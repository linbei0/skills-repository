use serde::{Deserialize, Serialize};
use super::agent_registry::AgentCapability;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub language: String,
    pub theme_mode: String,
    #[serde(default = "default_visible_skill_target_ids")]
    pub visible_skills_target_ids: Vec<String>,
    #[serde(default)]
    pub custom_skills_targets: Vec<CustomSkillsTarget>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomSkillsTarget {
    pub id: String,
    pub label: String,
    pub relative_path: String,
}

fn default_visible_skill_target_ids() -> Vec<String> {
    vec![
        "universal".into(),
        "antigravity".into(),
        "claude-code".into(),
        "codebuddy".into(),
        "kiro-cli".into(),
        "openclaw".into(),
        "qoder".into(),
        "trae".into(),
        "windsurf".into(),
    ]
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemInfo {
    pub os: String,
    pub arch: String,
    pub locale: String,
    pub theme: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BootstrapPayload {
    pub app_version: String,
    pub system: SystemInfo,
    pub settings: AppSettings,
    pub agents: Vec<AgentCapability>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketSearchRequest {
    pub query: String,
    pub page: u32,
    pub page_size: u32,
    pub enabled_providers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderStatus {
    pub provider: String,
    pub status: String,
    pub message: Option<String>,
    pub cache_hit: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketSkillSummary {
    pub id: String,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub provider: String,
    pub source_url: String,
    pub download_url: Option<String>,
    pub version: Option<String>,
    pub author: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketSearchResponse {
    pub results: Vec<MarketSkillSummary>,
    pub providers: Vec<ProviderStatus>,
    pub page: u32,
    pub page_size: u32,
    pub total: u32,
    pub cache_hit: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RepositorySkillSummary {
    pub id: String,
    pub name: String,
    pub source_type: String,
    pub source_market: Option<String>,
    pub installed_at: i64,
    pub security_level: String,
    pub blocked: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RepositorySkillDetail {
    pub id: String,
    pub name: String,
    pub canonical_path: String,
    pub source_type: String,
    pub source_market: Option<String>,
    pub source_url: Option<String>,
    pub installed_at: i64,
    pub security_level: String,
    pub blocked: bool,
    pub skill_markdown: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryUninstallResult {
    pub skill_id: String,
    pub removed_paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentGlobalSkillEntry {
    pub id: String,
    pub name: String,
    pub path: String,
    pub relationship: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentGlobalScanResult {
    pub agent_id: String,
    pub agent_label: String,
    pub root_path: String,
    pub entries: Vec<AgentGlobalSkillEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentGlobalScanRequest {
    pub agent_id: String,
    pub agent_label: String,
    pub relative_path: String,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DistributionRequest {
    pub skill_id: String,
    pub target_kind: String,
    pub target_agent: String,
    pub install_mode: String,
    pub project_root: Option<String>,
    pub custom_target_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DistributionResult {
    pub distribution_id: String,
    pub skill_id: String,
    pub target_agent: String,
    pub target_path: String,
    pub status: String,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallSkillRequest {
    pub provider: String,
    pub market_skill_id: String,
    pub source_url: String,
    pub download_url: Option<String>,
    pub name: String,
    pub slug: String,
    pub version: Option<String>,
    pub author: Option<String>,
    pub requested_targets: Vec<DistributionRequest>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallSkillResult {
    pub skill_id: String,
    pub canonical_path: String,
    pub blocked: bool,
    pub security_level: String,
    pub operation_log_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SecurityIssue {
    pub rule_id: String,
    pub severity: String,
    pub title: String,
    pub description: String,
    pub file_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SecurityRecommendation {
    pub action: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SecurityReport {
    pub id: String,
    pub skill_id: Option<String>,
    pub skill_name: Option<String>,
    pub source_path: Option<String>,
    pub scan_scope: String,
    pub level: String,
    pub score: u32,
    pub blocked: bool,
    pub issues: Vec<SecurityIssue>,
    pub recommendations: Vec<SecurityRecommendation>,
    pub scanned_files: Vec<String>,
    pub engine_version: String,
    pub scanned_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TemplateItem {
    pub id: String,
    pub skill_ref_type: String,
    pub skill_ref: String,
    pub display_name: Option<String>,
    pub required: bool,
    pub order_index: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TemplateRecord {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub target_agents: Vec<String>,
    pub scope: String,
    pub is_builtin: bool,
    pub items: Vec<TemplateItem>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveTemplateRequest {
    pub id: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
}

