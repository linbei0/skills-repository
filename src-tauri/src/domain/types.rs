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

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanSkillsResult {
    pub skills: Vec<SkillRecord>,
    pub distributions: Vec<DistributionRecord>,
    pub duplicates: Vec<DuplicateGroup>,
    pub projects: Vec<ProjectRecord>,
    pub overview: OverviewStats,
}
