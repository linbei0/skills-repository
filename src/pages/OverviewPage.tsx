import { useTranslation } from 'react-i18next'
import { useSettingsStore } from '../stores/use-settings-store'
import { useSkillsStore } from '../stores/use-skills-store'

export function OverviewPage() {
  const { t } = useTranslation()
  const scanSkills = useSkillsStore((state) => state.scanSkills)
  const projects = useSkillsStore((state) => state.projects)
  const settings = useSettingsStore((state) => state.settings)

  return (
    <div className="space-y-6">
      <section className="rounded-box border border-base-300 bg-base-100 p-6">
        <div className="flex items-start justify-between gap-4">
          <div>
            <p className="text-sm uppercase tracking-[0.24em] text-primary">
              {t('overview.phaseBadge')}
            </p>
            <h2 className="mt-2 text-3xl font-semibold">{t('overview.title')}</h2>
            <p className="mt-3 max-w-3xl text-sm leading-6 text-base-content/65">
              {t('overview.description')}
            </p>
          </div>

          <button
            className="btn btn-primary"
            onClick={() =>
              void scanSkills({
                includeSystem: true,
                includeProjects: true,
                projectRoots: settings.scan.projectRoots,
                customRoots: settings.scan.customRoots,
              })
            }
          >
            {t('overview.scanNow')}
          </button>
        </div>
      </section>

      <section className="grid gap-6 xl:grid-cols-[1.4fr_1fr]">
        <div className="rounded-box border border-base-300 bg-base-100 p-6">
          <h3 className="text-lg font-semibold">{t('overview.moduleStatusTitle')}</h3>
          <p className="mt-2 text-sm text-base-content/60">{t('overview.moduleStatusHint')}</p>
          <div className="mt-6 grid gap-3">
            <div className="rounded-box border border-dashed border-base-300 bg-base-200/60 p-4 text-sm text-base-content/60">
              {t('overview.moduleStatusNotice')}
            </div>
            <div className="rounded-box border border-dashed border-warning/40 bg-warning/5 p-4 text-sm text-base-content/70">
              {t('overview.metricsNotice')}
            </div>
          </div>
        </div>

        <div className="rounded-box border border-base-300 bg-base-100 p-6">
          <h3 className="text-lg font-semibold">{t('overview.agents')}</h3>
          <div className="mt-4 space-y-3">
            {projects.length === 0 ? (
              <div className="rounded-box border border-dashed border-base-300 bg-base-200/60 p-4 text-sm text-base-content/60">
                {t('overview.projectsEmpty')}
              </div>
            ) : (
              projects.map((project) => (
                <div key={project.id} className="rounded-box border border-base-300 bg-base-200/60 p-4">
                  <p className="font-medium">{project.name}</p>
                  <p className="mt-1 text-xs text-base-content/55">{project.rootPath}</p>
                </div>
              ))
            )}
          </div>
        </div>
      </section>
    </div>
  )
}
