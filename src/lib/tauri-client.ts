import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import type {
  AppSettings,
  BootstrapPayload,
  DistributionRequest,
  InstallSkillRequest,
  MarketSearchRequest,
  MarketSearchResponse,
  ScanSkillsRequest,
  SecurityReport,
  SaveTemplateRequest,
  TaskHandle,
  TaskProgress,
  TemplateInjectionRequest,
  TemplateRecord,
} from '../types/app'

export const EVENT_TASK_PROGRESS = 'task:progress'
export const EVENT_TASK_COMPLETED = 'task:completed'
export const EVENT_TASK_FAILED = 'task:failed'

export const bootstrapApp = () => invoke<BootstrapPayload>('bootstrap_app')

export const getSettings = () => invoke<AppSettings>('get_settings')

export const saveSettings = (settings: AppSettings) =>
  invoke<AppSettings>('save_settings', { settings })

export const searchMarketSkills = (request: MarketSearchRequest) =>
  invoke<MarketSearchResponse>('search_market_skills', { request })

export const installSkill = (request: InstallSkillRequest) =>
  invoke<TaskHandle>('install_skill', { request })

export const distributeSkill = (request: DistributionRequest) =>
  invoke<TaskHandle>('distribute_skill', { request })

export const getSecurityReports = () =>
  invoke<SecurityReport[]>('get_security_reports')

export const rescanSecurity = () =>
  invoke<TaskHandle>('rescan_security')

export const listTemplates = () => invoke<TemplateRecord[]>('list_templates')

export const getTemplate = (templateId: string) =>
  invoke<TemplateRecord | null>('get_template', { templateId })

export const saveTemplate = (request: SaveTemplateRequest) =>
  invoke<TemplateRecord>('save_template', { request })

export const deleteTemplate = (templateId: string) =>
  invoke<void>('delete_template', { templateId })

export const injectTemplate = (request: TemplateInjectionRequest) =>
  invoke<TaskHandle>('inject_template', { request })

export const scanSkills = (request: ScanSkillsRequest) =>
  invoke<TaskHandle>('scan_skills', { request })

export const onTaskProgress = (
  handler: (event: TaskProgress) => void,
) => listen<TaskProgress>(EVENT_TASK_PROGRESS, ({ payload }) => handler(payload))

export const onTaskCompleted = (
  handler: (event: TaskProgress) => void,
) => listen<TaskProgress>(EVENT_TASK_COMPLETED, ({ payload }) => handler(payload))

export const onTaskFailed = (handler: (event: TaskProgress) => void) =>
  listen<TaskProgress>(EVENT_TASK_FAILED, ({ payload }) => handler(payload))
