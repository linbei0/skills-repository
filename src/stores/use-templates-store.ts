import { create } from 'zustand'
import {
  deleteTemplate as deleteTemplateCommand,
  injectTemplate as injectTemplateCommand,
  listRepositorySkills as listRepositorySkillsCommand,
  listTemplates as listTemplatesCommand,
  saveTemplate as saveTemplateCommand,
} from '../lib/tauri-client'
import type {
  InjectTemplateRequest,
  InjectTemplateResult,
  RepositorySkillSummary,
  SaveTemplateRequest,
  TemplateRecord,
} from '../types/app'

interface TemplatesStoreState {
  templates: TemplateRecord[]
  repositorySkills: RepositorySkillSummary[]
  loading: boolean
  saving: boolean
  deleting: boolean
  injecting: boolean
  loaded: boolean
  error: string | null
  lastInjectResult: InjectTemplateResult | null
  refresh: () => Promise<void>
  refreshRepositorySkills: () => Promise<void>
  saveTemplate: (request: SaveTemplateRequest) => Promise<TemplateRecord>
  deleteTemplate: (templateId: string) => Promise<void>
  injectTemplate: (request: InjectTemplateRequest) => Promise<InjectTemplateResult>
  clearInjectResult: () => void
}

const toErrorMessage = (error: unknown) =>
  error instanceof Error ? error.message : String(error)

export const useTemplatesStore = create<TemplatesStoreState>((set) => ({
  templates: [],
  repositorySkills: [],
  loading: false,
  saving: false,
  deleting: false,
  injecting: false,
  loaded: false,
  error: null,
  lastInjectResult: null,
  refresh: async () => {
    set({ loading: true, error: null })
    try {
      const [templates, repositorySkills] = await Promise.all([
        listTemplatesCommand(),
        listRepositorySkillsCommand(),
      ])

      set({
        templates,
        repositorySkills,
        loading: false,
        loaded: true,
      })
    } catch (error) {
      set({
        loading: false,
        loaded: true,
        error: toErrorMessage(error),
      })
    }
  },
  refreshRepositorySkills: async () => {
    try {
      const repositorySkills = await listRepositorySkillsCommand()
      set({
        repositorySkills,
        error: null,
      })
    } catch (error) {
      set({
        error: toErrorMessage(error),
      })
      throw error
    }
  },
  saveTemplate: async (request) => {
    set({ saving: true, error: null })
    try {
      const template = await saveTemplateCommand(request)
      const templates = await listTemplatesCommand()
      set({
        templates,
        saving: false,
      })
      return template
    } catch (error) {
      set({ saving: false, error: toErrorMessage(error) })
      throw error
    }
  },
  deleteTemplate: async (templateId) => {
    set({ deleting: true, error: null })
    try {
      await deleteTemplateCommand(templateId)
      const templates = await listTemplatesCommand()
      set({
        templates,
        deleting: false,
      })
    } catch (error) {
      set({ deleting: false, error: toErrorMessage(error) })
      throw error
    }
  },
  injectTemplate: async (request) => {
    set({ injecting: true, error: null, lastInjectResult: null })
    try {
      const result = await injectTemplateCommand(request)
      set({
        injecting: false,
        lastInjectResult: result,
      })
      return result
    } catch (error) {
      set({
        injecting: false,
        error: toErrorMessage(error),
      })
      throw error
    }
  },
  clearInjectResult: () => set({ lastInjectResult: null }),
}))
