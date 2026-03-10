import type { AppSettings, CustomSkillsTarget } from '../types/app'

export interface BuiltinSkillsTarget {
  id: string
  label: string
  relativePath: string
}

export interface SkillsTargetOption {
  id: string
  label: string
  relativePath: string
  isCustom: boolean
}

export const BUILTIN_SKILLS_TARGETS: BuiltinSkillsTarget[] = [
  { id: 'universal', label: '通用', relativePath: '.agents/skills' },
  { id: 'antigravity', label: 'Antigravity', relativePath: '.agent/skills' },
  { id: 'augment', label: 'Augment', relativePath: '.augment/skills' },
  { id: 'claude-code', label: 'Claude Code', relativePath: '.claude/skills' },
  { id: 'openclaw', label: 'OpenClaw', relativePath: 'skills' },
  { id: 'codebuddy', label: 'CodeBuddy', relativePath: '.codebuddy/skills' },
  { id: 'command-code', label: 'Command Code', relativePath: '.commandcode/skills' },
  { id: 'continue', label: 'Continue', relativePath: '.continue/skills' },
  { id: 'cortex-code', label: 'Cortex Code', relativePath: '.cortex/skills' },
  { id: 'crush', label: 'Crush', relativePath: '.crush/skills' },
  { id: 'droid', label: 'Droid', relativePath: '.factory/skills' },
  { id: 'goose', label: 'Goose', relativePath: '.goose/skills' },
  { id: 'junie', label: 'Junie', relativePath: '.junie/skills' },
  { id: 'iflow-cli', label: 'iFlow CLI', relativePath: '.iflow/skills' },
  { id: 'kilo-code', label: 'Kilo Code', relativePath: '.kilocode/skills' },
  { id: 'kiro-cli', label: 'Kiro CLI', relativePath: '.kiro/skills' },
  { id: 'kode', label: 'Kode', relativePath: '.kode/skills' },
  { id: 'mcpjam', label: 'MCPJam', relativePath: '.mcpjam/skills' },
  { id: 'mistral-vibe', label: 'Mistral Vibe', relativePath: '.vibe/skills' },
  { id: 'mux', label: 'Mux', relativePath: '.mux/skills' },
  { id: 'openhands', label: 'OpenHands', relativePath: '.openhands/skills' },
  { id: 'pi', label: 'Pi', relativePath: '.pi/skills' },
  { id: 'qoder', label: 'Qoder', relativePath: '.qoder/skills' },
  { id: 'qwen-code', label: 'Qwen Code', relativePath: '.qwen/skills' },
  { id: 'roo-code', label: 'Roo Code', relativePath: '.roo/skills' },
  { id: 'trae', label: 'Trae', relativePath: '.trae/skills' },
  { id: 'trae-cn', label: 'Trae CN', relativePath: '.trae/skills' },
  { id: 'windsurf', label: 'Windsurf', relativePath: '.windsurf/skills' },
  { id: 'zencoder', label: 'Zencoder', relativePath: '.zencoder/skills' },
  { id: 'neovate', label: 'Neovate', relativePath: '.neovate/skills' },
  { id: 'pochi', label: 'Pochi', relativePath: '.pochi/skills' },
  { id: 'adal', label: 'AdaL', relativePath: '.adal/skills' },
]

export const DEFAULT_VISIBLE_SKILLS_TARGET_IDS = [
  'universal',
  'antigravity',
  'claude-code',
  'codebuddy',
  'kiro-cli',
  'openclaw',
  'qoder',
  'trae',
  'windsurf',
] as const

export const DEFAULT_SETTINGS_SKILLS_TARGET_IDS = [...DEFAULT_VISIBLE_SKILLS_TARGET_IDS]

export const resolveSkillsTargets = (settings: AppSettings): SkillsTargetOption[] => [
  ...BUILTIN_SKILLS_TARGETS.map((target) => ({ ...target, isCustom: false })),
  ...settings.customSkillsTargets.map((target) => ({
    ...target,
    isCustom: true,
  })),
]

export const createCustomSkillsTargetId = (label: string) => {
  const normalized = label
    .toLowerCase()
    .trim()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '')

  return `custom-${normalized || crypto.randomUUID()}`
}

export const normalizeRelativeSkillsPath = (relativePath: string) =>
  relativePath.replace(/\\/g, '/').trim()

export const hasSkillsTarget = (settings: AppSettings, targetId: string) =>
  resolveSkillsTargets(settings).some((target) => target.id === targetId)

export const removeCustomSkillsTarget = (
  customSkillsTargets: CustomSkillsTarget[],
  targetId: string,
) => customSkillsTargets.filter((target) => target.id !== targetId)
