import { create } from 'zustand'
import { searchMarketSkills as searchMarketSkillsCommand } from '../lib/tauri-client'
import type { MarketSearchResponse, MarketSkillSummary, ProviderStatus } from '../types/app'

interface MarketStoreState {
  query: string
  loading: boolean
  searched: boolean
  error: string | null
  results: MarketSkillSummary[]
  providers: ProviderStatus[]
  cacheHit: boolean
  total: number
  setQuery: (query: string) => void
  search: () => Promise<void>
}

export const useMarketStore = create<MarketStoreState>((set, get) => ({
  query: '',
  loading: false,
  searched: false,
  error: null,
  results: [],
  providers: [],
  cacheHit: false,
  total: 0,
  setQuery: (query) => set({ query }),
  search: async () => {
    set({ loading: true, error: null })
    try {
      const response: MarketSearchResponse = await searchMarketSkillsCommand({
        query: get().query,
        page: 1,
        pageSize: 10,
        enabledProviders: ['github'],
      })

      set({
        loading: false,
        searched: true,
        results: response.results,
        providers: response.providers,
        cacheHit: response.cacheHit,
        total: response.total,
      })
    } catch (error) {
      set({
        loading: false,
        searched: true,
        error: error instanceof Error ? error.message : String(error),
        results: [],
        providers: [],
        cacheHit: false,
        total: 0,
      })
    }
  },
}))
