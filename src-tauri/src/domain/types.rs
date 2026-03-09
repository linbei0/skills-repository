use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::agent_registry::AgentCapability;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub language: String,
    pub theme_mode: String,
    pub scan: ScanSettings,
    pub agent_preferences: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanSettings {
    pub project_roots: Vec<String>,
    pub custom_roots: Vec<String>,
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
pub struct OverviewStats {
    pub total_skills: usize,
    pub risky_skills: Option<usize>,
    pub duplicate_paths: usize,
    pub reclaimable_bytes: Option<u64>,
    pub template_count: Option<usize>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BootstrapPayload {
    pub app_version: String,
    pub system: SystemInfo,
    pub settings: AppSettings,
    pub agents: Vec<AgentCapability>,
    pub overview: OverviewStats,
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

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanSkillsRequest {
    pub include_system: bool,
    pub include_projects: bool,
    pub project_roots: Vec<String>,
    pub custom_roots: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskHandle {
    pub task_id: String,
    pub task_type: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskProgressEvent {
    pub task_id: String,
    pub task_type: String,
    pub status: String,
    pub step: String,
    pub current: u32,
    pub total: u32,
    pub message: String,
    pub payload: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillAgentBinding {
    pub primary: String,
    pub aliases: Vec<String>,
    pub priority: u32,
    pub compatible_agents: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillRecord {
    pub id: String,
    pub name: String,
    pub path: String,
    pub agent: SkillAgentBinding,
    pub scope: String,
    pub source: String,
    pub managed: bool,
    pub project_root: Option<String>,
    pub last_seen_at: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DistributionRecord {
    pub id: String,
    pub skill_id: String,
    pub target_agent: String,
    pub target_path: String,
    pub status: String,
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

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectRecord {
    pub id: String,
    pub name: String,
    pub root_path: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DuplicateGroup {
    pub name: String,
    pub paths: Vec<String>,
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
    pub target_agents: Vec<String>,
    pub scope: String,
    pub items: Vec<TemplateItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TemplateInjectionRequest {
    pub template_id: String,
    pub target_project_path: String,
    pub overwrite_strategy: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TemplateInjectionItemResult {
    pub skill_ref: String,
    pub status: String,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TemplateInjectionResult {
    pub template_id: String,
    pub target_project_path: String,
    pub status: String,
    pub installed_count: u32,
    pub skipped_count: u32,
    pub failed_count: u32,
    pub results: Vec<TemplateInjectionItemResult>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanSkillsResult {
    pub skills: Vec<SkillRecord>,
    pub distributions: Vec<DistributionRecord>,
    pub duplicates: Vec<DuplicateGroup>,
    pub projects: Vec<ProjectRecord>,
    pub overview: OverviewStats,
}
