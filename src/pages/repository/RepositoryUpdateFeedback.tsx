import { useTranslation } from 'react-i18next'
import { flattenBatchUpdateResult } from '../../stores/use-repository-store'
import type {
  BatchRepositorySkillUpdateResult,
  RepositorySkillUpdateItemResult,
} from '../../types/app'
import {
  batchResultCount,
  formatUpdateMessage,
  shouldShowSingleUpdateFeedback,
} from './repository-page-helpers'

interface RepositoryUpdateFeedbackProps {
  updateError: string | null
  lastUpdateResult: RepositorySkillUpdateItemResult | null
  lastBatchUpdateResult: BatchRepositorySkillUpdateResult | null
  onClear: () => void
}

export function RepositoryUpdateFeedback({
  updateError,
  lastUpdateResult,
  lastBatchUpdateResult,
  onClear,
}: RepositoryUpdateFeedbackProps) {
  const { t } = useTranslation()

  return (
    <>
      {updateError ? (
        <div className="rounded-lg border border-error/20 bg-error/5 px-5 py-4 text-sm text-error">
          <div className="flex items-start justify-between gap-3">
            <p className="flex-1">{updateError}</p>
            <button
              className="btn btn-circle btn-ghost btn-xs text-error/70 hover:bg-error/10 hover:text-error"
              onClick={onClear}
              title={t('common.close')}
            >
              <i className="hn hn-times text-sm"></i>
            </button>
          </div>
        </div>
      ) : null}

      {shouldShowSingleUpdateFeedback(lastUpdateResult) ? (
        <section className="rounded-lg border border-[var(--border-subtle)] bg-base-100 p-5 shadow-[inset_0_0_20px_rgba(var(--color-primary),0.02)]">
          <div className="flex flex-wrap items-center justify-between gap-3">
            <div>
              <h3 className="text-base font-bold text-base-content">{t('repository.update.resultTitle')}</h3>
              <p className="mt-1 text-sm text-base-content/60">{lastUpdateResult?.skillName}</p>
            </div>
            <div className="flex items-center gap-2">
              <span
                className={`badge border-0 ${
                  lastUpdateResult?.status === 'updated'
                    ? 'bg-success/10 text-success'
                    : lastUpdateResult?.status === 'skipped'
                      ? 'bg-info/10 text-info'
                      : 'bg-error/10 text-error'
                }`}
              >
                {t(`repository.update.statuses.${lastUpdateResult?.status ?? 'failed'}`)}
              </span>
              <button
                className="btn btn-circle btn-ghost btn-xs text-base-content/50 hover:bg-base-content/10 hover:text-base-content"
                onClick={onClear}
                title={t('common.close')}
              >
                <i className="hn hn-times text-sm"></i>
              </button>
            </div>
          </div>
          <p className="mt-3 text-sm leading-6 text-base-content/70">
            {lastUpdateResult ? formatUpdateMessage(lastUpdateResult, t) : ''}
          </p>
        </section>
      ) : null}

      {lastBatchUpdateResult && batchResultCount(lastBatchUpdateResult) > 0 ? (
        <section className="rounded-lg border border-[var(--border-subtle)] bg-base-100 p-5 shadow-[inset_0_0_20px_rgba(var(--color-primary),0.02)]">
          <div className="flex flex-wrap items-center justify-between gap-3">
            <div>
              <h3 className="text-base font-bold text-base-content">{t('repository.update.batchTitle')}</h3>
              <p className="mt-1 text-sm text-base-content/60">
                {t('repository.update.batchSummary', {
                  updated: lastBatchUpdateResult.updated.length,
                  skipped: lastBatchUpdateResult.skipped.length,
                  failed: lastBatchUpdateResult.failed.length,
                })}
              </p>
            </div>
            <div className="flex items-center gap-2">
              <div className="flex flex-wrap gap-2">
                <span className="badge border-0 bg-success/10 text-success">
                  {t('repository.update.batchUpdated', { count: lastBatchUpdateResult.updated.length })}
                </span>
                <span className="badge border-0 bg-info/10 text-info">
                  {t('repository.update.batchSkipped', { count: lastBatchUpdateResult.skipped.length })}
                </span>
                <span className="badge border-0 bg-error/10 text-error">
                  {t('repository.update.batchFailed', { count: lastBatchUpdateResult.failed.length })}
                </span>
              </div>
              <button
                className="btn btn-circle btn-ghost btn-xs text-base-content/50 hover:bg-base-content/10 hover:text-base-content"
                onClick={onClear}
                title={t('common.close')}
              >
                <i className="hn hn-times text-sm"></i>
              </button>
            </div>
          </div>
          <div className="mt-4 space-y-3">
            {flattenBatchUpdateResult(lastBatchUpdateResult).map((item) => (
              <article
                key={`${item.skillId}-${item.status}`}
                className="rounded-lg border border-[var(--border-subtle)] bg-base-200/30 px-4 py-3"
              >
                <div className="flex flex-wrap items-center justify-between gap-3">
                  <p className="font-medium text-base-content">{item.skillName}</p>
                  <span
                    className={`badge border-0 ${
                      item.status === 'updated'
                        ? 'bg-success/10 text-success'
                        : item.status === 'skipped'
                          ? 'bg-info/10 text-info'
                          : 'bg-error/10 text-error'
                    }`}
                  >
                    {t(`repository.update.statuses.${item.status}`)}
                  </span>
                </div>
                <p className="mt-2 text-sm leading-6 text-base-content/65">
                  {formatUpdateMessage(item, t)}
                </p>
              </article>
            ))}
          </div>
        </section>
      ) : null}
    </>
  )
}
