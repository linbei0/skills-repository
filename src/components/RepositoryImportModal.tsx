import { open as openDialog } from '@tauri-apps/plugin-dialog'
import { useEffect, useMemo, useState } from 'react'
import { useTranslation } from 'react-i18next'
import type {
  ImportRepositorySkillRequest,
  RepositoryImportSourceKind,
  ResolveRepositoryImportResult,
  ResolvedRepositoryImportCandidate,
} from '../types/app'

interface RepositoryImportModalProps {
  open: boolean
  resolving: boolean
  importing: boolean
  importError: string | null
  importBlockedLevel: string | null
  resolvedImport: ResolveRepositoryImportResult | null
  onReset: () => void
  onClose: () => void
  onResolve: (sourceKind: RepositoryImportSourceKind, input: string) => Promise<void>
  onImport: (request: ImportRepositorySkillRequest) => Promise<void>
}

const sourceKinds: RepositoryImportSourceKind[] = ['github', 'local_directory', 'local_zip']

export function RepositoryImportModal({
  open,
  resolving,
  importing,
  importError,
  importBlockedLevel,
  resolvedImport,
  onReset,
  onClose,
  onResolve,
  onImport,
}: RepositoryImportModalProps) {
  const { t } = useTranslation()
  const [sourceKind, setSourceKind] = useState<RepositoryImportSourceKind>('github')
  const [input, setInput] = useState('')
  const [selectedManifestPath, setSelectedManifestPath] = useState('')

  const activeResolvedImport =
    resolvedImport && resolvedImport.sourceKind === sourceKind ? resolvedImport : null

  useEffect(() => {
    if (!open) {
      setSourceKind('github')
      setInput('')
      setSelectedManifestPath('')
    }
  }, [open])

  useEffect(() => {
    if (!activeResolvedImport) {
      setSelectedManifestPath('')
      return
    }

    setInput(activeResolvedImport.normalizedInput)
    setSelectedManifestPath((current) => {
      if (
        current &&
        activeResolvedImport.candidates.some((candidate) => candidate.manifestPath === current)
      ) {
        return current
      }

      return activeResolvedImport.candidates[0]?.manifestPath ?? ''
    })
  }, [activeResolvedImport])

  const selectedCandidate = useMemo<ResolvedRepositoryImportCandidate | null>(
    () =>
      activeResolvedImport?.candidates.find(
        (candidate) => candidate.manifestPath === selectedManifestPath,
      ) ?? null,
    [activeResolvedImport, selectedManifestPath],
  )

  if (!open) return null

  const setInputAndReset = (value: string) => {
    setInput(value)
    if (activeResolvedImport) {
      onReset()
    }
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

  const canResolve = !resolving && !importing && input.trim().length > 0
  const canImport =
    !resolving &&
    !importing &&
    activeResolvedImport !== null &&
    selectedCandidate !== null &&
    input.trim().length > 0

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-base-content/45 p-4 backdrop-blur-sm md:p-6">
      <div className="flex max-h-[92vh] w-full max-w-5xl flex-col overflow-hidden rounded-[28px] border border-base-300 bg-base-100 shadow-2xl">
        <div className="flex items-start justify-between gap-4 border-b border-base-300 px-6 py-5 md:px-7">
          <div>
            <h3 className="text-2xl font-semibold">{t('repository.import.title')}</h3>
            <p className="mt-2 max-w-3xl text-sm leading-6 text-base-content/60">
              {t('repository.import.subtitle')}
            </p>
          </div>
          <button className="btn btn-ghost btn-circle" aria-label={t('common.close')} onClick={onClose}>
            <span className="text-xl font-semibold leading-none">x</span>
          </button>
        </div>

        <div className="space-y-6 overflow-y-auto p-6 md:p-7">
          <section className="rounded-[24px] border border-base-300 bg-base-200/50 p-5">
            <div className="flex flex-wrap gap-3">
              {sourceKinds.map((kind) => (
                <button
                  key={kind}
                  className={sourceKind === kind ? 'btn btn-primary btn-sm' : 'btn btn-outline btn-sm'}
                  onClick={() => {
                    setSourceKind(kind)
                    setInput('')
                    setSelectedManifestPath('')
                    onReset()
                  }}
                >
                  {t(`repository.import.sourceKinds.${kind}`)}
                </button>
              ))}
            </div>

            <div className="mt-5 grid gap-4 md:grid-cols-[1fr_auto]">
              <label className="form-control">
                <span className="label-text">{t('repository.import.inputLabel')}</span>
                <input
                  className="input input-bordered"
                  value={input}
                  onChange={(event) => setInputAndReset(event.target.value)}
                  placeholder={t(`repository.import.placeholders.${sourceKind}`)}
                />
              </label>

              {sourceKind === 'local_directory' ? (
                <div className="form-control">
                  <span className="label-text">{t('repository.import.pickDirectory')}</span>
                  <button className="btn btn-outline" onClick={() => void pickDirectory()}>
                    {t('repository.import.browse')}
                  </button>
                </div>
              ) : null}

              {sourceKind === 'local_zip' ? (
                <div className="form-control">
                  <span className="label-text">{t('repository.import.pickZip')}</span>
                  <button className="btn btn-outline" onClick={() => void pickZipFile()}>
                    {t('repository.import.browse')}
                  </button>
                </div>
              ) : null}
            </div>

            <div className="mt-5 flex flex-wrap items-center justify-between gap-3">
              <div className="text-sm text-base-content/60">{t('repository.import.supportedHint')}</div>
              <button
                className="btn btn-primary btn-sm"
                disabled={!canResolve}
                onClick={() => void onResolve(sourceKind, input.trim())}
              >
                {resolving ? t('repository.import.resolving') : t('repository.import.resolve')}
              </button>
            </div>
          </section>

          {importError ? (
            <section className="rounded-[24px] border border-error/30 bg-error/5 p-4 text-sm leading-6 text-error">
              {importError}
            </section>
          ) : null}

          {importBlockedLevel ? (
            <section className="rounded-[24px] border border-warning/30 bg-warning/10 p-4 text-sm leading-6 text-warning">
              {t('repository.import.blocked', { level: importBlockedLevel })}
            </section>
          ) : null}

          {activeResolvedImport ? (
            <section className="grid gap-6 xl:grid-cols-[1.05fr_0.95fr]">
              <article className="rounded-[24px] border border-base-300 bg-base-200/50 p-5">
                <div className="flex items-start justify-between gap-4">
                  <div>
                    <h4 className="text-lg font-semibold">{t('repository.import.candidatesTitle')}</h4>
                    <p className="mt-1 text-sm text-base-content/60">
                      {t('repository.import.candidatesCount', {
                        count: activeResolvedImport.candidates.length,
                      })}
                    </p>
                  </div>
                  <span className="badge badge-outline">
                    {t(`repository.import.sourceKinds.${activeResolvedImport.sourceKind}`)}
                  </span>
                </div>

                <div className="mt-5 space-y-3">
                  {activeResolvedImport.candidates.map((candidate) => (
                    <label
                      key={candidate.manifestPath}
                      className="flex cursor-pointer items-start gap-4 rounded-[20px] border border-base-300 bg-base-100 p-4"
                    >
                      <input
                        type="radio"
                        className="radio radio-sm mt-1"
                        checked={selectedManifestPath === candidate.manifestPath}
                        onChange={() => setSelectedManifestPath(candidate.manifestPath)}
                      />
                      <div className="min-w-0">
                        <div className="flex flex-wrap items-center gap-2">
                          <p className="font-semibold">{candidate.name}</p>
                          <span className="badge badge-outline">{candidate.slug}</span>
                        </div>
                        <p className="mt-2 break-all text-xs text-base-content/55">
                          {candidate.manifestPath}
                        </p>
                        <p className="mt-1 break-all text-xs text-base-content/55">
                          {candidate.skillRoot || '/'}
                        </p>
                        {candidate.description ? (
                          <p className="mt-2 text-sm text-base-content/60">{candidate.description}</p>
                        ) : null}
                      </div>
                    </label>
                  ))}
                </div>
              </article>

              <article className="rounded-[24px] border border-base-300 bg-base-200/50 p-5">
                <h4 className="text-lg font-semibold">{t('repository.import.previewTitle')}</h4>

                {selectedCandidate ? (
                  <div className="mt-5 space-y-4 text-sm">
                    <div className="rounded-[20px] border border-base-300 bg-base-100 p-4">
                      <p className="font-medium">{selectedCandidate.name}</p>
                      <div className="mt-3 space-y-2 text-base-content/65">
                        <p>{t('repository.import.preview.slug', { slug: selectedCandidate.slug })}</p>
                        <p>
                          {t('repository.import.preview.sourceType', {
                            type: t(`repository.import.sourceKinds.${sourceKind}`),
                          })}
                        </p>
                        <p className="break-all">
                          {t('repository.import.preview.sourceUrl', {
                            sourceUrl: selectedCandidate.sourceUrl,
                          })}
                        </p>
                        <p className="break-all">
                          {t('repository.import.preview.canonicalPath', {
                            path: selectedCandidate.slug,
                          })}
                        </p>
                      </div>
                    </div>

                    {activeResolvedImport.warnings.length > 0 ? (
                      <div className="rounded-[20px] border border-warning/30 bg-warning/10 p-4 text-warning">
                        {activeResolvedImport.warnings.join('；')}
                      </div>
                    ) : null}
                  </div>
                ) : (
                  <div className="mt-5 rounded-[20px] border border-dashed border-base-300 bg-base-100 p-5 text-sm text-base-content/60">
                    {t('repository.import.noCandidateSelected')}
                  </div>
                )}
              </article>
            </section>
          ) : null}
        </div>

        <div className="flex justify-end gap-3 border-t border-base-300 px-6 py-5 md:px-7">
          <button className="btn btn-ghost" onClick={onClose}>
            {t('common.close')}
          </button>
          <button
            className="btn btn-primary"
            disabled={!canImport}
            onClick={() => {
              if (!selectedCandidate) return

              void onImport({
                sourceKind,
                input: input.trim(),
                selectedManifestPath: selectedCandidate.manifestPath,
                selectedSkillRoot: selectedCandidate.skillRoot,
                name: selectedCandidate.name,
                slug: selectedCandidate.slug,
                sourceUrl: selectedCandidate.sourceUrl,
                repoUrl: selectedCandidate.repoUrl,
                version: selectedCandidate.version,
                author: selectedCandidate.author,
              })
            }}
          >
            {importing ? t('repository.import.importing') : t('repository.import.confirm')}
          </button>
        </div>
      </div>
    </div>
  )
}
