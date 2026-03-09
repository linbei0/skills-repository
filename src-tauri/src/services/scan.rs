use anyhow::{anyhow, Context, Result};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};
use time::OffsetDateTime;
use uuid::Uuid;
use walkdir::WalkDir;

use crate::domain::{
    agent_registry::{AgentPathClaim, AgentPathRole, AgentRegistry},
    types::{
        DuplicateGroup, OverviewStats, ProjectRecord, ScanSkillsRequest, ScanSkillsResult,
        SkillAgentBinding, SkillRecord,
    },
};
use crate::repositories::scan as scan_repository;

fn home_dir() -> Option<PathBuf> {
    dirs::home_dir()
}

fn canonical_string(path: &Path) -> Result<String> {
    std::fs::canonicalize(path)
        .with_context(|| format!("failed to canonicalize path {}", path.display()))
        .map(|canonical| canonical.to_string_lossy().to_string())
}

fn validate_scan_root(root: &Path, label: &str) -> Result<()> {
    if !root.exists() {
        return Err(anyhow!("{} does not exist: {}", label, root.display()));
    }

    if !root.is_dir() {
        return Err(anyhow!("{} is not a directory: {}", label, root.display()));
    }

    Ok(())
}

fn discover_skill_dirs(root: &Path) -> Result<Vec<PathBuf>> {
    validate_scan_root(root, "scan root")?;

    let mut skill_dirs = Vec::new();
    let mut walk_errors = Vec::new();

    for entry in WalkDir::new(root).min_depth(1).max_depth(4) {
        match entry {
            Ok(entry) => {
                if entry.file_type().is_file() && entry.file_name() == "SKILL.md" {
                    if let Some(parent) = entry.path().parent() {
                        skill_dirs.push(parent.to_path_buf());
                    }
                }
            }
            Err(error) => walk_errors.push(error.to_string()),
        }
    }

    if !walk_errors.is_empty() {
        return Err(anyhow!(
            "failed to walk {}: {}",
            root.display(),
            walk_errors.join("; ")
        ));
    }

    Ok(skill_dirs)
}

fn discover_optional_skill_dirs(root: &Path) -> Result<Vec<PathBuf>> {
    if !root.exists() {
        return Ok(Vec::new());
    }

    discover_skill_dirs(root)
}

fn insert_skill_record(
    skills: &mut Vec<SkillRecord>,
    seen_paths: &mut HashMap<String, usize>,
    skill_dir: &Path,
    claim: &AgentPathClaim,
    scope: &str,
    project_root: Option<String>,
    last_seen_at: i64,
) -> Result<()> {
    let path = canonical_string(skill_dir)?;
    if let Some(existing_index) = seen_paths.get(&path).copied() {
        merge_agent_binding(&mut skills[existing_index].agent, claim);
        return Ok(());
    }

    let agent = build_agent_binding(claim);
    let next_index = skills.len();
    seen_paths.insert(path.clone(), next_index);

    skills.push(SkillRecord {
        id: Uuid::new_v4().to_string(),
        name: skill_dir
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string(),
        path,
        agent,
        scope: scope.into(),
        source: "discovered".into(),
        managed: false,
        project_root,
        last_seen_at,
    });

    Ok(())
}

fn dedupe_and_sort(values: &mut Vec<String>) {
    values.sort();
    values.dedup();
}

fn build_agent_binding(claim: &AgentPathClaim) -> SkillAgentBinding {
    let mut compatible_agents = claim.compatible_agents.clone();
    dedupe_and_sort(&mut compatible_agents);

    SkillAgentBinding {
        primary: claim.agent_label.clone(),
        aliases: Vec::new(),
        priority: claim.priority,
        compatible_agents,
    }
}

fn merge_agent_binding(binding: &mut SkillAgentBinding, claim: &AgentPathClaim) {
    if claim.priority > binding.priority {
        if binding.primary != claim.agent_label {
            binding.aliases.push(binding.primary.clone());
        }
        binding.primary = claim.agent_label.clone();
        binding.priority = claim.priority;
    } else if claim.agent_label != binding.primary || matches!(claim.role, AgentPathRole::Alias) {
        binding.aliases.push(claim.agent_label.clone());
    }

    binding
        .compatible_agents
        .extend(claim.compatible_agents.iter().cloned());
    binding.compatible_agents.extend(
        binding
            .aliases
            .iter()
            .cloned()
            .filter(|alias| alias != &binding.primary),
    );

    dedupe_and_sort(&mut binding.aliases);
    binding.aliases.retain(|alias| alias != &binding.primary);
    dedupe_and_sort(&mut binding.compatible_agents);
    binding
        .compatible_agents
        .retain(|agent| agent != &binding.primary);
}

fn build_overview(skills: &[SkillRecord], duplicates: &[DuplicateGroup]) -> OverviewStats {
    OverviewStats {
        total_skills: skills.len(),
        risky_skills: None,
        duplicate_paths: duplicates.len(),
        reclaimable_bytes: None,
        template_count: None,
    }
}

fn canonical_project_root(path: &Path) -> Result<String> {
    validate_scan_root(path, "project root")?;
    canonical_string(path)
}

fn canonical_custom_root(path: &Path) -> Result<String> {
    validate_scan_root(path, "custom root")?;
    canonical_string(path)
}

fn global_scan_roots(registry: &AgentRegistry) -> Result<Vec<(AgentPathClaim, PathBuf)>> {
    let Some(home) = home_dir() else {
        return Ok(Vec::new());
    };

    Ok(registry
        .global_path_claims()
        .iter()
        .map(|claim| (claim.clone(), home.join(&claim.path)))
        .collect())
}

fn project_scan_roots(
    registry: &AgentRegistry,
    project_root: &Path,
) -> Vec<(AgentPathClaim, PathBuf)> {
    registry
        .project_path_claims()
        .iter()
        .map(|claim| (claim.clone(), project_root.join(&claim.path)))
        .collect()
}

pub fn scan_skills(
    registry: &AgentRegistry,
    request: &ScanSkillsRequest,
) -> Result<ScanSkillsResult> {
    let mut skills = Vec::new();
    let mut projects = Vec::new();
    let mut seen_paths = HashMap::new();
    let now = OffsetDateTime::now_utc().unix_timestamp();

    if request.include_system {
        for (claim, root) in global_scan_roots(registry)? {
            for skill_dir in discover_optional_skill_dirs(&root)? {
                insert_skill_record(
                    &mut skills,
                    &mut seen_paths,
                    &skill_dir,
                    &claim,
                    "system",
                    None,
                    now,
                )?;
            }
        }
    }

    if request.include_projects {
        for root in &request.project_roots {
            let project_root = PathBuf::from(root);
            let project_root_path = canonical_project_root(&project_root)?;

            projects.push(ProjectRecord {
                id: Uuid::new_v4().to_string(),
                name: project_root
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string(),
                root_path: project_root_path.clone(),
            });

            for (claim, candidate) in project_scan_roots(registry, &project_root) {
                for skill_dir in discover_optional_skill_dirs(&candidate)? {
                    insert_skill_record(
                        &mut skills,
                        &mut seen_paths,
                        &skill_dir,
                        &claim,
                        "project",
                        Some(project_root_path.clone()),
                        now,
                    )?;
                }
            }
        }
    }

    for root in &request.custom_roots {
        let custom_root = PathBuf::from(root);
        let _ = canonical_custom_root(&custom_root)?;
        for skill_dir in discover_skill_dirs(&custom_root)? {
            insert_skill_record(
                &mut skills,
                &mut seen_paths,
                &skill_dir,
                &AgentPathClaim {
                    agent_label: "Custom".into(),
                    path: root.clone(),
                    role: AgentPathRole::Primary,
                    priority: 100,
                    compatible_agents: Vec::new(),
                },
                "custom",
                None,
                now,
            )?;
        }
    }

    let mut duplicates_map: HashMap<String, Vec<String>> = HashMap::new();
    for skill in &skills {
        duplicates_map
            .entry(skill.name.clone())
            .or_default()
            .push(skill.path.clone());
    }

    let duplicates: Vec<DuplicateGroup> = duplicates_map
        .into_iter()
        .filter(|(_, paths)| paths.len() > 1)
        .map(|(name, paths)| DuplicateGroup { name, paths })
        .collect();

    Ok(ScanSkillsResult {
        overview: build_overview(&skills, &duplicates),
        skills,
        distributions: Vec::new(),
        duplicates,
        projects,
    })
}

pub fn persist_scan_snapshot(
    db_path: &Path,
    result: &ScanSkillsResult,
) -> Result<ScanSkillsResult> {
    scan_repository::replace_scan_snapshot(db_path, result)?;
    scan_repository::load_scan_snapshot(db_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repositories::{db::run_migrations, scan as scan_repository};
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn finds_project_level_skills() {
        let dir = tempdir().unwrap();
        let skill_dir = dir.path().join(".claude/skills/python-helper");
        fs::create_dir_all(&skill_dir).unwrap();
        fs::write(skill_dir.join("SKILL.md"), "# test").unwrap();

        let registry = AgentRegistry::new();
        let result = scan_skills(
            &registry,
            &ScanSkillsRequest {
                include_system: false,
                include_projects: true,
                project_roots: vec![dir.path().to_string_lossy().to_string()],
                custom_roots: vec![],
            },
        )
        .unwrap();

        assert_eq!(result.skills.len(), 1);
        assert_eq!(result.skills[0].name, "python-helper");
        assert_eq!(result.skills[0].scope, "project");
        assert_eq!(result.skills[0].agent.primary, "Claude Code");
        assert_eq!(result.overview.total_skills, 1);
        assert_eq!(result.overview.duplicate_paths, 0);
        assert_eq!(result.overview.risky_skills, None);
    }

    #[test]
    fn merges_shared_agents_for_standard_agents_path() {
        let dir = tempdir().unwrap();
        let skill_dir = dir.path().join(".agents/skills/shared-helper");
        fs::create_dir_all(&skill_dir).unwrap();
        fs::write(skill_dir.join("SKILL.md"), "# shared").unwrap();

        let registry = AgentRegistry::new();
        let result = scan_skills(
            &registry,
            &ScanSkillsRequest {
                include_system: false,
                include_projects: true,
                project_roots: vec![dir.path().to_string_lossy().to_string()],
                custom_roots: vec![],
            },
        )
        .unwrap();

        assert_eq!(result.skills.len(), 1);
        assert_eq!(result.skills[0].agent.primary, "OpenAI Codex");
        assert_eq!(
            result.skills[0].agent.aliases,
            vec!["Cursor".to_string(), "GitHub Copilot / VS Code".to_string()]
        );
        assert_eq!(result.skills[0].agent.priority, 90);
        assert_eq!(
            result.skills[0].agent.compatible_agents,
            vec!["Cursor".to_string(), "GitHub Copilot / VS Code".to_string()]
        );
    }

    #[test]
    fn keeps_primary_agent_for_native_codex_path() {
        let dir = tempdir().unwrap();
        let skill_dir = dir.path().join(".codex/skills/codex-helper");
        fs::create_dir_all(&skill_dir).unwrap();
        fs::write(skill_dir.join("SKILL.md"), "# codex").unwrap();

        let registry = AgentRegistry::new();
        let result = scan_skills(
            &registry,
            &ScanSkillsRequest {
                include_system: false,
                include_projects: true,
                project_roots: vec![dir.path().to_string_lossy().to_string()],
                custom_roots: vec![],
            },
        )
        .unwrap();

        assert_eq!(result.skills.len(), 1);
        assert_eq!(result.skills[0].agent.primary, "OpenAI Codex");
        assert!(result.skills[0].agent.aliases.is_empty());
        assert!(result.skills[0].agent.compatible_agents.is_empty());
    }

    #[test]
    fn canonical_string_fails_for_missing_path() {
        let dir = tempdir().unwrap();
        let missing = dir.path().join("missing");

        let result = canonical_string(&missing);

        assert!(result.is_err());
    }

    #[test]
    fn fails_when_project_root_is_missing() {
        let dir = tempdir().unwrap();
        let missing_root = dir.path().join("missing-project-root");
        let registry = AgentRegistry::new();

        let result = scan_skills(
            &registry,
            &ScanSkillsRequest {
                include_system: false,
                include_projects: true,
                project_roots: vec![missing_root.to_string_lossy().to_string()],
                custom_roots: vec![],
            },
        );

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("project root does not exist"));
    }

    #[test]
    fn fails_when_custom_root_is_missing() {
        let dir = tempdir().unwrap();
        let missing_root = dir.path().join("missing-custom-root");
        let registry = AgentRegistry::new();

        let result = scan_skills(
            &registry,
            &ScanSkillsRequest {
                include_system: false,
                include_projects: false,
                project_roots: vec![],
                custom_roots: vec![missing_root.to_string_lossy().to_string()],
            },
        );

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("custom root does not exist"));
    }

    #[test]
    fn persists_scan_snapshot_for_reload() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("scan.db");
        run_migrations(&db_path).unwrap();

        let project_skill_dir = dir.path().join(".claude/skills/python-helper");
        fs::create_dir_all(&project_skill_dir).unwrap();
        fs::write(project_skill_dir.join("SKILL.md"), "# project").unwrap();

        let custom_root = dir.path().join("custom");
        let custom_skill_dir = custom_root.join("python-helper");
        fs::create_dir_all(&custom_skill_dir).unwrap();
        fs::write(custom_skill_dir.join("SKILL.md"), "# custom").unwrap();

        let registry = AgentRegistry::new();
        let scan_result = scan_skills(
            &registry,
            &ScanSkillsRequest {
                include_system: false,
                include_projects: true,
                project_roots: vec![dir.path().to_string_lossy().to_string()],
                custom_roots: vec![custom_root.to_string_lossy().to_string()],
            },
        )
        .unwrap();

        let persisted = persist_scan_snapshot(&db_path, &scan_result).unwrap();
        let reloaded = scan_repository::load_scan_snapshot(&db_path).unwrap();
        let overview = scan_repository::load_overview_stats(&db_path).unwrap();

        assert_eq!(persisted.projects.len(), 1);
        assert_eq!(persisted.skills.len(), 2);
        assert_eq!(persisted.distributions.len(), 2);
        assert_eq!(persisted.duplicates.len(), 1);
        assert_eq!(persisted.overview.total_skills, 2);
        assert_eq!(persisted.overview.duplicate_paths, 1);
        assert_eq!(persisted.overview.risky_skills, None);

        assert_eq!(reloaded.projects.len(), 1);
        assert_eq!(reloaded.skills.len(), 2);
        assert_eq!(reloaded.distributions.len(), 2);
        assert_eq!(reloaded.duplicates.len(), 1);
        assert_eq!(reloaded.overview.total_skills, 2);
        assert_eq!(reloaded.overview.duplicate_paths, 1);
        assert_eq!(reloaded.overview.template_count, None);
        assert!(reloaded
            .skills
            .iter()
            .any(|skill| skill.scope == "project" && skill.project_root.is_some()));
        assert!(reloaded
            .skills
            .iter()
            .any(|skill| skill.scope == "custom" && skill.project_root.is_none()));

        assert_eq!(overview.total_skills, 2);
        assert_eq!(overview.duplicate_paths, 1);
        assert_eq!(overview.reclaimable_bytes, None);
    }
}
