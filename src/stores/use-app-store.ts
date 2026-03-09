import { create } from 'zustand'
import type {
  AgentCapability,
  BootstrapPayload,
  OverviewStats,
  SystemInfo,
} from '../types/app'

interface AppStoreState {
  bootstrapping: boolean
  bootstrapped: boolean
  error: string | null
  system: SystemInfo | null
  agents: AgentCapability[]
  overview: OverviewStats
  setBootstrapPayload: (payload: BootstrapPayload) => void
  setBootstrapError: (message: string) => void
  setOverview: (overview: OverviewStats) => void
}

const emptyOverview: OverviewStats = {
  totalSkills: 0,
  riskySkills: null,
  duplicatePaths: 0,
  reclaimableBytes: null,
  templateCount: null,
}

export const useAppStore = create<AppStoreState>((set) => ({
  bootstrapping: true,
  bootstrapped: false,
  error: null,
  system: null,
  agents: [],
  overview: emptyOverview,
  setBootstrapPayload: (payload) =>
    set({
      bootstrapping: false,
      bootstrapped: true,
      error: null,
      system: payload.system,
      agents: payload.agents,
      overview: payload.overview,
    }),
  setBootstrapError: (message) =>
    set({
      bootstrapping: false,
      error: message,
    }),
  setOverview: (overview) => set({ overview }),
}))
