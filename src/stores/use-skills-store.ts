import { create } from 'zustand'
import { distributeSkill as distributeSkillCommand, scanSkills as scanSkillsCommand } from '../lib/tauri-client'
import type {
  DistributionRecord,
  DistributionRequest,
  DistributionResult,
  ProjectRecord,
  ScanSkillsRequest,
  ScanSkillsResult,
  SkillRecord,
} from '../types/app'

interface SkillsStoreState {
  skills: SkillRecord[]
  projects: ProjectRecord[]
  duplicates: Array<{ name: string; paths: string[] }>
  distributions: DistributionRecord[]
  scanTaskId: string | null
  scanSkills: (request: ScanSkillsRequest) => Promise<void>
  distributeSkill: (request: DistributionRequest) => Promise<void>
  applyScanResult: (result: ScanSkillsResult) => void
  applyDistributionResult: (result: DistributionResult) => void
}

export const useSkillsStore = create<SkillsStoreState>((set) => ({
  skills: [],
  projects: [],
  duplicates: [],
  distributions: [],
  scanTaskId: null,
  scanSkills: async (request) => {
    const handle = await scanSkillsCommand(request)
    set({ scanTaskId: handle.taskId })
  },
  distributeSkill: async (request) => {
    await distributeSkillCommand(request)
  },
  applyScanResult: (result) =>
    set({
      skills: result.skills,
      projects: result.projects,
      duplicates: result.duplicates,
      distributions: result.distributions,
      scanTaskId: null,
    }),
  applyDistributionResult: (result) =>
    set((state) => ({
      distributions: [
        {
          id: result.distributionId || `failed:${result.skillId}:${result.targetAgent}`,
          skillId: result.skillId,
          targetAgent: result.targetAgent,
          targetPath: result.targetPath,
          status: result.status as DistributionRecord['status'],
        },
        ...state.distributions.filter(
          (item) =>
            item.id !== result.distributionId &&
            !(item.skillId === result.skillId && item.targetAgent === result.targetAgent),
        ),
      ],
    })),
}))
