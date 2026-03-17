import { useTranslation } from 'react-i18next'
import { normalizeDisplayPath } from '../../lib/normalize-display-path'
import type { RepositorySkillDeletionPreview } from '../../types/app'

interface RepositoryDeletePreviewModalProps {
  deletePreview: RepositorySkillDeletionPreview | null
  deletePreviewLoading: boolean
  deletePreviewError: string | null
  uninstallingSkillId: string | null
  onClose: () => void
  onUninstall: (skillId: string) => void
}

export function RepositoryDeletePreviewModal({
  deletePreview,
  deletePreviewLoading,
  deletePreviewError,
  uninstallingSkillId,
  onClose,
  onUninstall,
}: RepositoryDeletePreviewModalProps) {
  const { t } = useTranslation()

  if (!deletePreview && !deletePreviewLoading && !deletePreviewError) {
    return null
  }

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-[var(--bg-modal-overlay)] p-6 backdrop-blur-sm transition-all duration-300">
      <div className="relative flex max-h-[85vh] w-full max-w-2xl flex-col overflow-hidden rounded-xl border border-[var(--border-subtle)] bg-[var(--bg-modal-panel)] shadow-[0_0_50px_rgba(0,0,0,0.5)]">
        <div className="flex items-start justify-between gap-4 border-b border-[var(--border-subtle)] bg-base-100/50 px-8 py-6 backdrop-blur-md">
          <div className="min-w-0">
            <h3 className="truncate text-2xl font-bold text-base-content">
              {t('repository.deleteConfirmTitle')}
            </h3>
            <p className="mt-2 text-sm leading-relaxed text-base-content/60">
              {deletePreview
                ? t('repository.deleteConfirmBody', { name: deletePreview.skillName })
                : t('repository.deleteConfirmLoading')}
            </p>
          </div>
          <button
            className="btn btn-circle btn-ghost btn-sm text-base-content/50 hover:bg-base-content/10 hover:text-base-content"
            onClick={onClose}
          >
            <i className="hn hn-times text-lg"></i>
          </button>
        </div>

        <div className="overflow-y-auto p-8 custom-scrollbar">
          {deletePreviewLoading ? (
            <div className="flex justify-center py-12">
              <span className="loading loading-spinner loading-lg text-primary"></span>
            </div>
          ) : deletePreviewError ? (
            <div className="rounded border border-error/20 bg-error/5 p-4 text-sm text-error">
              {deletePreviewError}
            </div>
          ) : deletePreview ? (
            <div className="space-y-5">
              <div className="rounded-lg border border-warning/20 bg-warning/5 p-4 text-sm leading-6 text-base-content/75">
                {t('repository.deleteConfirmWarning')}
              </div>
              <div className="rounded-lg border border-[var(--border-subtle)] bg-base-200/30 p-4">
                <p className="text-xs font-semibold uppercase tracking-wide text-base-content/40">
                  {t('repository.deleteCanonicalPath')}
                </p>
                <p className="mt-2 break-all font-mono text-xs text-base-content/50">
                  {normalizeDisplayPath(deletePreview.canonicalPath)}
                </p>
              </div>
              <div className="rounded-lg border border-[var(--border-subtle)] bg-base-200/30 p-4">
                <div className="flex items-center justify-between gap-3">
                  <p className="text-xs font-semibold uppercase tracking-wide text-base-content/40">
                    {t('repository.deleteDistributedPaths')}
                  </p>
                  <span className="badge badge-outline">
                    {t('repository.deleteDistributedCount', {
                      count: deletePreview.distributionPaths.length,
                    })}
                  </span>
                </div>
                {deletePreview.distributionPaths.length === 0 ? (
                  <p className="mt-2 text-sm text-base-content/60">
                    {t('repository.deleteNoDistributions')}
                  </p>
                ) : (
                  <ul className="mt-3 space-y-2">
                    {deletePreview.distributionPaths.map((path) => (
                      <li
                        key={path}
                        className="break-all rounded bg-base-100/60 px-3 py-2 font-mono text-xs text-base-content/55"
                      >
                        {normalizeDisplayPath(path)}
                      </li>
                    ))}
                  </ul>
                )}
              </div>
            </div>
          ) : null}
        </div>

        <div className="flex items-center justify-end gap-3 border-t border-[var(--border-subtle)] px-8 py-5">
          <button className="btn btn-ghost" onClick={onClose} disabled={uninstallingSkillId !== null}>
            {t('common.cancel')}
          </button>
          <button
            className="btn btn-error"
            onClick={() => (deletePreview ? onUninstall(deletePreview.skillId) : undefined)}
            disabled={!deletePreview || uninstallingSkillId === deletePreview.skillId}
          >
            {deletePreview && uninstallingSkillId === deletePreview.skillId ? (
              <span className="loading loading-spinner loading-xs"></span>
            ) : (
              <i className="hn hn-trash"></i>
            )}
            {t('repository.confirmUninstall')}
          </button>
        </div>
      </div>
    </div>
  )
}
