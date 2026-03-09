mod adapters;
mod commands;
mod domain;
mod repositories;
mod security;
mod services;
mod tasks;
mod utils;

use commands::app::{
    bootstrap_app, delete_template, distribute_skill, get_security_reports, get_settings,
    get_template, inject_template, install_skill, list_templates, rescan_security, save_settings,
    save_template, scan_skills, search_market_skills,
};
use domain::app_state::AppState;
use repositories::db::run_migrations;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            let app_handle = app.handle();
            let state = AppState::new(&app_handle)?;
            run_migrations(&state.paths.db_file)?;
            app.manage(state);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            bootstrap_app,
            distribute_skill,
            delete_template,
            get_settings,
            get_security_reports,
            get_template,
            install_skill,
            inject_template,
            list_templates,
            rescan_security,
            save_settings,
            save_template,
            search_market_skills,
            scan_skills,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
