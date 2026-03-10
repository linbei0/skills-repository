import { NavLink, Outlet } from 'react-router-dom'
import { useTranslation } from 'react-i18next'
import { cn } from '../../lib/cn'
import { resolveThemeMode } from '../../lib/theme'
import type { AppLocale } from '../../types/app'
import { useAppStore } from '../../stores/use-app-store'
import { useSettingsStore } from '../../stores/use-settings-store'

const navItems = [
  { to: '/', key: 'repository', icon: 'hn-home' },
  { to: '/skills', key: 'skills', icon: 'hn-folder' },
  { to: '/market', key: 'market', icon: 'hn-search' },
  { to: '/security', key: 'security', icon: 'hn-lock-alt' },
  { to: '/templates', key: 'templates', icon: 'hn-grid' },
] as const

export function AppShell() {
  const { t } = useTranslation()
  const system = useAppStore((state) => state.system)
  const settings = useSettingsStore((state) => state.settings)
  const setLanguage = useSettingsStore((state) => state.setLanguage)
  const setThemeMode = useSettingsStore((state) => state.setThemeMode)
  const saveSettings = useSettingsStore((state) => state.save)
  const saving = useSettingsStore((state) => state.saving)
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
              <i className={isDarkTheme ? 'hn hn-sun' : 'hn hn-moon'} aria-hidden />
            </button>
          </div>
        </header>

        <main className="min-h-0 flex-1 overflow-y-auto px-6 py-6">
          <Outlet />
        </main>
      </div>
    </div>
  )
}

