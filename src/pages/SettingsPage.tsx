import { open as openDialog } from '@tauri-apps/plugin-dialog'
import { useEffect, useMemo, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { migrateRepositoryStorage } from '../lib/tauri-client'
import { useAppStore } from '../stores/use-app-store'
import { useRepositoryStore } from '../stores/use-repository-store'
import { useSettingsStore } from '../stores/use-settings-store'
import type { AppLocale, RepositoryStorageInfo, ThemeMode } from '../types/app'
import { cn } from '../lib/cn'

export function SettingsPage() {
  const { t } = useTranslation()
  const settings = useSettingsStore((state) => state.settings)
  const saving = useSettingsStore((state) => state.saving)
  const setLanguage = useSettingsStore((state) => state.setLanguage)
  const setThemeMode = useSettingsStore((state) => state.setThemeMode)
  const setRepositoryStoragePath = useSettingsStore((state) => state.setRepositoryStoragePath)
  const saveSettings = useSettingsStore((state) => state.save)
  const repositoryStorage = useAppStore((state) => state.repositoryStorage)
  const setRepositoryStorage = useAppStore((state) => state.setRepositoryStorage)

  const [repositoryPathInput, setRepositoryPathInput] = useState('')
  const [settingsError, setSettingsError] = useState<string | null>(null)
  const [settingsSaved, setSettingsSaved] = useState(false)
  const [migrating, setMigrating] = useState(false)
  const [migrationError, setMigrationError] = useState<string | null>(null)
  const [migrationSuccess, setMigrationSuccess] = useState<string | null>(null)

  useEffect(() => {
    if (!repositoryStorage) return
    setRepositoryPathInput(repositoryStorage.currentPath)
  }, [repositoryStorage])

  // Reset saved state after 3 seconds
  useEffect(() => {
    if (settingsSaved) {
      const timer = setTimeout(() => setSettingsSaved(false), 3000)
      return () => clearTimeout(timer)
    }
  }, [settingsSaved])

  const languageOptions: Array<{ value: AppLocale; label: string }> = useMemo(
    () => [
      { value: 'zh-CN', label: t('topbar.languageOptions.zhCN') },
      { value: 'en-US', label: t('topbar.languageOptions.enUS') },
      { value: 'ja-JP', label: t('topbar.languageOptions.jaJP') },
    ],
    [t],
  )

  const themeOptions: Array<{ value: ThemeMode; label: string; icon: string }> = useMemo(
    () => [
      {
        value: 'system',
        label: t('settings.themeOptions.system'),
        icon: 'hn-laptop',
      },
      {
        value: 'light',
        label: t('settings.themeOptions.light'),
        icon: 'hn-sun',
      },
      {
        value: 'dark',
        label: t('settings.themeOptions.dark'),
        icon: 'hn-moon',
      },
    ],
    [t],
  )

  const persistPreferences = async () => {
    setSettingsError(null)
    setSettingsSaved(false)
    try {
      await saveSettings()
      setSettingsSaved(true)
    } catch (error) {
      setSettingsError(error instanceof Error ? error.message : String(error))
    }
  }

  const handleLanguageChange = async (language: AppLocale) => {
    setLanguage(language)
    await persistPreferences()
  }

  const handleThemeChange = async (theme: ThemeMode) => {
    setThemeMode(theme)
    await persistPreferences()
  }

  const pickRepositoryDirectory = async () => {
    const selected = await openDialog({
      directory: true,
      multiple: false,
      defaultPath: repositoryPathInput || repositoryStorage?.currentPath,
      title: t('settings.repository.pickTitle'),
    })

    if (typeof selected === 'string') {
      setRepositoryPathInput(selected)
    }
  }

  const applyRepositoryStorage = async (targetPath: string, storageInfo: RepositoryStorageInfo) => {
    const trimmedTargetPath = targetPath.trim()
    if (!trimmedTargetPath) {
      setMigrationError(t('settings.repository.validation.required') || 'Path is required')
      return
    }
    
    // Only skip if paths are identical
    if (trimmedTargetPath === storageInfo.currentPath) return

    const confirmed = window.confirm(
      t('settings.repository.confirmMessage', {
        currentPath: storageInfo.currentPath,
        targetPath: trimmedTargetPath,
      }) || `Move repository from \n${storageInfo.currentPath}\nto\n${trimmedTargetPath}?`
    )
    if (!confirmed) return

    setMigrating(true)
    setMigrationError(null)
    setMigrationSuccess(null)

    try {
      const result = await migrateRepositoryStorage({ targetPath: trimmedTargetPath })
      
      const nextStorage = {
        defaultPath: storageInfo.defaultPath,
        currentPath: result.currentPath,
        isCustom: storageInfo.defaultPath !== result.currentPath,
      }

      setRepositoryStorage(nextStorage)
      setRepositoryStoragePath(nextStorage.isCustom ? result.currentPath : null)
      setRepositoryPathInput(result.currentPath)
      await useRepositoryStore.getState().refresh()
      
      setMigrationSuccess(
        t('settings.repository.success', {
          count: result.migratedSkillCount,
          targetPath: result.currentPath,
        }) || 'Repository migrated successfully'
      )
    } catch (error) {
      console.error('Migration failed:', error)
      setMigrationError(error instanceof Error ? error.message : String(error))
    } finally {
      setMigrating(false)
    }
  }

  const storageInfo = repositoryStorage

  return (
    <div className="mx-auto w-full max-w-5xl space-y-8 p-6 lg:p-10 animate-in fade-in slide-in-from-bottom-4 duration-500">
      
      {/* Header */}
      <header className="flex flex-col gap-3 border-b border-[var(--border-subtle)] pb-6">
        <div className="flex items-center gap-3">
          <h1 className="text-3xl font-bold tracking-tight">{t('settings.title')}</h1>
        </div>
        <p className="max-w-3xl text-base text-base-content/60">
          {t('settings.description')}
        </p>
      </header>

      <div className="card overflow-hidden border border-[var(--border-subtle)] bg-[var(--bg-modal-panel)] shadow-sm">
        <div className="card-body p-6 lg:p-8 space-y-0 divide-y divide-[var(--border-subtle)]">
          
          {/* Section 1: Preferences */}
          <section className="grid gap-6 lg:grid-cols-12 lg:gap-12 pb-8">
            <div className="lg:col-span-4 space-y-4">
              <div>
                <h2 className="card-title text-lg font-semibold">{t('settings.preferencesTitle')}</h2>
                <p className="mt-1 text-sm text-base-content/60">
                  {t('settings.preferencesDescription') || 'Customize the look and feel of the application.'}
                </p>
                {/* Show saving status in text if needed */}
                {saving && (
                  <div className="flex items-center gap-2 text-xs text-base-content/50 mt-2 animate-pulse">
                    <span className="loading loading-spinner loading-xs"></span>
                    {t('settings.saving') || 'Saving...'}
                  </div>
                )}
                {settingsSaved && !saving && (
                  <div className="flex items-center gap-2 text-xs text-success mt-2 animate-in fade-in zoom-in duration-300">
                    <i className="hn hn-check-circle"></i>
                    {t('settings.saved')}
                  </div>
                )}
              </div>
            </div>

            <div className="lg:col-span-8 grid gap-6 sm:grid-cols-2">
              {/* Language */}
              <div className="form-control w-full">
                <label className="label pt-0">
                  <span className="label-text font-medium text-base-content/80">{t('topbar.language')}</span>
                </label>
                <select
                  className="select select-bordered w-full bg-base-100 border-[var(--border-subtle)] focus:border-primary focus:outline-none"
                  value={settings.language}
                  onChange={(event) => handleLanguageChange(event.target.value as AppLocale)}
                  disabled={saving}
                >
                  {languageOptions.map((option) => (
                    <option key={option.value} value={option.value}>
                      {option.label}
                    </option>
                  ))}
                </select>
              </div>

              {/* Theme */}
              <div className="form-control w-full">
                <label className="label pt-0">
                  <span className="label-text font-medium text-base-content/80">{t('topbar.theme')}</span>
                </label>
                <div className="grid grid-cols-3 gap-2">
                  {themeOptions.map((option) => {
                    const isActive = settings.themeMode === option.value
                    return (
                      <button
                        key={option.value}
                        type="button"
                        className={cn(
                          "flex flex-col items-center justify-center gap-2 rounded-lg border p-3 transition-all hover:scale-[1.02] active:scale-[0.98]",
                          isActive
                            ? "border-primary bg-primary/10 text-primary"
                            : "border-[var(--border-subtle)] bg-base-100 hover:border-primary/50 hover:bg-base-200"
                        )}
                        onClick={() => handleThemeChange(option.value)}
                        disabled={saving}
                      >
                        <i className={cn("hn text-xl", option.icon)} />
                        <span className="text-[10px] font-medium uppercase tracking-wide">{option.label}</span>
                      </button>
                    )
                  })}
                </div>
              </div>

              {settingsError && (
                <div className="alert alert-error text-sm py-2 sm:col-span-2">
                  <i className="hn hn-alert-circle"></i>
                  <span>{settingsError}</span>
                </div>
              )}
            </div>
          </section>

          {/* Section 2: Repository Storage */}
          <section className="grid gap-6 lg:grid-cols-12 lg:gap-12 pt-8">
            <div className="lg:col-span-4 space-y-4">
              <div>
                <div className="flex items-center gap-2 flex-wrap mb-1">
                  <h2 className="card-title text-lg font-semibold">{t('settings.repository.title')}</h2>
                  {storageInfo?.isCustom ? (
                    <div className="badge badge-primary badge-outline badge-sm gap-1">
                      <i className="hn hn-check-circle text-[10px]"></i>
                      {t('settings.repository.customBadge')}
                    </div>
                  ) : (
                    <div className="badge badge-ghost badge-sm gap-1 text-base-content/60">
                      <i className="hn hn-info text-[10px]"></i>
                      {t('settings.repository.defaultBadge')}
                    </div>
                  )}
                </div>
                <p className="text-sm text-base-content/60 leading-relaxed">
                  {t('settings.repository.description')}
                </p>
              </div>

              <div className="alert bg-warning/5 border border-warning/20 rounded-lg py-3">
                <i className="hn hn-alert-triangle text-warning text-lg"></i>
                <div className="text-xs text-base-content/80">
                  <span className="font-bold block mb-1 text-warning">{t('settings.repository.warningTitle')}</span>
                  {t('settings.repository.warningBody')}
                </div>
              </div>
            </div>

            <div className="lg:col-span-8 space-y-6">
              <div className="space-y-4">
                {/* Current Path Display */}
                <div className="rounded-lg border border-[var(--border-subtle)] bg-base-200/30 p-4">
                  <div className="flex items-center justify-between mb-2">
                    <span className="text-xs font-bold uppercase tracking-wider text-base-content/50">
                      {t('settings.repository.currentPath')}
                    </span>
                    <span className="text-xs text-base-content/40 font-mono">
                      {storageInfo?.isCustom ? 'Custom' : 'System Default'}
                    </span>
                  </div>
                  <code className="block w-full break-all rounded bg-base-300/50 p-2 font-mono text-sm text-base-content/90">
                    {storageInfo?.currentPath ?? t('settings.repository.unavailable')}
                  </code>
                </div>

                {/* Default Path Info */}
                <div className="collapse collapse-arrow border border-[var(--border-subtle)] bg-base-100 rounded-lg">
                  <input type="checkbox" /> 
                  <div className="collapse-title min-h-[2.5rem] py-2 text-sm font-medium text-base-content/70">
                    {t('settings.repository.defaultPath')}
                  </div>
                  <div className="collapse-content">
                    <code className="block break-all rounded bg-base-200 p-2 font-mono text-xs text-base-content/60">
                      {storageInfo?.defaultPath ?? t('settings.repository.unavailable')}
                    </code>
                  </div>
                </div>

                {/* Action Area */}
                <div className="form-control gap-3 pt-2">
                  <label className="label p-0">
                    <span className="label-text font-medium">{t('settings.repository.targetLabel')}</span>
                  </label>
                  <div className="join w-full">
                    <input
                      className="input input-bordered join-item w-full bg-base-100 font-mono text-sm focus:outline-none"
                      value={repositoryPathInput}
                      onChange={(event) => setRepositoryPathInput(event.target.value)}
                      disabled={migrating || !storageInfo}
                      placeholder={storageInfo?.currentPath ?? ''}
                    />
                    <button
                      type="button"
                      className="btn btn-outline border-base-300 bg-base-200/50 join-item hover:bg-base-200"
                      onClick={() => void pickRepositoryDirectory()}
                      disabled={migrating || !storageInfo}
                    >
                      <i className="hn hn-folder-open"></i>
                      {t('settings.repository.browse')}
                    </button>
                  </div>
                </div>

                <div className="flex flex-wrap gap-3">
                  <button
                    type="button"
                    className="btn btn-primary flex-1 sm:flex-none sm:min-w-[140px]"
                    onClick={() =>
                      storageInfo
                        ? void applyRepositoryStorage(repositoryPathInput, storageInfo)
                        : undefined
                    }
                    disabled={migrating || !storageInfo || repositoryPathInput === storageInfo?.currentPath}
                  >
                    {migrating ? (
                      <>
                        <span className="loading loading-spinner loading-xs"></span>
                        {t('settings.repository.migrating')}
                      </>
                    ) : (
                      <>
                        <i className="hn hn-save"></i>
                        {t('settings.repository.applyAction')}
                      </>
                    )}
                  </button>

                  <button
                    type="button"
                    className="btn btn-ghost border border-[var(--border-subtle)] text-base-content/70 hover:bg-base-200 flex-1 sm:flex-none"
                    onClick={() =>
                      storageInfo
                        ? void applyRepositoryStorage(storageInfo.defaultPath, storageInfo)
                        : undefined
                    }
                    disabled={migrating || !storageInfo || !storageInfo.isCustom}
                  >
                    {t('settings.repository.resetAction')}
                  </button>
                </div>

                {migrationSuccess && (
                  <div className="alert alert-success text-sm py-2">
                    <i className="hn hn-check-circle"></i>
                    <span>{migrationSuccess}</span>
                  </div>
                )}

                {migrationError && (
                  <div className="alert alert-error text-sm py-2">
                    <i className="hn hn-alert-circle"></i>
                    <span>{migrationError}</span>
                  </div>
                )}
              </div>
            </div>
          </section>

        </div>
      </div>
    </div>
  )
}
