use anyhow::Result;
use rusqlite::{params, OptionalExtension};
use serde_json::json;
use std::path::Path;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::domain::types::InstallSkillRequest;

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
    task_id: Option<&str>,
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
            task_id,
            operation_type,
            entity_type,
            entity_id,
            status,
            summary,
            detail_json,
            created_at
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
        ",
        params![
            log_id,
            task_id,
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

pub fn find_installed_skill_id(
    path: &Path,
    slug: &str,
    source_url: &str,
) -> Result<Option<String>> {
    let conn = open_connection(path)?;
    conn.query_row(
        "
        SELECT id
        FROM skills
        WHERE slug = ?1 OR source_url = ?2
        ORDER BY updated_at DESC
        LIMIT 1
        ",
        params![slug, source_url],
        |row| row.get(0),
    )
    .optional()
    .map_err(Into::into)
}
