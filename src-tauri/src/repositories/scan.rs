use anyhow::Result;
use rusqlite::params;
use std::{collections::HashMap, path::Path};

use crate::domain::types::{
    DistributionRecord, DuplicateGroup, OverviewStats, ProjectRecord, ScanSkillsResult,
    SkillAgentBinding, SkillRecord,
};

use super::db::open_connection;

const DISCOVERED_SOURCE: &str = "discovered";
const ACTIVE_STATUS: &str = "active";
const UNMANAGED_MODE: &str = "unmanaged";
const UNKNOWN_SECURITY_LEVEL: &str = "unknown";
const NATIVE_INSTALL_MODE: &str = "native";

fn target_kind_for_scope(scope: &str) -> &str {
    match scope {
        "system" => "global",
        "project" => "project",
        "custom" => "custom",
        _ => "custom",
    }
}

fn scope_for_target_kind(target_kind: &str) -> &str {
    match target_kind {
        "global" => "system",
        "project" => "project",
        "custom" => "custom",
        _ => "custom",
    }
}

fn build_duplicates(skills: &[SkillRecord]) -> Vec<DuplicateGroup> {
    let mut duplicates_map: HashMap<String, Vec<String>> = HashMap::new();

    for skill in skills {
        duplicates_map
            .entry(skill.name.clone())
            .or_default()
            .push(skill.path.clone());
    }

    duplicates_map
        .into_iter()
        .filter(|(_, paths)| paths.len() > 1)
        .map(|(name, paths)| DuplicateGroup { name, paths })
        .collect()
}

pub fn replace_scan_snapshot(path: &Path, snapshot: &ScanSkillsResult) -> Result<()> {
    let mut conn = open_connection(path)?;
    let tx = conn.transaction()?;

    tx.execute(
        "
        DELETE FROM skill_distributions
        WHERE skill_id IN (
            SELECT id FROM skills WHERE source_type = ?1
        )
        ",
        params![DISCOVERED_SOURCE],
    )?;
    tx.execute(
        "DELETE FROM skills WHERE source_type = ?1",
        params![DISCOVERED_SOURCE],
    )?;
    tx.execute(
        "
        DELETE FROM projects
        WHERE id NOT IN (
            SELECT DISTINCT project_id
            FROM skill_distributions
            WHERE project_id IS NOT NULL
        )
        ",
        [],
    )?;

    for project in &snapshot.projects {
        tx.execute(
            "
            INSERT INTO projects (id, name, root_path, labels_json, created_at, updated_at)
            VALUES (?1, ?2, ?3, NULL, ?4, ?5)
            ON CONFLICT(root_path) DO UPDATE SET
                name = excluded.name,
                updated_at = excluded.updated_at
            ",
            params![project.id, project.name, project.root_path, 0_i64, 0_i64,],
        )?;
    }

    let mut project_id_by_root = HashMap::new();
    for project in &snapshot.projects {
        let project_id: String = tx.query_row(
            "SELECT id FROM projects WHERE root_path = ?1",
            params![project.root_path],
            |row| row.get(0),
        )?;
        project_id_by_root.insert(project.root_path.clone(), project_id);
    }

    for skill in &snapshot.skills {
        tx.execute(
            "
            INSERT INTO skills (
                id,
                slug,
                name,
                description,
                source_type,
                source_market,
                source_url,
                version,
                author,
                canonical_path,
                file_hash,
                size_bytes,
                management_mode,
                security_level,
                blocked,
                installed_at,
                updated_at,
                last_scanned_at,
                metadata_json
            )
            VALUES (?1, NULL, ?2, NULL, ?3, NULL, NULL, NULL, NULL, NULL, NULL, 0, ?4, ?5, 0, ?6, ?7, ?8, NULL)
            ",
            params![
                skill.id,
                skill.name,
                DISCOVERED_SOURCE,
                if skill.managed { "managed" } else { UNMANAGED_MODE },
                UNKNOWN_SECURITY_LEVEL,
                skill.last_seen_at,
                skill.last_seen_at,
                skill.last_seen_at,
            ],
        )?;

        let target_kind = target_kind_for_scope(&skill.scope);
        let project_id = skill
            .project_root
            .as_ref()
            .and_then(|root| project_id_by_root.get(root));

        tx.execute(
            "
            INSERT INTO skill_distributions (
                id,
                skill_id,
                target_kind,
                target_agent,
                project_id,
                target_path,
                install_mode,
                status,
                created_at,
                updated_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
            ",
            params![
                format!("distribution:{}", skill.id),
                skill.id,
                target_kind,
                skill.agent.primary.clone(),
                project_id,
                skill.path,
                NATIVE_INSTALL_MODE,
                ACTIVE_STATUS,
                skill.last_seen_at,
                skill.last_seen_at,
            ],
        )?;
    }

    tx.commit()?;
    Ok(())
}

pub fn load_overview_stats(path: &Path) -> Result<OverviewStats> {
    let conn = open_connection(path)?;
    let total_skills: usize = conn.query_row(
        "SELECT COUNT(*) FROM skills WHERE source_type = ?1",
        params![DISCOVERED_SOURCE],
        |row| row.get(0),
    )?;
    let duplicate_paths: usize = conn.query_row(
        "
        SELECT COUNT(*)
        FROM (
            SELECT name
            FROM skills
            WHERE source_type = ?1
            GROUP BY name
            HAVING COUNT(*) > 1
        )
        ",
        params![DISCOVERED_SOURCE],
        |row| row.get(0),
    )?;

    Ok(OverviewStats {
        total_skills,
        risky_skills: None,
        duplicate_paths,
        reclaimable_bytes: None,
        template_count: None,
    })
}

pub fn load_scan_snapshot(path: &Path) -> Result<ScanSkillsResult> {
    let conn = open_connection(path)?;

    let projects = {
        let mut stmt = conn.prepare(
            "
            SELECT id, name, root_path
            FROM projects
            ORDER BY root_path
            ",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(ProjectRecord {
                id: row.get(0)?,
                name: row.get(1)?,
                root_path: row.get(2)?,
            })
        })?;
        rows.collect::<rusqlite::Result<Vec<_>>>()?
    };

    let skills = {
        let mut stmt = conn.prepare(
            "
            SELECT
                s.id,
                s.name,
                d.target_path,
                d.target_agent,
                d.target_kind,
                s.source_type,
                s.management_mode,
                p.root_path,
                COALESCE(s.last_scanned_at, s.updated_at, s.installed_at)
            FROM skills s
            JOIN skill_distributions d ON d.skill_id = s.id
            LEFT JOIN projects p ON p.id = d.project_id
            WHERE s.source_type = ?1
            ORDER BY s.name, d.target_path
            ",
        )?;
        let rows = stmt.query_map(params![DISCOVERED_SOURCE], |row| {
            let target_kind: String = row.get(4)?;
            let management_mode: String = row.get(6)?;

            Ok(SkillRecord {
                id: row.get(0)?,
                name: row.get(1)?,
                path: row.get(2)?,
                agent: SkillAgentBinding {
                    primary: row.get(3)?,
                    aliases: Vec::new(),
                    priority: 0,
                    compatible_agents: Vec::new(),
                },
                scope: scope_for_target_kind(&target_kind).to_string(),
                source: row.get(5)?,
                managed: management_mode == "managed",
                project_root: row.get(7)?,
                last_seen_at: row.get(8)?,
            })
        })?;
        rows.collect::<rusqlite::Result<Vec<_>>>()?
    };

    let distributions = {
        let mut stmt = conn.prepare(
            "
            SELECT id, skill_id, target_agent, target_path, status
            FROM skill_distributions
            WHERE skill_id IN (
                SELECT id FROM skills WHERE source_type = ?1
            )
            ORDER BY target_path
            ",
        )?;
        let rows = stmt.query_map(params![DISCOVERED_SOURCE], |row| {
            Ok(DistributionRecord {
                id: row.get(0)?,
                skill_id: row.get(1)?,
                target_agent: row.get(2)?,
                target_path: row.get(3)?,
                status: row.get(4)?,
            })
        })?;
        rows.collect::<rusqlite::Result<Vec<_>>>()?
    };

    let duplicates = build_duplicates(&skills);
    let overview = load_overview_stats(path)?;

    Ok(ScanSkillsResult {
        overview,
        duplicates,
        distributions,
        projects,
        skills,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repositories::db::run_migrations;
    use tempfile::tempdir;

    fn sample_snapshot() -> ScanSkillsResult {
        ScanSkillsResult {
            overview: OverviewStats {
                total_skills: 2,
                risky_skills: None,
                duplicate_paths: 1,
                reclaimable_bytes: None,
                template_count: None,
            },
            skills: vec![
                SkillRecord {
                    id: "skill-project".into(),
                    name: "python-helper".into(),
                    path: "E:/project/.claude/skills/python-helper".into(),
                    agent: SkillAgentBinding {
                        primary: "Claude".into(),
                        aliases: Vec::new(),
                        priority: 100,
                        compatible_agents: Vec::new(),
                    },
                    scope: "project".into(),
                    source: "discovered".into(),
                    managed: false,
                    project_root: Some("E:/project".into()),
                    last_seen_at: 100,
                },
                SkillRecord {
                    id: "skill-custom".into(),
                    name: "python-helper".into(),
                    path: "E:/custom/python-helper".into(),
                    agent: SkillAgentBinding {
                        primary: "Custom".into(),
                        aliases: Vec::new(),
                        priority: 100,
                        compatible_agents: Vec::new(),
                    },
                    scope: "custom".into(),
                    source: "discovered".into(),
                    managed: false,
                    project_root: None,
                    last_seen_at: 100,
                },
            ],
            distributions: Vec::new(),
            duplicates: vec![DuplicateGroup {
                name: "python-helper".into(),
                paths: vec![
                    "E:/project/.claude/skills/python-helper".into(),
                    "E:/custom/python-helper".into(),
                ],
            }],
            projects: vec![ProjectRecord {
                id: "project-1".into(),
                name: "project".into(),
                root_path: "E:/project".into(),
            }],
        }
    }

    #[test]
    fn loads_overview_with_unknown_metrics_after_snapshot_replace() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("scan.db");
        run_migrations(&db_path).unwrap();

        replace_scan_snapshot(&db_path, &sample_snapshot()).unwrap();
        let overview = load_overview_stats(&db_path).unwrap();

        assert_eq!(overview.total_skills, 2);
        assert_eq!(overview.duplicate_paths, 1);
        assert_eq!(overview.risky_skills, None);
        assert_eq!(overview.reclaimable_bytes, None);
        assert_eq!(overview.template_count, None);
    }

    #[test]
    fn loads_scan_snapshot_with_backend_overview() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("scan.db");
        run_migrations(&db_path).unwrap();

        replace_scan_snapshot(&db_path, &sample_snapshot()).unwrap();
        let snapshot = load_scan_snapshot(&db_path).unwrap();

        assert_eq!(snapshot.skills.len(), 2);
        assert_eq!(snapshot.projects.len(), 1);
        assert_eq!(snapshot.distributions.len(), 2);
        assert_eq!(snapshot.duplicates.len(), 1);
        assert_eq!(snapshot.overview.total_skills, 2);
        assert_eq!(snapshot.overview.duplicate_paths, 1);
        assert_eq!(snapshot.overview.risky_skills, None);
    }
}
