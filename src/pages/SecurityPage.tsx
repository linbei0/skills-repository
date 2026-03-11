import { useTranslation } from 'react-i18next'
import { useEffect, useMemo } from 'react'
import { useSecurityStore } from '../stores/use-security-store'

export function SecurityPage() {
  const { t } = useTranslation()
  const reports = useSecurityStore((state) => state.reports)
  const loading = useSecurityStore((state) => state.loading)
  const loaded = useSecurityStore((state) => state.loaded)
  const error = useSecurityStore((state) => state.error)
  const refresh = useSecurityStore((state) => state.refresh)
  const rescan = useSecurityStore((state) => state.rescan)

  useEffect(() => {
    if (!loaded) {
      void refresh()
    }
  }, [loaded, refresh])

  const statusCards = useMemo(
    () => [
      {
        key: 'safe',
        accent: 'border-success/30 bg-success/5 text-success',
        count: reports.filter((report) => report.level === 'safe' || report.level === 'low').length,
      },
      {
        key: 'review',
        accent: 'border-warning/30 bg-warning/5 text-warning',
        count: reports.filter((report) => !report.blocked && !['safe', 'low'].includes(report.level)).length,
      },
      {
        key: 'blocked',
        accent: 'border-error/30 bg-error/5 text-error',
        count: reports.filter((report) => report.blocked).length,
      },
    ],
    [reports],
  )

  return (
    <div className="space-y-6">
      <section className="rounded-box border border-base-300 bg-base-100 p-6">
        <div className="flex items-start justify-between gap-4">
          <div>
            <h2 className="text-3xl font-semibold">{t('security.title')}</h2>
            <p className="mt-3 max-w-3xl text-sm text-base-content/65">{t('security.description')}</p>
          </div>
          <button className="btn btn-primary" onClick={() => void rescan()}>
            {loading ? t('security.rescanning') : t('security.rescan')}
          </button>
        </div>
      </section>

      <section className="grid gap-4 md:grid-cols-3">
        {statusCards.map((card) => (
          <div key={card.key} className="rounded-box border border-base-300 bg-base-100 p-5">
            <div className="flex items-start justify-between gap-3">
              <p className="text-sm uppercase tracking-[0.2em] text-base-content/50">
                {t(`security.cards.${card.key}.label`)}
              </p>
              <span className={`badge badge-outline ${card.accent}`}>
                {t('security.liveData')}
              </span>
            </div>
            <p className="mt-4 text-3xl font-semibold">{card.count}</p>
            <p className="mt-2 text-lg font-semibold">
              {t(`security.cards.${card.key}.title`)}
            </p>
            <p className="mt-2 text-sm leading-6 text-base-content/60">
              {t(`security.cards.${card.key}.description`)}
            </p>
          </div>
        ))}
      </section>

      {error ? (
        <section className="rounded-box border border-error/30 bg-error/5 p-5 text-sm leading-6 text-error">
          {error}
        </section>
      ) : null}

      <section className="rounded-box border border-base-300 bg-base-100 p-6">
        <div className="flex items-center justify-between gap-3">
          <h3 className="text-lg font-semibold">{t('security.reportsTitle')}</h3>
          <span className="text-sm text-base-content/55">
            {t('security.reportsCount', { count: reports.length })}
          </span>
        </div>

        {reports.length === 0 ? (
          <div className="mt-4 rounded-box border border-dashed border-base-300 bg-base-200/60 p-4 text-sm text-base-content/60">
            {loading ? t('security.loading') : t('security.empty')}
          </div>
        ) : (
          <div className="mt-4 space-y-4">
            {reports.map((report) => (
              <article key={report.id} className="rounded-box border border-base-300 bg-base-200/60 p-4">
                <div className="flex flex-wrap items-start justify-between gap-3">
                  <div>
                    <div className="flex flex-wrap items-center gap-2">
                      <p className="font-semibold">
                        {report.skillName ?? t('security.unlinkedSkill')}
                      </p>
                      <span className="badge badge-outline">
                        {t(`security.levels.${report.level}`)}
                      </span>
                      {report.blocked ? (
                        <span className="badge badge-error">{t('security.blockedBadge')}</span>
                      ) : null}
                    </div>
                    <p className="mt-2 text-sm text-base-content/60">
                      {report.sourcePath ?? t('security.temporaryScan')}
                    </p>
                  </div>
                  <div className="text-right text-sm text-base-content/60">
                    <p>{t('security.score', { score: report.score })}</p>
                    <p className="mt-1">{t('security.scope', { scope: report.scanScope })}</p>
                  </div>
                </div>

                {report.blockingReasons && report.blockingReasons.length > 0 ? (
                  <div className="mt-4 rounded-box border border-error/20 bg-error/5 p-3">
                    <p className="text-sm font-medium text-error">{t('security.blockingReasonsTitle')}</p>
                    <ul className="mt-2 space-y-2 text-sm text-error/80">
                      {report.blockingReasons.map((reason, index) => (
                        <li key={`${report.id}-reason-${index}`}>{reason}</li>
                      ))}
                    </ul>
                  </div>
                ) : null}

                {report.categoryBreakdown && report.categoryBreakdown.length > 0 ? (
                  <div className="mt-4 flex flex-wrap gap-2">
                    {report.categoryBreakdown.map((entry) => (
                      <span key={`${report.id}-${entry.category}`} className="badge badge-outline">
                        {t(`security.categories.${entry.category}`)} · {entry.count} · {entry.score}
                      </span>
                    ))}
                  </div>
                ) : null}

                <div className="mt-4 grid gap-3 lg:grid-cols-2">
                  <div className="rounded-box border border-base-300 bg-base-100 p-3">
                    <p className="text-sm font-medium">{t('security.issuesTitle')}</p>
                    {report.issues.length === 0 ? (
                      <p className="mt-2 text-sm text-base-content/60">{t('security.issuesEmpty')}</p>
                    ) : (
                      <ul className="mt-2 space-y-2 text-sm text-base-content/70">
                        {report.issues.map((issue) => (
                          <li key={`${report.id}-${issue.ruleId}`} className="rounded-box border border-base-300 bg-base-200/50 p-3">
                            <div className="flex flex-wrap items-center gap-2">
                              <span className="font-medium">{issue.title}</span>
                              <span className="badge badge-outline">{t(`security.categories.${issue.category || 'system'}`)}</span>
                              <span className="badge badge-outline">{t(`security.levels.${issue.severity}`)}</span>
                              {issue.fileKind ? (
                                <span className="badge badge-ghost">{t(`security.fileKinds.${issue.fileKind || 'unknown'}`)}</span>
                              ) : null}
                              {issue.blocking ? (
                                <span className="badge badge-error">{t('security.blockedBadge')}</span>
                              ) : null}
                            </div>
                            <p className="mt-2">{issue.description}</p>
                            <div className="mt-2 space-y-1 text-xs text-base-content/60">
                              {issue.filePath ? <p>{issue.filePath}</p> : null}
                              {issue.line ? <p>{t('security.line', { line: issue.line })}</p> : null}
                              {issue.evidence ? <p>{t('security.evidence', { evidence: issue.evidence })}</p> : null}
                            </div>
                          </li>
                        ))}
                      </ul>
                    )}
                  </div>

                  <div className="rounded-box border border-base-300 bg-base-100 p-3">
                    <p className="text-sm font-medium">{t('security.recommendationsTitle')}</p>
                    <ul className="mt-2 space-y-2 text-sm text-base-content/70">
                      {report.recommendations.map((recommendation, index) => (
                        <li key={`${report.id}-rec-${index}`}>
                          {recommendation.description}
                        </li>
                      ))}
                    </ul>
                  </div>
                </div>
              </article>
            ))}
          </div>
        )}
      </section>
    </div>
  )
}
