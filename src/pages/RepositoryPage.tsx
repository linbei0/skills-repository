import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { RepositoryImportModal } from '../components/RepositoryImportModal'
import { useRepositoryStore } from '../stores/use-repository-store'
import type {
  ImportRepositorySkillRequest,
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

const resolveStatusKey = (securityLevel: string, blocked: boolean) => {
  if (blocked) return 'blocked'
  if (securityLevel === 'safe') return 'safe'
  if (securityLevel === 'low') return 'low'
  if (securityLevel === 'medium') return 'medium'
  return 'unknown'
}

export function RepositoryPage() {
  const { t, i18n } = useTranslation()
  const [importOpen, setImportOpen] = useState(false)
  const items = useRepositoryStore((state) => state.items)
  const loading = useRepositoryStore((state) => state.loading)
  const loaded = useRepositoryStore((state) => state.loaded)
  const error = useRepositoryStore((state) => state.error)
  const selectedDetail = useRepositoryStore((state) => state.selectedDetail)
  const detailLoading = useRepositoryStore((state) => state.detailLoading)
  const detailError = useRepositoryStore((state) => state.detailError)
  const uninstallingSkillId = useRepositoryStore((state) => state.uninstallingSkillId)
  const resolvingImport = useRepositoryStore((state) => state.resolvingImport)
  const importing = useRepositoryStore((state) => state.importing)
  const importError = useRepositoryStore((state) => state.importError)
  const importBlockedLevel = useRepositoryStore((state) => state.importBlockedLevel)
  const resolvedImport = useRepositoryStore((state) => state.resolvedImport)
  const refresh = useRepositoryStore((state) => state.refresh)
  const loadDetail = useRepositoryStore((state) => state.loadDetail)
  const closeDetail = useRepositoryStore((state) => state.closeDetail)
  const uninstall = useRepositoryStore((state) => state.uninstall)
  const resolveImport = useRepositoryStore((state) => state.resolveImport)
  const importSkill = useRepositoryStore((state) => state.importSkill)
  const resetImportState = useRepositoryStore((state) => state.resetImportState)

  useEffect(() => {
    void refresh()
  }, [refresh])

  const openImportModal = () => {
    resetImportState()
    setImportOpen(true)
  }

  const closeImportModal = () => {
    setImportOpen(false)
    resetImportState()
  }

  const handleResolveImport = async (sourceKind: RepositoryImportSourceKind, input: string) => {
    await resolveImport({ sourceKind, input })
  }

  const handleImportSkill = async (request: ImportRepositorySkillRequest) => {
    return importSkill(request)
  }

  return (
    <div className="space-y-6">
      <section className="rounded-box border border-base-300 bg-base-100 p-6">
        <div className="flex items-start justify-between gap-4">
          <div>
            <h2 className="text-3xl font-semibold">{t('repository.title')}</h2>
            <p className="mt-3 max-w-3xl text-sm text-base-content/65">
              {t('repository.description')}
            </p>
          </div>
          <button className="btn btn-primary" onClick={openImportModal}>
            {t('repository.import.open')}
          </button>
        </div>
      </section>

      <section className="overflow-hidden rounded-box border border-base-300 bg-base-100">
        {loading ? (
          <div className="p-6 text-sm text-base-content/60">{t('repository.loading')}</div>
        ) : error ? (
          <div className="p-6 text-sm text-error">{error}</div>
        ) : loaded && items.length === 0 ? (
          <div className="p-6 text-sm text-base-content/60">{t('repository.empty')}</div>
        ) : (
          <div className="overflow-x-auto">
            <table className="table">
              <thead>
                <tr>
                  <th>{t('common.name')}</th>
                  <th>{t('repository.source')}</th>
                  <th>{t('repository.installedAt')}</th>
                  <th>{t('common.status')}</th>
                  <th>{t('repository.actions')}</th>
                </tr>
              </thead>
              <tbody>
                {items.map((item) => (
                  <tr key={item.id}>
                    <td className="font-medium">{item.name}</td>
                    <td>{resolveSourceLabel(item.sourceType, item.sourceMarket, t)}</td>
                    <td>{formatInstalledAt(item.installedAt, i18n.language)}</td>
                    <td>
                      <span className="badge badge-outline">
                        {t(
                          `repository.statusValues.${resolveStatusKey(item.securityLevel, item.blocked)}`,
                        )}
                      </span>
                    </td>
                    <td>
                      <div className="flex flex-wrap gap-2">
                        <button
                          className="btn btn-sm btn-outline"
                          onClick={() => void loadDetail(item.id)}
                        >
                          {t('repository.view')}
                        </button>
                        <button
                          className="btn btn-sm btn-outline btn-error"
                          onClick={() => void uninstall(item.id)}
                          disabled={uninstallingSkillId === item.id}
                        >
                          {uninstallingSkillId === item.id
                            ? t('repository.uninstalling')
                            : t('repository.uninstall')}
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

      {selectedDetail || detailLoading || detailError ? (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-base-content/45 p-6 backdrop-blur-sm">
          <div className="flex max-h-[90vh] w-full max-w-5xl flex-col overflow-hidden rounded-box border border-base-300 bg-base-100 shadow-2xl">
            <div className="flex items-start justify-between gap-4 border-b border-base-300 px-6 py-5">
              <div className="min-w-0">
                <h3 className="truncate text-3xl font-semibold">
                  {selectedDetail?.name ?? t('repository.detailTitle')}
                </h3>
                {selectedDetail ? (
                  <>
                    <p className="mt-2 break-all font-mono text-xs text-base-content/55">
                      {selectedDetail.canonicalPath}
                    </p>
                    <div className="mt-4 flex flex-wrap items-center gap-4 text-sm text-base-content/70">
                      <span>{formatInstalledAt(selectedDetail.installedAt, i18n.language)}</span>
                      <span>
                        {resolveSourceLabel(
                          selectedDetail.sourceType,
                          selectedDetail.sourceMarket,
                          t,
                        )}
                      </span>
                      <span>
                        {t(
                          `repository.statusValues.${resolveStatusKey(selectedDetail.securityLevel, selectedDetail.blocked)}`,
                        )}
                      </span>
                    </div>
                  </>
                ) : null}
              </div>
              <button className="btn btn-ghost btn-circle" aria-label={t('common.close')} onClick={closeDetail}>
                <span className="text-xl font-semibold leading-none">x</span>
              </button>
            </div>

            <div className="overflow-y-auto p-6">
              {detailLoading ? (
                <div className="text-sm text-base-content/60">{t('repository.detailLoading')}</div>
              ) : detailError ? (
                <div className="text-sm text-error">{detailError}</div>
              ) : selectedDetail ? (
                <div className="space-y-4">
                  {selectedDetail.sourceUrl ? (
                    <p className="break-all text-sm text-base-content/70">
                      {selectedDetail.sourceUrl}
                    </p>
                  ) : null}
                  <pre className="overflow-x-auto whitespace-pre-wrap rounded-box border border-base-300 bg-base-200/60 p-5 text-sm leading-7">
                    {selectedDetail.skillMarkdown}
                  </pre>
                </div>
              ) : null}
            </div>
          </div>
        </div>
      ) : null}

      <RepositoryImportModal
        open={importOpen}
        resolving={resolvingImport}
        importing={importing}
        importError={importError}
        importBlockedLevel={importBlockedLevel}
        resolvedImport={resolvedImport}
        existingSlugs={items.map((item) => item.slug)}
        onReset={resetImportState}
        onClose={closeImportModal}
        onResolve={handleResolveImport}
        onImport={handleImportSkill}
      />
    </div>
  )
}
