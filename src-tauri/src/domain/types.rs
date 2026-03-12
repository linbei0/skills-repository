use super::agent_registry::AgentCapability;
use serde::{Deserialize, Serialize};

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
    pub source_type: String,
    pub source_url: String,
    pub repo_url: Option<String>,
    pub download_url: Option<String>,
    pub package_ref: Option<String>,
    pub manifest_path: Option<String>,
    pub skill_root: Option<String>,
    pub version: Option<String>,
    pub author: Option<String>,
    pub tags: Vec<String>,
    pub installable: bool,
    pub resolver_status: String,
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
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
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
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
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
pub struct RepositorySkillDeletionPreview {
    pub skill_id: String,
    pub skill_name: String,
    pub canonical_path: String,
    pub distribution_paths: Vec<String>,
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
    pub source_type: String,
    pub source_url: String,
    pub repo_url: Option<String>,
    pub download_url: Option<String>,
    pub package_ref: Option<String>,
    pub manifest_path: Option<String>,
    pub skill_root: Option<String>,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
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
pub struct ResolveRepositoryImportRequest {
    pub source_kind: String,
    pub input: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedRepositoryImportCandidate {
    pub name: String,
    pub slug: String,
    pub manifest_path: String,
    pub skill_root: String,
    pub source_url: String,
    pub repo_url: Option<String>,
    pub version: Option<String>,
    pub author: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolveRepositoryImportResult {
    pub source_kind: String,
    pub normalized_input: String,
    pub candidates: Vec<ResolvedRepositoryImportCandidate>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportRepositorySkillRequest {
    pub source_kind: String,
    pub input: String,
    pub selected_manifest_path: String,
    pub selected_skill_root: String,
    pub name: String,
    pub slug: String,
    pub source_url: String,
    pub repo_url: Option<String>,
    pub version: Option<String>,
    pub author: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SecurityIssue {
    pub rule_id: String,
    #[serde(default)]
    pub category: String,
    pub severity: String,
    pub title: String,
    pub description: String,
    #[serde(default)]
    pub file_path: Option<String>,
    #[serde(default)]
    pub file_kind: Option<String>,
    #[serde(default)]
    pub line: Option<u32>,
    #[serde(default)]
    pub evidence: Option<String>,
    #[serde(default)]
    pub blocking: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SecurityRecommendation {
    pub action: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SecurityCategoryBreakdown {
    pub category: String,
    pub count: u32,
    pub score: u32,
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
    #[serde(default)]
    pub category_breakdown: Vec<SecurityCategoryBreakdown>,
    #[serde(default)]
    pub blocking_reasons: Vec<String>,
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
pub struct SaveTemplateItemRequest {
    pub skill_ref_type: String,
    pub skill_ref: String,
    pub display_name: Option<String>,
    pub order_index: Option<u32>,
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
    #[serde(default)]
    pub items: Vec<SaveTemplateItemRequest>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InjectTemplateRequest {
    pub template_id: String,
    pub project_root: String,
    pub target_type: String,
    pub target_agent_id: Option<String>,
    pub custom_relative_path: Option<String>,
    pub install_mode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InjectTemplateItemResult {
    pub skill_id: String,
    pub skill_name: String,
    pub target_path: String,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InjectTemplateResult {
    pub installed: Vec<InjectTemplateItemResult>,
    pub skipped: Vec<InjectTemplateItemResult>,
    pub failed: Vec<InjectTemplateItemResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchDistributeRepositorySkillsRequest {
    pub target_scope: String,
    pub skill_ids: Vec<String>,
    pub project_root: Option<String>,
    pub target_type: String,
    pub target_agent_id: Option<String>,
    pub custom_relative_path: Option<String>,
    pub install_mode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchDistributeItemResult {
    pub skill_id: String,
    pub skill_name: String,
    pub target_path: String,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchDistributeResult {
    pub installed: Vec<BatchDistributeItemResult>,
    pub skipped: Vec<BatchDistributeItemResult>,
    pub failed: Vec<BatchDistributeItemResult>,
}
