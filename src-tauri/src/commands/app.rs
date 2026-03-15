use tauri::{async_runtime::spawn_blocking, AppHandle, State};

use crate::{
    domain::{
        app_state::AppState,
        types::{
            AgentGlobalScanRequest, AgentGlobalScanResult, AppSettings,
            BatchDistributeRepositorySkillsRequest, BatchDistributeResult,
            BatchRepositorySkillUpdateResult, DistributionRequest, DistributionResult,
            ImportRepositorySkillRequest, InjectTemplateRequest, InjectTemplateResult,
            InstallSkillRequest, InstallSkillResult, MarketSearchRequest, MarketSearchResponse,
            MigrateRepositoryStorageRequest, MigrateRepositoryStorageResult,
            RepositorySkillDeletionPreview, RepositorySkillDetail, RepositorySkillSummary,
            RepositorySkillUpdateItemResult, RepositoryUninstallResult,
            ResolveRepositoryImportRequest, ResolveRepositoryImportResult, SaveTemplateRequest,
            SecurityReport, TemplateRecord,
        },
    },
    repositories::security as security_repository,
    repositories::skills as skills_repository,
    services::{
        agent_scan, bootstrap, distribution, install, market, project_distribution, repository,
        repository_import, repository_update, settings, source_reference, templates,
    },
};

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
pub fn save_settings(
    state: State<'_, AppState>,
    settings: AppSettings,
) -> Result<AppSettings, String> {
    log::info!("save_settings invoked");
    settings::save_settings(&state, &settings).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn migrate_repository_storage(
    state: State<'_, AppState>,
    request: MigrateRepositoryStorageRequest,
) -> Result<MigrateRepositoryStorageResult, String> {
    log::info!("migrate_repository_storage invoked");
    settings::migrate_repository_storage(&state, &request).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn open_source_reference(
    app_handle: AppHandle,
    _state: State<'_, AppState>,
    reference: String,
) -> Result<(), String> {
    log::info!("open_source_reference invoked");
    source_reference::open_source_reference(&app_handle, &reference)
        .map_err(|error| error.to_string())
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
pub fn inject_template(
    state: State<'_, AppState>,
    request: InjectTemplateRequest,
) -> Result<InjectTemplateResult, String> {
    log::info!("inject_template invoked");
    templates::inject_template(&state, &request).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn install_skill(
    state: State<'_, AppState>,
    request: InstallSkillRequest,
) -> Result<InstallSkillResult, String> {
    log::info!("install_skill invoked");
    let paths = settings::runtime_paths(&state).map_err(|error| error.to_string())?;
    install::install_skill(&paths, &request).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn resolve_repository_import_source(
    state: State<'_, AppState>,
    request: ResolveRepositoryImportRequest,
) -> Result<ResolveRepositoryImportResult, String> {
    log::info!("resolve_repository_import_source invoked");
    let paths = settings::runtime_paths(&state).map_err(|error| error.to_string())?;
    repository_import::resolve_repository_import_source(&paths, &request)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn import_repository_skill(
    state: State<'_, AppState>,
    request: ImportRepositorySkillRequest,
) -> Result<InstallSkillResult, String> {
    log::info!("import_repository_skill invoked");
    let paths = settings::runtime_paths(&state).map_err(|error| error.to_string())?;
    repository_import::import_repository_skill(&paths, &request).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn list_repository_skills(
    state: State<'_, AppState>,
) -> Result<Vec<RepositorySkillSummary>, String> {
    log::info!("list_repository_skills invoked");
    let paths = settings::runtime_paths(&state).map_err(|error| error.to_string())?;
    repository::list_repository_skills(&paths.db_file, &paths.canonical_store_dir)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn get_repository_skill_detail(
    state: State<'_, AppState>,
    skill_id: String,
) -> Result<RepositorySkillDetail, String> {
    log::info!("get_repository_skill_detail invoked");
    let paths = settings::runtime_paths(&state).map_err(|error| error.to_string())?;
    repository::get_repository_skill_detail(&paths.db_file, &paths.canonical_store_dir, &skill_id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn update_repository_skill(
    state: State<'_, AppState>,
    skill_id: String,
) -> Result<RepositorySkillUpdateItemResult, String> {
    log::info!("update_repository_skill invoked");
    let paths = settings::runtime_paths(&state).map_err(|error| error.to_string())?;
    spawn_blocking(move || repository_update::update_repository_skill(&paths, &skill_id))
        .await
        .map_err(|error| error.to_string())?
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn update_github_repository_skills(
    state: State<'_, AppState>,
) -> Result<BatchRepositorySkillUpdateResult, String> {
    log::info!("update_github_repository_skills invoked");
    let paths = settings::runtime_paths(&state).map_err(|error| error.to_string())?;
    spawn_blocking(move || repository_update::update_github_repository_skills(&paths))
        .await
        .map_err(|error| error.to_string())?
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn get_security_reports(state: State<'_, AppState>) -> Result<Vec<SecurityReport>, String> {
    log::info!("get_security_reports invoked");
    security_repository::list_security_reports(&state.paths.db_file)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn rescan_security(state: State<'_, AppState>) -> Result<Vec<SecurityReport>, String> {
    log::info!("rescan_security invoked");

    let result = (|| -> anyhow::Result<Vec<SecurityReport>> {
        let installed = crate::repositories::skills::list_installed_skills(&state.paths.db_file)?;
        let mut reports = Vec::new();

        for skill in installed {
            let report = crate::security::scan_skill_directory_with_context(
                std::path::Path::new(&skill.canonical_path),
                Some(skill.skill_id.clone()),
                "rescan",
                &crate::security::SecurityScanSourceContext {
                    source_url: skill.source_url.clone(),
                    repo_url: skill.repo_url.clone(),
                    download_url: None,
                    version: skill.version.clone(),
                    manifest_path: None,
                    skill_root: None,
                },
            )?;
            let mut persisted = report.clone();
            persisted.skill_name = Some(skill.name.clone());
            persisted.source_path = Some(skill.canonical_path.clone());
            security_repository::save_security_report(&state.paths.db_file, &persisted)?;
            let risk_override_applied = skill.risk_override_applied && persisted.blocked;
            crate::repositories::skills::update_skill_risk_override_state(
                &state.paths.db_file,
                &skill.skill_id,
                risk_override_applied,
            )?;
            crate::repositories::skills::update_skill_security_status(
                &state.paths.db_file,
                &skill.skill_id,
                &persisted.level,
                persisted.blocked && !risk_override_applied,
                persisted.scanned_at,
            )?;
            reports.push(persisted);
        }

        Ok(reports)
    })();

    result.map_err(|error| error.to_string())
}

#[tauri::command]
pub fn uninstall_repository_skill(
    state: State<'_, AppState>,
    skill_id: String,
) -> Result<RepositoryUninstallResult, String> {
    log::info!("uninstall_repository_skill invoked");
    let paths = settings::runtime_paths(&state).map_err(|error| error.to_string())?;
    repository::uninstall_repository_skill(&paths.db_file, &paths.canonical_store_dir, &skill_id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn scan_agent_global_skills(
    request: AgentGlobalScanRequest,
) -> Result<AgentGlobalScanResult, String> {
    log::info!("scan_agent_global_skills invoked");
    agent_scan::scan_agent_global_skills(&request).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn distribute_skill(
    state: State<'_, AppState>,
    request: DistributionRequest,
) -> Result<DistributionResult, String> {
    log::info!("distribute_skill invoked");
    distribution::distribute_skill(
        state.agent_registry.as_ref(),
        &state.paths.db_file,
        &request,
    )
    .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn batch_distribute_repository_skills(
    state: State<'_, AppState>,
    request: BatchDistributeRepositorySkillsRequest,
) -> Result<BatchDistributeResult, String> {
    log::info!("batch_distribute_repository_skills invoked");

    let selections = request
        .skill_ids
        .iter()
        .map(
            |skill_id| project_distribution::ProjectDistributionSelection {
                skill_id: skill_id.clone(),
                skill_name: skills_repository::load_skill_name(&state.paths.db_file, skill_id)
                    .unwrap_or_else(|_| skill_id.clone()),
            },
        )
        .collect::<Vec<_>>();

    project_distribution::distribute_repository_skills_to_project(
        &state,
        &selections,
        &project_distribution::ProjectDistributionRequest {
            target_scope: request.target_scope,
            project_root: request.project_root.unwrap_or_default(),
            target_type: request.target_type,
            target_agent_id: request.target_agent_id,
            custom_relative_path: request.custom_relative_path,
            install_mode: request.install_mode,
        },
    )
    .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn get_repository_skill_deletion_preview(
    state: State<'_, AppState>,
    skill_id: String,
) -> Result<RepositorySkillDeletionPreview, String> {
    log::info!("get_repository_skill_deletion_preview invoked");
    let paths = settings::runtime_paths(&state).map_err(|error| error.to_string())?;
    repository::get_repository_skill_deletion_preview(
        &paths.db_file,
        &paths.canonical_store_dir,
        &skill_id,
    )
    .map_err(|error| error.to_string())
}
