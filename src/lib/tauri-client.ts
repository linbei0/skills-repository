import { invoke } from '@tauri-apps/api/core'
import type {
  AgentGlobalScanRequest,
  AgentGlobalScanResult,
  AppSettings,
  BatchRepositorySkillUpdateResult,
  BatchDistributeRepositorySkillsRequest,
  BatchDistributeResult,
  BootstrapPayload,
  DistributionRequest,
  InstallSkillRequest,
  InstallSkillResult,
  ImportRepositorySkillRequest,
  MarketSearchRequest,
  MarketSearchResponse,
  MigrateRepositoryStorageRequest,
  MigrateRepositoryStorageResult,
  ResolveRepositoryImportRequest,
  ResolveRepositoryImportResult,
  RepositorySkillDetail,
  RepositorySkillDeletionPreview,
  RepositorySkillSummary,
  RepositorySkillUpdateItemResult,
  RepositoryUninstallResult,
  SecurityReport,
  SaveTemplateRequest,
  TemplateRecord,
  DistributionResult,
  InjectTemplateRequest,
  InjectTemplateResult,
} from '../types/app'

export const bootstrapApp = () => invoke<BootstrapPayload>('bootstrap_app')


export const saveSettings = (settings: AppSettings) =>
  invoke<AppSettings>('save_settings', { settings })

export const migrateRepositoryStorage = (request: MigrateRepositoryStorageRequest) =>
  invoke<MigrateRepositoryStorageResult>('migrate_repository_storage', { request })

export const openSourceReference = (reference: string) =>
  invoke<void>('open_source_reference', { reference })

export const listRepositorySkills = () =>
  invoke<RepositorySkillSummary[]>('list_repository_skills')

export const getRepositorySkillDetail = (skillId: string) =>
  invoke<RepositorySkillDetail>('get_repository_skill_detail', { skillId })

export const getRepositorySkillDeletionPreview = (skillId: string) =>
  invoke<RepositorySkillDeletionPreview>('get_repository_skill_deletion_preview', { skillId })

export const uninstallRepositorySkill = (skillId: string) =>
  invoke<RepositoryUninstallResult>('uninstall_repository_skill', { skillId })

export const updateRepositorySkill = (skillId: string) =>
  invoke<RepositorySkillUpdateItemResult>('update_repository_skill', { skillId })

export const updateGithubRepositorySkills = () =>
  invoke<BatchRepositorySkillUpdateResult>('update_github_repository_skills')

export const scanAgentGlobalSkills = (request: AgentGlobalScanRequest) =>
  invoke<AgentGlobalScanResult>('scan_agent_global_skills', { request })

export const searchMarketSkills = (request: MarketSearchRequest) =>
  invoke<MarketSearchResponse>('search_market_skills', { request })

export const installSkill = (request: InstallSkillRequest) =>
  invoke<InstallSkillResult>('install_skill', { request })

export const resolveRepositoryImportSource = (request: ResolveRepositoryImportRequest) =>
  invoke<ResolveRepositoryImportResult>('resolve_repository_import_source', { request })

export const importRepositorySkill = (request: ImportRepositorySkillRequest) =>
  invoke<InstallSkillResult>('import_repository_skill', { request })

export const distributeSkill = (request: DistributionRequest) =>
  invoke<DistributionResult>('distribute_skill', { request })

export const batchDistributeRepositorySkills = (
  request: BatchDistributeRepositorySkillsRequest,
) => invoke<BatchDistributeResult>('batch_distribute_repository_skills', { request })

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

export const injectTemplate = (request: InjectTemplateRequest) =>
  invoke<InjectTemplateResult>('inject_template', { request })

