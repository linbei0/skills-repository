use anyhow::Result;
use dark_light::Mode;
use sys_locale::get_locale;

use crate::domain::{
    agent_registry::AgentCapability,
    app_state::AppState,
    types::{BootstrapPayload, SystemInfo},
};
use crate::repositories::scan as scan_repository;

use super::settings::load_or_create_settings;

pub fn system_locale() -> String {
    get_locale().unwrap_or_else(|| "en-US".to_string())
}

pub fn normalize_language(locale: &str) -> String {
    let locale = locale.to_ascii_lowercase();

    if locale.starts_with("zh") {
        "zh-CN".into()
    } else if locale.starts_with("ja") {
        "ja-JP".into()
    } else {
        "en-US".into()
    }
}

pub fn detect_theme() -> String {
    match dark_light::detect() {
        Ok(Mode::Dark) => "dark".into(),
        _ => "light".into(),
    }
}

pub fn build_system_info() -> SystemInfo {
    SystemInfo {
        os: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
        locale: system_locale(),
        theme: detect_theme(),
    }
}

pub fn bootstrap_payload(state: &AppState, version: String) -> Result<BootstrapPayload> {
    let system = build_system_info();
    let settings = load_or_create_settings(state, normalize_language(&system.locale))?;
    let agents: Vec<AgentCapability> = state.agent_registry.agents().to_vec();

    Ok(BootstrapPayload {
        app_version: version,
        system,
        settings,
        agents,
        overview: scan_repository::load_overview_stats(&state.paths.db_file)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_supported_languages() {
        assert_eq!(normalize_language("zh-CN"), "zh-CN");
        assert_eq!(normalize_language("ja-JP"), "ja-JP");
        assert_eq!(normalize_language("en-GB"), "en-US");
        assert_eq!(normalize_language("fr-FR"), "en-US");
    }
}
