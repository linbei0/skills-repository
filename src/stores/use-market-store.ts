import { create } from 'zustand'
import {
  installSkill as installSkillCommand,
  searchMarketSkills as searchMarketSkillsCommand,
} from '../lib/tauri-client'
import { useRepositoryStore } from './use-repository-store'
import type {
  InstallSkillResult,
  MarketSearchResponse,
  MarketSkillSummary,
  ProviderStatus,
} from '../types/app'

type MarketInstallStatus = 'installing' | 'installed' | 'blocked' | 'failed'

interface MarketInstallState {
  status: MarketInstallStatus
  message?: string
  canonicalPath?: string
  securityLevel?: string
  skillId?: string
}

const toErrorMessage = (error: unknown) => (error instanceof Error ? error.message : String(error))

interface MarketStoreState {
  query: string
  loading: boolean
  searched: boolean
  error: string | null
  results: MarketSkillSummary[]
  providers: ProviderStatus[]
  cacheHit: boolean
  total: number
  installStates: Record<string, MarketInstallState>
  setQuery: (query: string) => void
  search: () => Promise<void>
  install: (skill: MarketSkillSummary) => Promise<InstallSkillResult | null>
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
  installStates: {},
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
  install: async (skill) => {
    set((state) => ({
      installStates: {
        ...state.installStates,
        [skill.id]: {
          status: 'installing',
        },
      },
    }))

    try {
      const result = await installSkillCommand({
        provider: skill.provider,
        marketSkillId: skill.id,
        sourceType: skill.sourceType,
        sourceUrl: skill.sourceUrl,
        repoUrl: skill.repoUrl,
        downloadUrl: skill.downloadUrl,
        packageRef: skill.packageRef,
        manifestPath: skill.manifestPath,
        skillRoot: skill.skillRoot,
        name: skill.name,
        slug: skill.slug,
        version: skill.version,
        author: skill.author,
        requestedTargets: [],
      })

      set((state) => ({
        installStates: {
          ...state.installStates,
          [skill.id]: result.blocked
            ? {
                status: 'blocked',
                message: result.securityLevel,
                securityLevel: result.securityLevel,
              }
            : {
                status: 'installed',
                canonicalPath: result.canonicalPath,
                securityLevel: result.securityLevel,
                skillId: result.skillId,
              },
        },
      }))

      if (!result.blocked) {
        void useRepositoryStore.getState().refresh()
      }

      return result
    } catch (error) {
      set((state) => ({
        installStates: {
          ...state.installStates,
          [skill.id]: {
            status: 'failed',
            message: toErrorMessage(error),
          },
        },
      }))

      return null
    }
  },
}))
