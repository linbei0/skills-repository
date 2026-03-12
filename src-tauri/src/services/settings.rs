use anyhow::Result;

use crate::{
    domain::{app_state::AppState, types::AppSettings},
    repositories::settings as settings_repo,
};

pub fn load_or_create_settings(state: &AppState, language: String) -> Result<AppSettings> {
    if let Some(settings) = settings_repo::load_settings(&state.paths.db_file)? {
        return Ok(settings);
    }

    let settings = settings_repo::default_settings(language);
    settings_repo::save_settings(&state.paths.db_file, &settings)
}

pub fn save_settings(state: &AppState, settings: &AppSettings) -> Result<AppSettings> {
    settings_repo::save_settings(&state.paths.db_file, settings)
}
