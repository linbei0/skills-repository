use anyhow::{anyhow, Result};
use std::{
    fs,
    io::{Error, ErrorKind},
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

use crate::{
    domain::{
        agent_registry::AgentRegistry,
        types::{DistributionRequest, DistributionResult},
    },
    repositories::{distributions as distributions_repository, skills as skills_repository},
};

fn copy_dir_all(source: &Path, target: &Path) -> Result<()> {
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

fn remove_existing_target(path: &Path) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }

    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() || metadata.is_file() {
        fs::remove_file(path)?;
    } else {
        fs::remove_dir_all(path)?;
    }

    Ok(())
}

fn create_directory_symlink(source: &Path, target: &Path) -> Result<()> {
    #[cfg(target_os = "windows")]
    {
        std::os::windows::fs::symlink_dir(source, target)
            .map_err(|error| format_symlink_error(target, error))?;
    }

    #[cfg(not(target_os = "windows"))]
    {
        std::os::unix::fs::symlink(source, target)?;
    }

    Ok(())
}

fn format_symlink_error(target: &Path, error: Error) -> anyhow::Error {
    if error.kind() == ErrorKind::PermissionDenied {
        anyhow!(
            "Windows symlink permission denied for {}. Enable Developer Mode or run with elevated permission.",
            target.display()
        )
    } else {
        anyhow!(error)
    }
}

fn resolve_target_root(registry: &AgentRegistry, request: &DistributionRequest) -> Result<PathBuf> {
    match request.target_kind.as_str() {
        "global" => {
            let home =
                dirs::home_dir().ok_or_else(|| anyhow!("failed to resolve home directory"))?;
            let relative = registry
                .preferred_global_path_for(&request.target_agent)
                .ok_or_else(|| anyhow!("unknown target agent {}", request.target_agent))?;
            Ok(home.join(relative))
        }
        "project" => {
            let project_root = request
                .project_root
                .as_ref()
                .ok_or_else(|| anyhow!("project_root is required for project distribution"))?;
            let relative = registry
                .preferred_project_path_for(&request.target_agent)
                .ok_or_else(|| anyhow!("unknown target agent {}", request.target_agent))?;
            Ok(PathBuf::from(project_root).join(relative))
        }
        "custom" => request
            .custom_target_path
            .as_ref()
            .map(PathBuf::from)
            .ok_or_else(|| anyhow!("custom_target_path is required for custom distribution")),
        other => Err(anyhow!("unsupported target kind {}", other)),
    }
}

pub fn resolve_distribution_target_path(
    registry: &AgentRegistry,
    db_path: &Path,
    request: &DistributionRequest,
) -> Result<PathBuf> {
    let skill_source = skills_repository::load_skill_source(db_path, &request.skill_id)?;
    let target_root = resolve_target_root(registry, request)?;
    Ok(target_root.join(skill_source.target_name))
}

pub fn distribute_skill(
    registry: &AgentRegistry,
    db_path: &Path,
    request: &DistributionRequest,
) -> Result<DistributionResult> {
    let skill_source = skills_repository::load_skill_source(db_path, &request.skill_id)?;
    let source_path = PathBuf::from(&skill_source.source_path);
    if !source_path.exists() {
        return Err(anyhow!(
            "skill source path does not exist: {}",
            source_path.display()
        ));
    }

    let target_path = resolve_distribution_target_path(registry, db_path, request)?;
    let target_root = target_path
        .parent()
        .ok_or_else(|| anyhow!("distribution target has no parent directory"))?;
    fs::create_dir_all(target_root)?;
    remove_existing_target(&target_path)?;

    match request.install_mode.as_str() {
        "symlink" => create_directory_symlink(&source_path, &target_path)?,
        "copy" | "native" => copy_dir_all(&source_path, &target_path)?,
        other => return Err(anyhow!("unsupported install mode {}", other)),
    }

    let project_id = request
        .project_root
        .as_deref()
        .map(|root| distributions_repository::find_project_id_by_root(db_path, root))
        .transpose()?
        .flatten();

    distributions_repository::save_distribution(
        db_path,
        &request.skill_id,
        &request.target_kind,
        &request.target_agent,
        project_id.as_deref(),
        &target_path.to_string_lossy(),
        &request.install_mode,
        "active",
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        domain::{agent_registry::AgentRegistry, app_state::AppPaths, types::InstallSkillRequest},
        repositories::{db::run_migrations, skills as skills_repository},
    };
    use tempfile::tempdir;

    fn setup_paths(root: &Path) -> AppPaths {
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

    fn seed_installed_skill(paths: &AppPaths) -> String {
        let source_dir = paths.canonical_store_dir.join("demo-skill");
        fs::create_dir_all(&source_dir).unwrap();
        fs::write(source_dir.join("SKILL.md"), "# demo").unwrap();

        skills_repository::save_installed_skill(
            &paths.db_file,
            &InstallSkillRequest {
                provider: "github".into(),
                market_skill_id: "demo".into(),
                source_type: "github-resolved-skill".into(),
                source_url: "https://example.com/demo".into(),
                repo_url: Some("https://example.com/demo".into()),
                download_url: None,
                package_ref: Some("example/demo@skills/demo-skill".into()),
                manifest_path: Some("skills/demo-skill/SKILL.md".into()),
                skill_root: Some("skills/demo-skill".into()),
                name: "Demo Skill".into(),
                slug: "demo-skill".into(),
                version: Some("main".into()),
                author: Some("tester".into()),
                requested_targets: Vec::new(),
            },
            &source_dir.to_string_lossy(),
            "safe",
            false,
        )
        .unwrap()
    }

    #[test]
    fn copies_skill_into_project_target() {
        let dir = tempdir().unwrap();
        let paths = setup_paths(dir.path());
        run_migrations(&paths.db_file).unwrap();
        let skill_id = seed_installed_skill(&paths);
        let project_root = dir.path().join("workspace");
        fs::create_dir_all(&project_root).unwrap();

        let result = distribute_skill(
            &AgentRegistry::new(),
            &paths.db_file,
            &DistributionRequest {
                skill_id,
                target_kind: "project".into(),
                target_agent: "Claude Code".into(),
                install_mode: "copy".into(),
                project_root: Some(project_root.to_string_lossy().to_string()),
                custom_target_path: None,
            },
        )
        .unwrap();

        assert_eq!(result.status, "active");
        assert!(PathBuf::from(&result.target_path).join("SKILL.md").exists());
    }

    #[test]
    fn formats_windows_symlink_permission_errors_explicitly() {
        let target = PathBuf::from("E:/target-skill");
        let error = format_symlink_error(&target, Error::from(ErrorKind::PermissionDenied));

        #[cfg(target_os = "windows")]
        assert!(error
            .to_string()
            .contains("Windows symlink permission denied"));
    }
}
