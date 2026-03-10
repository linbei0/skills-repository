import { useEffect, useRef, useState } from 'react'
import { RouterProvider } from 'react-router-dom'
import { useTranslation } from 'react-i18next'
import { router } from './app/router'
import { AppErrorBoundary } from './components/common/AppErrorBoundary'
import { bootstrapApp } from './lib/tauri-client'
import { applyResolvedTheme, resolveThemeMode } from './lib/theme'
import { useAppStore } from './stores/use-app-store'
import { useSettingsStore } from './stores/use-settings-store'
import { useTaskStore } from './stores/use-task-store'

function App() {
  const { i18n } = useTranslation()
  const startedRef = useRef(false)
  const [startupStep, setStartupStep] = useState('准备附加任务监听')
  const { bootstrapped, bootstrapping, error, setBootstrapPayload, setBootstrapError } =
    useAppStore()
  const attachTaskListeners = useTaskStore((state) => state.attachTaskListeners)
  const setSettings = useSettingsStore((state) => state.setSettings)
  const settings = useSettingsStore((state) => state.settings)
  const system = useAppStore((state) => state.system)

  useEffect(() => {
    if (startedRef.current) return
    startedRef.current = true

    let mounted = true
    let disposeTasks: VoidFunction | undefined
    let bootstrapGuardsActive = true

    const removeBootstrapGlobalListeners = () => {
      if (!bootstrapGuardsActive) return
      bootstrapGuardsActive = false
      window.removeEventListener('error', handleWindowError)
      window.removeEventListener('unhandledrejection', handleUnhandledRejection)
    }

    const bootstrapTimeout = window.setTimeout(() => {
      if (!mounted || !bootstrapGuardsActive) return
      console.error('[startup] bootstrap_app timed out')
      removeBootstrapGlobalListeners()
      setBootstrapError('bootstrap_app timed out after 5 seconds')
    }, 5000)

    const handleWindowError = (event: ErrorEvent) => {
      console.error('[startup] window error', event.error ?? event.message)
      if (!mounted || !bootstrapGuardsActive) return
      setBootstrapError(
        event.error instanceof Error ? event.error.message : String(event.message),
      )
    }

    const handleUnhandledRejection = (event: PromiseRejectionEvent) => {
      console.error('[startup] unhandled rejection', event.reason)
      if (!mounted || !bootstrapGuardsActive) return
      const reason =
        event.reason instanceof Error ? event.reason.message : String(event.reason)
      setBootstrapError(reason)
    }

    window.addEventListener('error', handleWindowError)
    window.addEventListener('unhandledrejection', handleUnhandledRejection)

    void attachTaskListeners()
      .then((cleanup: VoidFunction) => {
        disposeTasks = cleanup
        setStartupStep('任务监听已附加')
      })
      .catch((cause) => {
        console.error('Failed to attach task listeners:', cause)
      })

    console.info('[startup] bootstrap_app started')
    void bootstrapApp()
      .then((payload) => {
        if (!mounted) return

        window.clearTimeout(bootstrapTimeout)
        removeBootstrapGlobalListeners()
        console.info('[startup] bootstrap_app resolved', payload)
        setStartupStep('bootstrap_app 已返回，正在应用状态')
        setBootstrapPayload(payload)
        setSettings(payload.settings)
      })
      .catch((cause) => {
        if (!mounted) return
        window.clearTimeout(bootstrapTimeout)
        removeBootstrapGlobalListeners()
        console.error('[startup] bootstrap_app failed', cause)
        setBootstrapError(cause instanceof Error ? cause.message : String(cause))
      })

    return () => {
      mounted = false
      window.clearTimeout(bootstrapTimeout)
      removeBootstrapGlobalListeners()
      disposeTasks?.()
    }
  }, [attachTaskListeners, setBootstrapError, setBootstrapPayload, setSettings])

  useEffect(() => {
    if (!bootstrapped || !system) return

    const resolvedTheme = resolveThemeMode(settings.themeMode, system.theme)
    applyResolvedTheme(resolvedTheme)

    void i18n.changeLanguage(settings.language).catch((cause) => {
      console.error('Failed to change language:', cause)
    })
  }, [bootstrapped, i18n, settings.language, settings.themeMode, system])

  if (bootstrapping) {
    return (
      <div className="flex min-h-screen items-center justify-center bg-base-200 text-base-content">
        <div className="rounded-box border border-base-300 bg-base-100 p-8 shadow-sm">
          <p className="text-lg font-semibold">正在启动 skills管理器...</p>
          <p className="mt-2 text-sm text-base-content/60">
            正在加载系统设置、主题、语言和 Agent 能力矩阵。
          </p>
          <p className="mt-4 font-mono text-xs text-base-content/50">{startupStep}</p>
        </div>
      </div>
    )
  }

  if (error) {
    return (
      <div className="flex min-h-screen items-center justify-center bg-base-200 text-base-content">
        <div className="max-w-xl rounded-box border border-error/40 bg-base-100 p-8 shadow-sm">
          <p className="text-lg font-semibold text-error">启动失败</p>
          <p className="mt-2 text-sm text-base-content/70">{error}</p>
          <p className="mt-4 text-xs text-base-content/50">
            请检查本地数据库初始化、权限以及 Tauri 环境是否正常。
          </p>
        </div>
      </div>
    )
  }

  if (!bootstrapped) return null

  return (
    <AppErrorBoundary>
      <RouterProvider router={router} />
    </AppErrorBoundary>
  )
}

export default App
