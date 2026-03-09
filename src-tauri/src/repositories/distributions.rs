use anyhow::Result;
use rusqlite::{params, OptionalExtension};
use std::path::Path;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::domain::types::DistributionResult;

use super::db::open_connection;

pub fn save_distribution(
    path: &Path,
    skill_id: &str,
    target_kind: &str,
    target_agent: &str,
    project_id: Option<&str>,
    target_path: &str,
    install_mode: &str,
    status: &str,
) -> Result<DistributionResult> {
    let conn = open_connection(path)?;
    let distribution_id = Uuid::new_v4().to_string();
    let now = OffsetDateTime::now_utc().unix_timestamp();

    conn.execute(
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
            distribution_id,
            skill_id,
            target_kind,
            target_agent,
            project_id,
            target_path,
            install_mode,
            status,
            now,
            now,
        ],
    )?;

    Ok(DistributionResult {
        distribution_id,
        skill_id: skill_id.to_string(),
        target_agent: target_agent.to_string(),
        target_path: target_path.to_string(),
        status: status.to_string(),
        message: None,
    })
}

pub fn find_project_id_by_root(path: &Path, project_root: &str) -> Result<Option<String>> {
    let conn = open_connection(path)?;
    conn.query_row(
        "SELECT id FROM projects WHERE root_path = ?1",
        params![project_root],
        |row| row.get::<_, String>(0),
    )
    .optional()
    .map_err(Into::into)
}
