use anyhow::{anyhow, Result};
use std::path::Path;

use crate::{
    domain::{
        app_state::AppState,
        types::{
            InjectTemplateItemResult, InjectTemplateRequest, InjectTemplateResult,
            SaveTemplateRequest, TemplateRecord,
        },
    },
    repositories::templates as templates_repository,
    services::project_distribution::{
        self, ProjectDistributionRequest, ProjectDistributionSelection,
    },
};

fn validate_template(request: &SaveTemplateRequest) -> Result<()> {
    if request.name.trim().is_empty() {
        return Err(anyhow!("template name is required"));
    }

    for item in &request.items {
        if item.skill_ref_type.trim().is_empty() {
            return Err(anyhow!("template item skill_ref_type is required"));
        }
        if item.skill_ref.trim().is_empty() {
            return Err(anyhow!("template item skill_ref is required"));
        }
    }

    Ok(())
}

fn build_result_item(
    skill_id: &str,
    skill_name: &str,
    target_path: &Path,
    reason: Option<String>,
) -> InjectTemplateItemResult {
    InjectTemplateItemResult {
        skill_id: skill_id.to_string(),
        skill_name: skill_name.to_string(),
        target_path: target_path.to_string_lossy().to_string(),
        reason,
    }
}

pub fn list_templates(path: &Path) -> Result<Vec<TemplateRecord>> {
    templates_repository::list_templates(path)
}

pub fn get_template(path: &Path, template_id: &str) -> Result<Option<TemplateRecord>> {
    templates_repository::get_template(path, template_id)
}

pub fn save_template(path: &Path, request: &SaveTemplateRequest) -> Result<TemplateRecord> {
    validate_template(request)?;
    templates_repository::save_template(path, request)
}

pub fn delete_template(path: &Path, template_id: &str) -> Result<()> {
    templates_repository::delete_template(path, template_id)
}

pub fn inject_template(state: &AppState, request: &InjectTemplateRequest) -> Result<InjectTemplateResult> {
    if request.template_id.trim().is_empty() {
        return Err(anyhow!("template_id is required"));
    }

    let template = templates_repository::get_template(&state.paths.db_file, &request.template_id)?
        .ok_or_else(|| anyhow!("template {} does not exist", request.template_id))?;
    if template.items.is_empty() {
        return Err(anyhow!("template must contain at least one skill before injection"));
    }

    let distribution_request = ProjectDistributionRequest {
        target_scope: "project".into(),
        project_root: request.project_root.clone(),
        target_type: request.target_type.clone(),
        target_agent_id: request.target_agent_id.clone(),
        custom_relative_path: request.custom_relative_path.clone(),
        install_mode: request.install_mode.clone(),
    };
    let target_root = project_distribution::resolve_project_target_root(state, &distribution_request)?;

    let mut failed = Vec::new();
    let mut distribution_items = Vec::new();

    for item in &template.items {
        if item.skill_ref_type != "repository_skill" {
            failed.push(build_result_item(
                &item.skill_ref,
                item.display_name.as_deref().unwrap_or(&item.skill_ref),
                &target_root.join(&item.skill_ref),
                Some(format!("unsupported template item type {}", item.skill_ref_type)),
            ));
            continue;
        }

        distribution_items.push(ProjectDistributionSelection {
            skill_id: item.skill_ref.clone(),
            skill_name: item.display_name.clone().unwrap_or_else(|| item.skill_ref.clone()),
        });
    }

    if distribution_items.is_empty() {
        return Err(anyhow!("template must contain at least one repository skill before injection"));
    }

    let distribution_result = project_distribution::distribute_repository_skills_to_project(
        state,
        &distribution_items,
        &distribution_request,
    )?;

    Ok(InjectTemplateResult {
        installed: distribution_result
            .installed
            .into_iter()
            .map(|item| build_result_item(&item.skill_id, &item.skill_name, Path::new(&item.target_path), item.reason))
            .collect(),
        skipped: distribution_result
            .skipped
            .into_iter()
            .map(|item| build_result_item(&item.skill_id, &item.skill_name, Path::new(&item.target_path), item.reason))
            .collect(),
        failed: failed
            .into_iter()
            .chain(distribution_result.failed.into_iter().map(|item| {
                build_result_item(&item.skill_id, &item.skill_name, Path::new(&item.target_path), item.reason)
            }))
            .collect(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        domain::{
            agent_registry::AgentRegistry,
            app_state::{AppPaths, AppState},
            types::{AppSettings, CustomSkillsTarget, DistributionRequest, InstallSkillRequest, SaveTemplateItemRequest},
        },
        repositories::{db::run_migrations, settings as settings_repository, skills as skills_repository},
    };
    use std::{fs, sync::Arc};
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
                source_type: "github-resolved-skill".into(),
                source_url: "https://example.com/demo".into(),
                repo_url: Some("https://example.com/demo".into()),
                download_url: None,
                package_ref: Some(format!("example/demo@{}", slug)),
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

    fn seed_template(state: &AppState, skill_ids: &[(&str, &str)]) -> String {
        templates_repository::save_template(
            &state.paths.db_file,
            &SaveTemplateRequest {
                id: None,
                name: "Vue Template".into(),
                description: Some("starter".into()),
                tags: vec!["vue".into()],
                items: skill_ids
                    .iter()
                    .enumerate()
                    .map(|(index, (skill_id, display_name))| SaveTemplateItemRequest {
                        skill_ref_type: "repository_skill".into(),
                        skill_ref: (*skill_id).into(),
                        display_name: Some((*display_name).into()),
                        order_index: Some(index as u32),
                    })
                    .collect(),
            },
        )
        .unwrap()
        .id
    }

    #[test]
    fn injects_template_into_tag_path_with_copy_mode() {
        let dir = tempdir().unwrap();
        let state = test_state(dir.path());
        run_migrations(&state.paths.db_file).unwrap();
        let skill_id = seed_skill(&state, "demo-skill", "Demo Skill");
        let template_id = seed_template(&state, &[(&skill_id, "Demo Skill")]);
        let project_root = dir.path().join("workspace");
        fs::create_dir_all(&project_root).unwrap();

        let result = inject_template(
            &state,
            &InjectTemplateRequest {
                template_id,
                project_root: project_root.to_string_lossy().to_string(),
                target_type: "tag".into(),
                target_agent_id: Some("claude-code".into()),
                custom_relative_path: None,
                install_mode: "copy".into(),
            },
        )
        .unwrap();

        assert_eq!(result.installed.len(), 1);
        assert!(project_root
            .join(".claude/skills/demo-skill/SKILL.md")
            .exists());
    }

    #[test]
    fn skips_existing_targets_and_marks_missing_skills_failed() {
        let dir = tempdir().unwrap();
        let state = test_state(dir.path());
        run_migrations(&state.paths.db_file).unwrap();
        let skill_id = seed_skill(&state, "demo-skill", "Demo Skill");
        let template_id =
            seed_template(&state, &[(&skill_id, "Demo Skill"), ("missing-skill", "Missing Skill")]);
        let project_root = dir.path().join("workspace");
        fs::create_dir_all(project_root.join(".claude/skills/demo-skill")).unwrap();

        let result = inject_template(
            &state,
            &InjectTemplateRequest {
                template_id,
                project_root: project_root.to_string_lossy().to_string(),
                target_type: "tag".into(),
                target_agent_id: Some("claude-code".into()),
                custom_relative_path: None,
                install_mode: "copy".into(),
            },
        )
        .unwrap();

        assert_eq!(result.installed.len(), 0);
        assert_eq!(result.skipped.len(), 1);
        assert_eq!(result.failed.len(), 1);
    }

    #[test]
    fn uses_custom_relative_path_and_custom_target_id() {
        let dir = tempdir().unwrap();
        let state = test_state(dir.path());
        run_migrations(&state.paths.db_file).unwrap();
        let skill_id = seed_skill(&state, "demo-skill", "Demo Skill");
        let template_id = seed_template(&state, &[(&skill_id, "Demo Skill")]);
        let project_root = dir.path().join("workspace");
        fs::create_dir_all(&project_root).unwrap();

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

        let tag_result = inject_template(
            &state,
            &InjectTemplateRequest {
                template_id: template_id.clone(),
                project_root: project_root.to_string_lossy().to_string(),
                target_type: "tag".into(),
                target_agent_id: Some("custom-demo".into()),
                custom_relative_path: None,
                install_mode: "copy".into(),
            },
        )
        .unwrap();
        assert_eq!(tag_result.installed.len(), 1);
        assert!(project_root.join(".demo/skills/demo-skill/SKILL.md").exists());

        let project_root_custom = dir.path().join("workspace-custom");
        fs::create_dir_all(&project_root_custom).unwrap();
        let custom_result = inject_template(
            &state,
            &InjectTemplateRequest {
                template_id,
                project_root: project_root_custom.to_string_lossy().to_string(),
                target_type: "custom".into(),
                target_agent_id: None,
                custom_relative_path: Some(".company/skills/vue".into()),
                install_mode: "copy".into(),
            },
        )
        .unwrap();
        assert_eq!(custom_result.installed.len(), 1);
        assert!(project_root_custom
            .join(".company/skills/vue/demo-skill/SKILL.md")
            .exists());
    }

    #[test]
    fn rejects_template_injection_when_template_has_no_skills() {
        let dir = tempdir().unwrap();
        let state = test_state(dir.path());
        run_migrations(&state.paths.db_file).unwrap();
        let template_id = seed_template(&state, &[]);
        let project_root = dir.path().join("workspace");
        fs::create_dir_all(&project_root).unwrap();

        let error = inject_template(
            &state,
            &InjectTemplateRequest {
                template_id,
                project_root: project_root.to_string_lossy().to_string(),
                target_type: "tag".into(),
                target_agent_id: Some("claude-code".into()),
                custom_relative_path: None,
                install_mode: "copy".into(),
            },
        )
        .unwrap_err();

        assert!(error.to_string().contains("at least one skill"));
    }

    #[test]
    fn rejects_invalid_custom_relative_path() {
        let dir = tempdir().unwrap();
        let state = test_state(dir.path());
        run_migrations(&state.paths.db_file).unwrap();
        let skill_id = seed_skill(&state, "demo-skill", "Demo Skill");
        let template_id = seed_template(&state, &[(&skill_id, "Demo Skill")]);
        let project_root = dir.path().join("workspace");
        fs::create_dir_all(&project_root).unwrap();

        let error = inject_template(
            &state,
            &InjectTemplateRequest {
                template_id,
                project_root: project_root.to_string_lossy().to_string(),
                target_type: "custom".into(),
                target_agent_id: None,
                custom_relative_path: Some("../escape".into()),
                install_mode: "copy".into(),
            },
        )
        .unwrap_err();

        assert!(error.to_string().contains("cannot contain parent directory"));
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn reports_symlink_permission_failures_without_fallback() {
        let dir = tempdir().unwrap();
        let state = test_state(dir.path());
        run_migrations(&state.paths.db_file).unwrap();
        let skill_id = seed_skill(&state, "demo-skill", "Demo Skill");
        let template_id = seed_template(&state, &[(&skill_id, "Demo Skill")]);
        let project_root = dir.path().join("workspace");
        fs::create_dir_all(&project_root).unwrap();

        let result = inject_template(
            &state,
            &InjectTemplateRequest {
                template_id,
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
}
