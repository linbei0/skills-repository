import { create } from 'zustand'
import i18n from '../lib/i18n'
import {
  DEFAULT_SETTINGS_SKILLS_TARGET_IDS,
  removeCustomSkillsTarget as removeCustomSkillsTargetFromList,
} from '../lib/skills-targets'
import { saveSettings as saveSettingsCommand } from '../lib/tauri-client'
import { applyResolvedTheme, resolveThemeMode } from '../lib/theme'
import type { AppLocale, AppSettings, CustomSkillsTarget, ThemeMode } from '../types/app'
import { useAppStore } from './use-app-store'

interface SettingsStoreState {
  settings: AppSettings
  saving: boolean
  setSettings: (settings: AppSettings) => void
  setLanguage: (language: AppLocale) => void
  setThemeMode: (themeMode: ThemeMode) => void
  toggleVisibleSkillsTarget: (targetId: string) => void
  addCustomSkillsTarget: (target: CustomSkillsTarget) => void
  removeCustomSkillsTarget: (targetId: string) => void
  save: () => Promise<void>
}

const defaultSettings: AppSettings = {
  language: 'en-US',
  themeMode: 'system',
  visibleSkillsTargetIds: [...DEFAULT_SETTINGS_SKILLS_TARGET_IDS],
  customSkillsTargets: [],
}

export const useSettingsStore = create<SettingsStoreState>((set, get) => ({
  settings: defaultSettings,
  saving: false,
  setSettings: (settings) => set({ settings }),
  setLanguage: (language) =>
    set((state) => ({
      settings: { ...state.settings, language },
    })),
  setThemeMode: (themeMode) =>
    set((state) => ({
      settings: { ...state.settings, themeMode },
    })),
  toggleVisibleSkillsTarget: (targetId) =>
    set((state) => {
      const visibleSkillsTargetIds = state.settings.visibleSkillsTargetIds.includes(targetId)
        ? state.settings.visibleSkillsTargetIds.filter((id) => id !== targetId)
        : [...state.settings.visibleSkillsTargetIds, targetId]

      return {
        settings: {
          ...state.settings,
          visibleSkillsTargetIds,
        },
      }
    }),
  addCustomSkillsTarget: (target) =>
    set((state) => ({
      settings: {
        ...state.settings,
        customSkillsTargets: [...state.settings.customSkillsTargets, target],
        visibleSkillsTargetIds: [...state.settings.visibleSkillsTargetIds, target.id],
      },
    })),
  removeCustomSkillsTarget: (targetId) =>
    set((state) => ({
      settings: {
        ...state.settings,
        customSkillsTargets: removeCustomSkillsTargetFromList(
          state.settings.customSkillsTargets,
          targetId,
        ),
        visibleSkillsTargetIds: state.settings.visibleSkillsTargetIds.filter(
          (id) => id !== targetId,
        ),
      },
    })),
  save: async () => {
    set({ saving: true })
    try {
      const saved = await saveSettingsCommand(get().settings)
      set({ settings: saved, saving: false })

      const systemTheme = useAppStore.getState().system?.theme ?? 'light'
      applyResolvedTheme(resolveThemeMode(saved.themeMode, systemTheme))
      await i18n.changeLanguage(saved.language)
    } catch (error) {
      set({ saving: false })
      throw error
    }
  },
}))
