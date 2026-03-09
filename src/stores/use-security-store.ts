import { create } from 'zustand'
import {
  getSecurityReports as getSecurityReportsCommand,
  rescanSecurity as rescanSecurityCommand,
} from '../lib/tauri-client'
import type { SecurityReport } from '../types/app'

interface SecurityStoreState {
  reports: SecurityReport[]
  loading: boolean
  loaded: boolean
  error: string | null
  refresh: () => Promise<void>
  rescan: () => Promise<void>
}

export const useSecurityStore = create<SecurityStoreState>((set) => ({
  reports: [],
  loading: false,
  loaded: false,
  error: null,
  refresh: async () => {
    set({ loading: true, error: null })
    try {
      const reports = await getSecurityReportsCommand()
      set({ reports, loading: false, loaded: true })
    } catch (error) {
      set({
        loading: false,
        loaded: true,
        error: error instanceof Error ? error.message : String(error),
      })
    }
  },
  rescan: async () => {
    await rescanSecurityCommand()
  },
}))
