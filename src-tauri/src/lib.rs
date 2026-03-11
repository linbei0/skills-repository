mod adapters;
mod commands;
mod domain;
mod repositories;
mod security;
mod services;

use commands::app::{
    batch_distribute_repository_skills, bootstrap_app, delete_template, distribute_skill,
    get_repository_skill_detail, get_security_reports, get_template, import_repository_skill,
    inject_template, install_skill, list_repository_skills, list_templates, rescan_security,
    resolve_repository_import_source, save_settings, save_template, scan_agent_global_skills,
    search_market_skills, uninstall_repository_skill,
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
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            bootstrap_app,
            batch_distribute_repository_skills,
            distribute_skill,
            delete_template,
            get_repository_skill_detail,
            get_security_reports,
            get_template,
            import_repository_skill,
            inject_template,
            install_skill,
            list_repository_skills,
            list_templates,
            rescan_security,
            resolve_repository_import_source,
            save_settings,
            save_template,
            scan_agent_global_skills,
            search_market_skills,
            uninstall_repository_skill,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
