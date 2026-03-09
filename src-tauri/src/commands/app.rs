use tauri::{AppHandle, State};

use crate::{
    domain::{
        app_state::AppState,
        types::{
            AppSettings, DistributionRequest, InstallSkillRequest, MarketSearchRequest,
            MarketSearchResponse, SaveTemplateRequest, ScanSkillsRequest, SecurityReport,
            TaskHandle, TemplateInjectionRequest, TemplateRecord,
        },
    },
    repositories::security as security_repository,
    services::{
        bootstrap, distribution, install, market, scan, settings, template_injection, templates,
    },
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
pub fn search_market_skills(
    state: State<'_, AppState>,
    request: MarketSearchRequest,
) -> Result<MarketSearchResponse, String> {
    log::info!("search_market_skills invoked");
    market::search_market_skills(&state.paths.db_file, &request).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn list_templates(state: State<'_, AppState>) -> Result<Vec<TemplateRecord>, String> {
    log::info!("list_templates invoked");
    templates::list_templates(&state.paths.db_file).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn get_template(
    state: State<'_, AppState>,
    template_id: String,
) -> Result<Option<TemplateRecord>, String> {
    log::info!("get_template invoked");
    templates::get_template(&state.paths.db_file, &template_id).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn save_template(
    state: State<'_, AppState>,
    request: SaveTemplateRequest,
) -> Result<TemplateRecord, String> {
    log::info!("save_template invoked");
    templates::save_template(&state.paths.db_file, &request).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn delete_template(state: State<'_, AppState>, template_id: String) -> Result<(), String> {
    log::info!("delete_template invoked");
    templates::delete_template(&state.paths.db_file, &template_id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn install_skill(
    app: AppHandle,
    state: State<'_, AppState>,
    request: InstallSkillRequest,
) -> Result<TaskHandle, String> {
    log::info!("install_skill invoked");
    let task = tasks::new_task_handle("install");
    let task_handle = task.clone();
    let app_handle = app.clone();
    let state = state.inner().clone();

    log_task_emit_error(
        "install.queued",
        tasks::emit_progress(
            &app,
            &task,
            "queued",
            "prepare",
            0,
            4,
            "Install task queued",
        ),
    );

    tauri::async_runtime::spawn(async move {
        log_task_emit_error(
            "install.download",
            tasks::emit_progress(
                &app_handle,
                &task_handle,
                "running",
                "download",
                1,
                4,
                "Downloading market skill source",
            ),
        );
        log_task_emit_error(
            "install.security_check",
            tasks::emit_progress(
                &app_handle,
                &task_handle,
                "running",
                "security_check",
                2,
                4,
                "Running security pre-scan before canonical store",
            ),
        );
        log_task_emit_error(
            "install.persist",
            tasks::emit_progress(
                &app_handle,
                &task_handle,
                "running",
                "persist",
                3,
                4,
                "Persisting installed skill into canonical store and SQLite",
            ),
        );

        match install::install_skill(&state.paths, &task_handle.task_id, &request) {
            Ok(result) => {
                let step = if result.blocked {
                    "security_check"
                } else {
                    "cleanup"
                };
                let message = if result.blocked {
                    "Install blocked by security pre-scan"
                } else {
                    "Install completed"
                };

                log_task_emit_error(
                    "install.completed",
                    tasks::emit_completed(&app_handle, &task_handle, step, message, result),
                );
            }
            Err(error) => {
                log_task_emit_error(
                    "install.failed",
                    tasks::emit_failed(&app_handle, &task_handle, "cleanup", &error.to_string()),
                );
            }
        }
    });

    Ok(task)
}

#[tauri::command]
pub fn get_security_reports(state: State<'_, AppState>) -> Result<Vec<SecurityReport>, String> {
    log::info!("get_security_reports invoked");
    security_repository::list_security_reports(&state.paths.db_file)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn rescan_security(app: AppHandle, state: State<'_, AppState>) -> Result<TaskHandle, String> {
    log::info!("rescan_security invoked");
    let task = tasks::new_task_handle("rescan_security");
    let task_handle = task.clone();
    let app_handle = app.clone();
    let state = state.inner().clone();

    log_task_emit_error(
        "security.rescan.queued",
        tasks::emit_progress(
            &app,
            &task,
            "queued",
            "prepare",
            0,
            3,
            "Security rescan task queued",
        ),
    );

    tauri::async_runtime::spawn(async move {
        log_task_emit_error(
            "security.rescan.running",
            tasks::emit_progress(
                &app_handle,
                &task_handle,
                "running",
                "security_check",
                1,
                3,
                "Rescanning installed skills for security issues",
            ),
        );

        let result = (|| -> anyhow::Result<Vec<SecurityReport>> {
            let installed =
                crate::repositories::skills::list_installed_skills(&state.paths.db_file)?;
            let mut reports = Vec::new();

            for skill in installed {
                let report = crate::security::scan_skill_directory(
                    std::path::Path::new(&skill.canonical_path),
                    Some(skill.skill_id.clone()),
                    "rescan",
                )?;
                let mut persisted = report.clone();
                persisted.skill_name = Some(skill.name.clone());
                persisted.source_path = Some(skill.canonical_path.clone());
                security_repository::save_security_report(&state.paths.db_file, &persisted)?;
                crate::repositories::skills::update_skill_security_status(
                    &state.paths.db_file,
                    &skill.skill_id,
                    &persisted.level,
                    persisted.blocked,
                    persisted.scanned_at,
                )?;
                reports.push(persisted);
            }

            Ok(reports)
        })();

        match result {
            Ok(reports) => {
                log_task_emit_error(
                    "security.rescan.completed",
                    tasks::emit_completed(
                        &app_handle,
                        &task_handle,
                        "cleanup",
                        "Security rescan completed",
                        reports,
                    ),
                );
            }
            Err(error) => {
                log_task_emit_error(
                    "security.rescan.failed",
                    tasks::emit_failed(
                        &app_handle,
                        &task_handle,
                        "security_check",
                        &error.to_string(),
                    ),
                );
            }
        }
    });

    Ok(task)
}

#[tauri::command]
pub fn distribute_skill(
    app: AppHandle,
    state: State<'_, AppState>,
    request: DistributionRequest,
) -> Result<TaskHandle, String> {
    log::info!("distribute_skill invoked");
    let task = tasks::new_task_handle("distribute");
    let task_handle = task.clone();
    let app_handle = app.clone();
    let state = state.inner().clone();

    log_task_emit_error(
        "distribute.queued",
        tasks::emit_progress(
            &app,
            &task,
            "queued",
            "prepare",
            0,
            3,
            "Distribution task queued",
        ),
    );

    tauri::async_runtime::spawn(async move {
        log_task_emit_error(
            "distribute.running",
            tasks::emit_progress(
                &app_handle,
                &task_handle,
                "running",
                "distribute",
                1,
                3,
                "Distributing skill to target agent path",
            ),
        );

        match distribution::distribute_skill(
            state.agent_registry.as_ref(),
            &state.paths.db_file,
            &request,
        ) {
            Ok(result) => {
                log_task_emit_error(
                    "distribute.completed",
                    tasks::emit_completed(
                        &app_handle,
                        &task_handle,
                        "cleanup",
                        "Distribution completed",
                        result,
                    ),
                );
            }
            Err(error) => {
                let failed_payload = crate::domain::types::DistributionResult {
                    distribution_id: String::new(),
                    skill_id: request.skill_id.clone(),
                    target_agent: request.target_agent.clone(),
                    target_path: request
                        .custom_target_path
                        .clone()
                        .or(request.project_root.clone())
                        .unwrap_or_default(),
                    status: "failed".to_string(),
                    message: Some(error.to_string()),
                };
                log_task_emit_error(
                    "distribute.failed",
                    tasks::emit_failed_with_payload(
                        &app_handle,
                        &task_handle,
                        "distribute",
                        &error.to_string(),
                        failed_payload,
                    ),
                );
            }
        }
    });

    Ok(task)
}

#[tauri::command]
pub fn inject_template(
    app: AppHandle,
    state: State<'_, AppState>,
    request: TemplateInjectionRequest,
) -> Result<TaskHandle, String> {
    log::info!("inject_template invoked");
    let task = tasks::new_task_handle("inject_template");
    let task_handle = task.clone();
    let app_handle = app.clone();
    let state = state.inner().clone();

    log_task_emit_error(
        "inject_template.queued",
        tasks::emit_progress(
            &app,
            &task,
            "queued",
            "prepare",
            0,
            3,
            "Template injection task queued",
        ),
    );

    tauri::async_runtime::spawn(async move {
        log_task_emit_error(
            "inject_template.install",
            tasks::emit_progress(
                &app_handle,
                &task_handle,
                "running",
                "download",
                1,
                3,
                "Resolving and installing missing template skills",
            ),
        );
        log_task_emit_error(
            "inject_template.distribute",
            tasks::emit_progress(
                &app_handle,
                &task_handle,
                "running",
                "distribute",
                2,
                3,
                "Injecting template skills into project targets",
            ),
        );

        match template_injection::inject_template(
            &state.paths,
            state.agent_registry.as_ref(),
            &task_handle.task_id,
            &request,
        ) {
            Ok(result) if result.status == "partial" => {
                log_task_emit_error(
                    "inject_template.partial",
                    tasks::emit_partial(
                        &app_handle,
                        &task_handle,
                        "cleanup",
                        "Template injection completed with partial failures",
                        result,
                    ),
                );
            }
            Ok(result) if result.status == "failed" => {
                log_task_emit_error(
                    "inject_template.failed",
                    tasks::emit_failed_with_payload(
                        &app_handle,
                        &task_handle,
                        "distribute",
                        "Template injection failed for all items",
                        result,
                    ),
                );
            }
            Ok(result) => {
                log_task_emit_error(
                    "inject_template.completed",
                    tasks::emit_completed(
                        &app_handle,
                        &task_handle,
                        "cleanup",
                        "Template injection completed",
                        result,
                    ),
                );
            }
            Err(error) => {
                log_task_emit_error(
                    "inject_template.command_failed",
                    tasks::emit_failed(&app_handle, &task_handle, "prepare", &error.to_string()),
                );
            }
        }
    });

    Ok(task)
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
