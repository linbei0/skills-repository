import { create } from 'zustand'
import {
  getRepositorySkillDetail as getRepositorySkillDetailCommand,
  importRepositorySkill as importRepositorySkillCommand,
  listRepositorySkills as listRepositorySkillsCommand,
  resolveRepositoryImportSource as resolveRepositoryImportSourceCommand,
  uninstallRepositorySkill as uninstallRepositorySkillCommand,
} from '../lib/tauri-client'
import type {
  InstallSkillResult,
  ImportRepositorySkillRequest,
  RepositorySkillDetail,
  RepositorySkillSummary,
  ResolveRepositoryImportRequest,
  ResolveRepositoryImportResult,
} from '../types/app'

interface RepositoryStoreState {
  items: RepositorySkillSummary[]
  loading: boolean
  loaded: boolean
  error: string | null
  selectedDetail: RepositorySkillDetail | null
  detailLoading: boolean
  detailError: string | null
  uninstallingSkillId: string | null
  resolvingImport: boolean
  importing: boolean
  importError: string | null
  importBlockedLevel: string | null
  resolvedImport: ResolveRepositoryImportResult | null
  refresh: () => Promise<void>
  loadDetail: (skillId: string) => Promise<void>
  closeDetail: () => void
  uninstall: (skillId: string) => Promise<void>
  resolveImport: (request: ResolveRepositoryImportRequest) => Promise<ResolveRepositoryImportResult>
  importSkill: (request: ImportRepositorySkillRequest) => Promise<InstallSkillResult>
  resetImportState: () => void
}

export const useRepositoryStore = create<RepositoryStoreState>((set, get) => ({
  items: [],
  loading: false,
  loaded: false,
  error: null,
  selectedDetail: null,
  detailLoading: false,
  detailError: null,
  uninstallingSkillId: null,
  resolvingImport: false,
  importing: false,
  importError: null,
  importBlockedLevel: null,
  resolvedImport: null,
  refresh: async () => {
    set({ loading: true, error: null })
    try {
      const items = await listRepositorySkillsCommand()
      set({ items, loading: false, loaded: true, error: null })
    } catch (error) {
      set({
        loading: false,
        loaded: true,
        error: error instanceof Error ? error.message : String(error),
      })
    }
  },
  loadDetail: async (skillId) => {
    set({ detailLoading: true, detailError: null, selectedDetail: null })
    try {
      const selectedDetail = await getRepositorySkillDetailCommand(skillId)
      set({ selectedDetail, detailLoading: false, detailError: null })
    } catch (error) {
      set({
        detailLoading: false,
        detailError: error instanceof Error ? error.message : String(error),
      })
    }
  },
  closeDetail: () => set({ selectedDetail: null, detailLoading: false, detailError: null }),
  uninstall: async (skillId) => {
    set({ uninstallingSkillId: skillId })
    try {
      const result = await uninstallRepositorySkillCommand(skillId)
      set((state) => ({
        items: state.items.filter((item) => item.id !== result.skillId),
        selectedDetail: state.selectedDetail?.id === result.skillId ? null : state.selectedDetail,
      }))
    } finally {
      set({ uninstallingSkillId: null })
    }
  },
  resolveImport: async (request) => {
    set({ resolvingImport: true, importError: null, importBlockedLevel: null, resolvedImport: null })
    try {
      const resolvedImport = await resolveRepositoryImportSourceCommand(request)
      set({ resolvingImport: false, resolvedImport, importError: null, importBlockedLevel: null })
      return resolvedImport
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error)
      set({ resolvingImport: false, importError: message })
      throw error
    }
  },
  importSkill: async (request) => {
    set({ importing: true, importError: null, importBlockedLevel: null })
    try {
      const result = await importRepositorySkillCommand(request)
      const items = await listRepositorySkillsCommand()
      set({
        items,
        importing: false,
        importError: null,
        importBlockedLevel: result.blocked ? result.securityLevel : null,
        resolvedImport: result.blocked ? get().resolvedImport : null,
        loaded: true,
      })
      return result
    } catch (error) {
      set({
        importing: false,
        importError: error instanceof Error ? error.message : String(error),
        importBlockedLevel: null,
      })
      throw error
    }
  },
  resetImportState: () =>
    set({
      resolvingImport: false,
      importing: false,
      importError: null,
      importBlockedLevel: null,
      resolvedImport: null,
    }),
}))
