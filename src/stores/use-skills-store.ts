import { create } from 'zustand'
import { scanAgentGlobalSkills as scanAgentGlobalSkillsCommand } from '../lib/tauri-client'
import type { AgentGlobalScanResult, AgentGlobalSkillEntry } from '../types/app'

interface SkillsStoreState {
  selectedAgentId: string
  loading: boolean
  loaded: boolean
  error: string | null
  rootPath: string | null
  entries: AgentGlobalSkillEntry[]
  setSelectedAgentId: (agentId: string) => void
  scanAgentGlobalSkills: (agentId: string) => Promise<void>
  applyScanResult: (result: AgentGlobalScanResult) => void
}

export const useSkillsStore = create<SkillsStoreState>((set) => ({
  selectedAgentId: 'codex',
  loading: false,
  loaded: false,
  error: null,
  rootPath: null,
  entries: [],
  setSelectedAgentId: (agentId) => set({ selectedAgentId: agentId }),
  scanAgentGlobalSkills: async (agentId) => {
    set({ selectedAgentId: agentId, loading: true, error: null })
    try {
      const result = await scanAgentGlobalSkillsCommand(agentId)
      set({
        selectedAgentId: agentId,
        loading: false,
        loaded: true,
        error: null,
        rootPath: result.rootPath,
        entries: result.entries,
      })
    } catch (error) {
      set({
        selectedAgentId: agentId,
        loading: false,
        loaded: true,
        rootPath: null,
        entries: [],
        error: error instanceof Error ? error.message : String(error),
      })
    }
  },
  applyScanResult: (result) =>
    set({
      selectedAgentId: result.agentId,
      loading: false,
      loaded: true,
      error: null,
      rootPath: result.rootPath,
      entries: result.entries,
    }),
}))
