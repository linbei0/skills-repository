use tauri::{AppHandle, State};

use crate::{
    domain::{
        app_state::AppState,
        types::{AppSettings, ScanSkillsRequest, TaskHandle},
    },
    services::{bootstrap, scan, settings},
    tasks,
};

fn log_task_emit_error(stage: &str, result: anyhow::Result<()>) {
    if let Err(error) = result {
        log::error!("task event emit failed at {}: {}", stage, error);
    }
}

#[tauri::command]
pub fn bootstrap_app(
    state: State<'_, AppState>,
) -> Result<crate::domain::types::BootstrapPayload, String> {
    log::info!("bootstrap_app invoked");
    bootstrap::bootstrap_payload(&state, env!("CARGO_PKG_VERSION").to_string())
        .map(|payload| {
            log::info!("bootstrap_app resolved");
            payload
        })
        .map_err(|error| {
            log::error!("bootstrap_app failed: {}", error);
            error.to_string()
        })
}

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> Result<AppSettings, String> {
    log::info!("get_settings invoked");
    settings::get_settings(
        &state,
        bootstrap::normalize_language(&bootstrap::system_locale()),
    )
    .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn save_settings(
    state: State<'_, AppState>,
    settings: AppSettings,
) -> Result<AppSettings, String> {
    log::info!("save_settings invoked");
    settings::save_settings(&state, &settings).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn scan_skills(
    app: AppHandle,
    state: State<'_, AppState>,
    request: ScanSkillsRequest,
) -> Result<TaskHandle, String> {
    log::info!("scan_skills invoked");
    let task = tasks::new_task_handle("scan");
    let app_handle = app.clone();
    let task_handle = task.clone();
    let state = state.inner().clone();

    log_task_emit_error(
        "scan.queued",
        tasks::emit_progress(&app, &task, "queued", "prepare", 0, 3, "Scan task queued"),
    );

    tauri::async_runtime::spawn(async move {
        log_task_emit_error(
            "scan.running",
            tasks::emit_progress(
                &app_handle,
                &task_handle,
                "running",
                "scan",
                1,
                3,
                "Scanning configured skill roots",
            ),
        );

        match scan::scan_skills(state.agent_registry.as_ref(), &request) {
            Ok(result) => {
                log_task_emit_error(
                    "scan.persist",
                    tasks::emit_progress(
                        &app_handle,
                        &task_handle,
                        "running",
                        "persist",
                        2,
                        3,
                        "Persisting scan snapshot to SQLite",
                    ),
                );

                match scan::persist_scan_snapshot(&state.paths.db_file, &result) {
                    Ok(snapshot) => {
                        log_task_emit_error(
                            "scan.completed",
                            tasks::emit_completed(
                                &app_handle,
                                &task_handle,
                                "cleanup",
                                "Scan completed",
                                snapshot,
                            ),
                        );
                    }
                    Err(error) => {
                        log_task_emit_error(
                            "scan.persist_failed",
                            tasks::emit_failed(
                                &app_handle,
                                &task_handle,
                                "persist",
                                &error.to_string(),
                            ),
                        );
                    }
                }
            }
            Err(error) => {
                log_task_emit_error(
                    "scan.failed",
                    tasks::emit_failed(&app_handle, &task_handle, "scan", &error.to_string()),
                );
            }
        }
    });

    Ok(task)
}
