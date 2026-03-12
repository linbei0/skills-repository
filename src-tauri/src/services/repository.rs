use anyhow::{anyhow, Context, Result};
use std::{fs, path::Path};

use crate::{
    domain::types::{
        RepositorySkillDeletionPreview, RepositorySkillDetail, RepositorySkillSummary,
        RepositoryUninstallResult,
    },
    repositories::skills as skills_repository,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RemovalKind {
    File,
    Directory,
}

fn resolve_removal_kind(path: &Path, metadata: &fs::Metadata) -> RemovalKind {
    if metadata.file_type().is_symlink() {
        if path.is_dir() {
            RemovalKind::Directory
        } else {
            RemovalKind::File
        }
    } else if metadata.is_file() {
        RemovalKind::File
    } else {
        RemovalKind::Directory
    }
}

fn remove_path_if_present(path: &Path) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }

    let metadata = fs::symlink_metadata(path)
        .with_context(|| format!("failed to read metadata for {}", path.display()))?;
    if resolve_removal_kind(path, &metadata) == RemovalKind::File {
        fs::remove_file(path)
            .with_context(|| format!("failed to remove file {}", path.display()))?;
    } else {
        fs::remove_dir_all(path)
            .with_context(|| format!("failed to remove directory {}", path.display()))?;
    }

    Ok(())
}

pub fn list_repository_skills(
    db_path: &Path,
    canonical_store_dir: &Path,
) -> Result<Vec<RepositorySkillSummary>> {
    skills_repository::list_repository_skills(db_path, canonical_store_dir)
}

pub fn get_repository_skill_detail(
    db_path: &Path,
    canonical_store_dir: &Path,
    skill_id: &str,
) -> Result<RepositorySkillDetail> {
    skills_repository::get_repository_skill_detail(db_path, canonical_store_dir, skill_id)
}

pub fn get_repository_skill_deletion_preview(
    db_path: &Path,
    canonical_store_dir: &Path,
    skill_id: &str,
) -> Result<RepositorySkillDeletionPreview> {
    let plan = skills_repository::load_repository_skill_removal_plan(
        db_path,
        canonical_store_dir,
        skill_id,
    )?;

    Ok(RepositorySkillDeletionPreview {
        skill_id: plan.skill_id,
        skill_name: plan.skill_name,
        canonical_path: plan.canonical_path,
        distribution_paths: plan.distribution_paths,
    })
}

pub fn uninstall_repository_skill(
    db_path: &Path,
    canonical_store_dir: &Path,
    skill_id: &str,
) -> Result<RepositoryUninstallResult> {
    let plan = skills_repository::load_repository_skill_removal_plan(
        db_path,
        canonical_store_dir,
        skill_id,
    )?;

    let mut removed_paths = Vec::new();
    for distribution_path in &plan.distribution_paths {
        let target = Path::new(distribution_path);
        remove_path_if_present(target)?;
        removed_paths.push(target.to_string_lossy().to_string());
    }

    let canonical_path = Path::new(&plan.canonical_path);
    if !canonical_path.exists() {
        return Err(anyhow!(
            "canonical skill path does not exist: {}",
            canonical_path.display()
        ));
    }
    remove_path_if_present(canonical_path)?;
    removed_paths.push(canonical_path.to_string_lossy().to_string());

    skills_repository::delete_repository_skill(db_path, &plan.skill_id)?;

    Ok(RepositoryUninstallResult {
        skill_id: plan.skill_id,
        removed_paths,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        domain::types::{DistributionRequest, InstallSkillRequest},
        repositories::{
            db::run_migrations, distributions as distributions_repository,
            skills as skills_repository,
        },
    };
    use std::{fs, io::ErrorKind};
    use tempfile::tempdir;

    fn setup_skill_fixture() -> (
        std::path::PathBuf,
        std::path::PathBuf,
        String,
        std::path::PathBuf,
    ) {
        let dir = tempdir().unwrap();
        let root = dir.keep();
        let app_data_dir = root.join("app-data");
        let db_dir = app_data_dir.join("db");
        let canonical_store_dir = app_data_dir.join("skills");
        fs::create_dir_all(&db_dir).unwrap();
        fs::create_dir_all(&canonical_store_dir).unwrap();
        let db_path = db_dir.join("skills-manager.db");
        run_migrations(&db_path).unwrap();

        let skill_dir = canonical_store_dir.join("demo-skill");
        fs::create_dir_all(&skill_dir).unwrap();
        fs::write(skill_dir.join("SKILL.md"), "# demo").unwrap();

        let skill_id = skills_repository::save_installed_skill(
            &db_path,
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
                name: "Demo".into(),
                slug: "demo-skill".into(),
                description: Some("Demo repository skill".into()),
                version: None,
                author: None,
                requested_targets: Vec::<DistributionRequest>::new(),
            },
            &skill_dir.to_string_lossy(),
            "safe",
            false,
        )
        .unwrap();

        let distributed_path = root.join("distributed").join("demo-skill");
        fs::create_dir_all(distributed_path.parent().unwrap()).unwrap();
        fs::create_dir_all(&distributed_path).unwrap();
        fs::write(distributed_path.join("SKILL.md"), "# dist").unwrap();

        distributions_repository::save_distribution(
            &db_path,
            &skill_id,
            "global",
            "Codex",
            &distributed_path.to_string_lossy(),
            "copy",
            "active",
        )
        .unwrap();

        (db_path, canonical_store_dir, skill_id, distributed_path)
    }

    #[test]
    fn uninstalls_repository_skill_and_distributions() {
        let (db_path, canonical_store_dir, skill_id, distributed_path) = setup_skill_fixture();
        let canonical_skill_dir = canonical_store_dir.join("demo-skill");

        let result = uninstall_repository_skill(&db_path, &canonical_store_dir, &skill_id).unwrap();

        assert_eq!(result.skill_id, skill_id);
        assert!(!distributed_path.exists());
        assert!(!canonical_skill_dir.exists());
        let remaining =
            skills_repository::list_repository_skills(&db_path, &canonical_store_dir).unwrap();
        assert!(remaining.is_empty());
    }

    #[test]
    fn returns_deletion_preview_with_distribution_paths() {
        let (db_path, canonical_store_dir, skill_id, distributed_path) = setup_skill_fixture();

        let preview =
            get_repository_skill_deletion_preview(&db_path, &canonical_store_dir, &skill_id)
                .unwrap();

        assert_eq!(preview.skill_id, skill_id);
        assert_eq!(preview.skill_name, "Demo");
        assert_eq!(
            preview.distribution_paths,
            vec![distributed_path.to_string_lossy().to_string()]
        );
    }

    #[test]
    fn resolves_directory_symlink_as_directory_removal_kind() {
        let root = tempfile::tempdir().unwrap();
        let target_dir = root.path().join("target-dir");
        fs::create_dir_all(&target_dir).unwrap();
        let metadata = fs::symlink_metadata(&target_dir).unwrap();

        assert_eq!(
            resolve_removal_kind(&target_dir, &metadata),
            RemovalKind::Directory
        );
    }

    #[test]
    fn removes_directory_symlink_without_removing_source_directory() {
        let root = tempfile::tempdir().unwrap();
        let source_dir = root.path().join("source");
        let link_dir = root.path().join("link");
        fs::create_dir_all(&source_dir).unwrap();
        fs::write(source_dir.join("SKILL.md"), "# demo").unwrap();

        #[cfg(target_os = "windows")]
        let create_result = std::os::windows::fs::symlink_dir(&source_dir, &link_dir);
        #[cfg(not(target_os = "windows"))]
        let create_result = std::os::unix::fs::symlink(&source_dir, &link_dir);

        if let Err(error) = create_result {
            #[cfg(target_os = "windows")]
            if error.kind() == ErrorKind::PermissionDenied {
                return;
            }
            panic!("failed to create symlink for test: {}", error);
        }

        remove_path_if_present(&link_dir).unwrap();

        assert!(!link_dir.exists());
        assert!(source_dir.exists());
        assert!(source_dir.join("SKILL.md").exists());
    }
}
