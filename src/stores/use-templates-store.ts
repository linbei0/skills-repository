import { create } from 'zustand'
import {
  deleteTemplate as deleteTemplateCommand,
  getTemplate as getTemplateCommand,
  injectTemplate as injectTemplateCommand,
  listTemplates as listTemplatesCommand,
  saveTemplate as saveTemplateCommand,
} from '../lib/tauri-client'
import type {
  SaveTemplateRequest,
  TemplateInjectionRequest,
  TemplateInjectionResult,
  TemplateRecord,
} from '../types/app'

interface TemplatesStoreState {
  templates: TemplateRecord[]
  selectedTemplateId: string | null
  selectedTemplate: TemplateRecord | null
  loading: boolean
  saving: boolean
  deleting: boolean
  injecting: boolean
  loaded: boolean
  error: string | null
  injectResult: TemplateInjectionResult | null
  refresh: () => Promise<void>
  selectTemplate: (templateId: string | null) => Promise<TemplateRecord | null>
  saveTemplate: (request: SaveTemplateRequest) => Promise<TemplateRecord>
  deleteTemplate: (templateId: string) => Promise<void>
  injectTemplate: (request: TemplateInjectionRequest) => Promise<void>
  applyInjectionResult: (result: TemplateInjectionResult) => void
  setInjectionError: (message: string) => void
}

const toErrorMessage = (error: unknown) =>
  error instanceof Error ? error.message : String(error)

export const useTemplatesStore = create<TemplatesStoreState>((set, get) => ({
  templates: [],
  selectedTemplateId: null,
  selectedTemplate: null,
  loading: false,
  saving: false,
  deleting: false,
  injecting: false,
  loaded: false,
  error: null,
  injectResult: null,
  refresh: async () => {
    set({ loading: true, error: null })
    try {
      const templates = await listTemplatesCommand()
      const selectedTemplateId = get().selectedTemplateId
      const nextSelectedId =
        selectedTemplateId && templates.some((item) => item.id === selectedTemplateId)
          ? selectedTemplateId
          : null
      const selectedTemplate =
        templates.find((item) => item.id === nextSelectedId) ?? null
      set({
        templates,
        selectedTemplateId: nextSelectedId,
        selectedTemplate,
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
  selectTemplate: async (templateId) => {
    if (!templateId) {
      set({ selectedTemplateId: null, selectedTemplate: null, injectResult: null })
      return null
    }

    set({ loading: true, error: null, selectedTemplateId: templateId, injectResult: null })
    try {
      const template = await getTemplateCommand(templateId)
      set({
        selectedTemplate: template,
        loading: false,
      })
      return template
    } catch (error) {
      set({
        selectedTemplate: null,
        loading: false,
        error: toErrorMessage(error),
      })
      return null
    }
  },
  saveTemplate: async (request) => {
    set({ saving: true, error: null })
    try {
      const template = await saveTemplateCommand(request)
      const templates = await listTemplatesCommand()
      set({
        templates,
        selectedTemplateId: template.id,
        selectedTemplate: template,
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
        selectedTemplateId: null,
        selectedTemplate: null,
        deleting: false,
        injectResult: null,
      })
    } catch (error) {
      set({ deleting: false, error: toErrorMessage(error) })
      throw error
    }
  },
  injectTemplate: async (request) => {
    set({ injecting: true, error: null, injectResult: null })
    try {
      await injectTemplateCommand(request)
    } catch (error) {
      set({ injecting: false, error: toErrorMessage(error) })
      throw error
    }
  },
  applyInjectionResult: (result) =>
    set({
      injecting: false,
      injectResult: result,
      error: null,
    }),
  setInjectionError: (message) =>
    set({
      injecting: false,
      error: message,
    }),
}))
