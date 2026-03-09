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
  status: 'active' | 'broken' | 'removed'
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
