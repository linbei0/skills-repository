import { useTranslation } from 'react-i18next'
import { useEffect, useMemo } from 'react'
import { normalizeDisplayPath } from '../lib/normalize-display-path'
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
    <div className="space-y-8 p-8">
      {/* Header Section */}
      <section className="relative overflow-hidden rounded-lg border border-[var(--border-subtle)] bg-base-100 p-8 shadow-[inset_0_0_20px_rgba(var(--color-primary),0.05)]">
        <div className="absolute top-0 left-0 h-1 w-full bg-gradient-to-r from-primary/0 via-primary/50 to-primary/0 opacity-20"></div>
        <div className="flex flex-col gap-6 md:flex-row md:items-start md:justify-between">
          <div>
            <h2 className="text-3xl font-bold tracking-tight text-base-content">{t('security.title')}</h2>
            <p className="mt-3 max-w-3xl text-sm leading-relaxed text-base-content/70">
              {t('security.description')}
            </p>
          </div>
          <button
            className={`btn btn-primary border-none bg-primary text-[var(--text-inverse)] transition-all duration-300 hover:bg-primary hover:shadow-[var(--shadow-neon-primary)] ${
              loading ? 'animate-pulse' : ''
            }`}
            onClick={() => void rescan()}
            disabled={loading}
          >
            {loading ? (
              <>
                <span className="loading loading-spinner loading-sm"></span>
                {t('security.rescanning')}
              </>
            ) : (
              t('security.rescan')
            )}
          </button>
        </div>
      </section>

      {/* Status Cards Grid */}
      <section className="grid gap-6 md:grid-cols-2 xl:grid-cols-3">
        {statusCards.map((card) => (
          <div
            key={card.key}
            className="group relative overflow-hidden rounded-lg border border-[var(--border-subtle)] bg-[var(--card-bg)] p-6 transition-all duration-300 hover:scale-[1.02] hover:border-primary/30 hover:shadow-[inset_0_0_30px_rgba(var(--color-primary),0.1)]"
          >
            <div className="absolute -right-10 -top-10 h-32 w-32 rounded-full bg-primary/5 blur-3xl transition-all duration-500 group-hover:bg-primary/10"></div>
            
            <div className="relative flex items-start justify-between gap-3">
              <p className="text-xs font-bold uppercase tracking-[0.2em] text-base-content/40 transition-colors group-hover:text-primary/70">
                {t(`security.cards.${card.key}.label`)}
              </p>
              <span className={`badge badge-sm border-0 bg-opacity-10 font-mono text-xs ${card.accent}`}>
                {t('security.liveData')}
              </span>
            </div>
            
            <p className="mt-6 font-mono text-4xl font-bold text-base-content transition-all group-hover:text-primary group-hover:text-shadow-neon">
              {card.count}
            </p>
            <p className="mt-2 text-lg font-semibold text-base-content/90">
              {t(`security.cards.${card.key}.title`)}
            </p>
            <p className="mt-3 text-sm leading-6 text-base-content/60">
              {t(`security.cards.${card.key}.description`)}
            </p>
          </div>
        ))}
      </section>

      {error ? (
        <section className="rounded-lg border border-error/30 bg-error/10 p-5 text-sm leading-6 text-error shadow-[0_0_20px_rgba(255,0,92,0.1)]">
          <div className="flex items-center gap-3">
             <i className="hn hn-exclaimation text-lg"></i>
             {error}
          </div>
        </section>
      ) : null}

      {/* Reports Section */}
      <section className="rounded-lg border border-[var(--border-subtle)] bg-base-100 p-8 shadow-[inset_0_0_20px_rgba(var(--color-primary),0.02)]">
        <div className="flex items-center justify-between gap-3 border-b border-[var(--border-subtle)] pb-6">
          <h3 className="text-xl font-bold text-base-content">{t('security.reportsTitle')}</h3>
          <span className="font-mono text-sm text-primary/80">
            {t('security.reportsCount', { count: reports.length })}
          </span>
        </div>

        {reports.length === 0 ? (
          <div className="mt-8 flex flex-col items-center justify-center rounded-lg border border-dashed border-[var(--border-subtle)] bg-base-200/30 p-12 text-center">
            <div className="mb-4 rounded-full bg-base-200 p-4 text-primary/50">
               <i className="hn hn-shield text-3xl"></i>
            </div>
            <p className="text-base font-medium text-base-content/60">
              {loading ? t('security.loading') : t('security.empty')}
            </p>
          </div>
        ) : (
          <div className="mt-6 space-y-4">
            {reports.map((report) => (
              <article
                key={report.id}
                className="group overflow-hidden rounded-lg border border-[var(--border-subtle)] bg-base-200/40 p-0 transition-all hover:border-primary/20 hover:bg-base-200/60"
              >
                {/* Report Header */}
                <div className="flex flex-wrap items-start justify-between gap-4 bg-base-100/50 p-5">
                  <div className="space-y-1">
                    <div className="flex flex-wrap items-center gap-3">
                      <p className="text-lg font-bold text-base-content group-hover:text-primary transition-colors">
                        {report.skillName ?? t('security.unlinkedSkill')}
                      </p>
                      <span className={`badge badge-sm border-0 font-bold ${
                        report.level === 'safe' ? 'bg-success/20 text-success' :
                        report.level === 'medium' ? 'bg-warning/20 text-warning' :
                        report.level === 'high' || report.level === 'critical' ? 'bg-error/20 text-error' :
                        'bg-base-content/10 text-base-content/60'
                      }`}>
                        {t(`security.levels.${report.level}`)}
                      </span>
                      {report.blocked ? (
                        <span className="badge badge-error badge-sm gap-1">
                          <i className="hn hn-lock text-xs"></i>
                          {t('security.blockedBadge')}
                        </span>
                      ) : null}
                    </div>
                    <p className="font-mono text-xs text-base-content/40">
                  {report.sourcePath ? normalizeDisplayPath(report.sourcePath) : t('security.temporaryScan')}
                    </p>
                  </div>
                  <div className="text-right">
                    <div className="flex items-center justify-end gap-2">
                       <span className="text-xs text-base-content/50">{t('security.score', { score: '' })}</span>
                       <span className={`font-mono text-xl font-bold ${
                         report.score >= 80 ? 'text-success' : report.score >= 60 ? 'text-warning' : 'text-error'
                       }`}>{report.score}</span>
                    </div>
                    <p className="mt-1 text-xs text-base-content/40">{t('security.scope', { scope: report.scanScope })}</p>
                  </div>
                </div>

                <div className="p-5 pt-2">
                    {/* Blocking Reasons */}
                    {report.blockingReasons && report.blockingReasons.length > 0 ? (
                      <div className="mt-3 rounded border border-error/20 bg-error/5 p-4">
                        <p className="flex items-center gap-2 text-sm font-bold text-error">
                          <i className="hn hn-octagon-times"></i>
                          {t('security.blockingReasonsTitle')}
                        </p>
                        <ul className="mt-2 list-inside list-disc space-y-1 text-sm text-error/80">
                          {report.blockingReasons.map((reason, index) => (
                            <li key={`${report.id}-reason-${index}`}>{reason}</li>
                          ))}
                        </ul>
                      </div>
                    ) : null}

                    {/* Breakdown Badges */}
                    {report.categoryBreakdown && report.categoryBreakdown.length > 0 ? (
                      <div className="mt-4 flex flex-wrap gap-2">
                        {report.categoryBreakdown.map((entry) => (
                          <span key={`${report.id}-${entry.category}`} className="badge badge-ghost badge-sm gap-2 border-[var(--border-subtle)] bg-base-100 text-xs text-base-content/60">
                            <span>{t(`security.categories.${entry.category}`)}</span>
                            <span className="opacity-50">|</span>
                            <span className={entry.score < 80 ? 'text-warning' : 'text-success'}>{entry.score}</span>
                          </span>
                        ))}
                      </div>
                    ) : null}

                    {/* Issues & Recommendations Grid */}
                    <div className="mt-6 grid gap-6 lg:grid-cols-2">
                      {/* Issues Column */}
                      <div>
                        <p className="mb-3 flex items-center gap-2 text-xs font-bold uppercase tracking-wider text-base-content/40">
                           <i className="hn hn-bug"></i>
                           {t('security.issuesTitle')}
                        </p>
                        {report.issues.length === 0 ? (
                          <div className="rounded border border-dashed border-[var(--border-subtle)] bg-base-100/50 p-4 text-center text-sm text-base-content/40">
                             {t('security.issuesEmpty')}
                          </div>
                        ) : (
                          <ul className="space-y-3">
                            {report.issues.map((issue) => (
                              <li key={`${report.id}-${issue.ruleId}`} className="relative overflow-hidden rounded border border-[var(--border-subtle)] bg-base-100 p-4 transition-all hover:border-[var(--border-highlight)]">
                                <div className="absolute left-0 top-0 h-full w-1 bg-warning/50"></div>
                                <div className="flex flex-wrap items-center gap-2 pl-3">
                                  <span className="font-bold text-base-content/90">{issue.title}</span>
                                  <span className="badge badge-xs border-0 bg-base-content/10 text-base-content/60">{t(`security.categories.${issue.category || 'system'}`)}</span>
                                  <span className="badge badge-xs border-0 bg-warning/20 text-warning">{t(`security.levels.${issue.severity}`)}</span>
                                </div>
                                <p className="mt-2 pl-3 text-sm text-base-content/70">{issue.description}</p>
                              </li>
                            ))}
                          </ul>
                        )}
                      </div>

                      {/* Recommendations Column */}
                      <div>
                         <p className="mb-3 flex items-center gap-2 text-xs font-bold uppercase tracking-wider text-base-content/40">
                           <i className="hn hn-lightbulb"></i>
                           {t('security.recommendationsTitle')}
                        </p>
                        <ul className="space-y-2">
                          {report.recommendations.map((recommendation, index) => (
                            <li key={`${report.id}-rec-${index}`} className="flex gap-3 rounded bg-base-100/50 p-3 text-sm text-base-content/70">
                              <i className="hn hn-check-circle mt-0.5 text-success/70"></i>
                              <span>{recommendation.description}</span>
                            </li>
                          ))}
                        </ul>
                      </div>
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
