use anyhow::{anyhow, Context, Result};
use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
};
use url::Url;
use uuid::Uuid;

use crate::{
    adapters::market::GithubMarketProvider,
    domain::{
        app_state::AppPaths,
        types::{
            ImportRepositorySkillRequest, InstallSkillRequest, ResolveRepositoryImportRequest,
            ResolveRepositoryImportResult, ResolvedRepositoryImportCandidate,
        },
    },
    path_utils::display_path,
    repositories::skills as skills_repository,
    services::{
        fs_utils::{ensure_clean_dir, remove_dir_if_present},
        install,
    },
};

#[derive(Debug, Clone)]
struct GithubRepoMetadata {
    owner: String,
    repo: String,
    html_url: String,
    repo_name: String,
    default_branch: String,
    resolved_ref: String,
    description: Option<String>,
    author: Option<String>,
}

#[derive(Debug, Clone)]
struct ParsedGithubInput {
    owner: String,
    repo: String,
    requested_tree_path: Option<String>,
    requested_ref: Option<String>,
}

fn slugify_name(value: &str) -> String {
    let mut slug = String::new();
    let mut previous_dash = false;

    for ch in value.trim().chars() {
        let normalized = if ch.is_ascii_alphanumeric() {
            previous_dash = false;
            Some(ch.to_ascii_lowercase())
        } else if previous_dash {
            None
        } else {
            previous_dash = true;
            Some('-')
        };

        if let Some(ch) = normalized {
            slug.push(ch);
        }
    }

    slug.trim_matches('-').to_string()
}

fn display_local_path(path: &Path) -> String {
    display_path(&path.to_string_lossy())
}

fn parse_github_input(input: &str) -> Result<ParsedGithubInput> {
    let url = Url::parse(input.trim()).context("invalid GitHub URL")?;
    let host = url
        .host_str()
        .ok_or_else(|| anyhow!("invalid GitHub URL host"))?;
    if host != "github.com" && host != "www.github.com" {
        return Err(anyhow!(
            "only github.com public repository URLs are supported"
        ));
    }

    let segments = url
        .path_segments()
        .ok_or_else(|| anyhow!("invalid GitHub URL path"))?
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>();

    if segments.len() < 2 {
        return Err(anyhow!("GitHub URL must include owner and repository"));
    }

    let owner = segments[0].to_string();
    let repo = segments[1].trim_end_matches(".git").to_string();

    if segments.len() == 2 {
        return Ok(ParsedGithubInput {
            owner,
            repo,
            requested_tree_path: None,
            requested_ref: None,
        });
    }

    if segments.len() >= 5 && segments[2] == "tree" {
        return Ok(ParsedGithubInput {
            owner,
            repo,
            requested_tree_path: Some(segments[4..].join("/")),
            requested_ref: Some(segments[3].to_string()),
        });
    }

    Err(anyhow!(
        "unsupported GitHub URL format; use a repository URL or a default-branch /tree/... URL"
    ))
}

fn github_get_json(url: &str) -> Result<Value> {
    let response = ureq::get(url)
        .set("User-Agent", concat!("skills-manager/", env!("CARGO_PKG_VERSION")))
        .set("Accept", "application/vnd.github+json")
        .call()
        .map_err(|error| anyhow!("github request failed: {}", error))?;

    let body = response
        .into_string()
        .context("failed to read GitHub response body")?;
    serde_json::from_str(&body).context("failed to parse GitHub response")
}

fn fetch_github_repo_metadata_with<F>(
    parsed: &ParsedGithubInput,
    fetch_json: &F,
) -> Result<GithubRepoMetadata>
where
    F: Fn(&str) -> Result<Value>,
{
    let repo_api_url = format!(
        "https://api.github.com/repos/{}/{}",
        parsed.owner, parsed.repo
    );
    let payload = fetch_json(&repo_api_url)?;
    let default_branch = payload
        .get("default_branch")
        .and_then(Value::as_str)
        .unwrap_or("main")
        .to_string();
    let branch_api_url = format!(
        "https://api.github.com/repos/{}/{}/branches/{}",
        parsed.owner, parsed.repo, default_branch
    );
    let branch_payload = fetch_json(&branch_api_url)?;
    let resolved_ref = branch_payload
        .get("commit")
        .and_then(|commit| commit.get("sha"))
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .map(ToString::to_string)
        .ok_or_else(|| anyhow!("failed to resolve GitHub default branch head commit"))?;

    Ok(GithubRepoMetadata {
        owner: parsed.owner.clone(),
        repo: parsed.repo.clone(),
        html_url: payload
            .get("html_url")
            .and_then(Value::as_str)
            .map(ToString::to_string)
            .unwrap_or_else(|| format!("https://github.com/{}/{}", parsed.owner, parsed.repo)),
        repo_name: payload
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or(&parsed.repo)
            .to_string(),
        default_branch,
        resolved_ref,
        description: payload
            .get("description")
            .and_then(Value::as_str)
            .map(ToString::to_string),
        author: payload
            .get("owner")
            .and_then(|owner| owner.get("login"))
            .and_then(Value::as_str)
            .map(ToString::to_string),
    })
}

fn build_github_tree_url(repo: &GithubRepoMetadata) -> String {
    format!(
        "https://api.github.com/repos/{}/{}/git/trees/{}?recursive=1",
        repo.owner, repo.repo, repo.default_branch
    )
}

fn github_slug_for(repo: &GithubRepoMetadata, skill_root: &str) -> String {
    let mut slug = format!("{}-{}", repo.owner, repo.repo).to_ascii_lowercase();
    let suffix = skill_root
        .replace('/', "-")
        .replace('\\', "-")
        .replace('.', "-")
        .to_ascii_lowercase();

    if !suffix.is_empty() {
        slug.push('-');
        slug.push_str(&suffix);
    }

    slug
}

fn github_display_name_for(repo: &GithubRepoMetadata, skill_root: &str) -> String {
    if skill_root.is_empty() {
        return repo.repo_name.clone();
    }

    skill_root
        .split(['/', '\\'])
        .filter(|segment| !segment.is_empty())
        .next_back()
        .unwrap_or(&repo.repo_name)
        .to_string()
}

fn github_source_url_for(repo: &GithubRepoMetadata, skill_root: &str) -> String {
    if skill_root.is_empty() {
        repo.html_url.clone()
    } else {
        format!(
            "{}/tree/{}/{}",
            repo.html_url, repo.resolved_ref, skill_root
        )
    }
}

fn github_candidates_from_tree(
    repo: &GithubRepoMetadata,
    tree_payload: &Value,
) -> Vec<ResolvedRepositoryImportCandidate> {
    GithubMarketProvider::discover_manifest_paths(tree_payload)
        .into_iter()
        .map(|manifest_path| {
            let skill_root = GithubMarketProvider::skill_root_from_manifest_path(&manifest_path);

            ResolvedRepositoryImportCandidate {
                name: github_display_name_for(repo, &skill_root),
                slug: github_slug_for(repo, &skill_root),
                manifest_path,
                skill_root: skill_root.clone(),
                source_url: github_source_url_for(repo, &skill_root),
                repo_url: Some(repo.html_url.clone()),
                version: Some(repo.resolved_ref.clone()),
                author: repo.author.clone(),
                description: repo.description.clone(),
            }
        })
        .collect()
}

fn filter_github_candidates_by_tree_path(
    candidates: Vec<ResolvedRepositoryImportCandidate>,
    requested_path: &str,
) -> Vec<ResolvedRepositoryImportCandidate> {
    let requested_path = install::normalize_relative_path(requested_path);

    candidates
        .into_iter()
        .filter(|candidate| {
            let skill_root = install::normalize_relative_path(&candidate.skill_root);
            let manifest_path = install::normalize_relative_path(&candidate.manifest_path);
            skill_root == requested_path
                || manifest_path == requested_path
                || skill_root.starts_with(&format!("{}/", requested_path))
                || manifest_path.starts_with(&format!("{}/", requested_path))
        })
        .collect()
}

fn resolve_github_import_source_with<F>(
    request: &ResolveRepositoryImportRequest,
    fetch_json: F,
) -> Result<ResolveRepositoryImportResult>
where
    F: Fn(&str) -> Result<Value>,
{
    let parsed = parse_github_input(&request.input)?;
    let repo = fetch_github_repo_metadata_with(&parsed, &fetch_json)?;

    if let Some(requested_ref) = parsed.requested_ref.as_deref() {
        if requested_ref != repo.default_branch {
            return Err(anyhow!(
                "only default-branch GitHub tree URLs are supported in this build"
            ));
        }
    }

    let tree_payload = fetch_json(&build_github_tree_url(&repo))?;
    let mut candidates = github_candidates_from_tree(&repo, &tree_payload);

    if let Some(requested_path) = parsed.requested_tree_path.as_deref() {
        candidates = filter_github_candidates_by_tree_path(candidates, requested_path);
        if candidates.is_empty() {
            return Err(anyhow!(
                "no installable skill was found under the selected GitHub tree path"
            ));
        }
    }

    Ok(ResolveRepositoryImportResult {
        source_kind: request.source_kind.clone(),
        normalized_input: parsed
            .requested_tree_path
            .as_deref()
            .map(|path| format!("{}/tree/{}/{}", repo.html_url, repo.default_branch, path))
            .unwrap_or_else(|| repo.html_url.clone()),
        candidates,
        warnings: Vec::new(),
    })
}

fn canonicalize_existing_path(path: &Path) -> Result<PathBuf> {
    fs::canonicalize(path).with_context(|| format!("failed to canonicalize {}", path.display()))
}

fn local_candidate_display_name(skill_root: &Path) -> String {
    skill_root
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.trim().is_empty())
        .unwrap_or("imported-skill")
        .to_string()
}

fn parse_skill_frontmatter_description(skill_root: &Path) -> Option<String> {
    let content = fs::read_to_string(skill_root.join("SKILL.md")).ok()?;
    let normalized = content.replace("\r\n", "\n");
    let mut lines = normalized.lines();

    if lines.next()? != "---" {
        return None;
    }

    for line in lines {
        if line.trim() == "---" {
            break;
        }

        let trimmed = line.trim();
        if let Some(value) = trimmed.strip_prefix("description:") {
            let value = value.trim().trim_matches('"').trim_matches('\'').trim();
            if !value.is_empty() {
                return Some(value.to_string());
            }
        }
    }

    None
}

fn build_local_candidates(
    scan_root: &Path,
    source_url: &str,
    roots: Vec<PathBuf>,
) -> Result<Vec<ResolvedRepositoryImportCandidate>> {
    roots
        .into_iter()
        .map(|skill_root| {
            let manifest_path = skill_root.join("SKILL.md");
            let relative_manifest_path = manifest_path
                .strip_prefix(scan_root)
                .unwrap_or(&manifest_path)
                .to_string_lossy()
                .replace('\\', "/");
            let relative_skill_root = skill_root
                .strip_prefix(scan_root)
                .unwrap_or(&skill_root)
                .to_string_lossy()
                .replace('\\', "/");
            let name = local_candidate_display_name(&skill_root);
            let slug = slugify_name(&name);
            let description = parse_skill_frontmatter_description(&skill_root);

            Ok(ResolvedRepositoryImportCandidate {
                name,
                slug: if slug.is_empty() {
                    "imported-skill".to_string()
                } else {
                    slug
                },
                manifest_path: relative_manifest_path,
                skill_root: relative_skill_root,
                source_url: source_url.to_string(),
                repo_url: None,
                version: None,
                author: None,
                description,
            })
        })
        .collect()
}

fn resolve_local_directory_import_source(
    request: &ResolveRepositoryImportRequest,
) -> Result<ResolveRepositoryImportResult> {
    let input_path = canonicalize_existing_path(Path::new(request.input.trim()))?;
    if !input_path.is_dir() {
        return Err(anyhow!("selected local import source is not a directory"));
    }

    let roots = install::collect_skill_roots(&input_path)?;
    if roots.is_empty() {
        return Err(anyhow!(
            "no SKILL.md was found in the selected local directory"
        ));
    }

    Ok(ResolveRepositoryImportResult {
        source_kind: request.source_kind.clone(),
        normalized_input: display_local_path(&input_path),
        candidates: build_local_candidates(&input_path, &display_local_path(&input_path), roots)?,
        warnings: Vec::new(),
    })
}

fn validate_local_zip_path(path: &Path) -> Result<PathBuf> {
    let canonical_path = canonicalize_existing_path(path)?;
    if !canonical_path.is_file() {
        return Err(anyhow!("selected local zip source is not a file"));
    }

    let extension = canonical_path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    if extension != "zip" {
        return Err(anyhow!(
            "only .zip files are supported for local zip import"
        ));
    }

    Ok(canonical_path)
}

fn resolve_local_zip_import_source(
    paths: &AppPaths,
    request: &ResolveRepositoryImportRequest,
) -> Result<ResolveRepositoryImportResult> {
    let zip_path = validate_local_zip_path(Path::new(request.input.trim()))?;
    let scratch = paths
        .temp_dir
        .join(format!("repository-import-{}", Uuid::new_v4()));
    let extract_dir = scratch.join("extract");
    ensure_clean_dir(&extract_dir)?;
    let bytes = fs::read(&zip_path)
        .with_context(|| format!("failed to read local zip {}", zip_path.display()))?;
    let result = (|| -> Result<ResolveRepositoryImportResult> {
        install::extract_zip_bytes(&bytes, &extract_dir)?;

        let roots = install::collect_skill_roots(&extract_dir)?;
        if roots.is_empty() {
            return Err(anyhow!("no SKILL.md was found in the selected zip file"));
        }

        Ok(ResolveRepositoryImportResult {
            source_kind: request.source_kind.clone(),
            normalized_input: display_local_path(&zip_path),
            candidates: build_local_candidates(
                &extract_dir,
                &display_local_path(&zip_path),
                roots,
            )?,
            warnings: Vec::new(),
        })
    })();

    let _ = remove_dir_if_present(&scratch);

    result
}

fn github_download_url(repo_url: &str, resolved_ref: &str) -> String {
    format!(
        "{}/archive/{}.zip",
        repo_url.trim_end_matches('/'),
        resolved_ref
    )
}

fn github_package_ref(repo_url: &str, skill_root: &str) -> Option<String> {
    let parsed = Url::parse(repo_url).ok()?;
    let mut segments = parsed.path_segments()?.collect::<Vec<_>>();
    if segments.len() < 2 {
        return None;
    }

    let owner = segments.remove(0);
    let repo = segments.remove(0).trim_end_matches(".git");
    let base = format!("{owner}/{repo}");

    if skill_root.trim().is_empty() {
        Some(base)
    } else {
        Some(format!("{}@{}", base, skill_root.replace('\\', "/")))
    }
}

fn build_install_request_for_import(
    request: &ImportRepositorySkillRequest,
) -> Result<InstallSkillRequest> {
    match request.source_kind.as_str() {
        "github" => {
            let repo_url = request
                .repo_url
                .clone()
                .ok_or_else(|| anyhow!("GitHub import requires repoUrl"))?;
            let resolved_ref = request
                .version
                .clone()
                .filter(|value| !value.trim().is_empty())
                .ok_or_else(|| anyhow!("GitHub import requires a stable ref"))?;

            Ok(InstallSkillRequest {
                provider: "github".to_string(),
                market_skill_id: request.slug.clone(),
                source_type: "github".to_string(),
                source_url: request.source_url.clone(),
                repo_url: Some(repo_url.clone()),
                download_url: Some(github_download_url(&repo_url, &resolved_ref)),
                package_ref: github_package_ref(&repo_url, &request.selected_skill_root),
                manifest_path: Some(request.selected_manifest_path.clone()),
                skill_root: Some(request.selected_skill_root.clone()),
                name: request.name.clone(),
                slug: request.slug.clone(),
                description: request.description.clone(),
                version: Some(resolved_ref),
                author: request.author.clone(),
                requested_targets: Vec::new(),
            })
        }
        "local_directory" => {
            let input_root = canonicalize_existing_path(Path::new(request.input.trim()))?;
            let selected_root = if request.selected_skill_root.trim().is_empty() {
                input_root.clone()
            } else {
                canonicalize_existing_path(&input_root.join(&request.selected_skill_root))?
            };

            Ok(InstallSkillRequest {
                provider: "local".to_string(),
                market_skill_id: request.slug.clone(),
                source_type: "local".to_string(),
                source_url: display_local_path(&input_root),
                repo_url: None,
                download_url: Some(selected_root.to_string_lossy().to_string()),
                package_ref: Some(format!("local:{}", request.slug)),
                manifest_path: None,
                skill_root: None,
                name: request.name.clone(),
                slug: request.slug.clone(),
                description: request.description.clone(),
                version: None,
                author: request.author.clone(),
                requested_targets: Vec::new(),
            })
        }
        "local_zip" => Ok(InstallSkillRequest {
            provider: "local".to_string(),
            market_skill_id: request.slug.clone(),
            source_type: "local".to_string(),
            source_url: display_local_path(&canonicalize_existing_path(Path::new(
                request.input.trim(),
            ))?),
            repo_url: None,
            download_url: None,
            package_ref: Some(format!("local:{}", request.slug)),
            manifest_path: Some(request.selected_manifest_path.clone()),
            skill_root: Some(request.selected_skill_root.clone()),
            name: request.name.clone(),
            slug: request.slug.clone(),
            description: request.description.clone(),
            version: None,
            author: request.author.clone(),
            requested_targets: Vec::new(),
        }),
        _ => Err(anyhow!("unsupported repository import source kind")),
    }
}

pub fn resolve_repository_import_source(
    paths: &AppPaths,
    request: &ResolveRepositoryImportRequest,
) -> Result<ResolveRepositoryImportResult> {
    match request.source_kind.as_str() {
        "github" => resolve_github_import_source_with(request, github_get_json),
        "local_directory" => resolve_local_directory_import_source(request),
        "local_zip" => resolve_local_zip_import_source(paths, request),
        _ => Err(anyhow!("unsupported repository import source kind")),
    }
}

pub fn import_repository_skill(
    paths: &AppPaths,
    request: &ImportRepositorySkillRequest,
) -> Result<crate::domain::types::InstallSkillResult> {
    if request.slug.trim().is_empty() {
        return Err(anyhow!("imported skill slug cannot be empty"));
    }

    if request.selected_manifest_path.trim().is_empty() {
        return Err(anyhow!("selected manifest path cannot be empty"));
    }

    let canonical_path = paths
        .canonical_store_dir
        .join(install::sanitize_slug(&request.slug));
    if canonical_path.exists()
        || skills_repository::repository_skill_slug_exists(&paths.db_file, &request.slug)?
    {
        return Err(anyhow!(
            "a repository skill with slug '{}' already exists",
            request.slug
        ));
    }

    let install_request = build_install_request_for_import(request)?;
    install::install_skill_with_policy(paths, &install_request, request.allow_risk_override)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{domain::app_state::AppPaths, repositories::db::run_migrations};
    use tempfile::tempdir;

    fn test_paths(root: &Path) -> AppPaths {
        let app_root = root.join("app-data");
        let db_dir = app_root.join("db");
        let temp_dir = app_root.join("tmp");
        let canonical_store_dir = app_root.join("skills");

        fs::create_dir_all(&db_dir).unwrap();
        fs::create_dir_all(&temp_dir).unwrap();
        fs::create_dir_all(&canonical_store_dir).unwrap();

        AppPaths {
            db_file: db_dir.join("skills-manager.db"),
            temp_dir,
            canonical_store_dir,
        }
    }

    fn github_repo_payload(default_branch: &str) -> Value {
        serde_json::json!({
            "name": "skills",
            "html_url": "https://github.com/vercel-labs/skills",
            "default_branch": default_branch,
            "description": "demo repo",
            "owner": { "login": "vercel-labs" }
        })
    }

    fn github_tree_payload(paths: &[&str]) -> Value {
        serde_json::json!({
            "tree": paths.iter().map(|path| {
                serde_json::json!({
                    "path": path,
                    "type": "blob"
                })
            }).collect::<Vec<_>>()
        })
    }

    fn github_branch_payload(sha: &str) -> Value {
        serde_json::json!({
            "commit": {
                "sha": sha
            }
        })
    }

    #[test]
    fn resolves_github_repo_url_to_single_candidate() {
        let request = ResolveRepositoryImportRequest {
            source_kind: "github".into(),
            input: "https://github.com/vercel-labs/skills".into(),
        };

        let result = resolve_github_import_source_with(&request, |url| {
            if url.contains("/git/trees/") {
                Ok(github_tree_payload(&["skills/react/SKILL.md"]))
            } else if url.contains("/branches/") {
                Ok(github_branch_payload("0123456789abcdef"))
            } else {
                Ok(github_repo_payload("main"))
            }
        })
        .unwrap();

        assert_eq!(
            result.normalized_input,
            "https://github.com/vercel-labs/skills"
        );
        assert_eq!(result.candidates.len(), 1);
        assert_eq!(result.candidates[0].slug, "vercel-labs-skills-skills-react");
    }

    #[test]
    fn resolves_github_tree_url_to_matching_candidate() {
        let request = ResolveRepositoryImportRequest {
            source_kind: "github".into(),
            input: "https://github.com/vercel-labs/skills/tree/main/skills/rust".into(),
        };

        let result = resolve_github_import_source_with(&request, |url| {
            if url.contains("/git/trees/") {
                Ok(github_tree_payload(&[
                    "skills/react/SKILL.md",
                    "skills/rust/SKILL.md",
                ]))
            } else if url.contains("/branches/") {
                Ok(github_branch_payload("0123456789abcdef"))
            } else {
                Ok(github_repo_payload("main"))
            }
        })
        .unwrap();

        assert_eq!(result.candidates.len(), 1);
        assert_eq!(result.candidates[0].skill_root, "skills/rust");
    }

    #[test]
    fn rejects_non_default_branch_tree_url() {
        let request = ResolveRepositoryImportRequest {
            source_kind: "github".into(),
            input: "https://github.com/vercel-labs/skills/tree/dev/skills/rust".into(),
        };

        let error = resolve_github_import_source_with(&request, |url| {
            if url.contains("/branches/") {
                Ok(github_branch_payload("0123456789abcdef"))
            } else {
                Ok(github_repo_payload("main"))
            }
        })
        .unwrap_err();

        assert!(error
            .to_string()
            .contains("only default-branch GitHub tree URLs are supported"));
    }

    #[test]
    fn imports_local_directory_skill_into_repository() {
        let dir = tempdir().unwrap();
        let paths = test_paths(dir.path());
        run_migrations(&paths.db_file).unwrap();

        let source_dir = dir.path().join("source").join("demo-skill");
        fs::create_dir_all(&source_dir).unwrap();
        fs::write(source_dir.join("SKILL.md"), "# demo").unwrap();

        let result = import_repository_skill(
            &paths,
            &ImportRepositorySkillRequest {
                source_kind: "local_directory".into(),
                input: source_dir.to_string_lossy().to_string(),
                selected_manifest_path: "SKILL.md".into(),
                selected_skill_root: "".into(),
                name: "demo-skill".into(),
                slug: "demo-skill".into(),
                source_url: source_dir.to_string_lossy().to_string(),
                repo_url: None,
                version: None,
                author: None,
                description: Some("Import description".into()),
                allow_risk_override: false,
            },
        )
        .unwrap();

        assert!(!result.blocked);
        assert!(PathBuf::from(result.canonical_path)
            .join("SKILL.md")
            .exists());

        let conn = crate::repositories::db::open_connection(&paths.db_file).unwrap();
        let description: Option<String> = conn
            .query_row(
                "SELECT description FROM skills WHERE slug = 'demo-skill'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(description.as_deref(), Some("Import description"));
    }

    #[test]
    fn imports_selected_skill_from_multi_skill_local_directory() {
        let dir = tempdir().unwrap();
        let paths = test_paths(dir.path());
        run_migrations(&paths.db_file).unwrap();

        let source_root = dir.path().join("source");
        let adapt_root = source_root.join("adapt");
        let animate_root = source_root.join("animate");
        fs::create_dir_all(&adapt_root).unwrap();
        fs::create_dir_all(&animate_root).unwrap();
        fs::write(adapt_root.join("SKILL.md"), "# adapt").unwrap();
        fs::write(adapt_root.join("README.md"), "adapt").unwrap();
        fs::write(animate_root.join("SKILL.md"), "# animate").unwrap();
        fs::write(animate_root.join("README.md"), "animate").unwrap();

        let result = import_repository_skill(
            &paths,
            &ImportRepositorySkillRequest {
                source_kind: "local_directory".into(),
                input: source_root.to_string_lossy().to_string(),
                selected_manifest_path: "adapt/SKILL.md".into(),
                selected_skill_root: "adapt".into(),
                name: "adapt".into(),
                slug: "adapt".into(),
                source_url: source_root.to_string_lossy().to_string(),
                repo_url: None,
                version: None,
                author: None,
                description: Some("Adapt layouts across screens.".into()),
                allow_risk_override: false,
            },
        )
        .unwrap();

        let canonical_root = PathBuf::from(result.canonical_path);
        assert!(canonical_root.join("SKILL.md").exists());
        assert_eq!(
            fs::read_to_string(canonical_root.join("README.md")).unwrap(),
            "adapt"
        );
        assert!(!canonical_root.join("animate").exists());
    }

    #[test]
    fn rejects_duplicate_slug_imports() {
        let dir = tempdir().unwrap();
        let paths = test_paths(dir.path());
        run_migrations(&paths.db_file).unwrap();

        let source_dir = dir.path().join("source").join("demo-skill");
        fs::create_dir_all(&source_dir).unwrap();
        fs::write(source_dir.join("SKILL.md"), "# demo").unwrap();

        let request = ImportRepositorySkillRequest {
            source_kind: "local_directory".into(),
            input: source_dir.to_string_lossy().to_string(),
            selected_manifest_path: "SKILL.md".into(),
            selected_skill_root: "".into(),
            name: "demo-skill".into(),
            slug: "demo-skill".into(),
            source_url: source_dir.to_string_lossy().to_string(),
            repo_url: None,
            version: None,
            author: None,
            description: Some("Import description".into()),
            allow_risk_override: false,
        };

        import_repository_skill(&paths, &request).unwrap();
        let error = import_repository_skill(&paths, &request).unwrap_err();
        assert!(error.to_string().contains("already exists"));
    }

    #[test]
    fn resolves_local_directory_candidate_description_from_skill_frontmatter() {
        let dir = tempdir().unwrap();
        let source_dir = dir.path().join("source").join("polish");
        fs::create_dir_all(&source_dir).unwrap();
        fs::write(
            source_dir.join("SKILL.md"),
            "---\nname: polish\ndescription: Final quality pass before shipping.\n---\n\nBody",
        )
        .unwrap();

        let result = resolve_local_directory_import_source(&ResolveRepositoryImportRequest {
            source_kind: "local_directory".into(),
            input: source_dir.to_string_lossy().to_string(),
        })
        .unwrap();

        assert_eq!(
            result.candidates[0].description.as_deref(),
            Some("Final quality pass before shipping.")
        );
    }

    #[test]
    fn resolves_local_zip_candidate_description_from_skill_frontmatter() {
        let dir = tempdir().unwrap();
        let paths = test_paths(dir.path());
        let zip_path = dir.path().join("polish.zip");

        super::super::fs_utils::ensure_clean_dir(&paths.temp_dir).unwrap();
        let file = fs::File::create(&zip_path).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        let options = zip::write::SimpleFileOptions::default();
        zip.start_file("polish/SKILL.md", options).unwrap();
        use std::io::Write as _;
        zip.write_all(
            b"---\nname: polish\ndescription: Final quality pass before shipping.\n---\n\nBody",
        )
        .unwrap();
        zip.finish().unwrap();

        let result = resolve_local_zip_import_source(
            &paths,
            &ResolveRepositoryImportRequest {
                source_kind: "local_zip".into(),
                input: zip_path.to_string_lossy().to_string(),
            },
        )
        .unwrap();

        assert_eq!(
            result.candidates[0].description.as_deref(),
            Some("Final quality pass before shipping.")
        );
    }
}
