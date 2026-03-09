use anyhow::{anyhow, Result};
use std::{fs, path::PathBuf};

use crate::{
    domain::{
        agent_registry::AgentRegistry,
        app_state::AppPaths,
        types::{
            DistributionRequest, InstallSkillRequest, TemplateInjectionItemResult,
            TemplateInjectionRequest, TemplateInjectionResult, TemplateItem,
        },
    },
    repositories::{skills as skills_repository, templates as templates_repository},
};

use super::{distribution, install};

fn normalize_slug(value: &str) -> String {
    let normalized = value
        .trim()
        .to_ascii_lowercase()
        .chars()
        .map(|character| match character {
            'a'..='z' | '0'..='9' => character,
            _ => '-',
        })
        .collect::<String>();

    normalized
        .split('-')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

fn derive_template_item_slug(item: &TemplateItem) -> String {
    let candidate = item.display_name.clone().unwrap_or_else(|| {
        item.skill_ref
            .trim_end_matches('/')
            .rsplit('/')
            .next()
            .unwrap_or("template-skill")
            .trim_end_matches(".zip")
            .to_string()
    });

    let slug = normalize_slug(&candidate);
    if slug.is_empty() {
        format!("template-item-{}", item.order_index)
    } else {
        slug
    }
}

fn resolve_skill_for_template_item(
    paths: &AppPaths,
    task_id: &str,
    item: &TemplateItem,
) -> Result<String> {
    match item.skill_ref_type.as_str() {
        "skill_id" => Ok(item.skill_ref.clone()),
        "source_url" => {
            let slug = derive_template_item_slug(item);
            if let Some(existing_skill_id) =
                skills_repository::find_installed_skill_id(&paths.db_file, &slug, &item.skill_ref)?
            {
                return Ok(existing_skill_id);
            }

            let install_result = install::install_skill(
                paths,
                task_id,
                &InstallSkillRequest {
                    provider: "template".into(),
                    market_skill_id: item.id.clone(),
                    source_url: item.skill_ref.clone(),
                    download_url: None,
                    name: item.display_name.clone().unwrap_or_else(|| slug.clone()),
                    slug,
                    version: None,
                    author: None,
                    requested_targets: Vec::new(),
                },
            )?;

            if install_result.blocked {
                return Err(anyhow!(
                    "template item installation was blocked by security scan"
                ));
            }

            Ok(install_result.skill_id)
        }
        "market_ref" => Err(anyhow!(
            "template item type market_ref is not connected in this build"
        )),
        other => Err(anyhow!("unsupported template item type {}", other)),
    }
}

fn build_injection_result(
    request: &TemplateInjectionRequest,
    status: &str,
    results: Vec<TemplateInjectionItemResult>,
) -> TemplateInjectionResult {
    let installed_count = results
        .iter()
        .filter(|item| item.status == "installed")
        .count() as u32;
    let skipped_count = results
        .iter()
        .filter(|item| item.status == "skipped")
        .count() as u32;
    let failed_count = results
        .iter()
        .filter(|item| item.status == "failed")
        .count() as u32;

    TemplateInjectionResult {
        template_id: request.template_id.clone(),
        target_project_path: request.target_project_path.clone(),
        status: status.to_string(),
        installed_count,
        skipped_count,
        failed_count,
        results,
    }
}

pub fn inject_template(
    paths: &AppPaths,
    registry: &AgentRegistry,
    task_id: &str,
    request: &TemplateInjectionRequest,
) -> Result<TemplateInjectionResult> {
    let project_root = PathBuf::from(&request.target_project_path);
    if !project_root.exists() || !project_root.is_dir() {
        return Err(anyhow!(
            "target project path does not exist or is not a directory: {}",
            project_root.display()
        ));
    }
    let canonical_project_root = fs::canonicalize(&project_root)?;

    let template = templates_repository::get_template(&paths.db_file, &request.template_id)?
        .ok_or_else(|| anyhow!("template {} does not exist", request.template_id))?;

    let mut item_results = Vec::with_capacity(template.items.len());

    for item in &template.items {
        let skill_id = match resolve_skill_for_template_item(paths, task_id, item) {
            Ok(skill_id) => skill_id,
            Err(error) => {
                item_results.push(TemplateInjectionItemResult {
                    skill_ref: item.skill_ref.clone(),
                    status: "failed".into(),
                    message: Some(error.to_string()),
                });
                continue;
            }
        };

        if template.target_agents.is_empty() {
            item_results.push(TemplateInjectionItemResult {
                skill_ref: item.skill_ref.clone(),
                status: "failed".into(),
                message: Some("template has no target agents configured".into()),
            });
            continue;
        }

        let mut distribution_failures = Vec::new();
        let mut skipped = false;

        for target_agent in &template.target_agents {
            let request_payload = DistributionRequest {
                skill_id: skill_id.clone(),
                target_kind: "project".into(),
                target_agent: target_agent.clone(),
                install_mode: "copy".into(),
                project_root: Some(canonical_project_root.to_string_lossy().to_string()),
                custom_target_path: None,
            };

            if request.overwrite_strategy == "skip_existing" {
                let target_path = distribution::resolve_distribution_target_path(
                    registry,
                    &paths.db_file,
                    &request_payload,
                )?;
                if target_path.exists() {
                    skipped = true;
                    break;
                }
            } else if request.overwrite_strategy != "overwrite" {
                return Err(anyhow!(
                    "unsupported overwrite strategy {}",
                    request.overwrite_strategy
                ));
            }

            if let Err(error) =
                distribution::distribute_skill(registry, &paths.db_file, &request_payload)
            {
                distribution_failures.push(format!("{}: {}", target_agent, error));
            }
        }

        if skipped {
            item_results.push(TemplateInjectionItemResult {
                skill_ref: item.skill_ref.clone(),
                status: "skipped".into(),
                message: Some(
                    "target skill already exists in project and overwrite is disabled".into(),
                ),
            });
        } else if distribution_failures.is_empty() {
            item_results.push(TemplateInjectionItemResult {
                skill_ref: item.skill_ref.clone(),
                status: "installed".into(),
                message: None,
            });
        } else {
            item_results.push(TemplateInjectionItemResult {
                skill_ref: item.skill_ref.clone(),
                status: "failed".into(),
                message: Some(distribution_failures.join("; ")),
            });
        }
    }

    let failed_count = item_results
        .iter()
        .filter(|item| item.status == "failed")
        .count();

    if failed_count == item_results.len() && !item_results.is_empty() {
        return Ok(build_injection_result(request, "failed", item_results));
    }

    if failed_count > 0 {
        return Ok(build_injection_result(request, "partial", item_results));
    }

    Ok(build_injection_result(request, "completed", item_results))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        domain::{
            agent_registry::AgentRegistry,
            app_state::AppPaths,
            types::{SaveTemplateRequest, TemplateItem},
        },
        repositories::{
            db::run_migrations, skills as skills_repository, templates as templates_repository,
        },
    };
    use std::{fs, path::Path};
    use tempfile::tempdir;

    fn setup_paths(root: &Path) -> AppPaths {
        let app_data_dir = root.join("app-data");
        let db_dir = app_data_dir.join("db");
        let cache_dir = app_data_dir.join("cache");
        let temp_dir = app_data_dir.join("tmp");
        let canonical_store_dir = app_data_dir.join("skills");

        fs::create_dir_all(&db_dir).unwrap();
        fs::create_dir_all(&cache_dir).unwrap();
        fs::create_dir_all(&temp_dir).unwrap();
        fs::create_dir_all(&canonical_store_dir).unwrap();

        AppPaths {
            app_data_dir,
            db_file: db_dir.join("skills-manager.db"),
            cache_dir,
            temp_dir,
            canonical_store_dir,
        }
    }

    fn seed_installed_skill(paths: &AppPaths, slug: &str) -> String {
        let source_dir = paths.canonical_store_dir.join(slug);
        fs::create_dir_all(&source_dir).unwrap();
        fs::write(source_dir.join("SKILL.md"), "# demo").unwrap();

        skills_repository::save_installed_skill(
            &paths.db_file,
            &InstallSkillRequest {
                provider: "github".into(),
                market_skill_id: slug.into(),
                source_url: format!("https://example.com/{}", slug),
                download_url: None,
                name: slug.into(),
                slug: slug.into(),
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

    fn seed_template(paths: &AppPaths, items: Vec<TemplateItem>) -> String {
        templates_repository::save_template(
            &paths.db_file,
            &SaveTemplateRequest {
                id: None,
                name: "Starter".into(),
                description: Some("starter".into()),
                tags: vec!["starter".into()],
                target_agents: vec!["Claude Code".into()],
                scope: "user".into(),
                items,
            },
        )
        .unwrap()
        .id
    }

    #[test]
    fn injects_installed_skill_into_project() {
        let dir = tempdir().unwrap();
        let paths = setup_paths(dir.path());
        run_migrations(&paths.db_file).unwrap();
        let skill_id = seed_installed_skill(&paths, "demo-skill");
        let template_id = seed_template(
            &paths,
            vec![TemplateItem {
                id: String::new(),
                skill_ref_type: "skill_id".into(),
                skill_ref: skill_id,
                display_name: Some("Demo".into()),
                required: true,
                order_index: 0,
            }],
        );
        let project_root = dir.path().join("workspace");
        fs::create_dir_all(&project_root).unwrap();

        let result = inject_template(
            &paths,
            &AgentRegistry::new(),
            "inject-template-success",
            &TemplateInjectionRequest {
                template_id,
                target_project_path: project_root.to_string_lossy().to_string(),
                overwrite_strategy: "overwrite".into(),
            },
        )
        .unwrap();

        assert_eq!(result.status, "completed");
        assert_eq!(result.installed_count, 1);
        assert!(project_root
            .join(".claude/skills/demo-skill/SKILL.md")
            .exists());
    }

    #[test]
    fn fails_when_template_item_points_to_missing_skill() {
        let dir = tempdir().unwrap();
        let paths = setup_paths(dir.path());
        run_migrations(&paths.db_file).unwrap();
        let template_id = seed_template(
            &paths,
            vec![TemplateItem {
                id: String::new(),
                skill_ref_type: "skill_id".into(),
                skill_ref: "missing-skill-id".into(),
                display_name: Some("Missing".into()),
                required: true,
                order_index: 0,
            }],
        );
        let project_root = dir.path().join("workspace");
        fs::create_dir_all(&project_root).unwrap();

        let result = inject_template(
            &paths,
            &AgentRegistry::new(),
            "inject-template-missing",
            &TemplateInjectionRequest {
                template_id,
                target_project_path: project_root.to_string_lossy().to_string(),
                overwrite_strategy: "overwrite".into(),
            },
        )
        .unwrap();

        assert_eq!(result.status, "failed");
        assert_eq!(result.failed_count, 1);
        assert_eq!(result.results[0].status, "failed");
    }

    #[test]
    fn returns_partial_when_only_some_template_items_succeed() {
        let dir = tempdir().unwrap();
        let paths = setup_paths(dir.path());
        run_migrations(&paths.db_file).unwrap();
        let skill_id = seed_installed_skill(&paths, "demo-skill");
        let template_id = seed_template(
            &paths,
            vec![
                TemplateItem {
                    id: String::new(),
                    skill_ref_type: "skill_id".into(),
                    skill_ref: skill_id,
                    display_name: Some("Installed".into()),
                    required: true,
                    order_index: 0,
                },
                TemplateItem {
                    id: String::new(),
                    skill_ref_type: "skill_id".into(),
                    skill_ref: "missing-skill-id".into(),
                    display_name: Some("Missing".into()),
                    required: true,
                    order_index: 1,
                },
            ],
        );
        let project_root = dir.path().join("workspace");
        fs::create_dir_all(&project_root).unwrap();

        let result = inject_template(
            &paths,
            &AgentRegistry::new(),
            "inject-template-partial",
            &TemplateInjectionRequest {
                template_id,
                target_project_path: project_root.to_string_lossy().to_string(),
                overwrite_strategy: "overwrite".into(),
            },
        )
        .unwrap();

        assert_eq!(result.status, "partial");
        assert_eq!(result.installed_count, 1);
        assert_eq!(result.failed_count, 1);
    }
}
