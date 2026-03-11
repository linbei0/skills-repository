use anyhow::Result;
use rusqlite::Connection;
use std::path::Path;

const CURRENT_SCHEMA: &str = "
        CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value_json TEXT NOT NULL,
            updated_at INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS skills (
            id TEXT PRIMARY KEY,
            slug TEXT,
            name TEXT NOT NULL,
            description TEXT,
            source_type TEXT NOT NULL,
            source_market TEXT,
            source_url TEXT,
            version TEXT,
            author TEXT,
            canonical_path TEXT,
            file_hash TEXT,
            size_bytes INTEGER NOT NULL DEFAULT 0,
            management_mode TEXT NOT NULL DEFAULT 'unmanaged',
            security_level TEXT NOT NULL DEFAULT 'unknown',
            blocked INTEGER NOT NULL DEFAULT 0,
            installed_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            last_scanned_at INTEGER,
            metadata_json TEXT
        );

        CREATE TABLE IF NOT EXISTS skill_distributions (
            id TEXT PRIMARY KEY,
            skill_id TEXT NOT NULL,
            target_kind TEXT NOT NULL,
            target_agent TEXT NOT NULL,
            target_path TEXT NOT NULL,
            install_mode TEXT NOT NULL,
            status TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS templates (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            description TEXT,
            tags_json TEXT NOT NULL,
            target_agents_json TEXT NOT NULL,
            scope TEXT NOT NULL,
            is_builtin INTEGER NOT NULL DEFAULT 0,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS template_items (
            id TEXT PRIMARY KEY,
            template_id TEXT NOT NULL,
            skill_ref_type TEXT NOT NULL,
            skill_ref TEXT NOT NULL,
            display_name TEXT,
            required INTEGER NOT NULL DEFAULT 1,
            order_index INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS operation_logs (
            id TEXT PRIMARY KEY,
            operation_type TEXT NOT NULL,
            entity_type TEXT NOT NULL,
            entity_id TEXT,
            status TEXT NOT NULL,
            summary TEXT NOT NULL,
            detail_json TEXT,
            created_at INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS security_reports (
            id TEXT PRIMARY KEY,
            skill_id TEXT,
            scan_scope TEXT NOT NULL,
            level TEXT NOT NULL,
            score INTEGER NOT NULL,
            blocked INTEGER NOT NULL DEFAULT 0,
            issues_json TEXT NOT NULL,
            recommendations_json TEXT NOT NULL,
            scanned_files_json TEXT NOT NULL,
            engine_version TEXT NOT NULL,
            scanned_at INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS market_cache (
            key TEXT PRIMARY KEY,
            source_market TEXT NOT NULL,
            query TEXT NOT NULL,
            page INTEGER NOT NULL,
            payload_json TEXT NOT NULL,
            expires_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_skills_slug ON skills(slug);
        CREATE INDEX IF NOT EXISTS idx_skill_distributions_target_path ON skill_distributions(target_path);
        CREATE INDEX IF NOT EXISTS idx_security_reports_skill_id ON security_reports(skill_id);
        CREATE INDEX IF NOT EXISTS idx_security_reports_scanned_at ON security_reports(scanned_at);
        CREATE INDEX IF NOT EXISTS idx_market_cache_source_market ON market_cache(source_market);
        CREATE INDEX IF NOT EXISTS idx_market_cache_expires_at ON market_cache(expires_at);
";

pub fn open_connection(path: &Path) -> Result<Connection> {
    let conn = Connection::open(path)?;
    conn.execute_batch(
        "
        PRAGMA foreign_keys = ON;
        PRAGMA journal_mode = WAL;
        PRAGMA synchronous = NORMAL;
        ",
    )?;
    Ok(conn)
}

pub fn run_migrations(path: &Path) -> Result<()> {
    let conn = open_connection(path)?;
    conn.execute_batch(CURRENT_SCHEMA)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::OptionalExtension;
    use tempfile::tempdir;

    fn table_exists(conn: &Connection, table_name: &str) -> bool {
        conn.query_row(
            "SELECT name FROM sqlite_master WHERE type = 'table' AND name = ?1",
            [table_name],
            |row| row.get::<_, String>(0),
        )
        .optional()
        .unwrap()
        .is_some()
    }

    fn column_exists(conn: &Connection, table_name: &str, column_name: &str) -> bool {
        let mut stmt = conn
            .prepare(&format!("PRAGMA table_info({table_name})"))
            .unwrap();

        stmt.query_map([], |row| row.get::<_, String>(1))
            .unwrap()
            .collect::<rusqlite::Result<Vec<_>>>()
            .unwrap()
            .into_iter()
            .any(|column| column == column_name)
    }

    #[test]
    fn creates_current_schema_on_fresh_database() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("fresh.db");

        run_migrations(&db_path).unwrap();
        let conn = open_connection(&db_path).unwrap();

        assert!(table_exists(&conn, "settings"));
        assert!(table_exists(&conn, "skills"));
        assert!(table_exists(&conn, "skill_distributions"));
        assert!(table_exists(&conn, "templates"));
        assert!(table_exists(&conn, "template_items"));
        assert!(table_exists(&conn, "operation_logs"));
        assert!(table_exists(&conn, "security_reports"));
        assert!(table_exists(&conn, "market_cache"));
        assert!(!table_exists(&conn, "projects"));
    }

    #[test]
    fn current_distribution_schema_has_no_project_id_column() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("distribution.db");

        run_migrations(&db_path).unwrap();
        let conn = open_connection(&db_path).unwrap();

        assert!(!column_exists(&conn, "skill_distributions", "project_id"));
    }

    #[test]
    fn run_migrations_is_idempotent() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("idempotent.db");

        run_migrations(&db_path).unwrap();
        run_migrations(&db_path).unwrap();

        let conn = open_connection(&db_path).unwrap();
        assert!(table_exists(&conn, "security_reports"));
        assert!(table_exists(&conn, "market_cache"));
    }
}
