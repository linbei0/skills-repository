import { create } from 'zustand'
import {
  getRepositorySkillDetail as getRepositorySkillDetailCommand,
  listRepositorySkills as listRepositorySkillsCommand,
  uninstallRepositorySkill as uninstallRepositorySkillCommand,
} from '../lib/tauri-client'
import type { RepositorySkillDetail, RepositorySkillSummary } from '../types/app'

interface RepositoryStoreState {
  items: RepositorySkillSummary[]
  loading: boolean
  loaded: boolean
  error: string | null
  selectedDetail: RepositorySkillDetail | null
  detailLoading: boolean
  detailError: string | null
  uninstallingSkillId: string | null
  refresh: () => Promise<void>
  loadDetail: (skillId: string) => Promise<void>
  closeDetail: () => void
  uninstall: (skillId: string) => Promise<void>
  removeSkill: (skillId: string) => void
}

export const useRepositoryStore = create<RepositoryStoreState>((set) => ({
  items: [],
  loading: false,
  loaded: false,
  error: null,
  selectedDetail: null,
  detailLoading: false,
  detailError: null,
  uninstallingSkillId: null,
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
      await uninstallRepositorySkillCommand(skillId)
    } finally {
      set({ uninstallingSkillId: null })
    }
  },
  removeSkill: (skillId) =>
    set((state) => ({
      items: state.items.filter((item) => item.id !== skillId),
      selectedDetail: state.selectedDetail?.id === skillId ? null : state.selectedDetail,
    })),
}))
