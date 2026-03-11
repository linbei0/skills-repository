use anyhow::{anyhow, Result};
use serde_json::json;
use std::{fs, path::PathBuf};

use crate::{
    domain::{
        app_state::AppState,
        types::{BatchDistributeItemResult, BatchDistributeResult},
    },
    repositories::{
        distributions as distributions_repository, settings as settings_repository,
        skills as skills_repository,
    },
    services::distribution,
};

pub(crate) const BUILTIN_PROJECT_TARGETS: &[(&str, &str)] = &[
    ("universal", ".agents/skills"),
    ("antigravity", ".agent/skills"),
    ("augment", ".augment/skills"),
    ("claude-code", ".claude/skills"),
    ("openclaw", "skills"),
    ("codebuddy", ".codebuddy/skills"),
    ("command-code", ".commandcode/skills"),
    ("continue", ".continue/skills"),
    ("cortex-code", ".cortex/skills"),
    ("crush", ".crush/skills"),
    ("droid", ".factory/skills"),
    ("goose", ".goose/skills"),
    ("junie", ".junie/skills"),
    ("iflow-cli", ".iflow/skills"),
    ("kilo-code", ".kilocode/skills"),
    ("kiro-cli", ".kiro/skills"),
    ("kode", ".kode/skills"),
    ("mcpjam", ".mcpjam/skills"),
    ("mistral-vibe", ".vibe/skills"),
    ("mux", ".mux/skills"),
    ("openhands", ".openhands/skills"),
    ("pi", ".pi/skills"),
    ("qoder", ".qoder/skills"),
    ("qwen-code", ".qwen/skills"),
    ("roo-code", ".roo/skills"),
    ("trae", ".trae/skills"),
    ("trae-cn", ".trae/skills"),
    ("windsurf", ".windsurf/skills"),
    ("zencoder", ".zencoder/skills"),
    ("neovate", ".neovate/skills"),
    ("pochi", ".pochi/skills"),
    ("adal", ".adal/skills"),
];

#[derive(Debug, Clone)]
pub(crate) struct ProjectDistributionSelection {
    pub skill_id: String,
    pub skill_name: String,
}

#[derive(Debug, Clone)]
pub(crate) struct ProjectDistributionRequest {
    pub target_scope: String,
    pub project_root: String,
    pub target_type: String,
    pub target_agent_id: Option<String>,
    pub custom_relative_path: Option<String>,
    pub install_mode: String,
}

fn normalize_relative_path(relative_path: &str) -> String {
    relative_path
        .replace('\\', "/")
        .trim()
        .trim_matches('/')
        .to_string()
}

fn validate_custom_relative_path(relative_path: &str) -> Result<String> {
    let normalized = normalize_relative_path(relative_path);
    if normalized.is_empty() {
        return Err(anyhow!("custom relative path is required"));
    }
    if normalized.starts_with('/') || normalized.starts_with('\\') {
        return Err(anyhow!("custom relative path must be relative"));
    }
    if normalized.split('/').any(|segment| segment == "..") {
        return Err(anyhow!(
            "custom relative path cannot contain parent directory segments"
        ));
    }
    Ok(normalized)
}

fn resolve_tag_relative_path(state: &AppState, target_agent_id: &str) -> Result<String> {
    if let Some((_, relative_path)) = BUILTIN_PROJECT_TARGETS
        .iter()
        .find(|(id, _)| *id == target_agent_id)
    {
        return Ok((*relative_path).to_string());
    }

    let settings = settings_repository::load_settings(&state.paths.db_file)?
        .unwrap_or_else(|| settings_repository::default_settings("en-US".into()));

    settings
        .custom_skills_targets
        .into_iter()
        .find(|target| target.id == target_agent_id)
        .map(|target| target.relative_path)
        .ok_or_else(|| anyhow!("unknown target agent id {}", target_agent_id))
}

fn ensure_project_root(project_root: &str) -> Result<PathBuf> {
    let path = PathBuf::from(project_root);
    if !path.exists() {
        return Err(anyhow!("project root does not exist: {}", path.display()));
    }
    if !path.is_dir() {
        return Err(anyhow!("project root is not a directory: {}", path.display()));
    }
    Ok(path)
}

fn ensure_home_root() -> Result<PathBuf> {
    dirs::home_dir().ok_or_else(|| anyhow!("failed to resolve home directory"))
}

fn validate_install_mode(install_mode: &str) -> Result<()> {
    if install_mode != "symlink" && install_mode != "copy" {
        return Err(anyhow!("unsupported install mode {}", install_mode));
    }
    Ok(())
}

pub(crate) fn resolve_project_target_root(
    state: &AppState,
    request: &ProjectDistributionRequest,
) -> Result<PathBuf> {
    validate_install_mode(&request.install_mode)?;
    let relative_target = match request.target_type.as_str() {
        "tag" => {
            let target_agent_id = request
                .target_agent_id
                .as_deref()
                .ok_or_else(|| anyhow!("target_agent_id is required when target_type=tag"))?;
            resolve_tag_relative_path(state, target_agent_id)?
        }
        "custom" => validate_custom_relative_path(
            request
                .custom_relative_path
                .as_deref()
                .ok_or_else(|| {
                    anyhow!("custom_relative_path is required when target_type=custom")
                })?,
        )?,
        other => return Err(anyhow!("unsupported target_type {}", other)),
    };

    let target_root = match request.target_scope.as_str() {
        "project" => {
            if request.project_root.trim().is_empty() {
                return Err(anyhow!("project_root is required when target_scope=project"));
            }
            ensure_project_root(&request.project_root)?
        }
        "global" => ensure_home_root()?,
        other => return Err(anyhow!("unsupported target_scope {}", other)),
    };

    Ok(target_root.join(relative_target))
}

pub(crate) fn build_result_item(
    skill_id: &str,
    skill_name: &str,
    target_path: &std::path::Path,
    reason: Option<String>,
) -> BatchDistributeItemResult {
    BatchDistributeItemResult {
        skill_id: skill_id.to_string(),
        skill_name: skill_name.to_string(),
        target_path: target_path.to_string_lossy().to_string(),
        reason,
    }
}

fn save_distribution_operation_log(
    state: &AppState,
    status: &str,
    skill_id: &str,
    skill_name: &str,
    target_path: &std::path::Path,
    request: &ProjectDistributionRequest,
    reason: Option<&str>,
) -> Result<()> {
    skills_repository::save_operation_log(
        &state.paths.db_file,
        "project_distribute",
        "skill",
        Some(skill_id),
        status,
        reason.unwrap_or("repository skill distributed to project"),
        Some(json!({
            "skillName": skill_name,
            "targetScope": request.target_scope,
            "projectRoot": request.project_root,
            "targetType": request.target_type,
            "targetAgentId": request.target_agent_id,
            "customRelativePath": request.custom_relative_path,
            "installMode": request.install_mode,
            "targetPath": target_path.to_string_lossy(),
            "reason": reason,
        })),
    )?;
    Ok(())
}

pub(crate) fn distribute_repository_skills_to_project(
    state: &AppState,
    skills: &[ProjectDistributionSelection],
    request: &ProjectDistributionRequest,
) -> Result<BatchDistributeResult> {
    if skills.is_empty() {
        return Err(anyhow!("at least one repository skill is required for distribution"));
    }

    let target_root = resolve_project_target_root(state, request)?;
    fs::create_dir_all(&target_root)?;

    let mut installed = Vec::new();
    let mut skipped = Vec::new();
    let mut failed = Vec::new();

    for skill in skills {
        let source = match skills_repository::load_skill_source(&state.paths.db_file, &skill.skill_id)
        {
            Ok(source) => source,
            Err(error) => {
                let item = build_result_item(
                    &skill.skill_id,
                    &skill.skill_name,
                    &target_root.join(&skill.skill_name),
                    Some(format!("missing repository skill: {}", error)),
                );
                let _ = save_distribution_operation_log(
                    state,
                    "failed",
                    &skill.skill_id,
                    &skill.skill_name,
                    &PathBuf::from(&item.target_path),
                    request,
                    item.reason.as_deref(),
                );
                failed.push(item);
                continue;
            }
        };

        let source_path = PathBuf::from(&source.source_path);
        if !source_path.exists() {
            let target_path = target_root.join(&source.target_name);
            let item = build_result_item(
                &skill.skill_id,
                &skill.skill_name,
                &target_path,
                Some(format!("skill source path does not exist: {}", source_path.display())),
            );
            let _ = save_distribution_operation_log(
                state,
                "failed",
                &skill.skill_id,
                &skill.skill_name,
                &target_path,
                request,
                item.reason.as_deref(),
            );
            failed.push(item);
            continue;
        }

        let target_path = target_root.join(&source.target_name);
        if target_path.exists() || fs::symlink_metadata(&target_path).is_ok() {
            let item = build_result_item(
                &skill.skill_id,
                &skill.skill_name,
                &target_path,
                Some("target already exists".into()),
            );
            let _ = save_distribution_operation_log(
                state,
                "skipped",
                &skill.skill_id,
                &skill.skill_name,
                &target_path,
                request,
                item.reason.as_deref(),
            );
            skipped.push(item);
            continue;
        }

        let install_result: Result<()> = match request.install_mode.as_str() {
            "symlink" => distribution::create_directory_symlink(&source_path, &target_path),
            "copy" => distribution::copy_dir_all(&source_path, &target_path),
            _ => unreachable!(),
        };

        match install_result {
            Ok(()) => {
                distributions_repository::save_distribution(
                    &state.paths.db_file,
                    &skill.skill_id,
                    &request.target_scope,
                    request.target_agent_id.as_deref().unwrap_or("custom"),
                    &target_path.to_string_lossy(),
                    &request.install_mode,
                    "active",
                )?;

                let item = build_result_item(&skill.skill_id, &skill.skill_name, &target_path, None);
                let _ = save_distribution_operation_log(
                    state,
                    "success",
                    &skill.skill_id,
                    &skill.skill_name,
                    &target_path,
                    request,
                    None,
                );
                installed.push(item);
            }
            Err(error) => {
                let item = build_result_item(
                    &skill.skill_id,
                    &skill.skill_name,
                    &target_path,
                    Some(error.to_string()),
                );
                let _ = save_distribution_operation_log(
                    state,
                    "failed",
                    &skill.skill_id,
                    &skill.skill_name,
                    &target_path,
                    request,
                    item.reason.as_deref(),
                );
                failed.push(item);
            }
        }
    }

    Ok(BatchDistributeResult {
        installed,
        skipped,
        failed,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        domain::{
            agent_registry::AgentRegistry,
            app_state::{AppPaths, AppState},
            types::{AppSettings, CustomSkillsTarget, DistributionRequest, InstallSkillRequest},
        },
        repositories::{db::run_migrations, settings as settings_repository, skills as skills_repository},
    };
    use std::{fs, path::Path, sync::Arc};
    use tempfile::tempdir;

    fn test_state(root: &Path) -> AppState {
        let app_root = root.join("app-data");
        let db_dir = app_root.join("db");
        let temp_dir = app_root.join("tmp");
        let canonical_store_dir = app_root.join("skills");

        fs::create_dir_all(&db_dir).unwrap();
        fs::create_dir_all(&temp_dir).unwrap();
        fs::create_dir_all(&canonical_store_dir).unwrap();

        AppState {
            paths: AppPaths {
                db_file: db_dir.join("skills-manager.db"),
                temp_dir,
                canonical_store_dir,
            },
            agent_registry: Arc::new(AgentRegistry::new()),
        }
    }

    fn seed_skill(state: &AppState, slug: &str, name: &str) -> String {
        let source_dir = state.paths.canonical_store_dir.join(slug);
        fs::create_dir_all(&source_dir).unwrap();
        fs::write(source_dir.join("SKILL.md"), format!("# {}", name)).unwrap();

        skills_repository::save_installed_skill(
            &state.paths.db_file,
            &InstallSkillRequest {
                provider: "github".into(),
                market_skill_id: slug.into(),
                source_type: "market".into(),
                source_url: format!("https://example.com/{}", slug),
                repo_url: Some(format!("https://example.com/{}", slug)),
                download_url: None,
                package_ref: Some(format!("example/{}@skills/{}", slug, slug)),
                manifest_path: Some(format!("skills/{}/SKILL.md", slug)),
                skill_root: Some(format!("skills/{}", slug)),
                name: name.into(),
                slug: slug.into(),
                version: Some("main".into()),
                author: Some("tester".into()),
                requested_targets: Vec::<DistributionRequest>::new(),
            },
            &source_dir.to_string_lossy(),
            "safe",
            false,
        )
        .unwrap()
    }

    #[test]
    fn distributes_multiple_skills_into_tag_path() {
        let dir = tempdir().unwrap();
        let state = test_state(dir.path());
        run_migrations(&state.paths.db_file).unwrap();
        let skill_a = seed_skill(&state, "demo-a", "Demo A");
        let skill_b = seed_skill(&state, "demo-b", "Demo B");
        let project_root = dir.path().join("workspace");
        fs::create_dir_all(&project_root).unwrap();

        let result = distribute_repository_skills_to_project(
            &state,
            &[
                ProjectDistributionSelection { skill_id: skill_a.clone(), skill_name: "Demo A".into() },
                ProjectDistributionSelection { skill_id: skill_b.clone(), skill_name: "Demo B".into() },
            ],
            &ProjectDistributionRequest {
                target_scope: "project".into(),
                project_root: project_root.to_string_lossy().to_string(),
                target_type: "tag".into(),
                target_agent_id: Some("claude-code".into()),
                custom_relative_path: None,
                install_mode: "copy".into(),
            },
        )
        .unwrap();

        assert_eq!(result.installed.len(), 2);
        assert!(project_root.join(".claude/skills/demo-a/SKILL.md").exists());
        assert!(project_root.join(".claude/skills/demo-b/SKILL.md").exists());
    }

    #[test]
    fn distributes_skills_into_custom_relative_path() {
        let dir = tempdir().unwrap();
        let state = test_state(dir.path());
        run_migrations(&state.paths.db_file).unwrap();
        let skill_id = seed_skill(&state, "demo-skill", "Demo Skill");
        let project_root = dir.path().join("workspace");
        fs::create_dir_all(&project_root).unwrap();

        let result = distribute_repository_skills_to_project(
            &state,
            &[ProjectDistributionSelection { skill_id, skill_name: "Demo Skill".into() }],
            &ProjectDistributionRequest {
                target_scope: "project".into(),
                project_root: project_root.to_string_lossy().to_string(),
                target_type: "custom".into(),
                target_agent_id: None,
                custom_relative_path: Some(".company/skills/vue".into()),
                install_mode: "copy".into(),
            },
        )
        .unwrap();

        assert_eq!(result.installed.len(), 1);
        assert!(project_root
            .join(".company/skills/vue/demo-skill/SKILL.md")
            .exists());
    }

    #[test]
    fn skips_existing_targets() {
        let dir = tempdir().unwrap();
        let state = test_state(dir.path());
        run_migrations(&state.paths.db_file).unwrap();
        let skill_id = seed_skill(&state, "demo-skill", "Demo Skill");
        let project_root = dir.path().join("workspace");
        fs::create_dir_all(project_root.join(".claude/skills/demo-skill")).unwrap();

        let result = distribute_repository_skills_to_project(
            &state,
            &[ProjectDistributionSelection { skill_id, skill_name: "Demo Skill".into() }],
            &ProjectDistributionRequest {
                target_scope: "project".into(),
                project_root: project_root.to_string_lossy().to_string(),
                target_type: "tag".into(),
                target_agent_id: Some("claude-code".into()),
                custom_relative_path: None,
                install_mode: "copy".into(),
            },
        )
        .unwrap();

        assert_eq!(result.skipped.len(), 1);
        assert_eq!(result.skipped[0].reason.as_deref(), Some("target already exists"));
    }

    #[test]
    fn marks_missing_repository_skill_as_failed_and_continues() {
        let dir = tempdir().unwrap();
        let state = test_state(dir.path());
        run_migrations(&state.paths.db_file).unwrap();
        let existing_skill = seed_skill(&state, "demo-skill", "Demo Skill");
        let project_root = dir.path().join("workspace");
        fs::create_dir_all(&project_root).unwrap();

        let result = distribute_repository_skills_to_project(
            &state,
            &[
                ProjectDistributionSelection { skill_id: existing_skill, skill_name: "Demo Skill".into() },
                ProjectDistributionSelection { skill_id: "missing-id".into(), skill_name: "Missing Skill".into() },
            ],
            &ProjectDistributionRequest {
                target_scope: "project".into(),
                project_root: project_root.to_string_lossy().to_string(),
                target_type: "tag".into(),
                target_agent_id: Some("claude-code".into()),
                custom_relative_path: None,
                install_mode: "copy".into(),
            },
        )
        .unwrap();

        assert_eq!(result.installed.len(), 1);
        assert_eq!(result.failed.len(), 1);
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn reports_windows_symlink_failures_without_fallback() {
        let dir = tempdir().unwrap();
        let state = test_state(dir.path());
        run_migrations(&state.paths.db_file).unwrap();
        let skill_id = seed_skill(&state, "demo-skill", "Demo Skill");
        let project_root = dir.path().join("workspace");
        fs::create_dir_all(&project_root).unwrap();

        let result = distribute_repository_skills_to_project(
            &state,
            &[ProjectDistributionSelection { skill_id, skill_name: "Demo Skill".into() }],
            &ProjectDistributionRequest {
                target_scope: "project".into(),
                project_root: project_root.to_string_lossy().to_string(),
                target_type: "tag".into(),
                target_agent_id: Some("claude-code".into()),
                custom_relative_path: None,
                install_mode: "symlink".into(),
            },
        )
        .unwrap();

        if result.failed.is_empty() {
            assert!(project_root
                .join(".claude/skills/demo-skill")
                .symlink_metadata()
                .unwrap()
                .file_type()
                .is_symlink());
        } else {
            assert!(result.failed[0]
                .reason
                .as_deref()
                .unwrap_or_default()
                .contains("Windows symlink permission denied"));
        }
    }

    #[test]
    fn resolves_custom_target_from_settings() {
        let dir = tempdir().unwrap();
        let state = test_state(dir.path());
        run_migrations(&state.paths.db_file).unwrap();
        settings_repository::save_settings(
            &state.paths.db_file,
            &AppSettings {
                language: "zh-CN".into(),
                theme_mode: "system".into(),
                visible_skills_target_ids: vec!["custom-demo".into()],
                custom_skills_targets: vec![CustomSkillsTarget {
                    id: "custom-demo".into(),
                    label: "Demo IDE".into(),
                    relative_path: ".demo/skills".into(),
                }],
            },
        )
        .unwrap();
        let project_root = dir.path().join("workspace");
        fs::create_dir_all(&project_root).unwrap();

        let target_root = resolve_project_target_root(
            &state,
            &ProjectDistributionRequest {
                target_scope: "project".into(),
                project_root: project_root.to_string_lossy().to_string(),
                target_type: "tag".into(),
                target_agent_id: Some("custom-demo".into()),
                custom_relative_path: None,
                install_mode: "copy".into(),
            },
        )
        .unwrap();

        assert!(target_root.ends_with(".demo/skills"));
    }

    #[test]
    fn resolves_global_target_root_under_home_directory() {
        let dir = tempdir().unwrap();
        let state = test_state(dir.path());
        run_migrations(&state.paths.db_file).unwrap();

        let target_root = resolve_project_target_root(
            &state,
            &ProjectDistributionRequest {
                target_scope: "global".into(),
                project_root: String::new(),
                target_type: "tag".into(),
                target_agent_id: Some("claude-code".into()),
                custom_relative_path: None,
                install_mode: "copy".into(),
            },
        )
        .unwrap();

        assert!(target_root.ends_with(".claude/skills"));
    }
}
