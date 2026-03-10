import { create } from 'zustand'
import { scanAgentGlobalSkills as scanAgentGlobalSkillsCommand } from '../lib/tauri-client'
import type {
  AgentGlobalScanRequest,
  AgentGlobalSkillEntry,
} from '../types/app'

interface SkillsStoreState {
  selectedAgentId: string
  loading: boolean
  loaded: boolean
  error: string | null
  rootPath: string | null
  entries: AgentGlobalSkillEntry[]
  setSelectedAgentId: (agentId: string) => void
  scanAgentGlobalSkills: (request: AgentGlobalScanRequest) => Promise<void>
}

export const useSkillsStore = create<SkillsStoreState>((set) => ({
  selectedAgentId: 'universal',
  loading: false,
  loaded: false,
  error: null,
  rootPath: null,
  entries: [],
  setSelectedAgentId: (agentId) => set({ selectedAgentId: agentId }),
  scanAgentGlobalSkills: async (request) => {
    set({ selectedAgentId: request.agentId, loading: true, error: null })
    try {
      const result = await scanAgentGlobalSkillsCommand(request)
      set({
        selectedAgentId: request.agentId,
        loading: false,
        loaded: true,
        error: null,
        rootPath: result.rootPath,
        entries: result.entries,
      })
    } catch (error) {
      set({
        selectedAgentId: request.agentId,
        loading: false,
        loaded: true,
        rootPath: null,
        entries: [],
        error: error instanceof Error ? error.message : String(error),
      })
    }
  },
}))
