use anyhow::{Context, Result};
use std::{
    fs,
    path::{Component, Path, PathBuf},
};
use uuid::Uuid;

use crate::domain::types::{AgentGlobalScanRequest, AgentGlobalScanResult, AgentGlobalSkillEntry};

fn has_skill_marker(path: &Path) -> bool {
    path.join("SKILL.md").is_file()
}

fn relation_for_entry(path: &Path) -> Result<String> {
    let metadata = fs::symlink_metadata(path)
        .with_context(|| format!("failed to read metadata for {}", path.display()))?;
    if metadata.file_type().is_symlink() {
        let resolved = fs::canonicalize(path);
        return Ok(if resolved.is_ok() { "linked" } else { "broken" }.to_string());
    }

    Ok("directory".to_string())
}

fn entry_is_skill(path: &Path) -> bool {
    if has_skill_marker(path) {
        return true;
    }

    match fs::canonicalize(path) {
        Ok(resolved) => has_skill_marker(&resolved),
        Err(_) => match fs::symlink_metadata(path) {
            Ok(metadata) => metadata.file_type().is_symlink(),
            Err(_) => false,
        },
    }
}

fn resolve_relative_root_path(relative_path: &str) -> Result<PathBuf> {
    let normalized = relative_path.replace('\\', "/");
    let relative = PathBuf::from(&normalized);
    if relative.as_os_str().is_empty() {
        anyhow::bail!("relative path must not be empty");
    }
    if relative.is_absolute() {
        anyhow::bail!("relative path must stay under the user home directory");
    }
    for component in relative.components() {
        match component {
            Component::Normal(_) => {}
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                anyhow::bail!("relative path contains unsupported component");
            }
        }
    }

    let home_dir = dirs::home_dir().context("failed to resolve home directory")?;
    Ok(home_dir.join(relative))
}

pub fn scan_agent_global_skills(request: &AgentGlobalScanRequest) -> Result<AgentGlobalScanResult> {
    let root_path = resolve_relative_root_path(&request.relative_path)?;

    let mut entries = Vec::new();
    if root_path.exists() {
        for entry in fs::read_dir(&root_path)
            .with_context(|| format!("failed to read {}", root_path.display()))?
        {
            let entry = entry?;
            let path = entry.path();
            let metadata = fs::symlink_metadata(&path)
                .with_context(|| format!("failed to read metadata for {}", path.display()))?;
            if !metadata.file_type().is_symlink() && !metadata.is_dir() {
                continue;
            }
            if !entry_is_skill(&path) {
                continue;
            }

            entries.push(AgentGlobalSkillEntry {
                id: Uuid::new_v4().to_string(),
                name: entry.file_name().to_string_lossy().to_string(),
                path: PathBuf::from(&path).to_string_lossy().to_string(),
                relationship: relation_for_entry(&path)?,
            });
        }
        entries.sort_by(|left, right| left.name.cmp(&right.name));
    }

    Ok(AgentGlobalScanResult {
        agent_id: request.agent_id.clone(),
        agent_label: request.agent_label.clone(),
        root_path: root_path.to_string_lossy().to_string(),
        entries,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn classifies_directory_link_and_broken_link() {
        let dir = tempdir().unwrap();
        let skills_root = dir.path().join("skills");
        fs::create_dir_all(&skills_root).unwrap();

        let real_skill = skills_root.join("real");
        fs::create_dir_all(&real_skill).unwrap();
        fs::write(real_skill.join("SKILL.md"), "# real").unwrap();

        let linked_skill = skills_root.join("linked");
        #[cfg(target_os = "windows")]
        std::os::windows::fs::symlink_dir(&real_skill, &linked_skill).unwrap();
        #[cfg(not(target_os = "windows"))]
        std::os::unix::fs::symlink(&real_skill, &linked_skill).unwrap();

        let broken_target = skills_root.join("missing");
        let broken_link = skills_root.join("broken");
        #[cfg(target_os = "windows")]
        std::os::windows::fs::symlink_dir(&broken_target, &broken_link).unwrap();
        #[cfg(not(target_os = "windows"))]
        std::os::unix::fs::symlink(&broken_target, &broken_link).unwrap();

        assert_eq!(relation_for_entry(&real_skill).unwrap(), "directory");
        assert_eq!(relation_for_entry(&linked_skill).unwrap(), "linked");
        assert_eq!(relation_for_entry(&broken_link).unwrap(), "broken");
        assert!(entry_is_skill(&real_skill));
        assert!(entry_is_skill(&linked_skill));
        assert!(entry_is_skill(&broken_link));
    }

    #[test]
    fn rejects_absolute_or_parent_relative_paths() {
        assert!(resolve_relative_root_path("/tmp/skills").is_err());
        assert!(resolve_relative_root_path("../skills").is_err());
        assert!(resolve_relative_root_path("..\\skills").is_err());
    }
}
