use anyhow::Result;
use rusqlite::Connection;
use rusqlite_migration::{Migrations, M};
use std::path::Path;

const MIGRATIONS: &[M<'_>] = &[
    M::up(
        "
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
            task_id TEXT,
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
        ",
    ),
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
