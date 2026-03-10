use anyhow::{Context, Result};
use std::{
    fs,
    path::{Path, PathBuf},
};
use uuid::Uuid;

use crate::domain::types::{AgentGlobalScanResult, AgentGlobalSkillEntry};

struct AgentGlobalTarget {
    id: &'static str,
    label: &'static str,
    relative_path: &'static str,
}

const AGENT_GLOBAL_TARGETS: &[AgentGlobalTarget] = &[
    AgentGlobalTarget {
        id: "antigravity",
        label: "Antigravity",
        relative_path: ".agent/skills",
    },
    AgentGlobalTarget {
        id: "claude-code",
        label: "Claude Code",
        relative_path: ".claude/skills",
    },
    AgentGlobalTarget {
        id: "codebuddy",
        label: "CodeBuddy",
        relative_path: ".codebuddy/skills",
    },
    AgentGlobalTarget {
        id: "codex",
        label: "Codex",
        relative_path: ".agents/skills",
    },
    AgentGlobalTarget {
        id: "cursor",
        label: "Cursor",
        relative_path: ".agents/skills",
    },
    AgentGlobalTarget {
        id: "kiro",
        label: "Kiro",
        relative_path: ".kiro/skills",
    },
    AgentGlobalTarget {
        id: "openclaw",
        label: "OpenClaw",
        relative_path: "skills",
    },
    AgentGlobalTarget {
        id: "opencode",
        label: "OpenCode",
        relative_path: ".agents/skills",
    },
    AgentGlobalTarget {
        id: "qoder",
        label: "Qoder",
        relative_path: ".qoder/skills",
    },
    AgentGlobalTarget {
        id: "trae",
        label: "Trae",
        relative_path: ".trae/skills",
    },
    AgentGlobalTarget {
        id: "vscode",
        label: "VSCode",
        relative_path: ".agents/skills",
    },
    AgentGlobalTarget {
        id: "windsurf",
        label: "Windsurf",
        relative_path: ".windsurf/skills",
    },
];

fn resolve_target(agent_id: &str) -> Option<&'static AgentGlobalTarget> {
    AGENT_GLOBAL_TARGETS.iter().find(|target| target.id == agent_id)
}

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

pub fn scan_agent_global_skills(agent_id: &str) -> Result<AgentGlobalScanResult> {
    let target = resolve_target(agent_id)
        .with_context(|| format!("unsupported agent global scan target {}", agent_id))?;
    let home_dir = dirs::home_dir().context("failed to resolve home directory")?;
    let root_path = home_dir.join(target.relative_path);

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
        agent_id: target.id.to_string(),
        agent_label: target.label.to_string(),
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
}
