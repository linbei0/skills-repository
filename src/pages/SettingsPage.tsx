import { useMemo } from 'react'
import { useTranslation } from 'react-i18next'
import { useSettingsStore } from '../stores/use-settings-store'

export function SettingsPage() {
  const { t } = useTranslation()
  const settings = useSettingsStore((state) => state.settings)
  const saving = useSettingsStore((state) => state.saving)
  const setLanguage = useSettingsStore((state) => state.setLanguage)
  const setThemeMode = useSettingsStore((state) => state.setThemeMode)
  const setRootsText = useSettingsStore((state) => state.setRootsText)
  const save = useSettingsStore((state) => state.save)

  const projectRootsText = useMemo(() => settings.scan.projectRoots.join('\n'), [settings.scan.projectRoots])
  const customRootsText = useMemo(() => settings.scan.customRoots.join('\n'), [settings.scan.customRoots])

  return (
    <div className="space-y-6">
      <section className="rounded-box border border-base-300 bg-base-100 p-6">
        <h2 className="text-3xl font-semibold">{t('settings.title')}</h2>
        <p className="mt-3 max-w-3xl text-sm text-base-content/65">{t('settings.description')}</p>
      </section>

      <section className="grid gap-6 xl:grid-cols-[1fr_1fr]">
        <div className="rounded-box border border-base-300 bg-base-100 p-6">
          <h3 className="text-lg font-semibold">{t('settings.sections.basic')}</h3>
          <div className="mt-5 space-y-5">
            <label className="form-control">
              <span className="mb-2 text-sm font-medium">{t('settings.language')}</span>
              <select
                className="select select-bordered"
                value={settings.language}
                onChange={(event) => setLanguage(event.target.value as typeof settings.language)}
              >
                <option value="zh-CN">简体中文</option>
                <option value="en-US">English</option>
                <option value="ja-JP">日本語</option>
              </select>
            </label>

            <label className="form-control">
              <span className="mb-2 text-sm font-medium">{t('settings.themeMode')}</span>
              <select
                className="select select-bordered"
                value={settings.themeMode}
                onChange={(event) => setThemeMode(event.target.value as typeof settings.themeMode)}
              >
                <option value="system">{t('settings.themeModes.system')}</option>
                <option value="light">{t('settings.themeModes.light')}</option>
                <option value="dark">{t('settings.themeModes.dark')}</option>
              </select>
            </label>
          </div>
        </div>

        <div className="rounded-box border border-base-300 bg-base-100 p-6">
          <h3 className="text-lg font-semibold">{t('settings.sections.scan')}</h3>
          <div className="mt-5 space-y-5">
            <label className="form-control">
              <span className="mb-2 text-sm font-medium">{t('settings.projectRoots')}</span>
              <textarea
                className="textarea textarea-bordered min-h-32 font-mono text-sm"
                value={projectRootsText}
                onChange={(event) => setRootsText('projectRoots', event.target.value)}
                placeholder={'E:\\workspace\\python-app\nE:\\workspace\\frontend-app'}
              />
            </label>

            <label className="form-control">
              <span className="mb-2 text-sm font-medium">{t('settings.customRoots')}</span>
              <textarea
                className="textarea textarea-bordered min-h-32 font-mono text-sm"
                value={customRootsText}
                onChange={(event) => setRootsText('customRoots', event.target.value)}
                placeholder={'E:\\shared\\skills\nD:\\agent-labs'}
              />
            </label>
          </div>
        </div>
      </section>

      <div className="flex justify-end">
        <button className="btn btn-primary" onClick={() => void save()} disabled={saving}>
          {saving ? t('settings.saving') : t('settings.save')}
        </button>
      </div>
    </div>
  )
}
