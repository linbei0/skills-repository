use super::agent_registry::AgentRegistry;
use anyhow::{Context, Result};
use std::{path::PathBuf, sync::Arc};
use tauri::{AppHandle, Manager};

#[derive(Debug, Clone)]
pub struct AppPaths {
    pub db_file: PathBuf,
    pub temp_dir: PathBuf,
    pub canonical_store_dir: PathBuf,
}

impl AppPaths {
    pub fn from_app(app: &AppHandle) -> Result<Self> {
        let app_data_dir = app
            .path()
            .app_data_dir()
            .context("failed to resolve app data directory")?;
        let db_dir = app_data_dir.join("db");
        let cache_dir = app_data_dir.join("cache").join("market");
        let temp_dir = app_data_dir.join("tmp").join("tasks");
        let canonical_store_dir = app_data_dir.join("skills");
        let db_file = db_dir.join("skills-manager.db");

        std::fs::create_dir_all(&db_dir)?;
        std::fs::create_dir_all(&cache_dir)?;
        std::fs::create_dir_all(&temp_dir)?;
        std::fs::create_dir_all(&canonical_store_dir)?;

        Ok(Self {
            db_file,
            temp_dir,
            canonical_store_dir,
        })
    }
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub paths: AppPaths,
    pub agent_registry: Arc<AgentRegistry>,
}

impl AppState {
    pub fn new(app: &AppHandle) -> Result<Self> {
        Ok(Self {
            paths: AppPaths::from_app(app)?,
            agent_registry: Arc::new(AgentRegistry::new()),
        })
    }
}
