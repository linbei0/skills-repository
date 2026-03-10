import { useMemo, useState } from 'react'
import { NavLink, Outlet } from 'react-router-dom'
import { useTranslation } from 'react-i18next'
import { cn } from '../../lib/cn'
import { resolveThemeMode } from '../../lib/theme'
import type { AppLocale } from '../../types/app'
import { useAppStore } from '../../stores/use-app-store'
import { useSettingsStore } from '../../stores/use-settings-store'
import { useTaskStore } from '../../stores/use-task-store'

const navItems = [
  { to: '/', key: 'repository', icon: 'hn-home' },
  { to: '/skills', key: 'skills', icon: 'hn-folder' },
  { to: '/market', key: 'market', icon: 'hn-search' },
  { to: '/security', key: 'security', icon: 'hn-shield' },
  { to: '/templates', key: 'templates', icon: 'hn-list' },
] as const

export function AppShell() {
  const { t } = useTranslation()
  const [tasksOpen, setTasksOpen] = useState(false)
  const system = useAppStore((state) => state.system)
  const settings = useSettingsStore((state) => state.settings)
  const setLanguage = useSettingsStore((state) => state.setLanguage)
  const setThemeMode = useSettingsStore((state) => state.setThemeMode)
  const saveSettings = useSettingsStore((state) => state.save)
  const saving = useSettingsStore((state) => state.saving)
  const tasks = useTaskStore((state) => state.tasks)

  const activeTasks = useMemo(
    () => tasks.filter((task) => task.status === 'queued' || task.status === 'running'),
    [tasks],
  )
  const resolvedTheme = system
    ? resolveThemeMode(settings.themeMode, system.theme)
    : 'skills-light'
  const isDarkTheme = resolvedTheme === 'skills-dark'

  const languageOptions: Array<{ value: AppLocale; label: string }> = [
    { value: 'zh-CN', label: t('topbar.languageOptions.zhCN') },
    { value: 'en-US', label: t('topbar.languageOptions.enUS') },
    { value: 'ja-JP', label: t('topbar.languageOptions.jaJP') },
  ]

  const updateLanguageQuickly = async (language: AppLocale) => {
    setLanguage(language)
    await saveSettings()
  }

  const toggleThemeQuickly = async () => {
    const nextTheme = settings.themeMode === 'dark' ? 'light' : 'dark'
    setThemeMode(nextTheme)
    await saveSettings()
  }

  return (
    <div className="flex h-screen overflow-hidden bg-base-200 text-base-content">
      <aside className="sticky top-0 flex h-screen w-72 shrink-0 flex-col overflow-y-auto border-r border-base-300 bg-base-100/90 backdrop-blur">
        <div className="border-b border-base-300 px-5 py-5">
          <p className="text-xs uppercase tracking-[0.24em] text-primary">skills manager</p>
          <h1 className="mt-2 text-2xl font-semibold">{t('app.title')}</h1>
          <p className="mt-2 text-sm text-base-content/60">{t('app.subtitle')}</p>
        </div>

        <nav className="flex-1 px-3 py-4">
          <ul className="space-y-1">
            {navItems.map((item) => (
              <li key={item.key}>
                <NavLink
                  to={item.to}
                  end={item.to === '/'}
                  className={({ isActive }) =>
                    cn(
                      'flex items-center gap-3 rounded-box px-4 py-3 text-sm font-medium transition-colors',
                      isActive
                        ? 'bg-primary text-primary-content'
                        : 'text-base-content/70 hover:bg-base-200 hover:text-base-content',
                    )
                  }
                >
                  <i className={cn('hn text-base', item.icon)} aria-hidden />
                  <span>{t(`nav.${item.key}`)}</span>
                </NavLink>
              </li>
            ))}
          </ul>
        </nav>

        <div className="border-t border-base-300 px-5 py-4 text-xs text-base-content/50">
          {t('common.footer')}
        </div>
      </aside>

      <div className="flex min-w-0 flex-1 flex-col overflow-hidden">
        <header className="flex items-center justify-between gap-4 border-b border-base-300 bg-base-100/90 px-6 py-4 backdrop-blur">
          <div>
            <p className="text-sm font-medium text-base-content/70">{t('topbar.language')}</p>
            <p className="text-xs text-base-content/50">
              {system ? `${system.os.toUpperCase()} · ${system.arch}` : t('app.shellHint')}
            </p>
          </div>

          <div className="flex items-center gap-2">
            <label className="flex items-center gap-2 rounded-box border border-base-300 bg-base-100 px-3 py-2 text-sm">
              <span className="text-base-content/60">{t('topbar.language')}</span>
              <select
                className="select select-ghost select-xs min-h-0 h-auto border-0 bg-transparent pr-6 font-medium text-base-content focus:outline-none"
                value={settings.language}
                onChange={(event) => void updateLanguageQuickly(event.target.value as AppLocale)}
                disabled={saving}
                aria-label={t('topbar.language')}
              >
                {languageOptions.map((option) => (
                  <option key={option.value} value={option.value}>
                    {option.label}
                  </option>
                ))}
              </select>
            </label>

            <button
              className="btn btn-ghost btn-circle btn-sm text-xl"
              onClick={() => void toggleThemeQuickly()}
              aria-label={t('topbar.theme')}
              title={t('topbar.theme')}
            >
              <i className={cn('hn', isDarkTheme ? 'hn-sun' : 'hn-moon')} aria-hidden />
            </button>

            <button className="btn btn-primary btn-sm" onClick={() => setTasksOpen((open) => !open)}>
              <i className="hn hn-list" aria-hidden />
              <span>{t('topbar.tasks')}</span>
              {activeTasks.length > 0 ? (
                <span className="badge badge-sm bg-primary-content text-primary">{activeTasks.length}</span>
              ) : null}
            </button>
          </div>
        </header>

        <main className="flex min-h-0 flex-1 gap-6 overflow-hidden px-6 py-6">
          <section className="min-w-0 flex-1 overflow-y-auto pr-1">
            <Outlet />
          </section>

          <aside
            className={cn(
              'w-full max-w-[420px] shrink-0 self-start rounded-box border border-base-300 bg-base-100 transition-all duration-200',
              tasksOpen
                ? 'translate-x-0 opacity-100'
                : 'pointer-events-none hidden opacity-0 xl:block xl:translate-x-4',
            )}
          >
            <div className="border-b border-base-300 px-5 py-4">
              <p className="text-lg font-semibold">{t('tasks.title')}</p>
              <p className="mt-1 text-sm text-base-content/60">
                {system ? `${system.os.toUpperCase()} · ${system.arch}` : t('app.shellHint')}
              </p>
            </div>

            <div className="max-h-[calc(100vh-12rem)] space-y-3 overflow-y-auto p-4">
              {tasks.length === 0 ? (
                <div className="rounded-box border border-dashed border-base-300 bg-base-200/70 p-4 text-sm text-base-content/60">
                  {t('tasks.empty')}
                </div>
              ) : (
                tasks.map((task) => (
                  <article key={task.taskId} className="rounded-box border border-base-300 bg-base-200/60 p-4">
                    <div className="flex items-start justify-between gap-3">
                      <div>
                        <p className="text-sm font-semibold">{task.taskType}</p>
                        <p className="mt-1 text-xs text-base-content/55">{task.message}</p>
                      </div>
                      <span className="badge badge-outline">{t(`tasks.${task.status}`)}</span>
                    </div>
                    <progress
                      className="progress progress-primary mt-3 w-full"
                      value={task.current}
                      max={task.total || 1}
                    />
                    <div className="mt-2 flex items-center justify-between text-xs text-base-content/55">
                      <span>{task.step}</span>
                      <span>
                        {task.current}/{task.total}
                      </span>
                    </div>
                  </article>
                ))
              )}
            </div>
          </aside>
        </main>
      </div>
    </div>
  )
}
