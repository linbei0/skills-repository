use anyhow::Result;
use rusqlite::{params, OptionalExtension};
use std::path::Path;
use time::OffsetDateTime;

use crate::domain::types::MarketSearchResponse;

use super::db::open_connection;

const CACHE_TTL_SECONDS: i64 = 600;

fn cache_key(provider: &str, query: &str, page: u32, page_size: u32) -> String {
    format!(
        "{}:{}:{}:{}",
        provider,
        query.trim().to_ascii_lowercase(),
        page,
        page_size
    )
}

pub fn load_cached_search(
    path: &Path,
    provider: &str,
    query: &str,
    page: u32,
    page_size: u32,
) -> Result<Option<MarketSearchResponse>> {
    let conn = open_connection(path)?;
    let key = cache_key(provider, query, page, page_size);
    let now = OffsetDateTime::now_utc().unix_timestamp();

    let payload = conn
        .query_row(
            "
            SELECT payload_json
            FROM market_cache
            WHERE key = ?1 AND expires_at >= ?2
            ",
            params![key, now],
            |row| row.get::<_, String>(0),
        )
        .optional()?;

    payload
        .map(|json| serde_json::from_str::<MarketSearchResponse>(&json).map_err(Into::into))
        .transpose()
}

pub fn save_cached_search(
    path: &Path,
    provider: &str,
    query: &str,
    page: u32,
    page_size: u32,
    payload: &MarketSearchResponse,
) -> Result<()> {
    let conn = open_connection(path)?;
    let now = OffsetDateTime::now_utc().unix_timestamp();
    let expires_at = now + CACHE_TTL_SECONDS;
    let key = cache_key(provider, query, page, page_size);
    let payload_json = serde_json::to_string(payload)?;

    conn.execute(
        "
        INSERT INTO market_cache (key, source_market, query, page, payload_json, expires_at, updated_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
        ON CONFLICT(key) DO UPDATE SET
            payload_json = excluded.payload_json,
            expires_at = excluded.expires_at,
            updated_at = excluded.updated_at
        ",
        params![key, provider, query, page, payload_json, expires_at, now],
    )?;

    Ok(())
}
