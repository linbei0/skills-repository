use anyhow::{anyhow, Context, Result};
use serde_json::Value;

use crate::domain::types::{
    MarketSearchRequest, MarketSearchResponse, MarketSkillSummary, ProviderStatus,
};

pub const GITHUB_PROVIDER: &str = "github";

pub(crate) const SKILL_DIRECTORY_PREFIXES: &[&str] = &[
    "skills/",
    ".claude/skills/",
    ".agents/skills/",
    ".github/skills/",
];

pub trait MarketProviderAdapter: Send + Sync {
    fn provider_id(&self) -> &'static str;
    fn search(&self, request: &MarketSearchRequest) -> Result<MarketSearchResponse>;
}

#[derive(Debug, Default, Clone)]
pub struct GithubMarketProvider;

#[derive(Debug, Clone)]
struct GithubRepoCandidate {
    repo_id: String,
    full_name: String,
    repo_name: String,
    html_url: String,
    default_branch: String,
    description: Option<String>,
    author: Option<String>,
    tags: Vec<String>,
}

impl GithubMarketProvider {
    fn build_search_url(&self, request: &MarketSearchRequest) -> String {
        let query = if request.query.trim().is_empty() {
            "SKILL.md path:skills stars:>5".to_string()
        } else {
            format!("{} SKILL.md skills", request.query.trim())
        };

        format!(
            "https://api.github.com/search/repositories?q={}&sort=stars&order=desc&per_page={}&page={}",
            urlencoding::encode(&query),
            request.page_size.min(20),
            request.page.max(1)
        )
    }

    fn github_get_json(url: &str) -> Result<Value> {
        let response = ureq::get(url)
            .set("User-Agent", "skills-manager/0.1.0")
            .set("Accept", "application/vnd.github+json")
            .call()
            .map_err(|error| anyhow!("github provider request failed: {}", error))?;

        let body = response
            .into_string()
            .context("failed to read github provider response body")?;
        serde_json::from_str(&body).context("failed to parse github provider response")
    }

    fn parse_repository(item: &Value) -> Option<GithubRepoCandidate> {
        let full_name = item.get("full_name")?.as_str()?.to_string();
        let html_url = item.get("html_url")?.as_str()?.to_string();
        let repo_name = item.get("name")?.as_str()?.to_string();
        let default_branch = item
            .get("default_branch")
            .and_then(Value::as_str)
            .unwrap_or("main")
            .to_string();

        Some(GithubRepoCandidate {
            repo_id: item.get("id")?.as_i64()?.to_string(),
            full_name,
            repo_name,
            html_url,
            default_branch,
            description: item
                .get("description")
                .and_then(Value::as_str)
                .map(ToString::to_string),
            author: item
                .get("owner")
                .and_then(|owner| owner.get("login"))
                .and_then(Value::as_str)
                .map(ToString::to_string),
            tags: item
                .get("topics")
                .and_then(Value::as_array)
                .map(|topics| {
                    topics
                        .iter()
                        .filter_map(Value::as_str)
                        .map(ToString::to_string)
                        .collect()
                })
                .unwrap_or_default(),
        })
    }

    fn build_tree_url(repo: &GithubRepoCandidate) -> String {
        format!(
            "https://api.github.com/repos/{}/git/trees/{}?recursive=1",
            repo.full_name, repo.default_branch
        )
    }

    pub(crate) fn is_manifest_path_supported(path: &str) -> bool {
        if !path.ends_with("SKILL.md") {
            return false;
        }

        let normalized = path.replace('\\', "/");
        if normalized.contains("/node_modules/")
            || normalized.contains("/dist/")
            || normalized.contains("/build/")
            || normalized.starts_with("node_modules/")
            || normalized.starts_with("dist/")
            || normalized.starts_with("build/")
        {
            return false;
        }

        if normalized == "SKILL.md" {
            return true;
        }

        SKILL_DIRECTORY_PREFIXES
            .iter()
            .any(|prefix| normalized.starts_with(prefix))
            || normalized.matches('/').count() <= 3
    }

    pub(crate) fn discover_manifest_paths(tree_payload: &Value) -> Vec<String> {
        let mut manifests = tree_payload
            .get("tree")
            .and_then(Value::as_array)
            .map(|entries| {
                entries
                    .iter()
                    .filter(|entry| entry.get("type").and_then(Value::as_str) == Some("blob"))
                    .filter_map(|entry| entry.get("path").and_then(Value::as_str))
                    .filter(|path| Self::is_manifest_path_supported(path))
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        manifests.sort();
        manifests.dedup();
        manifests
    }

    fn package_ref_for(repo: &GithubRepoCandidate, skill_root: &str) -> String {
        if skill_root.is_empty() {
            repo.full_name.clone()
        } else {
            format!("{}@{}", repo.full_name, skill_root.replace('\\', "/"))
        }
    }

    fn slug_for(repo: &GithubRepoCandidate, skill_root: &str) -> String {
        let mut slug = repo.full_name.replace('/', "-");
        if !skill_root.is_empty() {
            slug.push('-');
            slug.push_str(
                &skill_root
                    .replace('/', "-")
                    .replace('\\', "-")
                    .replace('.', "-")
                    .to_ascii_lowercase(),
            );
        }
        slug
    }

    fn display_name_for(repo: &GithubRepoCandidate, skill_root: &str) -> String {
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

    pub(crate) fn skill_root_from_manifest_path(path: &str) -> String {
        path.rsplit_once('/')
            .map(|(parent, _)| parent.to_string())
            .unwrap_or_default()
    }

    fn source_url_for(repo: &GithubRepoCandidate, skill_root: &str) -> String {
        if skill_root.is_empty() {
            repo.html_url.clone()
        } else {
            format!(
                "{}/tree/{}/{}",
                repo.html_url, repo.default_branch, skill_root
            )
        }
    }

    fn resolve_repo_skills(repo: &GithubRepoCandidate, tree_payload: &Value) -> Vec<MarketSkillSummary> {
        Self::discover_manifest_paths(tree_payload)
            .into_iter()
            .map(|manifest_path| {
                let skill_root = Self::skill_root_from_manifest_path(&manifest_path);
                MarketSkillSummary {
                    id: format!("{}:{}", repo.repo_id, manifest_path),
                    slug: Self::slug_for(repo, &skill_root),
                    name: Self::display_name_for(repo, &skill_root),
                    description: repo.description.clone(),
                    provider: GITHUB_PROVIDER.to_string(),
                    source_type: "github-resolved-skill".to_string(),
                    source_url: Self::source_url_for(repo, &skill_root),
                    repo_url: Some(repo.html_url.clone()),
                    download_url: Some(format!(
                        "{}/archive/refs/heads/{}.zip",
                        repo.html_url, repo.default_branch
                    )),
                    package_ref: Some(Self::package_ref_for(repo, &skill_root)),
                    manifest_path: Some(manifest_path),
                    skill_root: Some(skill_root),
                    version: Some(repo.default_branch.clone()),
                    author: repo.author.clone(),
                    tags: repo.tags.clone(),
                    installable: true,
                    resolver_status: "resolved".to_string(),
                }
            })
            .collect()
    }
}

impl MarketProviderAdapter for GithubMarketProvider {
    fn provider_id(&self) -> &'static str {
        GITHUB_PROVIDER
    }

    fn search(&self, request: &MarketSearchRequest) -> Result<MarketSearchResponse> {
        let payload = Self::github_get_json(&self.build_search_url(request))?;
        let repos = payload
            .get("items")
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(Self::parse_repository)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        let mut results = Vec::new();
        let mut resolution_failures = 0_u32;

        for repo in &repos {
            match Self::github_get_json(&Self::build_tree_url(repo)) {
                Ok(tree_payload) => {
                    results.extend(Self::resolve_repo_skills(repo, &tree_payload));
                }
                Err(error) => {
                    resolution_failures += 1;
                    log::warn!(
                        "failed to resolve repo {} into installable skills: {}",
                        repo.full_name,
                        error
                    );
                }
            }
        }

        let provider_status = if resolution_failures > 0 {
            ProviderStatus {
                provider: self.provider_id().to_string(),
                status: "partial".to_string(),
                message: Some(format!(
                    "resolved {} installable skills from {} repositories; {} repository resolutions failed",
                    results.len(),
                    repos.len(),
                    resolution_failures
                )),
                cache_hit: false,
            }
        } else {
            ProviderStatus {
                provider: self.provider_id().to_string(),
                status: "ok".to_string(),
                message: Some(format!(
                    "resolved {} installable skills from {} repositories",
                    results.len(),
                    repos.len()
                )),
                cache_hit: false,
            }
        };

        Ok(MarketSearchResponse {
            total: results.len() as u32,
            results,
            providers: vec![provider_status],
            page: request.page.max(1),
            page_size: request.page_size.min(20),
            cache_hit: false,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn repo() -> GithubRepoCandidate {
        GithubRepoCandidate {
            repo_id: "42".into(),
            full_name: "vercel-labs/skills".into(),
            repo_name: "skills".into(),
            html_url: "https://github.com/vercel-labs/skills".into(),
            default_branch: "main".into(),
            description: Some("skill catalog".into()),
            author: Some("vercel-labs".into()),
            tags: vec!["skills".into()],
        }
    }

    #[test]
    fn discovers_supported_manifest_paths() {
        let tree_payload = json!({
            "tree": [
                { "path": "skills/react/SKILL.md", "type": "blob" },
                { "path": ".claude/skills/deploy/SKILL.md", "type": "blob" },
                { "path": "docs/SKILL.md", "type": "blob" },
                { "path": "node_modules/pkg/SKILL.md", "type": "blob" },
                { "path": "README.md", "type": "blob" }
            ]
        });

        let manifests = GithubMarketProvider::discover_manifest_paths(&tree_payload);

        assert_eq!(
            manifests,
            vec![
                ".claude/skills/deploy/SKILL.md".to_string(),
                "docs/SKILL.md".to_string(),
                "skills/react/SKILL.md".to_string()
            ]
        );
    }

    #[test]
    fn expands_each_manifest_to_installable_skill() {
        let tree_payload = json!({
            "tree": [
                { "path": "skills/react/SKILL.md", "type": "blob" },
                { "path": "skills/testing/SKILL.md", "type": "blob" }
            ]
        });

        let resolved = GithubMarketProvider::resolve_repo_skills(&repo(), &tree_payload);

        assert_eq!(resolved.len(), 2);
        assert_eq!(resolved[0].name, "react");
        assert_eq!(resolved[0].skill_root.as_deref(), Some("skills/react"));
        assert_eq!(
            resolved[0].package_ref.as_deref(),
            Some("vercel-labs/skills@skills/react")
        );
        assert!(resolved.iter().all(|skill| skill.installable));
    }
}
