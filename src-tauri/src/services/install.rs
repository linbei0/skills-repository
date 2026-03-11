use anyhow::{anyhow, Context, Result};
use serde_json::json;
use std::{
    fs,
    io::{Cursor, Read, Write},
    path::{Path, PathBuf},
};
use uuid::Uuid;
use walkdir::WalkDir;
use zip::ZipArchive;

use crate::{
    domain::{
        app_state::AppPaths,
        types::{InstallSkillRequest, InstallSkillResult},
    },
    repositories::{security as security_repository, skills as skills_repository},
    security,
};

pub(crate) fn ensure_clean_dir(path: &Path) -> Result<()> {
    if path.exists() {
        if path.is_dir() {
            fs::remove_dir_all(path)?;
        } else {
            fs::remove_file(path)?;
        }
    }
    fs::create_dir_all(path)?;
    Ok(())
}

pub(crate) fn sanitize_slug(slug: &str) -> String {
    slug.trim().replace('/', "-")
}

pub(crate) fn copy_dir_all(source: &Path, target: &Path) -> Result<()> {
    fs::create_dir_all(target)?;

    for entry in WalkDir::new(source) {
        let entry = entry?;
        let relative = entry.path().strip_prefix(source)?;
        let destination = target.join(relative);

        if entry.file_type().is_dir() {
            fs::create_dir_all(&destination)?;
        } else {
            if let Some(parent) = destination.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(entry.path(), &destination)?;
        }
    }

    Ok(())
}

pub(crate) fn extract_zip_bytes(bytes: &[u8], target_dir: &Path) -> Result<()> {
    ensure_clean_dir(target_dir)?;
    let reader = Cursor::new(bytes.to_vec());
    let mut archive = ZipArchive::new(reader).context("failed to open downloaded zip archive")?;

    for index in 0..archive.len() {
        let mut file = archive.by_index(index)?;
        let Some(enclosed_name) = file.enclosed_name().map(PathBuf::from) else {
            continue;
        };
        let out_path = target_dir.join(enclosed_name);

        if file.is_dir() {
            fs::create_dir_all(&out_path)?;
            continue;
        }

        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let mut out_file = fs::File::create(&out_path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        out_file.write_all(&buffer)?;
    }

    Ok(())
}

fn stage_source(temp_dir: &Path, request: &InstallSkillRequest) -> Result<PathBuf> {
    let staged_dir = temp_dir.join("staged");
    ensure_clean_dir(&staged_dir)?;

    let source = request
        .download_url
        .clone()
        .unwrap_or_else(|| request.source_url.clone());

    if source.starts_with("http://") || source.starts_with("https://") {
        let response = ureq::get(&source)
            .set("User-Agent", "skills-manager/0.1.0")
            .call()
            .map_err(|error| anyhow!("failed to download skill archive: {}", error))?;
        let mut bytes = Vec::new();
        response
            .into_reader()
            .read_to_end(&mut bytes)
            .context("failed to read downloaded archive bytes")?;
        extract_zip_bytes(&bytes, &staged_dir)?;
        return Ok(staged_dir);
    }

    let local_path = PathBuf::from(&source);
    if !local_path.exists() {
        return Err(anyhow!("install source does not exist: {}", source));
    }

    if local_path.is_dir() {
        copy_dir_all(&local_path, &staged_dir)?;
        return Ok(staged_dir);
    }

    let bytes = fs::read(&local_path)
        .with_context(|| format!("failed to read install source {}", source))?;
    extract_zip_bytes(&bytes, &staged_dir)?;
    Ok(staged_dir)
}

fn find_skill_root(root: &Path) -> Result<PathBuf> {
    collect_skill_roots(root)?
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("no SKILL.md found in downloaded source"))
}

pub(crate) fn collect_skill_roots(root: &Path) -> Result<Vec<PathBuf>> {
    let mut roots = Vec::new();
    for entry in WalkDir::new(root) {
        let entry = entry?;
        if entry.file_type().is_file() && entry.file_name() == "SKILL.md" {
            let parent = entry
                .path()
                .parent()
                .map(PathBuf::from)
                .ok_or_else(|| anyhow!("skill root has no parent"))?;
            roots.push(parent);
        }
    }

    roots.sort();
    roots.dedup();
    Ok(roots)
}

pub(crate) fn normalize_relative_path(value: &str) -> String {
    value
        .trim()
        .trim_matches('/')
        .replace('\\', "/")
        .trim_end_matches('/')
        .to_string()
}

pub(crate) fn path_suffix_matches(candidate: &str, expected: &str) -> bool {
    let expected = normalize_relative_path(expected);
    let candidate = normalize_relative_path(candidate);
    candidate == expected || candidate.ends_with(&format!("/{}", expected))
}

fn resolve_requested_skill_root(root: &Path, request: &InstallSkillRequest) -> Result<PathBuf> {
    let roots = collect_skill_roots(root)?;
    if roots.is_empty() {
        return Err(anyhow!("no SKILL.md found in downloaded source"));
    }

    if let Some(manifest_path) = request
        .manifest_path
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        if let Some(found) = roots.iter().find(|candidate| {
            let manifest_candidate = candidate.join("SKILL.md");
            let relative = manifest_candidate
                .strip_prefix(root)
                .unwrap_or(&manifest_candidate)
                .to_string_lossy()
                .to_string();
            path_suffix_matches(&relative, manifest_path)
        }) {
            return Ok(found.clone());
        }

        return Err(anyhow!(
            "requested manifest path was not found in downloaded source: {}",
            manifest_path
        ));
    }

    if let Some(skill_root) = request
        .skill_root
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        if let Some(found) = roots.iter().find(|candidate| {
            let relative = candidate
                .strip_prefix(root)
                .unwrap_or(candidate)
                .to_string_lossy()
                .to_string();
            path_suffix_matches(&relative, skill_root)
        }) {
            return Ok(found.clone());
        }

        return Err(anyhow!(
            "requested skill root was not found in downloaded source: {}",
            skill_root
        ));
    }

    find_skill_root(root)
}

pub fn install_skill(
    paths: &AppPaths,
    request: &InstallSkillRequest,
) -> Result<InstallSkillResult> {
    let install_temp_dir = paths.temp_dir.join(format!("install-{}", Uuid::new_v4()));
    ensure_clean_dir(&install_temp_dir)?;

    let install_result = (|| -> Result<InstallSkillResult> {
        let staged_dir = stage_source(&install_temp_dir, request)?;
        let skill_root = resolve_requested_skill_root(&staged_dir, request)?;
        let security_report = security::scan_skill_directory_with_context(
            &skill_root,
            None,
            "temp_install",
            &security::SecurityScanSourceContext {
                source_url: Some(request.source_url.clone()),
                repo_url: request.repo_url.clone(),
                download_url: request.download_url.clone(),
                version: request.version.clone(),
                manifest_path: request.manifest_path.clone(),
                skill_root: request.skill_root.clone(),
            },
        )?;

        if security_report.blocked {
            security_repository::save_security_report(&paths.db_file, &security_report)?;
            let operation_log_id = skills_repository::save_operation_log(
                &paths.db_file,
                "install",
                "skill",
                None,
                "failed",
                "skill installation blocked by security scan",
                Some(json!({ "securityReport": security_report })),
            )?;

            return Ok(InstallSkillResult {
                skill_id: String::new(),
                canonical_path: String::new(),
                blocked: true,
                security_level: security_report.level,
                operation_log_id: Some(operation_log_id),
            });
        }

        let canonical_path = paths.canonical_store_dir.join(sanitize_slug(&request.slug));
        ensure_clean_dir(&canonical_path)?;
        copy_dir_all(&skill_root, &canonical_path)?;

        let skill_id = skills_repository::save_installed_skill(
            &paths.db_file,
            request,
            &canonical_path.to_string_lossy(),
            &security_report.level,
            false,
        )?;

        let mut persisted_report = security_report.clone();
        persisted_report.id = Uuid::new_v4().to_string();
        persisted_report.skill_id = Some(skill_id.clone());
        persisted_report.skill_name = Some(request.name.clone());
        persisted_report.source_path = Some(canonical_path.to_string_lossy().to_string());
        security_repository::save_security_report(&paths.db_file, &persisted_report)?;
        skills_repository::update_skill_security_status(
            &paths.db_file,
            &skill_id,
            &persisted_report.level,
            persisted_report.blocked,
            persisted_report.scanned_at,
        )?;

        let operation_log_id = skills_repository::save_operation_log(
            &paths.db_file,
            "install",
            "skill",
            Some(&skill_id),
            "success",
            "skill installed into canonical store",
            Some(json!({
                "canonicalPath": canonical_path.to_string_lossy(),
                "securityLevel": security_report.level,
            })),
        )?;

        Ok(InstallSkillResult {
            skill_id,
            canonical_path: canonical_path.to_string_lossy().to_string(),
            blocked: false,
            security_level: security_report.level,
            operation_log_id: Some(operation_log_id),
        })
    })();

    let cleanup_result = fs::remove_dir_all(&install_temp_dir);
    if let Err(error) = cleanup_result {
        log::warn!(
            "failed to remove install temp dir {}: {}",
            install_temp_dir.display(),
            error
        );
    }

    if install_result.is_err() {
        let canonical_path = paths.canonical_store_dir.join(sanitize_slug(&request.slug));
        if canonical_path.exists() {
            let _ = fs::remove_dir_all(&canonical_path);
        }
    }

    install_result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        domain::app_state::AppPaths,
        repositories::{db::{open_connection, run_migrations}, security as security_repository},
    };
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

    fn write_zip(target: &Path, entries: &[(&str, &str)]) {
        let file = fs::File::create(target).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        let options = zip::write::SimpleFileOptions::default();

        for (name, content) in entries {
            zip.start_file(name, options).unwrap();
            zip.write_all(content.as_bytes()).unwrap();
        }

        zip.finish().unwrap();
    }

    fn request(download_url: String) -> InstallSkillRequest {
        InstallSkillRequest {
            provider: "github".into(),
            market_skill_id: "demo".into(),
            source_type: "github-resolved-skill".into(),
            source_url: download_url.clone(),
            repo_url: Some("https://github.com/demo/demo-skill".into()),
            download_url: Some(download_url),
            package_ref: Some("demo/demo-skill".into()),
            manifest_path: None,
            skill_root: None,
            name: "Demo Skill".into(),
            slug: "demo-skill".into(),
            version: Some("main".into()),
            author: Some("tester".into()),
            requested_targets: Vec::new(),
        }
    }

    fn temp_install_report_count(db_file: &Path) -> i64 {
        let conn = open_connection(db_file).unwrap();
        conn.query_row(
            "SELECT COUNT(*) FROM security_reports WHERE skill_id IS NULL AND scan_scope = 'temp_install'",
            [],
            |row| row.get(0),
        )
        .unwrap()
    }

    #[test]
    fn installs_skill_into_canonical_store() {
        let dir = tempdir().unwrap();
        let paths = test_paths(dir.path());
        run_migrations(&paths.db_file).unwrap();

        let zip_path = dir.path().join("skill.zip");
        write_zip(
            &zip_path,
            &[
                ("demo-skill/SKILL.md", "# demo"),
                ("demo-skill/README.md", "ok"),
            ],
        );

        let result = install_skill(
            &paths,
            &request(zip_path.to_string_lossy().to_string()),
        )
        .unwrap();

        assert!(!result.blocked);
        assert!(!result.skill_id.is_empty());
        assert!(PathBuf::from(&result.canonical_path)
            .join("SKILL.md")
            .exists());
    }

    #[test]
    fn successful_install_does_not_persist_temp_install_report() {
        let dir = tempdir().unwrap();
        let paths = test_paths(dir.path());
        run_migrations(&paths.db_file).unwrap();

        let zip_path = dir.path().join("skill.zip");
        write_zip(
            &zip_path,
            &[
                ("demo-skill/SKILL.md", "# demo"),
                ("demo-skill/README.md", "ok"),
            ],
        );

        let result = install_skill(
            &paths,
            &request(zip_path.to_string_lossy().to_string()),
        )
        .unwrap();

        let reports = security_repository::list_security_reports(&paths.db_file).unwrap();

        assert!(!result.blocked);
        assert_eq!(temp_install_report_count(&paths.db_file), 0);
        assert_eq!(reports.len(), 1);
        assert_eq!(reports[0].skill_id.as_deref(), Some(result.skill_id.as_str()));
    }

    #[test]
    fn blocks_high_risk_skill_before_persisting() {
        let dir = tempdir().unwrap();
        let paths = test_paths(dir.path());
        run_migrations(&paths.db_file).unwrap();

        let zip_path = dir.path().join("blocked.zip");
        write_zip(
            &zip_path,
            &[
                ("blocked-skill/SKILL.md", "# blocked"),
                ("blocked-skill/install.sh", "rm -rf /"),
            ],
        );

        let result = install_skill(
            &paths,
            &request(zip_path.to_string_lossy().to_string()),
        )
        .unwrap();

        assert!(result.blocked);
        assert_eq!(result.security_level, "high");
        assert!(result.skill_id.is_empty());
        assert!(!paths.canonical_store_dir.join("demo-skill").exists());
        assert_eq!(temp_install_report_count(&paths.db_file), 1);
    }

    #[test]
    fn blocks_insecure_http_source_before_persisting() {
        let dir = tempdir().unwrap();
        let paths = test_paths(dir.path());
        run_migrations(&paths.db_file).unwrap();

        let zip_path = dir.path().join("http-source.zip");
        write_zip(
            &zip_path,
            &[
                ("http-source-skill/SKILL.md", "# insecure"),
                ("http-source-skill/README.md", "content"),
            ],
        );

        let mut install_request = request(zip_path.to_string_lossy().to_string());
        install_request.source_url = "http://example.com/http-source-skill.zip".into();
        install_request.download_url = Some(zip_path.to_string_lossy().to_string());
        install_request.slug = "http-source-skill".into();

        let result = install_skill(&paths, &install_request).unwrap();

        assert!(result.blocked);
        assert!(result.skill_id.is_empty());
        assert!(!paths
            .canonical_store_dir
            .join("http-source-skill")
            .exists());
        assert_eq!(temp_install_report_count(&paths.db_file), 1);
    }

    #[test]
    fn rolls_back_when_skill_manifest_is_missing() {
        let dir = tempdir().unwrap();
        let paths = test_paths(dir.path());
        run_migrations(&paths.db_file).unwrap();

        let zip_path = dir.path().join("broken.zip");
        write_zip(&zip_path, &[("broken-skill/README.md", "no manifest")]);

        let result = install_skill(
            &paths,
            &request(zip_path.to_string_lossy().to_string()),
        );

        assert!(result.is_err());
        assert!(!paths.canonical_store_dir.join("demo-skill").exists());
    }

    #[test]
    fn installs_exact_skill_root_when_repository_contains_multiple_skills() {
        let dir = tempdir().unwrap();
        let paths = test_paths(dir.path());
        run_migrations(&paths.db_file).unwrap();

        let zip_path = dir.path().join("multi-skill.zip");
        write_zip(
            &zip_path,
            &[
                ("repo-main/skills/demo-skill/SKILL.md", "# demo"),
                ("repo-main/skills/demo-skill/README.md", "demo"),
                ("repo-main/skills/other-skill/SKILL.md", "# other"),
                ("repo-main/skills/other-skill/README.md", "other"),
            ],
        );

        let result = install_skill(
            &paths,
            &InstallSkillRequest {
                manifest_path: Some("skills/demo-skill/SKILL.md".into()),
                skill_root: Some("skills/demo-skill".into()),
                package_ref: Some("demo/demo-skill@skills/demo-skill".into()),
                ..request(zip_path.to_string_lossy().to_string())
            },
        )
        .unwrap();

        assert!(!result.blocked);
        assert!(PathBuf::from(&result.canonical_path).join("README.md").exists());
        assert_eq!(
            fs::read_to_string(PathBuf::from(&result.canonical_path).join("README.md")).unwrap(),
            "demo"
        );
        assert!(!PathBuf::from(&result.canonical_path)
            .join("../other-skill")
            .exists());
    }
}
