use anyhow::Result;
use rusqlite::params;
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
            target_path,
            install_mode,
            status,
            created_at,
            updated_at
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
        ",
        params![
            distribution_id,
            skill_id,
            target_kind,
            target_agent,
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
