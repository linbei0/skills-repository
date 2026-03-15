import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { RepositoryDistributeModal } from '../components/RepositoryDistributeModal'
import { RepositoryImportModal } from '../components/RepositoryImportModal'
import { normalizeDisplayPath } from '../lib/normalize-display-path'
import { resolveSkillsTargets } from '../lib/skills-targets'
import { openSourceReference } from '../lib/tauri-client'
import { useAppStore } from '../stores/use-app-store'
import {
  flattenBatchUpdateResult,
  useRepositoryStore,
} from '../stores/use-repository-store'
import { useSettingsStore } from '../stores/use-settings-store'
import type {
  BatchRepositorySkillUpdateResult,
  BatchDistributeRepositorySkillsRequest,
  ImportRepositorySkillRequest,
  RepositorySkillUpdateItemResult,
  RepositoryImportSourceKind,
} from '../types/app'

const formatInstalledAt = (value: number, locale: string) =>
  new Intl.DateTimeFormat(locale, {
    year: 'numeric',
    month: 'long',
    day: 'numeric',
  }).format(new Date(value * 1000))

const resolveSourceLabel = (
  sourceType: string,
  sourceMarket: string | null | undefined,
  t: (key: string, options?: Record<string, unknown>) => string,
) => {
  if (sourceType === 'market') {
    return t('repository.sourceMarket', { market: sourceMarket ?? 'market' })
  }
  if (sourceType === 'github') {
    return t('repository.sourceGithub')
  }
  if (sourceType === 'local') {
    return t('repository.sourceLocal')
  }
  return t('repository.sourceUnknown')
}

const resolveStatusKey = (
  securityLevel: string,
  blocked: boolean,
  riskOverrideApplied?: boolean,
) => {
  if (riskOverrideApplied) return 'overridden'
  if (blocked) return 'blocked'
  if (securityLevel === 'safe') return 'safe'
  if (securityLevel === 'low') return 'low'
  if (securityLevel === 'medium') return 'medium'
  return 'unknown'
}

const resolveDescription = (
  value: string | null | undefined,
  t: (key: string, options?: Record<string, unknown>) => string,
) => (value?.trim() ? value : t('repository.descriptionMissing'))

const logSourceOpenFailure = (error: unknown) => {
  console.error('Failed to open source reference:', error)
}

const shouldShowSingleUpdateFeedback = (result: RepositorySkillUpdateItemResult | null) =>
  Boolean(result && (result.status !== 'updated' || result.copyDistributionCount > 0))

const batchResultCount = (result: BatchRepositorySkillUpdateResult | null) =>
  result ? result.updated.length + result.skipped.length + result.failed.length : 0

const asRecord = (value: Record<string, unknown> | null | undefined) =>
  value && typeof value === 'object' ? value : null

const detailString = (details: Record<string, unknown> | null | undefined, key: string) => {
  const value = asRecord(details)?.[key]
  return typeof value === 'string' && value.trim() ? value : null
}

const formatUpdateMessage = (
  result: RepositorySkillUpdateItemResult,
  t: (key: string, options?: Record<string, unknown>) => string,
) => {
  const parts: string[] = []

  switch (result.reasonCode) {
    case 'updated_to_latest':
      parts.push(t('repository.update.reasons.updatedToLatest'))
      break
    case 'already_up_to_date':
      parts.push(t('repository.update.reasons.alreadyUpToDate'))
      break
    case 'blocked_by_security_scan':
      parts.push(t('repository.update.reasons.blockedBySecurityScan'))
      break
    case 'update_failed':
      parts.push(t('repository.update.reasons.updateFailed'))
      break
    default:
      parts.push(t('repository.update.reasons.unknown'))
      break
  }

  if (result.copyDistributionCount > 0) {
    parts.push(
      t('repository.update.copyDistributionNotice', {
        count: result.copyDistributionCount,
      }),
    )
  }

  const error = detailString(result.details, 'error')
  if (error) {
    parts.push(t('repository.update.errorDetail', { error }))
  }

  return parts.join(' ')
}

export function RepositoryPage() {
  const { t, i18n } = useTranslation()
  const [importOpen, setImportOpen] = useState(false)
  const settings = useSettingsStore((state) => state.settings)
  const builtinSkillsTargets = useAppStore((state) => state.builtinSkillsTargets)
  const items = useRepositoryStore((state) => state.items)
  const loading = useRepositoryStore((state) => state.loading)
  const loaded = useRepositoryStore((state) => state.loaded)
  const error = useRepositoryStore((state) => state.error)
  const selectedDetail = useRepositoryStore((state) => state.selectedDetail)
  const detailLoading = useRepositoryStore((state) => state.detailLoading)
  const detailError = useRepositoryStore((state) => state.detailError)
  const uninstallingSkillId = useRepositoryStore((state) => state.uninstallingSkillId)
  const deletePreview = useRepositoryStore((state) => state.deletePreview)
  const deletePreviewLoading = useRepositoryStore((state) => state.deletePreviewLoading)
  const deletePreviewError = useRepositoryStore((state) => state.deletePreviewError)
  const updatingSkillId = useRepositoryStore((state) => state.updatingSkillId)
  const batchUpdating = useRepositoryStore((state) => state.batchUpdating)
  const updateError = useRepositoryStore((state) => state.updateError)
  const lastUpdateResult = useRepositoryStore((state) => state.lastUpdateResult)
  const lastBatchUpdateResult = useRepositoryStore((state) => state.lastBatchUpdateResult)
  const distributionOpen = useRepositoryStore((state) => state.distributionOpen)
  const distributing = useRepositoryStore((state) => state.distributing)
  const distributionError = useRepositoryStore((state) => state.distributionError)
  const lastDistributionResult = useRepositoryStore((state) => state.lastDistributionResult)
  const resolvingImport = useRepositoryStore((state) => state.resolvingImport)
  const importing = useRepositoryStore((state) => state.importing)
  const importError = useRepositoryStore((state) => state.importError)
  const importBlockedReport = useRepositoryStore((state) => state.importBlockedReport)
  const resolvedImport = useRepositoryStore((state) => state.resolvedImport)
  const refresh = useRepositoryStore((state) => state.refresh)
  const loadDetail = useRepositoryStore((state) => state.loadDetail)
  const closeDetail = useRepositoryStore((state) => state.closeDetail)
  const loadDeletePreview = useRepositoryStore((state) => state.loadDeletePreview)
  const clearDeletePreview = useRepositoryStore((state) => state.clearDeletePreview)
  const updateSkill = useRepositoryStore((state) => state.updateSkill)
  const updateGithubSkills = useRepositoryStore((state) => state.updateGithubSkills)
  const clearUpdateState = useRepositoryStore((state) => state.clearUpdateState)
  const uninstall = useRepositoryStore((state) => state.uninstall)
  const openDistribution = useRepositoryStore((state) => state.openDistribution)
  const closeDistribution = useRepositoryStore((state) => state.closeDistribution)
  const batchDistributeSkills = useRepositoryStore((state) => state.batchDistributeSkills)
  const resetDistributionState = useRepositoryStore((state) => state.resetDistributionState)
  const resolveImport = useRepositoryStore((state) => state.resolveImport)
  const importSkill = useRepositoryStore((state) => state.importSkill)
  const resetImportState = useRepositoryStore((state) => state.resetImportState)

  useEffect(() => {
    void refresh()
  }, [refresh])

  const visibleTargets = resolveSkillsTargets(builtinSkillsTargets, settings).filter((target) =>
    settings.visibleSkillsTargetIds.includes(target.id),
  )
  const updatableCount = items.filter((item) => item.canUpdate).length

  const openImportModal = () => {
    resetImportState()
    clearUpdateState()
    setImportOpen(true)
  }

  const closeImportModal = () => {
    setImportOpen(false)
    resetImportState()
  }

  const handleOpenDistribution = () => {
    resetDistributionState()
    clearUpdateState()
    openDistribution()
  }

  const handleCloseDistribution = () => {
    closeDistribution()
    resetDistributionState()
  }

  const handleOpenDeletePreview = async (skillId: string) => {
    await loadDeletePreview(skillId)
  }

  const handleCloseDeletePreview = () => {
    clearDeletePreview()
  }

  const handleUpdateSkill = async (skillId: string) => {
    await updateSkill(skillId)
  }

  const handleUpdateGithubSkills = async () => {
    await updateGithubSkills()
  }

  const handleResolveImport = async (sourceKind: RepositoryImportSourceKind, input: string) => {
    await resolveImport({ sourceKind, input })
  }

  const handleImportSkill = async (request: ImportRepositorySkillRequest) => {
    return importSkill(request)
  }

  const handleBatchDistributeSkills = async (request: BatchDistributeRepositorySkillsRequest) => {
    await batchDistributeSkills(request)
  }

  return (
    <div className="space-y-8 p-8">
      {/* Header Section */}
      <section className="relative overflow-hidden rounded-lg border border-[var(--border-subtle)] bg-base-100 p-8 shadow-[inset_0_0_20px_rgba(var(--color-primary),0.05)]">
        <div className="absolute top-0 left-0 h-1 w-full bg-gradient-to-r from-primary/0 via-primary/50 to-primary/0 opacity-20"></div>
        <div className="flex flex-col gap-6 md:flex-row md:items-center md:justify-between">
          <div className="flex-1">
            <h2 className="text-3xl font-bold tracking-tight text-base-content">{t('repository.title')}</h2>
            <p className="mt-3 max-w-3xl text-sm leading-relaxed text-base-content/70">
              {t('repository.description')}
            </p>
          </div>
          <div className="flex flex-wrap gap-3">
            <button
              className="btn btn-outline h-10 min-h-[2.5rem] border-[var(--border-subtle)] px-4 text-base-content hover:border-primary hover:bg-primary/10 hover:text-primary"
              disabled={updatableCount === 0 || batchUpdating || Boolean(updatingSkillId)}
              onClick={() => void handleUpdateGithubSkills()}
            >
              {batchUpdating ? (
                <span className="loading loading-spinner loading-sm"></span>
              ) : (
                <i className="hn hn-refresh text-base"></i>
              )}
              {t('repository.update.open', { count: updatableCount })}
            </button>
            <button 
              className="btn btn-primary h-10 w-40 min-h-[2.5rem] border-none bg-primary px-4 text-[var(--text-inverse)] transition-all duration-300 hover:bg-primary hover:shadow-[var(--shadow-neon-primary)]" 
              onClick={openImportModal}
            >
              <i className="hn hn-download-alt text-lg"></i>
              {t('repository.import.open')}
            </button>
            <button
              className="btn btn-outline h-10 w-40 min-h-[2.5rem] border-[var(--border-subtle)] px-4 text-base-content hover:border-primary hover:bg-primary/10 hover:text-primary"
              disabled={items.length === 0}
              onClick={handleOpenDistribution}
            >
              <i className="hn hn-share text-lg"></i>
              {t('repository.distribute.open')}
            </button>
          </div>
        </div>
      </section>

      {updateError ? (
        <div className="rounded-lg border border-error/20 bg-error/5 px-5 py-4 text-sm text-error">
          <div className="flex items-start justify-between gap-3">
            <p className="flex-1">{updateError}</p>
            <button
              className="btn btn-circle btn-ghost btn-xs text-error/70 hover:bg-error/10 hover:text-error"
              onClick={clearUpdateState}
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
              <span className={`badge border-0 ${
                lastUpdateResult?.status === 'updated'
                  ? 'bg-success/10 text-success'
                  : lastUpdateResult?.status === 'skipped'
                    ? 'bg-info/10 text-info'
                    : 'bg-error/10 text-error'
              }`}>
                {t(`repository.update.statuses.${lastUpdateResult?.status ?? 'failed'}`)}
              </span>
              <button
                className="btn btn-circle btn-ghost btn-xs text-base-content/50 hover:bg-base-content/10 hover:text-base-content"
                onClick={clearUpdateState}
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
                onClick={clearUpdateState}
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
                    <span className={`badge border-0 ${
                      item.status === 'updated'
                        ? 'bg-success/10 text-success'
                        : item.status === 'skipped'
                          ? 'bg-info/10 text-info'
                          : 'bg-error/10 text-error'
                    }`}>
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

      {/* Skills List Section */}
      <section className="overflow-hidden rounded-lg border border-[var(--border-subtle)] bg-base-100 shadow-[inset_0_0_20px_rgba(var(--color-primary),0.02)]">
        {loading ? (
          <div className="flex flex-col items-center justify-center p-12 text-center">
            <span className="loading loading-spinner loading-lg text-primary"></span>
            <p className="mt-4 text-sm text-base-content/60">{t('repository.loading')}</p>
          </div>
        ) : error ? (
          <div className="flex items-center gap-3 bg-error/10 p-6 text-error">
             <i className="hn hn-exclaimation text-lg"></i>
             <span className="text-sm font-medium">{error}</span>
          </div>
        ) : loaded && items.length === 0 ? (
          <div className="flex flex-col items-center justify-center p-16 text-center">
            <div className="mb-4 rounded-full bg-base-200 p-4 text-base-content/30">
              <i className="hn hn-box-usd text-3xl"></i>
            </div>
            <p className="text-base font-medium text-base-content/60">{t('repository.empty')}</p>
          </div>
        ) : (
          <div className="overflow-x-auto">
            <table className="table-fixed table w-full">
              <thead>
                <tr className="border-b border-[var(--border-subtle)] bg-base-200/50 text-xs font-bold uppercase tracking-wider text-base-content/40">
                  <th className="py-4 pl-6 text-left">{t('common.name')}</th>
                  <th className="w-28 text-center">{t('repository.source')}</th>
                  <th className="w-32 text-center">{t('repository.installedAt')}</th>
                  <th className="w-28 text-center">{t('common.status')}</th>
                  <th className="w-40 pr-6 text-center">{t('repository.actions')}</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-[var(--border-subtle)]">
                {items.map((item) => (
                  <tr key={item.id} className="group transition-colors hover:bg-base-200/50">
                    <td className="py-4 pl-6 text-left">
                      <div className="flex items-start gap-3">
                        <div className="mt-1 flex h-8 w-8 shrink-0 items-center justify-center rounded bg-primary/10 text-primary">
                          <i className="hn hn-code-block text-base leading-none"></i>
                        </div>
                        <div className="min-w-0">
                          <p className="font-bold text-base-content/90 transition-colors group-hover:text-primary">
                            {item.name}
                          </p>
                          <p className="mt-1 line-clamp-2 text-sm leading-6 text-base-content/50">
                            {resolveDescription(item.description, t)}
                          </p>
                        </div>
                      </div>
                    </td>
                    <td className="text-center text-sm text-base-content/70">
                      {resolveSourceLabel(item.sourceType, item.sourceMarket, t)}
                    </td>
                    <td className="text-center font-mono text-xs text-base-content/50">
                      {formatInstalledAt(item.installedAt, i18n.language)}
                    </td>
                    <td className="text-center">
                      {(() => {
                        const statusKey = resolveStatusKey(
                          item.securityLevel,
                          item.blocked,
                          item.riskOverrideApplied,
                        )
                        return (
                          <span className={`inline-flex whitespace-nowrap badge badge-sm gap-1 border-0 font-bold ${
                            statusKey === 'blocked' ? 'bg-error/20 text-error' :
                            statusKey === 'overridden' ? 'bg-warning/20 text-warning' :
                            statusKey === 'safe' ? 'bg-success/20 text-success' :
                            statusKey === 'low' ? 'bg-success/10 text-success/80' :
                            'bg-warning/20 text-warning'
                          }`}>
                            <i className={`hn ${
                              statusKey === 'blocked' ? 'hn-lock' :
                              statusKey === 'overridden' ? 'hn-shield' :
                              statusKey === 'safe' ? 'hn-check-circle' :
                              'hn-exclaimation'
                            } text-xs`}></i>
                            {t(`repository.statusValues.${statusKey}`)}
                          </span>
                        )
                      })()}
                    </td>
                    <td className="pr-6 text-center">
                      <div className="flex justify-center gap-2 opacity-0 transition-opacity group-hover:opacity-100">
                        {item.canUpdate ? (
                          <button
                            className="btn btn-square btn-ghost btn-sm text-primary/80 hover:bg-primary/10 hover:text-primary"
                            onClick={() => void handleUpdateSkill(item.id)}
                            disabled={batchUpdating || updatingSkillId === item.id}
                            title={t('repository.update.single')}
                          >
                            {updatingSkillId === item.id ? (
                              <span className="loading loading-spinner loading-xs"></span>
                            ) : (
                              <i className="hn hn-refresh text-sm"></i>
                            )}
                          </button>
                        ) : null}
                        <button
                          className="btn btn-square btn-ghost btn-sm text-base-content/70 hover:bg-primary/10 hover:text-primary"
                          onClick={() => void loadDetail(item.id)}
                          title={t('repository.view')}
                        >
                          <i className="hn hn-eye"></i>
                        </button>
                        <button
                          className="btn btn-square btn-ghost btn-sm text-error/70 hover:bg-error/10 hover:text-error"
                          onClick={() => void handleOpenDeletePreview(item.id)}
                          disabled={uninstallingSkillId === item.id}
                          title={t('repository.uninstall')}
                        >
                          {uninstallingSkillId === item.id ? (
                            <span className="loading loading-spinner loading-xs"></span>
                          ) : (
                            <i className="hn hn-trash"></i>
                          )}
                        </button>
                      </div>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </section>

      {/* Detail Modal - Enhanced */}
      {selectedDetail || detailLoading || detailError ? (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-[var(--bg-modal-overlay)] p-6 backdrop-blur-sm transition-all duration-300">
          <div className="relative flex max-h-[85vh] w-full max-w-4xl flex-col overflow-hidden rounded-xl border border-[var(--border-subtle)] bg-[var(--bg-modal-panel)] shadow-[0_0_50px_rgba(0,0,0,0.5)]">
            
            {/* Modal Header */}
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
                        {formatInstalledAt(selectedDetail.installedAt, i18n.language)}
                      </span>
                      <span className="badge badge-outline border-[var(--border-subtle)] text-xs text-base-content/60">
                        {resolveSourceLabel(selectedDetail.sourceType, selectedDetail.sourceMarket, t)}
                      </span>
                      {selectedDetail.canUpdate ? (
                        <button
                          type="button"
                          onClick={() => void handleUpdateSkill(selectedDetail.id)}
                          disabled={batchUpdating || updatingSkillId === selectedDetail.id}
                          className="btn btn-ghost btn-xs text-primary hover:bg-primary/10"
                        >
                          {updatingSkillId === selectedDetail.id ? (
                            <span className="loading loading-spinner loading-xs"></span>
                          ) : null}
                          {t('repository.update.single')}
                        </button>
                      ) : null}
                      {selectedDetail.sourceUrl && (
                        <button
                          type="button"
                          onClick={() =>
                            void openSourceReference(selectedDetail.sourceUrl!).catch(
                              logSourceOpenFailure,
                            )
                          }
                          className="flex items-center gap-1 text-xs text-primary hover:underline"
                        >
                          <i className="hn hn-external-link"></i>
                          {t('repository.source')}
                        </button>
                      )}
                    </div>
                  </div>
                ) : null}
              </div>
              <button 
                className="btn btn-circle btn-ghost btn-sm text-base-content/50 hover:bg-base-content/10 hover:text-base-content" 
                onClick={closeDetail}
              >
                <i className="hn hn-times text-lg"></i>
              </button>
            </div>

            {/* Modal Content */}
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
      ) : null}

      {deletePreview || deletePreviewLoading || deletePreviewError ? (
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
                onClick={handleCloseDeletePreview}
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
              <button
                className="btn btn-ghost"
                onClick={handleCloseDeletePreview}
                disabled={uninstallingSkillId !== null}
              >
                {t('common.cancel')}
              </button>
              <button
                className="btn btn-error"
                onClick={() => deletePreview ? void uninstall(deletePreview.skillId) : undefined}
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
      ) : null}


      <RepositoryImportModal
        open={importOpen}
        resolving={resolvingImport}
        importing={importing}
        importError={importError}
        importBlockedReport={importBlockedReport}
        resolvedImport={resolvedImport}
        existingSlugs={items.map((item) => item.slug)}
        onReset={resetImportState}
        onClose={closeImportModal}
        onResolve={handleResolveImport}
        onImport={handleImportSkill}
      />

      <RepositoryDistributeModal
        open={distributionOpen}
        repositorySkills={items}
        targets={visibleTargets}
        distributing={distributing}
        error={distributionError}
        result={lastDistributionResult}
        onClose={handleCloseDistribution}
        onSubmit={handleBatchDistributeSkills}
      />
    </div>
  )
}
