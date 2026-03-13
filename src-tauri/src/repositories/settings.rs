use anyhow::Result;
use rusqlite::{params, Connection, OptionalExtension};
use std::path::Path;
use time::OffsetDateTime;

use crate::domain::types::AppSettings;

use super::db::open_connection;

pub(crate) const SETTINGS_KEY: &str = "app_settings";

pub fn load_settings(path: &Path) -> Result<Option<AppSettings>> {
    let conn = open_connection(path)?;
    let value = conn
        .query_row(
            "SELECT value_json FROM settings WHERE key = ?1",
            params![SETTINGS_KEY],
            |row| row.get::<_, String>(0),
        )
        .optional()?;

    value
        .map(|json| serde_json::from_str::<AppSettings>(&json).map_err(Into::into))
        .transpose()
}

pub fn save_settings(path: &Path, settings: &AppSettings) -> Result<AppSettings> {
    let conn = open_connection(path)?;
    save_settings_with_connection(&conn, settings)
}

pub(crate) fn save_settings_with_connection(
    conn: &Connection,
    settings: &AppSettings,
) -> Result<AppSettings> {
    let now = OffsetDateTime::now_utc().unix_timestamp();
    let json = serde_json::to_string(settings)?;

    conn.execute(
        "
        INSERT INTO settings (key, value_json, updated_at)
        VALUES (?1, ?2, ?3)
        ON CONFLICT(key) DO UPDATE SET
            value_json = excluded.value_json,
            updated_at = excluded.updated_at
        ",
        params![SETTINGS_KEY, json, now],
    )?;

    Ok(settings.clone())
}

pub fn default_settings(language: String) -> AppSettings {
    AppSettings {
        language,
        theme_mode: "system".into(),
        visible_skills_target_ids: vec![
            "universal".into(),
            "antigravity".into(),
            "claude-code".into(),
            "codebuddy".into(),
            "kiro-cli".into(),
            "openclaw".into(),
            "qoder".into(),
            "trae".into(),
            "windsurf".into(),
        ],
        custom_skills_targets: Vec::new(),
        repository_storage_path: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repositories::db::run_migrations;
    use tempfile::tempdir;

    #[test]
    fn saves_and_loads_settings() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("settings.db");
        run_migrations(&db_path).unwrap();

        let settings = AppSettings {
            language: "zh-CN".into(),
            theme_mode: "dark".into(),
            visible_skills_target_ids: vec!["universal".into(), "qoder".into()],
            custom_skills_targets: vec![crate::domain::types::CustomSkillsTarget {
                id: "custom-demo".into(),
                label: "Demo IDE".into(),
                relative_path: ".demo/skills".into(),
            }],
            repository_storage_path: Some("D:/skills-repository".into()),
        };

        save_settings(&db_path, &settings).unwrap();
        let loaded = load_settings(&db_path).unwrap().unwrap();

        assert_eq!(loaded.language, "zh-CN");
        assert_eq!(loaded.theme_mode, "dark");
        assert_eq!(loaded.visible_skills_target_ids.len(), 2);
        assert_eq!(loaded.custom_skills_targets.len(), 1);
        assert_eq!(
            loaded.repository_storage_path.as_deref(),
            Some("D:/skills-repository")
        );
    }
}
