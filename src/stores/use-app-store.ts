import { create } from 'zustand'
import type { AgentCapability, BootstrapPayload, SystemInfo } from '../types/app'

interface AppStoreState {
  bootstrapping: boolean
  bootstrapped: boolean
  error: string | null
  system: SystemInfo | null
  agents: AgentCapability[]
  setBootstrapPayload: (payload: BootstrapPayload) => void
  setBootstrapError: (message: string) => void
}

export const useAppStore = create<AppStoreState>((set) => ({
  bootstrapping: true,
  bootstrapped: false,
  error: null,
  system: null,
  agents: [],
  setBootstrapPayload: (payload) =>
    set({
      bootstrapping: false,
      bootstrapped: true,
      error: null,
      system: payload.system,
      agents: payload.agents,
    }),
  setBootstrapError: (message) =>
    set({
      bootstrapping: false,
      error: message,
    }),
}))
