export type ThemeMode = 'system' | 'light' | 'dark'
export type ResolvedTheme = 'skills-light' | 'skills-dark'
export type AppLocale = 'zh-CN' | 'en-US' | 'ja-JP'

export interface AppSettings {
  language: AppLocale
  themeMode: ThemeMode
  scan: {
    projectRoots: string[]
    customRoots: string[]
  }
  agentPreferences: Record<string, string>
}

export interface AgentCapability {
  id: string
  label: string
  globalPaths: string[]
  projectPaths: string[]
  defaultGlobalMode: 'symlink' | 'copy' | 'native'
  defaultProjectMode: 'symlink' | 'copy' | 'native'
}

export interface SystemInfo {
  os: 'windows' | 'macos' | 'linux'
  arch: string
  locale: string
  theme: 'light' | 'dark'
}

export interface OverviewStats {
  totalSkills: number
  riskySkills: number | null
  duplicatePaths: number
  reclaimableBytes: number | null
  templateCount: number | null
}

export interface BootstrapPayload {
  appVersion: string
  system: SystemInfo
  settings: AppSettings
  agents: AgentCapability[]
  overview: OverviewStats
}

export interface MarketSearchRequest {
  query: string
  page: number
  pageSize: number
  enabledProviders: string[]
}

export interface ProviderStatus {
  provider: string
  status: string
  message: string | null
  cacheHit: boolean
}

export interface MarketSkillSummary {
  id: string
  slug: string
  name: string
  description: string | null
  provider: string
  sourceUrl: string
  downloadUrl: string | null
  version: string | null
  author: string | null
  tags: string[]
}

export interface MarketSearchResponse {
  results: MarketSkillSummary[]
  providers: ProviderStatus[]
  page: number
  pageSize: number
  total: number
  cacheHit: boolean
}

export interface SkillAgentBinding {
  primary: string
  aliases: string[]
  priority: number
  compatibleAgents: string[]
}

export interface SkillRecord {
  id: string
  name: string
  path: string
  agent: SkillAgentBinding
  scope: 'system' | 'project' | 'custom'
  source: string
  managed: boolean
  projectRoot?: string | null
  lastSeenAt: number
}

export interface DistributionRecord {
  id: string
  skillId: string
  targetAgent: string
  targetPath: string
  status: 'active' | 'broken' | 'removed' | 'failed'
}

export interface DistributionRequest {
  skillId: string
  targetKind: string
  targetAgent: string
  installMode: string
  projectRoot?: string | null
  customTargetPath?: string | null
}

export interface DistributionResult {
  distributionId: string
  skillId: string
  targetAgent: string
  targetPath: string
  status: string
  message: string | null
}

export interface InstallSkillRequest {
  provider: string
  marketSkillId: string
  sourceUrl: string
  downloadUrl?: string | null
  name: string
  slug: string
  version?: string | null
  author?: string | null
  requestedTargets: DistributionRequest[]
}

export interface InstallSkillResult {
  skillId: string
  canonicalPath: string
  blocked: boolean
  securityLevel: string
  operationLogId?: string | null
}

export interface SecurityIssue {
  ruleId: string
  severity: string
  title: string
  description: string
  filePath?: string | null
}

export interface SecurityRecommendation {
  action: string
  description: string
}

export interface SecurityReport {
  id: string
  skillId?: string | null
  skillName?: string | null
  sourcePath?: string | null
  scanScope: string
  level: string
  score: number
  blocked: boolean
  issues: SecurityIssue[]
  recommendations: SecurityRecommendation[]
  scannedFiles: string[]
  engineVersion: string
  scannedAt: number
}

export interface ProjectRecord {
  id: string
  name: string
  rootPath: string
}

export interface DuplicateGroup {
  name: string
  paths: string[]
}

export interface TemplateItem {
  id: string
  skillRefType: string
  skillRef: string
  displayName?: string | null
  required: boolean
  orderIndex: number
}

export interface TemplateRecord {
  id: string
  name: string
  description?: string | null
  tags: string[]
  targetAgents: string[]
  scope: string
  isBuiltin: boolean
  items: TemplateItem[]
  createdAt: number
  updatedAt: number
}

export interface SaveTemplateRequest {
  id?: string | null
  name: string
  description?: string | null
  tags: string[]
  targetAgents: string[]
  scope: string
  items: TemplateItem[]
}

export interface TemplateInjectionRequest {
  templateId: string
  targetProjectPath: string
  overwriteStrategy: string
}

export interface TemplateInjectionItemResult {
  skillRef: string
  status: string
  message?: string | null
}

export interface TemplateInjectionResult {
  templateId: string
  targetProjectPath: string
  status: string
  installedCount: number
  skippedCount: number
  failedCount: number
  results: TemplateInjectionItemResult[]
}

export interface ScanSkillsRequest {
  includeSystem: boolean
  includeProjects: boolean
  projectRoots: string[]
  customRoots: string[]
}

export interface ScanSkillsResult {
  skills: SkillRecord[]
  distributions: DistributionRecord[]
  duplicates: DuplicateGroup[]
  projects: ProjectRecord[]
  overview: OverviewStats
}

export interface TaskHandle {
  taskId: string
  taskType: string
}

export type TaskType =
  | 'scan'
  | 'install'
  | 'distribute'
  | 'remove_distribution'
  | 'delete_skill'
  | 'update_skill'
  | 'inject_template'
  | 'rescan_security'

export interface TaskProgress {
  taskId: string
  taskType: TaskType
  status: 'queued' | 'running' | 'partial' | 'completed' | 'failed'
  step:
    | 'prepare'
    | 'scan'
    | 'download'
    | 'security_check'
    | 'persist'
    | 'distribute'
    | 'cleanup'
  current: number
  total: number
  message: string
  payload?: unknown
}
