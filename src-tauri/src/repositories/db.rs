use anyhow::Result;
use rusqlite::Connection;
use rusqlite_migration::{Migrations, M};
use std::path::Path;

const INITIAL_SCHEMA_MIGRATION: &str = "
        CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value_json TEXT NOT NULL,
            updated_at INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS projects (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            root_path TEXT NOT NULL UNIQUE,
            labels_json TEXT,
            created_at INTEGER NOT NULL,
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
            project_id TEXT,
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

        CREATE INDEX IF NOT EXISTS idx_projects_root_path ON projects(root_path);
        CREATE INDEX IF NOT EXISTS idx_skills_slug ON skills(slug);
        CREATE INDEX IF NOT EXISTS idx_skill_distributions_target_path ON skill_distributions(target_path);
";

const PHASE_TWO_SCHEMA_MIGRATION: &str = "
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

        CREATE INDEX IF NOT EXISTS idx_security_reports_skill_id ON security_reports(skill_id);
        CREATE INDEX IF NOT EXISTS idx_security_reports_scanned_at ON security_reports(scanned_at);
        CREATE INDEX IF NOT EXISTS idx_market_cache_source_market ON market_cache(source_market);
        CREATE INDEX IF NOT EXISTS idx_market_cache_expires_at ON market_cache(expires_at);
";

const PHASE_THREE_SCHEMA_MIGRATION: &str = "
        PRAGMA foreign_keys = OFF;

        CREATE TABLE IF NOT EXISTS operation_logs_v3 (
            id TEXT PRIMARY KEY,
            operation_type TEXT NOT NULL,
            entity_type TEXT NOT NULL,
            entity_id TEXT,
            status TEXT NOT NULL,
            summary TEXT NOT NULL,
            detail_json TEXT,
            created_at INTEGER NOT NULL
        );

        INSERT INTO operation_logs_v3 (
            id,
            operation_type,
            entity_type,
            entity_id,
            status,
            summary,
            detail_json,
            created_at
        )
        SELECT
            id,
            operation_type,
            entity_type,
            entity_id,
            status,
            summary,
            detail_json,
            created_at
        FROM operation_logs;

        DROP TABLE operation_logs;
        ALTER TABLE operation_logs_v3 RENAME TO operation_logs;

        PRAGMA foreign_keys = ON;
";

const MIGRATIONS: &[M<'_>] = &[
    M::up(INITIAL_SCHEMA_MIGRATION),
    M::up(PHASE_TWO_SCHEMA_MIGRATION),
    M::up(PHASE_THREE_SCHEMA_MIGRATION),
];

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
    let mut conn = open_connection(path)?;
    let migrations = Migrations::new(MIGRATIONS.to_vec());
    migrations.to_latest(&mut conn)?;
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

    #[test]
    fn creates_phase_two_tables_on_fresh_database() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("fresh.db");

        run_migrations(&db_path).unwrap();
        let conn = open_connection(&db_path).unwrap();

        assert!(table_exists(&conn, "market_cache"));
        assert!(table_exists(&conn, "security_reports"));
    }

    #[test]
    fn upgrades_existing_phase_one_database_to_phase_two_schema() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("upgraded.db");
        let conn = open_connection(&db_path).unwrap();

        conn.execute_batch(INITIAL_SCHEMA_MIGRATION).unwrap();
        conn.pragma_update(None, "user_version", 1).unwrap();

        run_migrations(&db_path).unwrap();
        let upgraded = open_connection(&db_path).unwrap();

        assert!(table_exists(&upgraded, "market_cache"));
        assert!(table_exists(&upgraded, "security_reports"));

        let user_version: i64 = upgraded
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .unwrap();
        assert_eq!(user_version, 3);
    }
}
