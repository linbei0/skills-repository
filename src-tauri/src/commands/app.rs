use tauri::State;

use crate::{
    domain::{
        app_state::AppState,
        types::{
            AgentGlobalScanRequest, AgentGlobalScanResult, AppSettings,
            BatchDistributeRepositorySkillsRequest, BatchDistributeResult, DistributionRequest,
            DistributionResult, ImportRepositorySkillRequest, InjectTemplateRequest,
            InjectTemplateResult, InstallSkillRequest, InstallSkillResult,
            MarketSearchRequest, MarketSearchResponse, RepositorySkillDetail,
            RepositorySkillSummary, RepositoryUninstallResult, ResolveRepositoryImportRequest,
            ResolveRepositoryImportResult, SaveTemplateRequest, SecurityReport, TemplateRecord,
        },
    },
    repositories::skills as skills_repository,
    repositories::security as security_repository,
    services::{
        agent_scan, bootstrap, distribution, install, market, project_distribution,
        repository, repository_import, settings, templates,
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
    install::install_skill(&state.paths, &request).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn resolve_repository_import_source(
    state: State<'_, AppState>,
    request: ResolveRepositoryImportRequest,
) -> Result<ResolveRepositoryImportResult, String> {
    log::info!("resolve_repository_import_source invoked");
    repository_import::resolve_repository_import_source(&state.paths, &request)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn import_repository_skill(
    state: State<'_, AppState>,
    request: ImportRepositorySkillRequest,
) -> Result<InstallSkillResult, String> {
    log::info!("import_repository_skill invoked");
    repository_import::import_repository_skill(&state.paths, &request)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn list_repository_skills(
    state: State<'_, AppState>,
) -> Result<Vec<RepositorySkillSummary>, String> {
    log::info!("list_repository_skills invoked");
    repository::list_repository_skills(&state.paths.db_file, &state.paths.canonical_store_dir)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn get_repository_skill_detail(
    state: State<'_, AppState>,
    skill_id: String,
) -> Result<RepositorySkillDetail, String> {
    log::info!("get_repository_skill_detail invoked");
    repository::get_repository_skill_detail(
        &state.paths.db_file,
        &state.paths.canonical_store_dir,
        &skill_id,
    )
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

    result.map_err(|error| error.to_string())
}

#[tauri::command]
pub fn uninstall_repository_skill(
    state: State<'_, AppState>,
    skill_id: String,
) -> Result<RepositoryUninstallResult, String> {
    log::info!("uninstall_repository_skill invoked");
    repository::uninstall_repository_skill(
        &state.paths.db_file,
        &state.paths.canonical_store_dir,
        &skill_id,
    )
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
    distribution::distribute_skill(state.agent_registry.as_ref(), &state.paths.db_file, &request)
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
        .map(|skill_id| project_distribution::ProjectDistributionSelection {
            skill_id: skill_id.clone(),
            skill_name: skills_repository::load_skill_name(&state.paths.db_file, skill_id)
                .unwrap_or_else(|_| skill_id.clone()),
        })
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
