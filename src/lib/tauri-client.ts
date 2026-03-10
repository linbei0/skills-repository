import { invoke } from '@tauri-apps/api/core'
import type {
  AgentGlobalScanResult,
  AppSettings,
  BootstrapPayload,
  DistributionRequest,
  InstallSkillRequest,
  InstallSkillResult,
  MarketSearchRequest,
  MarketSearchResponse,
  RepositorySkillDetail,
  RepositorySkillSummary,
  RepositoryUninstallResult,
  SecurityReport,
  SaveTemplateRequest,
  TemplateRecord,
  DistributionResult,
} from '../types/app'

export const bootstrapApp = () => invoke<BootstrapPayload>('bootstrap_app')


export const saveSettings = (settings: AppSettings) =>
  invoke<AppSettings>('save_settings', { settings })

export const listRepositorySkills = () =>
  invoke<RepositorySkillSummary[]>('list_repository_skills')

export const getRepositorySkillDetail = (skillId: string) =>
  invoke<RepositorySkillDetail>('get_repository_skill_detail', { skillId })

export const uninstallRepositorySkill = (skillId: string) =>
  invoke<RepositoryUninstallResult>('uninstall_repository_skill', { skillId })

export const scanAgentGlobalSkills = (agentId: string) =>
  invoke<AgentGlobalScanResult>('scan_agent_global_skills', { agentId })

export const searchMarketSkills = (request: MarketSearchRequest) =>
  invoke<MarketSearchResponse>('search_market_skills', { request })

export const installSkill = (request: InstallSkillRequest) =>
  invoke<InstallSkillResult>('install_skill', { request })

export const distributeSkill = (request: DistributionRequest) =>
  invoke<DistributionResult>('distribute_skill', { request })

export const getSecurityReports = () =>
  invoke<SecurityReport[]>('get_security_reports')

export const rescanSecurity = () =>
  invoke<SecurityReport[]>('rescan_security')

export const listTemplates = () => invoke<TemplateRecord[]>('list_templates')

export const getTemplate = (templateId: string) =>
  invoke<TemplateRecord | null>('get_template', { templateId })

export const saveTemplate = (request: SaveTemplateRequest) =>
  invoke<TemplateRecord>('save_template', { request })

export const deleteTemplate = (templateId: string) =>
  invoke<void>('delete_template', { templateId })

