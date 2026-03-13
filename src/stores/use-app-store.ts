import { create } from 'zustand'
import type { AgentCapability, BootstrapPayload, RepositoryStorageInfo, SystemInfo } from '../types/app'

interface AppStoreState {
  bootstrapping: boolean
  bootstrapped: boolean
  error: string | null
  system: SystemInfo | null
  agents: AgentCapability[]
  repositoryStorage: RepositoryStorageInfo | null
  setBootstrapPayload: (payload: BootstrapPayload) => void
  setRepositoryStorage: (repositoryStorage: RepositoryStorageInfo) => void
  setBootstrapError: (message: string) => void
}

export const useAppStore = create<AppStoreState>((set) => ({
  bootstrapping: true,
  bootstrapped: false,
  error: null,
  system: null,
  agents: [],
  repositoryStorage: null,
  setBootstrapPayload: (payload) =>
    set({
      bootstrapping: false,
      bootstrapped: true,
      error: null,
      system: payload.system,
      agents: payload.agents,
      repositoryStorage: payload.repositoryStorage,
    }),
  setRepositoryStorage: (repositoryStorage) => set({ repositoryStorage }),
  setBootstrapError: (message) =>
    set({
      bootstrapping: false,
      error: message,
    }),
}))
