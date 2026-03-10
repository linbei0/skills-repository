export type ThemeMode = 'system' | 'light' | 'dark'
export type ResolvedTheme = 'skills-light' | 'skills-dark'
export type AppLocale = 'zh-CN' | 'en-US' | 'ja-JP'

export interface AppSettings {
  language: AppLocale
  themeMode: ThemeMode
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

export interface BootstrapPayload {
  appVersion: string
  system: SystemInfo
  settings: AppSettings
  agents: AgentCapability[]
}

export interface RepositorySkillSummary {
  id: string
  name: string
  sourceType: string
  sourceMarket?: string | null
  installedAt: number
  securityLevel: string
  blocked: boolean
}

export interface RepositorySkillDetail {
  id: string
  name: string
  canonicalPath: string
  sourceType: string
  sourceMarket?: string | null
  sourceUrl?: string | null
  installedAt: number
  securityLevel: string
  blocked: boolean
  skillMarkdown: string
}

export interface RepositoryUninstallResult {
  skillId: string
  removedPaths: string[]
}

export interface AgentGlobalSkillEntry {
  id: string
  name: string
  path: string
  relationship: 'linked' | 'directory' | 'broken'
}

export interface AgentGlobalScanResult {
  agentId: string
  agentLabel: string
  rootPath: string
  entries: AgentGlobalSkillEntry[]
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
  createdAt: number
  updatedAt: number
}

export interface SaveTemplateRequest {
  id?: string | null
  name: string
  description?: string | null
  tags: string[]
}

