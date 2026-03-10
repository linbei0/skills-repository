use anyhow::{anyhow, Context, Result};
use rusqlite::params;
use serde_json::json;
use std::{fs, path::{Path, PathBuf}};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::domain::types::{
    InstallSkillRequest, RepositorySkillDetail, RepositorySkillSummary,
};

use super::db::open_connection;

pub fn save_installed_skill(
    path: &Path,
    request: &InstallSkillRequest,
    canonical_path: &str,
    security_level: &str,
    blocked: bool,
) -> Result<String> {
    let conn = open_connection(path)?;
    let now = OffsetDateTime::now_utc().unix_timestamp();
    let skill_id = Uuid::new_v4().to_string();

    conn.execute(
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
        VALUES (?1, ?2, ?3, NULL, 'market', ?4, ?5, ?6, ?7, ?8, NULL, 0, 'managed', ?9, ?10, ?11, ?12, ?13, ?14)
        ",
        params![
            skill_id,
            request.slug,
            request.name,
            request.provider,
            request.source_url,
            request.version,
            request.author,
            canonical_path,
            security_level,
            blocked as i64,
            now,
            now,
            now,
            json!({
                "downloadUrl": request.download_url,
                "requestedTargets": request.requested_targets,
            })
            .to_string(),
        ],
    )?;

    Ok(skill_id)
}

pub fn save_operation_log(
    path: &Path,
    operation_type: &str,
    entity_type: &str,
    entity_id: Option<&str>,
    status: &str,
    summary: &str,
    detail_json: Option<serde_json::Value>,
) -> Result<String> {
    let conn = open_connection(path)?;
    let log_id = Uuid::new_v4().to_string();
    let now = OffsetDateTime::now_utc().unix_timestamp();

    conn.execute(
        "
        INSERT INTO operation_logs (
            id,
            operation_type,
            entity_type,
            entity_id,
            status,
            summary,
            detail_json,
            created_at
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
        ",
        params![
            log_id,
            operation_type,
            entity_type,
            entity_id,
            status,
            summary,
            detail_json.map(|value| value.to_string()),
            now,
        ],
    )?;

    Ok(log_id)
}

pub struct SkillSource {
    pub source_path: String,
    pub target_name: String,
}

pub struct InstalledSkillSummary {
    pub skill_id: String,
    pub name: String,
    pub canonical_path: String,
}

pub struct RepositorySkillRemovalPlan {
    pub skill_id: String,
    pub canonical_path: String,
    pub distribution_paths: Vec<String>,
}

pub fn load_skill_source(path: &Path, skill_id: &str) -> Result<SkillSource> {
    let conn = open_connection(path)?;
    conn.query_row(
        "
        SELECT
            COALESCE(s.canonical_path, (
                SELECT target_path
                FROM skill_distributions
                WHERE skill_id = s.id
                ORDER BY created_at ASC
                LIMIT 1
            )) AS source_path,
            COALESCE(s.slug, s.name) AS target_name
        FROM skills s
        WHERE s.id = ?1
        ",
        params![skill_id],
        |row| {
            Ok(SkillSource {
                source_path: row.get(0)?,
                target_name: row.get(1)?,
            })
        },
    )
    .map_err(Into::into)
}

pub fn update_skill_security_status(
    path: &Path,
    skill_id: &str,
    security_level: &str,
    blocked: bool,
    scanned_at: i64,
) -> Result<()> {
    let conn = open_connection(path)?;
    conn.execute(
        "
        UPDATE skills
        SET security_level = ?2,
            blocked = ?3,
            last_scanned_at = ?4,
            updated_at = ?4
        WHERE id = ?1
        ",
        params![skill_id, security_level, blocked as i64, scanned_at],
    )?;
    Ok(())
}

pub fn list_installed_skills(path: &Path) -> Result<Vec<InstalledSkillSummary>> {
    let conn = open_connection(path)?;
    let mut stmt = conn.prepare(
        "
        SELECT id, name, canonical_path
        FROM skills
        WHERE canonical_path IS NOT NULL
        ORDER BY updated_at DESC
        ",
    )?;

    let rows = stmt.query_map([], |row| {
        Ok(InstalledSkillSummary {
            skill_id: row.get(0)?,
            name: row.get(1)?,
            canonical_path: row.get(2)?,
        })
    })?;

    rows.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(Into::into)
}


fn canonicalize_existing_path(path: &Path) -> Result<PathBuf> {
    fs::canonicalize(path).with_context(|| format!("failed to canonicalize {}", path.display()))
}

pub fn list_repository_skills(
    path: &Path,
    canonical_store_dir: &Path,
) -> Result<Vec<RepositorySkillSummary>> {
    let canonical_root = canonicalize_existing_path(canonical_store_dir)?;
    let conn = open_connection(path)?;
    let mut stmt = conn.prepare(
        "
        SELECT
            id,
            name,
            source_type,
            source_market,
            installed_at,
            security_level,
            blocked,
            canonical_path
        FROM skills
        WHERE canonical_path IS NOT NULL
        ORDER BY installed_at DESC, name ASC
        ",
    )?;

    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, Option<String>>(3)?,
            row.get::<_, i64>(4)?,
            row.get::<_, String>(5)?,
            row.get::<_, i64>(6)? != 0,
            row.get::<_, String>(7)?,
        ))
    })?;

    let mut skills = Vec::new();
    for row in rows {
        let (id, name, source_type, source_market, installed_at, security_level, blocked, raw_path) =
            row?;
        let skill_path = PathBuf::from(&raw_path);
        if !skill_path.exists() {
            continue;
        }
        let canonical_skill_path = match canonicalize_existing_path(&skill_path) {
            Ok(value) => value,
            Err(_) => continue,
        };
        if !canonical_skill_path.starts_with(&canonical_root) {
            continue;
        }

        skills.push(RepositorySkillSummary {
            id,
            name,
            source_type,
            source_market,
            installed_at,
            security_level,
            blocked,
        });
    }

    Ok(skills)
}

pub fn get_repository_skill_detail(
    path: &Path,
    canonical_store_dir: &Path,
    skill_id: &str,
) -> Result<RepositorySkillDetail> {
    let canonical_root = canonicalize_existing_path(canonical_store_dir)?;
    let conn = open_connection(path)?;
    let row = conn.query_row(
        "
        SELECT
            id,
            name,
            canonical_path,
            source_type,
            source_market,
            source_url,
            installed_at,
            security_level,
            blocked
        FROM skills
        WHERE id = ?1 AND canonical_path IS NOT NULL
        ",
        params![skill_id],
        |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, Option<String>>(4)?,
                row.get::<_, Option<String>>(5)?,
                row.get::<_, i64>(6)?,
                row.get::<_, String>(7)?,
                row.get::<_, i64>(8)? != 0,
            ))
        },
    )?;

    let (
        id,
        name,
        canonical_path,
        source_type,
        source_market,
        source_url,
        installed_at,
        security_level,
        blocked,
    ) = row;

    let skill_dir = PathBuf::from(&canonical_path);
    let canonical_skill_dir = canonicalize_existing_path(&skill_dir)?;
    if !canonical_skill_dir.starts_with(&canonical_root) {
        return Err(anyhow!(
            "skill {} is not inside canonical store: {}",
            skill_id,
            canonical_skill_dir.display()
        ));
    }

    let skill_markdown_path = canonical_skill_dir.join("SKILL.md");
    let skill_markdown = fs::read_to_string(&skill_markdown_path).with_context(|| {
        format!(
            "failed to read repository skill markdown {}",
            skill_markdown_path.display()
        )
    })?;

    Ok(RepositorySkillDetail {
        id,
        name,
        canonical_path: canonical_skill_dir.to_string_lossy().to_string(),
        source_type,
        source_market,
        source_url,
        installed_at,
        security_level,
        blocked,
        skill_markdown,
    })
}

pub fn load_repository_skill_removal_plan(
    path: &Path,
    canonical_store_dir: &Path,
    skill_id: &str,
) -> Result<RepositorySkillRemovalPlan> {
    let canonical_root = canonicalize_existing_path(canonical_store_dir)?;
    let conn = open_connection(path)?;
    let canonical_path: String = conn.query_row(
        "SELECT canonical_path FROM skills WHERE id = ?1 AND canonical_path IS NOT NULL",
        params![skill_id],
        |row| row.get(0),
    )?;

    let canonical_skill_dir = canonicalize_existing_path(Path::new(&canonical_path))?;
    if !canonical_skill_dir.starts_with(&canonical_root) {
        return Err(anyhow!(
            "skill {} is not inside canonical store: {}",
            skill_id,
            canonical_skill_dir.display()
        ));
    }

    let mut stmt = conn.prepare(
        "
        SELECT target_path
        FROM skill_distributions
        WHERE skill_id = ?1
        ORDER BY created_at ASC
        ",
    )?;
    let rows = stmt.query_map(params![skill_id], |row| row.get::<_, String>(0))?;
    let distribution_paths = rows.collect::<rusqlite::Result<Vec<_>>>()?;

    Ok(RepositorySkillRemovalPlan {
        skill_id: skill_id.to_string(),
        canonical_path: canonical_skill_dir.to_string_lossy().to_string(),
        distribution_paths,
    })
}

pub fn delete_repository_skill(path: &Path, skill_id: &str) -> Result<()> {
    let mut conn = open_connection(path)?;
    let tx = conn.transaction()?;

    tx.execute(
        "DELETE FROM skill_distributions WHERE skill_id = ?1",
        params![skill_id],
    )?;
    tx.execute(
        "DELETE FROM security_reports WHERE skill_id = ?1",
        params![skill_id],
    )?;
    tx.execute("DELETE FROM skills WHERE id = ?1", params![skill_id])?;

    tx.commit()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        domain::types::{DistributionRequest, SecurityReport},
        repositories::{
            db::run_migrations, distributions as distributions_repository,
            security as security_repository,
        },
    };
    use tempfile::tempdir;

    fn seed_skill(root: &Path, source_type: &str, source_market: Option<&str>) -> (PathBuf, String) {
        let app_data_dir = root.join("app-data");
        let db_dir = app_data_dir.join("db");
        let canonical_store_dir = app_data_dir.join("skills");
        fs::create_dir_all(&db_dir).unwrap();
        fs::create_dir_all(&canonical_store_dir).unwrap();
        let db_path = db_dir.join("skills-manager.db");
        run_migrations(&db_path).unwrap();

        let skill_dir = canonical_store_dir.join("demo-skill");
        fs::create_dir_all(&skill_dir).unwrap();
        fs::write(skill_dir.join("SKILL.md"), "# demo skill").unwrap();

        let request = InstallSkillRequest {
            provider: source_market.unwrap_or("local").to_string(),
            market_skill_id: "demo".into(),
            source_url: "https://example.com/demo".into(),
            download_url: None,
            name: "Demo Skill".into(),
            slug: "demo-skill".into(),
            version: Some("main".into()),
            author: Some("tester".into()),
            requested_targets: Vec::<DistributionRequest>::new(),
        };
        let skill_id = save_installed_skill(
            &db_path,
            &request,
            &skill_dir.to_string_lossy(),
            "safe",
            false,
        )
        .unwrap();

        let conn = open_connection(&db_path).unwrap();
        conn.execute(
            "UPDATE skills SET source_type = ?2, source_market = ?3 WHERE id = ?1",
            params![skill_id, source_type, source_market],
        )
        .unwrap();

        (db_path, skill_id)
    }

    #[test]
    fn lists_only_existing_skills_in_canonical_store() {
        let dir = tempdir().unwrap();
        let (db_path, skill_id) = seed_skill(dir.path(), "market", Some("github"));
        let canonical_store_dir = dir.path().join("app-data").join("skills");

        let skills = list_repository_skills(&db_path, &canonical_store_dir).unwrap();

        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].id, skill_id);
        assert_eq!(skills[0].source_market.as_deref(), Some("github"));
    }

    #[test]
    fn loads_repository_skill_detail_with_markdown() {
        let dir = tempdir().unwrap();
        let (db_path, skill_id) = seed_skill(dir.path(), "market", Some("github"));
        let canonical_store_dir = dir.path().join("app-data").join("skills");

        let detail = get_repository_skill_detail(&db_path, &canonical_store_dir, &skill_id).unwrap();

        assert_eq!(detail.id, skill_id);
        assert!(detail.skill_markdown.contains("demo skill"));
        assert_eq!(detail.source_market.as_deref(), Some("github"));
    }

    #[test]
    fn deletes_skill_distributions_and_reports_from_database() {
        let dir = tempdir().unwrap();
        let (db_path, skill_id) = seed_skill(dir.path(), "market", Some("github"));
        let target_path = dir.path().join("global").join("demo-skill");
        fs::create_dir_all(target_path.parent().unwrap()).unwrap();
        fs::create_dir_all(&target_path).unwrap();
        fs::write(target_path.join("SKILL.md"), "# distributed").unwrap();

        distributions_repository::save_distribution(
            &db_path,
            &skill_id,
            "global",
            "Codex",
            None,
            &target_path.to_string_lossy(),
            "copy",
            "active",
        )
        .unwrap();

        security_repository::save_security_report(
            &db_path,
            &SecurityReport {
                id: "report-1".into(),
                skill_id: Some(skill_id.clone()),
                skill_name: Some("Demo Skill".into()),
                source_path: Some(target_path.to_string_lossy().to_string()),
                scan_scope: "canonical_store".into(),
                level: "safe".into(),
                score: 100,
                blocked: false,
                issues: Vec::new(),
                recommendations: Vec::new(),
                scanned_files: vec![target_path.join("SKILL.md").to_string_lossy().to_string()],
                engine_version: "phase2-rules-v1".into(),
                scanned_at: 100,
            },
        )
        .unwrap();

        delete_repository_skill(&db_path, &skill_id).unwrap();
        let conn = open_connection(&db_path).unwrap();
        let skill_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM skills WHERE id = ?1", params![skill_id], |row| {
                row.get(0)
            })
            .unwrap();
        let distribution_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM skill_distributions", [], |row| row.get(0))
            .unwrap();
        let report_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM security_reports", [], |row| row.get(0))
            .unwrap();

        assert_eq!(skill_count, 0);
        assert_eq!(distribution_count, 0);
        assert_eq!(report_count, 0);
    }
}
