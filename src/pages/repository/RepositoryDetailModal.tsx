import { useTranslation } from 'react-i18next'
import { normalizeDisplayPath } from '../../lib/normalize-display-path'
import { openSourceReference } from '../../lib/tauri-client'
import type { RepositorySkillDetail } from '../../types/app'
import {
  formatInstalledAt,
  logSourceOpenFailure,
  resolveDescription,
  resolveSourceLabel,
} from './repository-page-helpers'

interface RepositoryDetailModalProps {
  selectedDetail: RepositorySkillDetail | null
  detailLoading: boolean
  detailError: string | null
  locale: string
  batchUpdating: boolean
  updatingSkillId: string | null
  onClose: () => void
  onUpdateSkill: (skillId: string) => void
}

export function RepositoryDetailModal({
  selectedDetail,
  detailLoading,
  detailError,
  locale,
  batchUpdating,
  updatingSkillId,
  onClose,
  onUpdateSkill,
}: RepositoryDetailModalProps) {
  const { t } = useTranslation()

  if (!selectedDetail && !detailLoading && !detailError) {
    return null
  }

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-[var(--bg-modal-overlay)] p-6 backdrop-blur-sm transition-all duration-300">
      <div className="relative flex max-h-[85vh] w-full max-w-4xl flex-col overflow-hidden rounded-xl border border-[var(--border-subtle)] bg-[var(--bg-modal-panel)] shadow-[0_0_50px_rgba(0,0,0,0.5)]">
        <div className="flex items-start justify-between gap-4 border-b border-[var(--border-subtle)] bg-base-100/50 px-8 py-6 backdrop-blur-md">
          <div className="min-w-0">
            <h3 className="truncate text-2xl font-bold text-base-content">
              {selectedDetail?.name ?? t('repository.detailTitle')}
            </h3>
            {selectedDetail ? (
              <div className="mt-3 flex flex-col gap-2">
                <div className="rounded-lg border border-[var(--border-subtle)] bg-base-200/40 p-4">
                  <p className="text-xs font-semibold uppercase tracking-wide text-base-content/40">
                    {t('repository.summaryTitle')}
                  </p>
                  <p className="mt-2 text-sm leading-6 text-base-content/75">
                    {resolveDescription(selectedDetail.description, t)}
                  </p>
                </div>
                <p className="break-all font-mono text-xs text-base-content/40">
                  {normalizeDisplayPath(selectedDetail.canonicalPath)}
                </p>
                <div className="flex flex-wrap items-center gap-3">
                  <span className="badge badge-outline border-[var(--border-subtle)] text-xs text-base-content/60">
                    {formatInstalledAt(selectedDetail.installedAt, locale)}
                  </span>
                  <span className="badge badge-outline border-[var(--border-subtle)] text-xs text-base-content/60">
                    {resolveSourceLabel(selectedDetail.sourceType, selectedDetail.sourceMarket, t)}
                  </span>
                  {selectedDetail.canUpdate ? (
                    <button
                      type="button"
                      onClick={() => onUpdateSkill(selectedDetail.id)}
                      disabled={batchUpdating || updatingSkillId === selectedDetail.id}
                      className="btn btn-ghost btn-xs text-primary hover:bg-primary/10"
                    >
                      {updatingSkillId === selectedDetail.id ? (
                        <span className="loading loading-spinner loading-xs"></span>
                      ) : null}
                      {t('repository.update.single')}
                    </button>
                  ) : null}
                  {selectedDetail.sourceUrl ? (
                    <button
                      type="button"
                      onClick={() =>
                        void openSourceReference(selectedDetail.sourceUrl!).catch(logSourceOpenFailure)
                      }
                      className="flex items-center gap-1 text-xs text-primary hover:underline"
                    >
                      <i className="hn hn-external-link"></i>
                      {t('repository.source')}
                    </button>
                  ) : null}
                </div>
              </div>
            ) : null}
          </div>
          <button
            className="btn btn-circle btn-ghost btn-sm text-base-content/50 hover:bg-base-content/10 hover:text-base-content"
            onClick={onClose}
          >
            <i className="hn hn-times text-lg"></i>
          </button>
        </div>

        <div className="overflow-y-auto p-8 custom-scrollbar">
          {detailLoading ? (
            <div className="flex justify-center py-12">
              <span className="loading loading-spinner loading-lg text-primary"></span>
            </div>
          ) : detailError ? (
            <div className="rounded border border-error/20 bg-error/5 p-4 text-sm text-error">
              {detailError}
            </div>
          ) : selectedDetail ? (
            <div className="prose prose-base max-w-none dark:prose-invert">
              <pre className="overflow-x-auto whitespace-pre-wrap rounded-lg border border-[var(--border-subtle)] bg-[var(--bg-input)] p-6 font-mono text-sm leading-relaxed text-base-content/80 shadow-inner">
                {selectedDetail.skillMarkdown}
              </pre>
            </div>
          ) : null}
        </div>
      </div>
    </div>
  )
}
