import { create } from 'zustand'
import {
  onTaskCompleted,
  onTaskFailed,
  onTaskProgress,
} from '../lib/tauri-client'
import type {
  DistributionResult,
  ScanSkillsResult,
  TaskProgress,
  TemplateInjectionResult,
} from '../types/app'
import { useAppStore } from './use-app-store'
import { useSecurityStore } from './use-security-store'
import { useSkillsStore } from './use-skills-store'
import { useTemplatesStore } from './use-templates-store'

interface TaskStoreState {
  attached: boolean
  tasks: TaskProgress[]
  registerTask: (task: TaskProgress) => void
  upsertTask: (task: TaskProgress) => void
  attachTaskListeners: () => Promise<VoidFunction>
}

const byMostRecent = (tasks: TaskProgress[]) =>
  [...tasks].sort((left, right) => right.taskId.localeCompare(left.taskId))

export const useTaskStore = create<TaskStoreState>((set, get) => ({
  attached: false,
  tasks: [],
  registerTask: (task) =>
    set((state) => ({
      tasks: byMostRecent([
        task,
        ...state.tasks.filter((item) => item.taskId !== task.taskId),
      ]),
    })),
  upsertTask: (task) =>
    set((state) => ({
      tasks: byMostRecent([
        task,
        ...state.tasks.filter((item) => item.taskId !== task.taskId),
      ]),
    })),
  attachTaskListeners: async () => {
    if (get().attached) return () => undefined

    set({ attached: true })

    const handleCompletedLikeEvent = (event: TaskProgress) => {
      get().upsertTask(event)

      if (event.taskType === 'scan' && event.payload) {
        const result = event.payload as ScanSkillsResult
        useSkillsStore.getState().applyScanResult(result)
        useAppStore.getState().setOverview(result.overview)
      }

      if (event.taskType === 'distribute' && event.payload) {
        const result = event.payload as DistributionResult
        useSkillsStore.getState().applyDistributionResult(result)
      }

      if (event.taskType === 'rescan_security') {
        void useSecurityStore.getState().refresh()
      }

      if (event.taskType === 'inject_template' && event.payload) {
        const result = event.payload as TemplateInjectionResult
        useTemplatesStore.getState().applyInjectionResult(result)
        void useSecurityStore.getState().refresh()
      }
    }

    const unlistenProgress = await onTaskProgress((event) => {
      get().upsertTask(event)
    })
    const unlistenCompleted = await onTaskCompleted(handleCompletedLikeEvent)
    const unlistenFailed = await onTaskFailed((event) => {
      get().upsertTask(event)

      if (event.taskType === 'distribute' && event.payload) {
        const result = event.payload as DistributionResult
        useSkillsStore.getState().applyDistributionResult(result)
      }

      if (event.taskType === 'rescan_security') {
        void useSecurityStore.getState().refresh()
      }

      if (event.taskType === 'inject_template') {
        if (event.payload) {
          const result = event.payload as TemplateInjectionResult
          useTemplatesStore.getState().applyInjectionResult(result)
        } else {
          useTemplatesStore.getState().setInjectionError(event.message)
        }
        void useSecurityStore.getState().refresh()
      }
    })

    return () => {
      unlistenProgress()
      unlistenCompleted()
      unlistenFailed()
      set({ attached: false })
    }
  },
}))
