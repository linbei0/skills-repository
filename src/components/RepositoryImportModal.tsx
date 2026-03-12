import { open as openDialog } from '@tauri-apps/plugin-dialog'
import { useEffect, useMemo, useState } from 'react'
import { useTranslation } from 'react-i18next'
import type {
  ImportRepositorySkillRequest,
  InstallSkillResult,
  RepositoryImportSourceKind,
  ResolveRepositoryImportResult,
  SecurityReport,
} from '../types/app'
import { normalizeDisplayPath } from '../lib/normalize-display-path'

interface RepositoryImportModalProps {
  open: boolean
  resolving: boolean
  importing: boolean
  importError: string | null
  importBlockedReport: SecurityReport | null
  resolvedImport: ResolveRepositoryImportResult | null
  existingSlugs: string[]
  onReset: () => void
  onClose: () => void
  onResolve: (sourceKind: RepositoryImportSourceKind, input: string) => Promise<void>
  onImport: (request: ImportRepositorySkillRequest) => Promise<InstallSkillResult>
}

type ImportBatchItemStatus = 'installed' | 'overridden' | 'blocked' | 'existing' | 'failed'

interface ImportBatchItemResult {
  manifestPath: string
  name: string
  slug: string
  status: ImportBatchItemStatus
  message?: string
  securityReport?: SecurityReport | null
  riskOverrideApplied?: boolean
}

const sourceKinds: RepositoryImportSourceKind[] = ['github', 'local_directory', 'local_zip']

export function RepositoryImportModal({
  open,
  resolving,
  importing,
  importError,
  importBlockedReport,
  resolvedImport,
  existingSlugs,
  onReset,
  onClose,
  onResolve,
  onImport,
}: RepositoryImportModalProps) {
  const { t } = useTranslation()
  const [sourceKind, setSourceKind] = useState<RepositoryImportSourceKind>('github')
  const [input, setInput] = useState('')
  const [selectedManifestPaths, setSelectedManifestPaths] = useState<string[]>([])
  const [batchResults, setBatchResults] = useState<ImportBatchItemResult[]>([])
  const [riskAcknowledgements, setRiskAcknowledgements] = useState<Record<string, boolean>>({})

  const activeResolvedImport =
    resolvedImport && resolvedImport.sourceKind === sourceKind ? resolvedImport : null

  const existingSlugSet = useMemo(() => new Set(existingSlugs), [existingSlugs])

  useEffect(() => {
    if (!open) {
      setSourceKind('github')
      setInput('')
      setSelectedManifestPaths([])
      setBatchResults([])
      setRiskAcknowledgements({})
    }
  }, [open])

  useEffect(() => {
    if (!activeResolvedImport) {
      setSelectedManifestPaths([])
      return
    }

    setInput(normalizeDisplayPath(activeResolvedImport.normalizedInput))
    setSelectedManifestPaths((current) => {
      const next = current.filter((manifestPath) =>
        activeResolvedImport.candidates.some((candidate) => candidate.manifestPath === manifestPath),
      )

      if (next.length > 0) {
        return next
      }

      return activeResolvedImport.candidates.length === 1
        ? [activeResolvedImport.candidates[0].manifestPath]
        : []
    })
  }, [activeResolvedImport])

  const selectedCandidates = useMemo(
    () =>
      activeResolvedImport?.candidates.filter((candidate) =>
        selectedManifestPaths.includes(candidate.manifestPath),
      ) ?? [],
    [activeResolvedImport, selectedManifestPaths],
  )

  const selectedCandidate = selectedCandidates[0] ?? null
  const selectedBatchResult = useMemo(
    () =>
      selectedCandidate
        ? batchResults.find((item) => item.manifestPath === selectedCandidate.manifestPath) ?? null
        : null,
    [batchResults, selectedCandidate],
  )
  const activeSecurityReport = selectedBatchResult?.securityReport ?? importBlockedReport
  const blockingReasons = useMemo(() => {
    if (!activeSecurityReport) return []
    if (activeSecurityReport.blockingReasons.length > 0) {
      return activeSecurityReport.blockingReasons
    }

    return activeSecurityReport.issues
      .filter((issue) => issue.blocking)
      .map((issue) => `${issue.title}: ${issue.description}`)
  }, [activeSecurityReport])

  if (!open) return null

  const resetForChangedInput = () => {
    setBatchResults([])
    setRiskAcknowledgements({})
    if (activeResolvedImport) {
      onReset()
    }
  }

  const setInputAndReset = (value: string) => {
    setInput(normalizeDisplayPath(value))
    resetForChangedInput()
  }

  const pickDirectory = async () => {
    const selected = await openDialog({
      directory: true,
      multiple: false,
    })

    if (typeof selected === 'string') {
      setInputAndReset(selected)
    }
  }

  const pickZipFile = async () => {
    const selected = await openDialog({
      directory: false,
      multiple: false,
      filters: [{ name: 'Zip', extensions: ['zip'] }],
    })

    if (typeof selected === 'string') {
      setInputAndReset(selected)
    }
  }

  const toggleManifestPath = (manifestPath: string) => {
    setBatchResults([])
    setSelectedManifestPaths((current) =>
      current.includes(manifestPath)
        ? current.filter((item) => item !== manifestPath)
        : [...current, manifestPath],
    )
  }

  const selectAllCandidates = () => {
    if (!activeResolvedImport) return
    setBatchResults([])
    setSelectedManifestPaths(activeResolvedImport.candidates.map((candidate) => candidate.manifestPath))
  }

  const clearSelectedCandidates = () => {
    setBatchResults([])
    setSelectedManifestPaths([])
  }

  const canResolve = !resolving && !importing && input.trim().length > 0
  const canImport =
    !resolving &&
    !importing &&
    activeResolvedImport !== null &&
    selectedCandidates.length > 0 &&
    input.trim().length > 0
  const canOverrideSelectedCandidate =
    !importing &&
    selectedCandidate !== null &&
    selectedBatchResult?.status === 'blocked' &&
    riskAcknowledgements[selectedCandidate.manifestPath] === true

  const resolveSecurityLevelLabel = (level: string) =>
    t(`security.levels.${level}`, { defaultValue: level })

  const buildImportRequest = (
    candidate: (typeof selectedCandidates)[number],
    allowRiskOverride = false,
  ): ImportRepositorySkillRequest => ({
    sourceKind,
    input: input.trim(),
    selectedManifestPath: candidate.manifestPath,
    selectedSkillRoot: candidate.skillRoot,
    name: candidate.name,
    slug: candidate.slug,
    sourceUrl: candidate.sourceUrl,
    repoUrl: candidate.repoUrl,
    version: candidate.version,
    author: candidate.author,
    description: candidate.description,
    allowRiskOverride,
  })

  const buildBatchResult = (
    candidate: (typeof selectedCandidates)[number],
    result: InstallSkillResult,
  ): ImportBatchItemResult => {
    if (result.blocked) {
      return {
        manifestPath: candidate.manifestPath,
        name: candidate.name,
        slug: candidate.slug,
        status: 'blocked',
        message: t('repository.import.result.blocked', {
          level: resolveSecurityLevelLabel(result.securityLevel),
        }),
        securityReport: result.securityReport ?? null,
        riskOverrideApplied: false,
      }
    }

    return {
      manifestPath: candidate.manifestPath,
      name: candidate.name,
      slug: candidate.slug,
      status: result.riskOverrideApplied ? 'overridden' : 'installed',
      message: result.riskOverrideApplied
        ? t('repository.import.result.overridden')
        : t('repository.import.result.installed'),
      securityReport: result.securityReport ?? null,
      riskOverrideApplied: result.riskOverrideApplied ?? false,
    }
  }

  const handleImport = async () => {
    const nextResults: ImportBatchItemResult[] = []
    const nextExistingSlugs = new Set(existingSlugSet)

    for (const candidate of selectedCandidates) {
      if (nextExistingSlugs.has(candidate.slug)) {
        nextResults.push({
          manifestPath: candidate.manifestPath,
          name: candidate.name,
          slug: candidate.slug,
          status: 'existing',
          message: t('repository.import.result.existing'),
        })
        continue
      }

      try {
        const result = await onImport(buildImportRequest(candidate))
        const batchResult = buildBatchResult(candidate, result)

        if (batchResult.status === 'blocked') {
          nextResults.push(batchResult)
        } else {
          nextExistingSlugs.add(candidate.slug)
          nextResults.push(batchResult)
        }
      } catch (error) {
        nextResults.push({
          manifestPath: candidate.manifestPath,
          name: candidate.name,
          slug: candidate.slug,
          status: 'failed',
          message: error instanceof Error ? error.message : String(error),
        })
      }
    }

    setBatchResults(nextResults)

    if (
      nextResults.length > 0 &&
      nextResults.every((item) => item.status === 'installed' || item.status === 'overridden')
    ) {
      onClose()
    }
  }

  const handleOverrideSelectedCandidate = async () => {
    if (!selectedCandidate) return

    try {
      const result = await onImport(buildImportRequest(selectedCandidate, true))
      const nextResult = buildBatchResult(selectedCandidate, result)

      setBatchResults((current) =>
        current.map((item) =>
          item.manifestPath === selectedCandidate.manifestPath ? nextResult : item,
        ),
      )
      setRiskAcknowledgements((current) => ({
        ...current,
        [selectedCandidate.manifestPath]: false,
      }))
    } catch (error) {
      setBatchResults((current) =>
        current.map((item) =>
          item.manifestPath === selectedCandidate.manifestPath
            ? {
                ...item,
                status: 'failed',
                message: error instanceof Error ? error.message : String(error),
              }
            : item,
        ),
      )
    }
  }

  const renderResultBadgeClass = (status: ImportBatchItemStatus) => {
    switch (status) {
      case 'installed':
        return 'badge badge-success'
      case 'overridden':
        return 'badge badge-warning'
      case 'blocked':
        return 'badge badge-warning'
      case 'existing':
        return 'badge badge-info'
      default:
        return 'badge badge-error'
    }
  }

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-[var(--bg-modal-overlay)] p-4 backdrop-blur-sm transition-all duration-300 md:p-6">
      <div className="flex max-h-[92vh] w-full max-w-6xl flex-col overflow-hidden rounded-xl border border-[var(--border-subtle)] bg-[var(--bg-modal-panel)] shadow-[0_0_50px_rgba(0,0,0,0.5)]">
        
        {/* Header */}
        <div className="flex items-start justify-between gap-4 border-b border-[var(--border-subtle)] bg-base-100/50 px-8 py-6 backdrop-blur-md md:px-8">
          <div>
            <h3 className="text-2xl font-bold text-base-content">{t('repository.import.title')}</h3>
            <p className="mt-2 max-w-3xl text-sm leading-relaxed text-base-content/60">
              {t('repository.import.subtitle')}
            </p>
          </div>
          <button 
            className="btn btn-circle btn-ghost btn-sm text-base-content/50 hover:bg-base-content/10 hover:text-base-content" 
            aria-label={t('common.close')} 
            onClick={onClose}
          >
            <i className="hn hn-times text-lg"></i>
          </button>
        </div>

        <div className="space-y-6 overflow-y-auto p-8 md:p-8 custom-scrollbar">
          {/* Source Selection Section */}
          <section className="rounded-lg border border-[var(--border-subtle)] bg-base-200/20 p-6">
            <div className="flex flex-wrap gap-3">
              {sourceKinds.map((kind) => (
                <button
                  key={kind}
                  className={`btn h-10 min-h-[2.5rem] px-6 transition-all duration-200 ${
                    sourceKind === kind 
                      ? 'btn-primary border-none bg-primary text-[var(--text-inverse)] shadow-[var(--shadow-neon-primary)]' 
                      : 'btn-outline border-[var(--border-subtle)] text-base-content/70 hover:border-primary/50 hover:text-base-content'
                  }`}
                  onClick={() => {
                    setSourceKind(kind)
                    setInput('')
                    setSelectedManifestPaths([])
                    setBatchResults([])
                    onReset()
                  }}
                >
                  {kind === 'github' && <i className="hn hn-github mr-2"></i>}
                  {kind === 'local_directory' && <i className="hn hn-folder mr-2"></i>}
                  {kind === 'local_zip' && <i className="hn hn-file-import mr-2"></i>}
                  {t(`repository.import.sourceKinds.${kind}`)}
                </button>
              ))}
            </div>

            <div className="mt-6 grid gap-4 md:grid-cols-[1fr_auto]">
              <div className="form-control w-full">
                <div className="relative">
                  <div className="pointer-events-none absolute inset-y-0 left-0 flex items-center pl-4">
                    <i className="hn hn-search text-base-content/40"></i>
                  </div>
                  <input
                    className="input input-bordered h-12 w-full border-[var(--border-subtle)] bg-[var(--bg-input)] pl-11 text-base-content placeholder:text-base-content/30 focus:border-primary/50 focus:bg-[var(--bg-input-focus)] focus:outline-none"
                    value={input}
                    onChange={(event) => setInputAndReset(event.target.value)}
                    placeholder={t(`repository.import.placeholders.${sourceKind}`)}
                  />
                </div>
              </div>

              {sourceKind === 'local_directory' ? (
                <button 
                  className="btn h-12 min-h-[3rem] border-[var(--border-subtle)] bg-base-200/50 text-base-content hover:bg-base-200 hover:border-[var(--border-subtle)]" 
                  onClick={() => void pickDirectory()}
                >
                  <i className="hn hn-folder-open mr-2"></i>
                  {t('repository.import.browse')}
                </button>
              ) : null}

              {sourceKind === 'local_zip' ? (
                <button 
                  className="btn h-12 min-h-[3rem] border-[var(--border-subtle)] bg-base-200/50 text-base-content hover:bg-base-200 hover:border-[var(--border-subtle)]" 
                  onClick={() => void pickZipFile()}
                >
                  <i className="hn hn-folder-open mr-2"></i>
                  {t('repository.import.browse')}
                </button>
              ) : null}
            </div>

            <div className="mt-6 flex flex-wrap items-center justify-between gap-3 border-t border-[var(--border-subtle)] pt-4">
              <div className="flex items-center gap-2 text-sm text-base-content/50">
                <i className="hn hn-info-circle"></i>
                {t('repository.import.supportedHint')}
              </div>
              <button
                className="btn btn-primary h-10 min-h-[2.5rem] border-none bg-primary px-8 text-[var(--text-inverse)] shadow-[var(--shadow-neon-primary)] hover:shadow-[0_0_25px_rgba(var(--color-primary),0.5)] disabled:bg-base-300 disabled:text-base-content/30"
                disabled={!canResolve}
                onClick={() => {
                  void onResolve(sourceKind, input.trim()).catch(() => undefined)
                }}
              >
                {resolving ? (
                  <>
                    <span className="loading loading-spinner loading-sm"></span>
                    {t('repository.import.resolving')}
                  </>
                ) : (
                  <>
                    <i className="hn hn-check mr-2"></i>
                    {t('repository.import.resolve')}
                  </>
                )}
              </button>
            </div>
          </section>

          {importError ? (
            <section className="rounded-lg border border-error/30 bg-error/10 p-4 text-sm leading-6 text-error shadow-[0_0_15px_rgba(255,0,0,0.1)]">
              <div className="flex items-center gap-3">
                <i className="hn hn-exclaimation text-lg"></i>
                {importError}
              </div>
            </section>
          ) : null}

          {importBlockedReport ? (
            <section className="rounded-lg border border-warning/30 bg-warning/10 p-4 text-sm leading-6 text-warning shadow-[0_0_15px_rgba(255,165,0,0.1)]">
              <div className="flex items-start gap-3">
                <i className="hn hn-shield text-lg"></i>
                <div className="space-y-2">
                  <p>
                    {t('repository.import.blocked', {
                      level: resolveSecurityLevelLabel(importBlockedReport.level),
                    })}
                  </p>
                  {blockingReasons.length > 0 ? (
                    <ul className="list-disc space-y-1 pl-5 text-xs text-warning/80">
                      {blockingReasons.slice(0, 3).map((reason) => (
                        <li key={reason}>{reason}</li>
                      ))}
                    </ul>
                  ) : null}
                </div>
              </div>
            </section>
          ) : null}

          {activeResolvedImport ? (
            <section className="grid gap-6 xl:grid-cols-[1.05fr_0.95fr]">
              {/* Candidates List */}
              <article className="flex flex-col rounded-lg border border-[var(--border-subtle)] bg-base-200/30">
                <div className="flex flex-wrap items-center justify-between gap-4 border-b border-[var(--border-subtle)] bg-base-100/50 p-5">
                  <div>
                    <h4 className="text-lg font-bold text-base-content">{t('repository.import.candidatesTitle')}</h4>
                    <p className="mt-1 text-sm text-base-content/50">
                      {t('repository.import.candidatesCount', {
                        count: activeResolvedImport.candidates.length,
                      })}
                    </p>
                  </div>
                  <div className="flex flex-wrap gap-2">
                    <button className="btn btn-xs btn-ghost text-primary hover:bg-primary/10" onClick={selectAllCandidates}>
                      {t('repository.import.selectAll')}
                    </button>
                    <button className="btn btn-xs btn-ghost text-base-content/50 hover:text-base-content" onClick={clearSelectedCandidates}>
                      {t('repository.import.clearSelection')}
                    </button>
                  </div>
                </div>

                <div className="flex-1 space-y-3 overflow-y-auto p-5 custom-scrollbar max-h-[400px]">
                  {activeResolvedImport.candidates.map((candidate) => {
                    const exists = existingSlugSet.has(candidate.slug)
                    const isSelected = selectedManifestPaths.includes(candidate.manifestPath)

                    return (
                      <label
                        key={candidate.manifestPath}
                        className={`group flex cursor-pointer items-start gap-4 rounded-lg border p-4 transition-all duration-200 ${
                          isSelected 
                            ? 'border-primary/50 bg-primary/5 shadow-[inset_0_0_10px_rgba(var(--color-primary),0.05)]' 
                            : 'border-[var(--border-subtle)] bg-base-100 hover:border-[var(--border-subtle)] hover:bg-base-200/50'
                        }`}
                      >
                        <input
                          type="checkbox"
                          className="checkbox checkbox-sm checkbox-primary mt-1 border-[var(--border-subtle)] bg-base-100"
                          checked={isSelected}
                          onChange={() => toggleManifestPath(candidate.manifestPath)}
                        />
                        <div className="min-w-0 flex-1">
                          <div className="flex flex-wrap items-center gap-2">
                            <p className={`font-bold ${isSelected ? 'text-primary' : 'text-base-content/90'}`}>
                              {candidate.name}
                            </p>
                            <span className="badge badge-outline border-[var(--border-subtle)] text-xs text-base-content/50">{candidate.slug}</span>
                            {exists ? (
                              <span className="badge badge-info badge-sm gap-1 bg-info/10 text-info border-0">
                                <i className="hn hn-check text-[10px]"></i>
                                {t('repository.import.existingBadge')}
                              </span>
                            ) : null}
                          </div>
                          <p className="mt-2 break-all font-mono text-xs text-base-content/40">
                            {candidate.manifestPath}
                          </p>
                          {candidate.description ? (
                            <p className="mt-2 line-clamp-2 text-sm text-base-content/60">{candidate.description}</p>
                          ) : null}
                        </div>
                      </label>
                    )
                  })}
                </div>
              </article>

              {/* Preview Panel */}
              <article className="flex flex-col rounded-lg border border-[var(--border-subtle)] bg-base-200/30">
                <div className="border-b border-[var(--border-subtle)] bg-base-100/50 p-5">
                  <h4 className="text-lg font-bold text-base-content">{t('repository.import.previewTitle')}</h4>
                </div>

                <div className="flex-1 max-h-[400px] overflow-y-auto p-5 custom-scrollbar">
                  {selectedCandidate ? (
                    <div className="space-y-4 text-sm">
                      <div className="rounded-lg border border-[var(--border-subtle)] bg-base-100 p-5 shadow-inner">
                        <div className="mb-4 flex items-center gap-3">
                          <div className="flex h-10 w-10 items-center justify-center rounded bg-primary/10 text-primary">
                            <i className="hn hn-code-block text-xl"></i>
                          </div>
                          <div>
                            <p className="text-lg font-bold text-base-content">{selectedCandidate.name}</p>
                            <p className="text-xs text-base-content/50">
                              {selectedCandidate.version || 'v1.0.0'}
                            </p>
                          </div>
                        </div>

                        <div className="space-y-3 rounded bg-[var(--bg-input)] p-4 font-mono text-xs text-base-content/60">
                          <div className="flex justify-between">
                            <span className="text-base-content/40">Source:</span>
                            <span className="text-right text-base-content">
                              {t(`repository.import.sourceKinds.${sourceKind}`)}
                            </span>
                          </div>
                          <div className="flex justify-between">
                            <span className="text-base-content/40">Slug:</span>
                            <span className="text-right text-primary">{selectedCandidate.slug}</span>
                          </div>
                          <div className="flex justify-between">
                            <span className="text-base-content/40">Author:</span>
                            <span className="text-right text-base-content">
                              {selectedCandidate.author || 'Unknown'}
                            </span>
                          </div>
                        </div>

                        <div className="mt-4 border-t border-[var(--border-subtle)] pt-4">
                          <p className="mb-2 text-xs uppercase tracking-wider text-base-content/40">
                            Source URL
                          </p>
                          <p className="break-all font-mono text-xs text-primary/80">
                            {normalizeDisplayPath(selectedCandidate.sourceUrl)}
                          </p>
                        </div>
                      </div>

                      {activeResolvedImport.warnings.length > 0 ? (
                        <div className="rounded-lg border border-warning/30 bg-warning/10 p-4 text-warning shadow-[0_0_15px_rgba(255,165,0,0.1)]">
                          <div className="flex items-start gap-2">
                            <i className="hn hn-exclaimation mt-0.5"></i>
                            <div>{activeResolvedImport.warnings.join(' / ')}</div>
                          </div>
                        </div>
                      ) : null}

                      {activeSecurityReport ? (
                        <div className="rounded-lg border border-warning/30 bg-warning/5 p-4 shadow-[0_0_15px_rgba(255,165,0,0.08)]">
                          <div className="flex flex-wrap items-center justify-between gap-3">
                            <div>
                              <h5 className="font-bold text-base-content">
                                {t('repository.import.security.title')}
                              </h5>
                              <p className="mt-1 text-xs text-base-content/60">
                                {t('repository.import.security.summary', {
                                  level: resolveSecurityLevelLabel(activeSecurityReport.level),
                                  count: activeSecurityReport.issues.length,
                                })}
                              </p>
                            </div>
                            <div className="flex flex-wrap gap-2">
                              <span className="badge badge-warning badge-sm border-0">
                                {resolveSecurityLevelLabel(activeSecurityReport.level)}
                              </span>
                              {activeSecurityReport.blocked ? (
                                <span className="badge badge-error badge-sm border-0">
                                  {t('repository.import.security.blockedBadge')}
                                </span>
                              ) : null}
                            </div>
                          </div>

                          {blockingReasons.length > 0 ? (
                            <div className="mt-4">
                              <p className="mb-2 text-xs font-bold uppercase tracking-wider text-base-content/40">
                                {t('repository.import.security.blockingReasons')}
                              </p>
                              <ul className="space-y-2">
                                {blockingReasons.map((reason) => (
                                  <li
                                    key={reason}
                                    className="rounded border border-warning/20 bg-base-100/70 px-3 py-2 text-xs text-base-content/80"
                                  >
                                    {reason}
                                  </li>
                                ))}
                              </ul>
                            </div>
                          ) : null}

                          {activeSecurityReport.issues.length > 0 ? (
                            <div className="mt-4">
                              <p className="mb-2 text-xs font-bold uppercase tracking-wider text-base-content/40">
                                {t('repository.import.security.issues')}
                              </p>
                              <div className="space-y-2">
                                {activeSecurityReport.issues.slice(0, 4).map((issue) => (
                                  <article
                                    key={`${issue.ruleId}-${issue.filePath ?? issue.title}`}
                                    className="rounded border border-[var(--border-subtle)] bg-base-100/70 p-3"
                                  >
                                    <div className="flex flex-wrap items-center gap-2">
                                      <span className="font-medium text-base-content">
                                        {issue.title}
                                      </span>
                                      <span className="badge badge-ghost badge-xs">
                                        {issue.ruleId}
                                      </span>
                                      <span className="badge badge-outline badge-xs">
                                        {issue.severity}
                                      </span>
                                    </div>
                                    <p className="mt-2 text-xs leading-5 text-base-content/70">
                                      {issue.description}
                                    </p>
                                    {issue.filePath ? (
                                      <p className="mt-2 break-all font-mono text-[11px] text-base-content/50">
                                        {normalizeDisplayPath(issue.filePath)}
                                      </p>
                                    ) : null}
                                  </article>
                                ))}
                              </div>
                            </div>
                          ) : null}

                          {selectedCandidate && selectedBatchResult?.status === 'blocked' ? (
                            <div className="mt-4 space-y-3 rounded-lg border border-warning/20 bg-base-100/70 p-4">
                              <label className="flex cursor-pointer items-start gap-3 text-sm text-base-content/80">
                                <input
                                  type="checkbox"
                                  className="checkbox checkbox-warning checkbox-sm mt-0.5"
                                  checked={riskAcknowledgements[selectedCandidate.manifestPath] ?? false}
                                  onChange={(event) =>
                                    setRiskAcknowledgements((current) => ({
                                      ...current,
                                      [selectedCandidate.manifestPath]: event.target.checked,
                                    }))
                                  }
                                />
                                <span>{t('repository.import.security.overrideAcknowledge')}</span>
                              </label>
                              <button
                                className="btn btn-warning btn-sm"
                                disabled={!canOverrideSelectedCandidate}
                                onClick={() => void handleOverrideSelectedCandidate()}
                              >
                                <i className="hn hn-shield text-sm"></i>
                                {t('repository.import.security.overrideAction')}
                              </button>
                            </div>
                          ) : null}

                          {selectedBatchResult?.status === 'overridden' ? (
                            <div className="mt-4 rounded-lg border border-success/20 bg-success/10 px-4 py-3 text-sm text-success">
                              {t('repository.import.security.overrideInstalled')}
                            </div>
                          ) : null}
                        </div>
                      ) : null}

                      {batchResults.length > 0 ? (
                        <div className="rounded-lg border border-[var(--border-subtle)] bg-base-100 p-4">
                          <div className="mb-4 flex flex-wrap items-center justify-between gap-3">
                            <h5 className="font-bold text-base-content">
                              {t('repository.import.result.title')}
                            </h5>
                            <div className="flex flex-wrap gap-2">
                              <span className="badge badge-sm border-0 bg-success/10 text-success">
                                {t('repository.import.result.installedCount', {
                                  count: batchResults.filter((item) => item.status === 'installed').length,
                                })}
                              </span>
                              <span className="badge badge-sm border-0 bg-warning/10 text-warning">
                                {t('repository.import.result.overriddenCount', {
                                  count: batchResults.filter((item) => item.status === 'overridden').length,
                                })}
                              </span>
                              <span className="badge badge-sm border-0 bg-info/10 text-info">
                                {t('repository.import.result.existingCount', {
                                  count: batchResults.filter((item) => item.status === 'existing').length,
                                })}
                              </span>
                              <span className="badge badge-sm border-0 bg-warning/10 text-warning">
                                {t('repository.import.result.blockedCount', {
                                  count: batchResults.filter((item) => item.status === 'blocked').length,
                                })}
                              </span>
                              <span className="badge badge-sm border-0 bg-error/10 text-error">
                                {t('repository.import.result.failedCount', {
                                  count: batchResults.filter((item) => item.status === 'failed').length,
                                })}
                              </span>
                            </div>
                          </div>
                          <div className="space-y-2 max-h-[150px] overflow-y-auto pr-1 custom-scrollbar">
                            {batchResults.map((item) => (
                              <article
                                key={item.manifestPath}
                                className={`rounded border p-3 text-xs ${
                                  item.status === 'installed'
                                    ? 'border-success/20 bg-success/5'
                                    : item.status === 'overridden'
                                      ? 'border-warning/20 bg-warning/5'
                                      : item.status === 'failed'
                                        ? 'border-error/20 bg-error/5'
                                        : 'border-[var(--border-subtle)] bg-base-200'
                                }`}
                              >
                                <div className="flex items-start justify-between gap-3">
                                  <span className="max-w-[150px] truncate font-medium text-base-content/80">
                                    {item.name}
                                  </span>
                                  <span className={renderResultBadgeClass(item.status)}>
                                    {t(`repository.import.result.statuses.${item.status}`)}
                                  </span>
                                </div>
                                {item.message ? (
                                  <p className="mt-2 text-[11px] leading-5 text-base-content/60">
                                    {item.message}
                                  </p>
                                ) : null}
                              </article>
                            ))}
                          </div>
                        </div>
                      ) : null}
                    </div>
                  ) : (
                    <div className="flex h-full min-h-[200px] flex-col items-center justify-center rounded-lg border border-dashed border-[var(--border-subtle)] bg-base-100/50 p-5 text-center">
                      <div className="mb-3 rounded-full bg-base-200 p-3 text-base-content/20">
                        <i className="hn hn-cursor text-2xl"></i>
                      </div>
                      <p className="text-sm text-base-content/40">
                        {t('repository.import.noCandidateSelected')}
                      </p>
                    </div>
                  )}
                </div>
              </article>
            </section>
          ) : null}
        </div>

        <div className="flex justify-end gap-3 border-t border-[var(--border-subtle)] bg-base-100/50 px-8 py-6 backdrop-blur-md md:px-8">
          <button 
            className="btn btn-ghost text-base-content/60 hover:bg-base-content/10 hover:text-base-content" 
            onClick={onClose}
          >
            {t('common.close')}
          </button>
          <button 
            className="btn btn-primary h-10 min-h-[2.5rem] border-none bg-primary px-8 text-[var(--text-inverse)] shadow-[var(--shadow-neon-primary)] transition-all duration-300 hover:bg-primary hover:brightness-110 hover:shadow-[0_0_30px_rgba(var(--color-primary),0.6)] disabled:bg-base-300 disabled:text-base-content/30 disabled:shadow-none" 
            disabled={!canImport} 
            onClick={() => void handleImport()}
          >
            {importing ? (
              <>
                <span className="loading loading-spinner loading-sm"></span>
                {t('repository.import.importing')}
              </>
            ) : (
              <>
                <i className="hn hn-download-alt mr-2"></i>
                {t('repository.import.confirm')}
              </>
            )}
          </button>
        </div>
      </div>
    </div>
  )
}
