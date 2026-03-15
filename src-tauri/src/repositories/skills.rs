use anyhow::{anyhow, Context, Result};
use rusqlite::{params, Connection, Transaction};
use serde_json::{json, Value};
use std::{
    fs,
    path::{Path, PathBuf},
};
use time::OffsetDateTime;
use url::Url;
use uuid::Uuid;

use crate::domain::types::{InstallSkillRequest, RepositorySkillDetail, RepositorySkillSummary};
use crate::path_utils::display_path;

use super::db::open_connection;

fn normalize_persisted_source_type(source_type: &str) -> &str {
    match source_type {
        "market" | "github" | "local" => source_type,
        _ => "market",
    }
}

fn normalize_persisted_source_market(
    request: &InstallSkillRequest,
    persisted_source_type: &str,
) -> Option<String> {
    match persisted_source_type {
        "market" => Some(request.provider.clone()),
        "github" => Some("github".to_string()),
        "local" => None,
        _ => None,
    }
}

fn display_source_url(source_type: &str, source_url: Option<String>) -> Option<String> {
    match (source_type, source_url) {
        ("local", Some(value)) => Some(display_path(&value)),
        (_, value) => value,
    }
}

fn repo_url_from_metadata(metadata_json: Option<String>) -> Option<String> {
    metadata_json
        .and_then(|raw| serde_json::from_str::<Value>(&raw).ok())
        .and_then(|metadata| {
            metadata
                .get("repoUrl")
                .and_then(Value::as_str)
                .map(ToString::to_string)
        })
}

fn metadata_from_json(metadata_json: Option<String>) -> Value {
    metadata_json
        .and_then(|raw| serde_json::from_str::<Value>(&raw).ok())
        .filter(|value| value.is_object())
        .unwrap_or_else(|| json!({}))
}

fn metadata_string_field(metadata_json: Option<String>, field: &str) -> Option<String> {
    metadata_from_json(metadata_json)
        .get(field)
        .and_then(Value::as_str)
        .map(ToString::to_string)
}

fn normalize_github_repo_url(value: &str) -> Option<String> {
    let parsed = Url::parse(value.trim()).ok()?;
    let host = parsed.host_str()?;
    if host != "github.com" && host != "www.github.com" {
        return None;
    }

    let segments = parsed
        .path_segments()?
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>();
    if segments.len() < 2 {
        return None;
    }

    let owner = segments[0];
    let repo = segments[1].trim_end_matches(".git");
    if owner.is_empty() || repo.is_empty() {
        return None;
    }

    Some(format!("https://github.com/{owner}/{repo}"))
}

pub fn resolve_github_repo_url(
    repo_url: Option<String>,
    source_url: Option<String>,
) -> Option<String> {
    repo_url
        .as_deref()
        .and_then(normalize_github_repo_url)
        .or_else(|| source_url.as_deref().and_then(normalize_github_repo_url))
}

fn can_update_from_source(repo_url: Option<String>, source_url: Option<String>) -> bool {
    resolve_github_repo_url(repo_url, source_url).is_some()
}

fn risk_override_from_metadata(metadata_json: Option<String>) -> bool {
    metadata_json
        .and_then(|raw| serde_json::from_str::<Value>(&raw).ok())
        .and_then(|metadata| metadata.get("riskOverrideApplied").and_then(Value::as_bool))
        .unwrap_or(false)
}

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
    let persisted_source_type = normalize_persisted_source_type(&request.source_type);
    let persisted_source_market = normalize_persisted_source_market(request, persisted_source_type);

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
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, NULL, 0, 'managed', ?11, ?12, ?13, ?14, ?15, ?16)
        ",
        params![
            skill_id,
            request.slug,
            request.name,
            request.description,
            persisted_source_type,
            persisted_source_market,
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
                "sourceType": request.source_type,
                "repoUrl": request.repo_url,
                "downloadUrl": request.download_url,
                "packageRef": request.package_ref,
                "manifestPath": request.manifest_path,
                "skillRoot": request.skill_root,
                "requestedTargets": request.requested_targets,
            })
            .to_string(),
        ],
    )?;

    Ok(skill_id)
}

pub fn repository_skill_slug_exists(path: &Path, slug: &str) -> Result<bool> {
    let conn = open_connection(path)?;
    let exists = conn.query_row(
        "
        SELECT EXISTS(
            SELECT 1
            FROM skills
            WHERE slug = ?1 AND canonical_path IS NOT NULL
        )
        ",
        params![slug],
        |row| row.get::<_, i64>(0),
    )?;

    Ok(exists != 0)
}

pub fn load_skill_name(path: &Path, skill_id: &str) -> Result<String> {
    let conn = open_connection(path)?;
    conn.query_row(
        "SELECT name FROM skills WHERE id = ?1",
        params![skill_id],
        |row| row.get(0),
    )
    .map_err(Into::into)
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
    save_operation_log_in_connection(
        &conn,
        operation_type,
        entity_type,
        entity_id,
        status,
        summary,
        detail_json,
    )
}

fn save_operation_log_in_connection(
    conn: &Connection,
    operation_type: &str,
    entity_type: &str,
    entity_id: Option<&str>,
    status: &str,
    summary: &str,
    detail_json: Option<serde_json::Value>,
) -> Result<String> {
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

pub fn save_operation_log_in_tx(
    tx: &Transaction<'_>,
    operation_type: &str,
    entity_type: &str,
    entity_id: Option<&str>,
    status: &str,
    summary: &str,
    detail_json: Option<serde_json::Value>,
) -> Result<String> {
    let log_id = Uuid::new_v4().to_string();
    let now = OffsetDateTime::now_utc().unix_timestamp();

    tx.execute(
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
    pub source_url: Option<String>,
    pub repo_url: Option<String>,
    pub version: Option<String>,
    pub risk_override_applied: bool,
}

#[derive(Debug, Clone)]
pub struct RepositorySkillUpdateTarget {
    pub skill_id: String,
    pub name: String,
    pub slug: String,
    pub canonical_path: String,
    pub source_type: String,
    pub repo_url: String,
    pub manifest_path: Option<String>,
    pub skill_root: Option<String>,
    pub version: Option<String>,
    pub copy_distribution_count: usize,
}

type RepositorySkillUpdateRow = (
    String,
    String,
    String,
    String,
    String,
    Option<String>,
    Option<String>,
    Option<String>,
    i64,
);

fn build_repository_skill_update_target(
    row: RepositorySkillUpdateRow,
) -> Result<RepositorySkillUpdateTarget> {
    let (
        skill_id,
        slug,
        name,
        canonical_path,
        source_type,
        source_url,
        version,
        metadata_json,
        copy_distribution_count,
    ) = row;

    let repo_url = resolve_github_repo_url(
        repo_url_from_metadata(metadata_json.clone()),
        source_url.clone(),
    )
    .ok_or_else(|| anyhow!("skill does not have an updatable GitHub repository source"))?;

    Ok(RepositorySkillUpdateTarget {
        skill_id,
        name,
        slug,
        canonical_path,
        source_type,
        repo_url,
        manifest_path: metadata_string_field(metadata_json.clone(), "manifestPath"),
        skill_root: metadata_string_field(metadata_json.clone(), "skillRoot"),
        version,
        copy_distribution_count: copy_distribution_count.max(0) as usize,
    })
}

pub struct RepositorySkillRemovalPlan {
    pub skill_id: String,
    pub skill_name: String,
    pub canonical_path: String,
    pub distribution_paths: Vec<String>,
}

pub struct RepositoryStorageEntry {
    pub skill_id: String,
    pub slug: String,
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

pub fn update_skill_risk_override_state(
    path: &Path,
    skill_id: &str,
    risk_override_applied: bool,
) -> Result<()> {
    let conn = open_connection(path)?;
    let metadata_json: Option<String> = conn.query_row(
        "SELECT metadata_json FROM skills WHERE id = ?1",
        params![skill_id],
        |row| row.get(0),
    )?;

    let mut metadata = metadata_json
        .and_then(|raw| serde_json::from_str::<Value>(&raw).ok())
        .unwrap_or_else(|| json!({}));
    if !metadata.is_object() {
        metadata = json!({});
    }

    if let Some(object) = metadata.as_object_mut() {
        object.insert(
            "riskOverrideApplied".to_string(),
            Value::Bool(risk_override_applied),
        );
    }

    let now = OffsetDateTime::now_utc().unix_timestamp();
    conn.execute(
        "
        UPDATE skills
        SET metadata_json = ?2,
            updated_at = ?3
        WHERE id = ?1
        ",
        params![skill_id, metadata.to_string(), now],
    )?;

    Ok(())
}

pub fn list_installed_skills(path: &Path) -> Result<Vec<InstalledSkillSummary>> {
    let conn = open_connection(path)?;
    let mut stmt = conn.prepare(
        "
        SELECT id, name, canonical_path, source_url, version, metadata_json
        FROM skills
        WHERE canonical_path IS NOT NULL
        ORDER BY updated_at DESC
        ",
    )?;

    let rows = stmt.query_map([], |row| {
        let metadata_json: Option<String> = row.get(5)?;
        Ok(InstalledSkillSummary {
            skill_id: row.get(0)?,
            name: row.get(1)?,
            canonical_path: row.get(2)?,
            source_url: row.get(3)?,
            repo_url: repo_url_from_metadata(metadata_json.clone()),
            version: row.get(4)?,
            risk_override_applied: risk_override_from_metadata(metadata_json),
        })
    })?;

    rows.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(Into::into)
}

pub fn list_repository_storage_entries(
    path: &Path,
    canonical_store_dir: &Path,
) -> Result<Vec<RepositoryStorageEntry>> {
    let canonical_root = canonicalize_existing_path(canonical_store_dir)?;
    let conn = open_connection(path)?;
    let mut stmt = conn.prepare(
        "
        SELECT id, slug, canonical_path
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
        ))
    })?;

    let mut entries = Vec::new();
    for row in rows {
        let (skill_id, slug, raw_path) = row?;
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

        entries.push(RepositoryStorageEntry {
            skill_id,
            slug,
            canonical_path: canonical_skill_path.to_string_lossy().to_string(),
        });
    }

    Ok(entries)
}

pub fn rewrite_repository_storage_paths_with_connection(
    conn: &Connection,
    previous_root: &Path,
    current_root: &Path,
) -> Result<usize> {
    let previous_root = canonicalize_existing_path(previous_root)?;
    let current_root = canonicalize_existing_path(current_root)?;
    let now = OffsetDateTime::now_utc().unix_timestamp();
    let mut stmt = conn.prepare(
        "
        SELECT id, canonical_path
        FROM skills
        WHERE canonical_path IS NOT NULL
        ",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })?;

    let mut updates = Vec::new();
    for row in rows {
        let (skill_id, canonical_path) = row?;
        let current_path = fs::canonicalize(PathBuf::from(&canonical_path))
            .unwrap_or_else(|_| PathBuf::from(&canonical_path));
        let relative = match current_path.strip_prefix(&previous_root) {
            Ok(value) => value.to_path_buf(),
            Err(_) => continue,
        };
        updates.push((
            skill_id,
            current_root.join(relative).to_string_lossy().to_string(),
        ));
    }
    drop(stmt);

    for (skill_id, next_path) in &updates {
        conn.execute(
            "UPDATE skills SET canonical_path = ?2, updated_at = ?3 WHERE id = ?1",
            params![skill_id, next_path, now],
        )?;
    }

    Ok(updates.len())
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
            slug,
            name,
            description,
            source_type,
            source_market,
            source_url,
            installed_at,
            security_level,
            blocked,
            canonical_path,
            metadata_json
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
            row.get::<_, String>(4)?,
            row.get::<_, Option<String>>(5)?,
            row.get::<_, Option<String>>(6)?,
            row.get::<_, i64>(7)?,
            row.get::<_, String>(8)?,
            row.get::<_, i64>(9)? != 0,
            row.get::<_, String>(10)?,
            row.get::<_, Option<String>>(11)?,
        ))
    })?;

    let mut skills = Vec::new();
    for row in rows {
        let (
            id,
            slug,
            name,
            description,
            source_type,
            source_market,
            source_url,
            installed_at,
            security_level,
            blocked,
            raw_path,
            metadata_json,
        ) = row?;
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

        let risk_override_applied = risk_override_from_metadata(metadata_json.clone());
        skills.push(RepositorySkillSummary {
            id,
            slug,
            name,
            description,
            source_type,
            source_market,
            installed_at,
            security_level,
            blocked,
            risk_override_applied,
            can_update: can_update_from_source(
                repo_url_from_metadata(metadata_json.clone()),
                source_url,
            ),
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
            slug,
            name,
            description,
            canonical_path,
            source_type,
            source_market,
            source_url,
            installed_at,
            security_level,
            blocked,
            metadata_json
        FROM skills
        WHERE id = ?1 AND canonical_path IS NOT NULL
        ",
        params![skill_id],
        |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, Option<String>>(6)?,
                row.get::<_, Option<String>>(7)?,
                row.get::<_, i64>(8)?,
                row.get::<_, String>(9)?,
                row.get::<_, i64>(10)? != 0,
                row.get::<_, Option<String>>(11)?,
            ))
        },
    )?;

    let (
        id,
        slug,
        name,
        description,
        canonical_path,
        source_type,
        source_market,
        source_url,
        installed_at,
        security_level,
        blocked,
        metadata_json,
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
    let display_source_url = display_source_url(&source_type, source_url);
    let can_update = can_update_from_source(
        repo_url_from_metadata(metadata_json.clone()),
        display_source_url.clone(),
    );

    Ok(RepositorySkillDetail {
        id,
        slug,
        name,
        description,
        canonical_path: display_path(&canonical_skill_dir.to_string_lossy()),
        source_type,
        source_market,
        source_url: display_source_url,
        installed_at,
        security_level,
        blocked,
        risk_override_applied: risk_override_from_metadata(metadata_json),
        can_update,
        skill_markdown,
    })
}

pub fn load_repository_skill_update_target(
    path: &Path,
    skill_id: &str,
) -> Result<RepositorySkillUpdateTarget> {
    let conn = open_connection(path)?;
    let row = conn.query_row(
        "
        SELECT
            id,
            slug,
            name,
            canonical_path,
            source_type,
            source_url,
            version,
            metadata_json,
            (
                SELECT COUNT(*)
                FROM skill_distributions
                WHERE skill_id = skills.id AND install_mode = 'copy'
            ) AS copy_distribution_count
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
                row.get::<_, String>(4)?,
                row.get::<_, Option<String>>(5)?,
                row.get::<_, Option<String>>(6)?,
                row.get::<_, Option<String>>(7)?,
                row.get::<_, i64>(8)?,
            ))
        },
    )?;
    build_repository_skill_update_target(row)
}

pub fn list_repository_skill_update_targets(
    path: &Path,
) -> Result<Vec<RepositorySkillUpdateTarget>> {
    let conn = open_connection(path)?;
    let mut stmt = conn.prepare(
        "
        SELECT
            id,
            slug,
            name,
            canonical_path,
            source_type,
            source_url,
            version,
            metadata_json,
            COALESCE(copy_stats.copy_distribution_count, 0)
        FROM skills
        LEFT JOIN (
            SELECT
                skill_id,
                COUNT(*) AS copy_distribution_count
            FROM skill_distributions
            WHERE install_mode = 'copy'
            GROUP BY skill_id
        ) AS copy_stats ON copy_stats.skill_id = skills.id
        WHERE canonical_path IS NOT NULL
        ORDER BY updated_at DESC, name ASC
        ",
    )?;

    let mut targets = Vec::new();
    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, String>(4)?,
            row.get::<_, Option<String>>(5)?,
            row.get::<_, Option<String>>(6)?,
            row.get::<_, Option<String>>(7)?,
            row.get::<_, i64>(8)?,
        ))
    })?;

    for row in rows {
        if let Ok(target) = build_repository_skill_update_target(row?) {
            targets.push(target);
        }
    }

    Ok(targets)
}

pub struct UpdateRepositorySkillRecordInput {
    pub description: Option<String>,
    pub version: Option<String>,
    pub author: Option<String>,
    pub source_url: String,
    pub repo_url: String,
    pub download_url: Option<String>,
    pub package_ref: Option<String>,
    pub manifest_path: Option<String>,
    pub skill_root: Option<String>,
    pub security_level: String,
    pub blocked: bool,
    pub scanned_at: i64,
}

pub fn update_repository_skill_record_in_tx(
    tx: &Transaction<'_>,
    skill_id: &str,
    input: &UpdateRepositorySkillRecordInput,
) -> Result<()> {
    let updated_at = OffsetDateTime::now_utc().unix_timestamp();

    let metadata_json: Option<String> = tx.query_row(
        "SELECT metadata_json FROM skills WHERE id = ?1",
        params![skill_id],
        |row| row.get(0),
    )?;
    let mut metadata = metadata_from_json(metadata_json);
    metadata["repoUrl"] = Value::String(input.repo_url.clone());
    metadata["downloadUrl"] = input
        .download_url
        .clone()
        .map(Value::String)
        .unwrap_or(Value::Null);
    metadata["packageRef"] = input
        .package_ref
        .clone()
        .map(Value::String)
        .unwrap_or(Value::Null);
    metadata["manifestPath"] = input
        .manifest_path
        .clone()
        .map(Value::String)
        .unwrap_or(Value::Null);
    metadata["skillRoot"] = input
        .skill_root
        .clone()
        .map(Value::String)
        .unwrap_or(Value::Null);
    metadata["riskOverrideApplied"] = Value::Bool(false);

    tx.execute(
        "
        UPDATE skills
        SET
            description = ?2,
            source_url = ?3,
            version = ?4,
            author = ?5,
            security_level = ?6,
            blocked = ?7,
            updated_at = ?8,
            last_scanned_at = ?9,
            metadata_json = ?10
        WHERE id = ?1
        ",
        params![
            skill_id,
            input.description,
            input.source_url,
            input.version,
            input.author,
            input.security_level,
            input.blocked as i64,
            updated_at,
            input.scanned_at,
            metadata.to_string(),
        ],
    )?;

    Ok(())
}

pub fn load_repository_skill_removal_plan(
    path: &Path,
    canonical_store_dir: &Path,
    skill_id: &str,
) -> Result<RepositorySkillRemovalPlan> {
    let canonical_root = canonicalize_existing_path(canonical_store_dir)?;
    let conn = open_connection(path)?;
    let (skill_name, canonical_path): (String, String) = conn.query_row(
        "SELECT name, canonical_path FROM skills WHERE id = ?1 AND canonical_path IS NOT NULL",
        params![skill_id],
        |row| Ok((row.get(0)?, row.get(1)?)),
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
    let distribution_paths = rows
        .collect::<rusqlite::Result<Vec<_>>>()?
        .into_iter()
        .map(|path| display_path(&path))
        .collect::<Vec<_>>();

    Ok(RepositorySkillRemovalPlan {
        skill_id: skill_id.to_string(),
        skill_name,
        canonical_path: display_path(&canonical_skill_dir.to_string_lossy()),
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

    fn seed_skill(
        root: &Path,
        source_type: &str,
        source_market: Option<&str>,
    ) -> (PathBuf, String) {
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
            source_type: "catalog".into(),
            source_url: "https://example.com/demo".into(),
            repo_url: Some("https://example.com/demo".into()),
            download_url: None,
            package_ref: Some("example/demo@skills/demo-skill".into()),
            manifest_path: Some("skills/demo-skill/SKILL.md".into()),
            skill_root: Some("skills/demo-skill".into()),
            name: "Demo Skill".into(),
            slug: "demo-skill".into(),
            description: Some("Improve confusing UX copy and labels.".into()),
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
        assert_eq!(
            skills[0].description.as_deref(),
            Some("Improve confusing UX copy and labels.")
        );
    }

    #[test]
    fn loads_repository_skill_detail_with_markdown() {
        let dir = tempdir().unwrap();
        let (db_path, skill_id) = seed_skill(dir.path(), "market", Some("github"));
        let canonical_store_dir = dir.path().join("app-data").join("skills");

        let detail =
            get_repository_skill_detail(&db_path, &canonical_store_dir, &skill_id).unwrap();

        assert_eq!(detail.id, skill_id);
        assert!(detail.skill_markdown.contains("demo skill"));
        assert_eq!(detail.source_market.as_deref(), Some("github"));
        assert_eq!(
            detail.description.as_deref(),
            Some("Improve confusing UX copy and labels.")
        );
    }

    #[test]
    fn marks_only_github_backed_skills_as_updatable() {
        let dir = tempdir().unwrap();
        let (db_path, github_skill_id) = seed_skill(dir.path(), "market", Some("github"));
        let (_db_path, local_skill_id) = seed_skill(dir.path(), "local", None);
        let canonical_store_dir = dir.path().join("app-data").join("skills");
        let conn = open_connection(&db_path).unwrap();
        conn.execute(
            "UPDATE skills SET source_url = ?2, metadata_json = ?3 WHERE id = ?1",
            params![
                github_skill_id,
                "https://github.com/demo/repo/tree/main/skills/demo-skill",
                json!({
                    "repoUrl": "https://github.com/demo/repo",
                    "manifestPath": "skills/demo-skill/SKILL.md",
                    "skillRoot": "skills/demo-skill",
                })
                .to_string(),
            ],
        )
        .unwrap();
        conn.execute(
            "UPDATE skills SET source_url = ?2, metadata_json = ?3 WHERE id = ?1",
            params![
                local_skill_id,
                "E:/skills/demo-skill",
                json!({
                    "repoUrl": Value::Null,
                })
                .to_string(),
            ],
        )
        .unwrap();

        let skills = list_repository_skills(&db_path, &canonical_store_dir).unwrap();
        let github_skill = skills
            .iter()
            .find(|item| item.id == github_skill_id)
            .unwrap();
        let local_skill = skills
            .iter()
            .find(|item| item.id == local_skill_id)
            .unwrap();
        assert!(github_skill.can_update);
        assert!(!local_skill.can_update);

        let github_detail =
            get_repository_skill_detail(&db_path, &canonical_store_dir, &github_skill_id).unwrap();
        let local_detail =
            get_repository_skill_detail(&db_path, &canonical_store_dir, &local_skill_id).unwrap();
        assert!(github_detail.can_update);
        assert!(!local_detail.can_update);
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
                category_breakdown: Vec::new(),
                blocking_reasons: Vec::new(),
                engine_version: "phase2-rules-v1".into(),
                scanned_at: 100,
            },
        )
        .unwrap();

        delete_repository_skill(&db_path, &skill_id).unwrap();
        let conn = open_connection(&db_path).unwrap();
        let skill_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM skills WHERE id = ?1",
                params![skill_id],
                |row| row.get(0),
            )
            .unwrap();
        let distribution_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM skill_distributions", [], |row| {
                row.get(0)
            })
            .unwrap();
        let report_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM security_reports", [], |row| {
                row.get(0)
            })
            .unwrap();

        assert_eq!(skill_count, 0);
        assert_eq!(distribution_count, 0);
        assert_eq!(report_count, 0);
    }
}
