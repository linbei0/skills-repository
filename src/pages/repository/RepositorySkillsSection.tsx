import { useTranslation } from 'react-i18next'
import { HighlightedText } from '../../components/common/HighlightedText'
import type { HighlightRange } from '../../lib/repository-search'
import type { RepositorySkillSummary } from '../../types/app'
import {
  formatInstalledAt,
  resolveDescription,
  resolveSourceLabel,
  resolveStatusKey,
} from './repository-page-helpers'

interface RepositoryListRow {
  item: RepositorySkillSummary
  highlights: {
    name: HighlightRange[]
    description: HighlightRange[]
    source: HighlightRange[]
  }
}

interface RepositorySkillsSectionProps {
  loading: boolean
  loaded: boolean
  error: string | null
  items: RepositorySkillSummary[]
  rows: RepositoryListRow[]
  searchExpanded: boolean
  isSearching: boolean
  hasSearchResults: boolean
  searchQueryDisplay: string
  visibleRangeStart: number
  visibleRangeEnd: number
  paginatedTotal: number
  currentPage: number
  pageCount: number
  searchPageNumbers: number[]
  locale: string
  batchUpdating: boolean
  updatingSkillId: string | null
  uninstallingSkillId: string | null
  onClearSearch: () => void
  onUpdateSkill: (skillId: string) => void
  onLoadDetail: (skillId: string) => void
  onOpenDeletePreview: (skillId: string) => void
  onChangePage: (page: number) => void
}

export function RepositorySkillsSection({
  loading,
  loaded,
  error,
  items,
  rows,
  searchExpanded,
  isSearching,
  hasSearchResults,
  searchQueryDisplay,
  visibleRangeStart,
  visibleRangeEnd,
  paginatedTotal,
  currentPage,
  pageCount,
  searchPageNumbers,
  locale,
  batchUpdating,
  updatingSkillId,
  uninstallingSkillId,
  onClearSearch,
  onUpdateSkill,
  onLoadDetail,
  onOpenDeletePreview,
  onChangePage,
}: RepositorySkillsSectionProps) {
  const { t } = useTranslation()

  return (
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
      ) : isSearching && !hasSearchResults ? (
        <div className="flex flex-col items-center justify-center p-16 text-center">
          <div className="mb-4 rounded-full bg-primary/10 p-4 text-primary/70">
            <i className="hn hn-search text-3xl"></i>
          </div>
          <h3 className="text-lg font-semibold text-base-content">{t('repository.search.emptyTitle')}</h3>
          <p className="mt-3 max-w-xl text-sm leading-6 text-base-content/60">
            {t('repository.search.emptyDescription', { query: searchQueryDisplay })}
          </p>
          <button type="button" className="btn btn-primary mt-6" onClick={onClearSearch}>
            {t('repository.search.clear')}
          </button>
        </div>
      ) : (
        <div>
          {searchExpanded ? (
            <div className="flex flex-col gap-3 border-b border-[var(--border-subtle)] px-6 py-4 sm:flex-row sm:items-center sm:justify-between">
              <div>
                <h3 className="text-base font-bold text-base-content">{t('repository.search.tableTitle')}</h3>
                <p className="mt-1 text-sm text-base-content/60">
                  {t('repository.search.pageSummary', {
                    start: visibleRangeStart,
                    end: visibleRangeEnd,
                    total: paginatedTotal,
                  })}
                </p>
              </div>
              {pageCount > 1 ? (
                <div className="badge border-0 bg-base-200/80 px-3 py-3 text-base-content/65">
                  {t('repository.search.currentPage', { page: currentPage, total: pageCount })}
                </div>
              ) : null}
            </div>
          ) : null}

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
                {rows.map(({ item, highlights }) => {
                  const descriptionText = resolveDescription(item.description, t)
                  const sourceLabel = resolveSourceLabel(item.sourceType, item.sourceMarket, t)
                  const statusKey = resolveStatusKey(
                    item.securityLevel,
                    item.blocked,
                    item.riskOverrideApplied,
                  )

                  return (
                    <tr key={item.id} className="group transition-colors hover:bg-base-200/50">
                      <td className="py-4 pl-6 text-left">
                        <div className="flex items-start gap-3">
                          <div className="mt-1 flex h-8 w-8 shrink-0 items-center justify-center rounded bg-primary/10 text-primary">
                            <i className="hn hn-code-block text-base leading-none"></i>
                          </div>
                          <div className="min-w-0">
                            <HighlightedText
                              text={item.name}
                              ranges={highlights.name}
                              className="block font-bold text-base-content/90 transition-colors group-hover:text-primary"
                            />
                            <HighlightedText
                              text={descriptionText}
                              ranges={highlights.description}
                              className="mt-1 block line-clamp-2 text-sm leading-6 text-base-content/50"
                              highlightClassName="rounded-sm bg-primary/12 px-0.5 text-base-content"
                            />
                          </div>
                        </div>
                      </td>
                      <td className="text-center text-sm text-base-content/70">
                        <HighlightedText
                          text={sourceLabel}
                          ranges={highlights.source}
                          className="text-sm text-base-content/70"
                        />
                      </td>
                      <td className="text-center font-mono text-xs text-base-content/50">
                        {formatInstalledAt(item.installedAt, locale)}
                      </td>
                      <td className="text-center">
                        <span
                          className={`inline-flex whitespace-nowrap badge badge-sm gap-1 border-0 font-bold ${
                            statusKey === 'blocked'
                              ? 'bg-error/20 text-error'
                              : statusKey === 'overridden'
                                ? 'bg-warning/20 text-warning'
                                : statusKey === 'safe'
                                  ? 'bg-success/20 text-success'
                                  : statusKey === 'low'
                                    ? 'bg-success/10 text-success/80'
                                    : 'bg-warning/20 text-warning'
                          }`}
                        >
                          <i
                            className={`hn ${
                              statusKey === 'blocked'
                                ? 'hn-lock'
                                : statusKey === 'overridden'
                                  ? 'hn-shield'
                                  : statusKey === 'safe'
                                    ? 'hn-check-circle'
                                    : 'hn-exclaimation'
                            } text-xs`}
                          ></i>
                          {t(`repository.statusValues.${statusKey}`)}
                        </span>
                      </td>
                      <td className="pr-6 text-center">
                        <div className="flex justify-center gap-2 opacity-0 transition-opacity group-hover:opacity-100">
                          {item.canUpdate ? (
                            <button
                              className="btn btn-square btn-ghost btn-sm text-primary/80 hover:bg-primary/10 hover:text-primary"
                              onClick={() => onUpdateSkill(item.id)}
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
                            onClick={() => onLoadDetail(item.id)}
                            title={t('repository.view')}
                          >
                            <i className="hn hn-eye"></i>
                          </button>
                          <button
                            className="btn btn-square btn-ghost btn-sm text-error/70 hover:bg-error/10 hover:text-error"
                            onClick={() => onOpenDeletePreview(item.id)}
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
                  )
                })}
              </tbody>
            </table>
          </div>

          {searchExpanded && pageCount > 1 ? (
            <div className="flex flex-col gap-4 border-t border-[var(--border-subtle)] px-6 py-5 sm:flex-row sm:items-center sm:justify-between">
              <p className="text-sm text-base-content/60">
                {t('repository.search.pageSummary', {
                  start: visibleRangeStart,
                  end: visibleRangeEnd,
                  total: paginatedTotal,
                })}
              </p>
              <div className="join self-start sm:self-auto">
                <button
                  type="button"
                  className="btn btn-sm join-item border-[var(--border-subtle)] bg-base-100"
                  onClick={() => onChangePage(Math.max(1, currentPage - 1))}
                  disabled={currentPage <= 1}
                >
                  {t('repository.search.previous')}
                </button>
                {searchPageNumbers.map((pageNumber) => (
                  <button
                    key={pageNumber}
                    type="button"
                    className={`btn btn-sm join-item border-[var(--border-subtle)] ${
                      pageNumber === currentPage ? 'btn-primary' : 'bg-base-100 text-base-content/70'
                    }`}
                    onClick={() => onChangePage(pageNumber)}
                  >
                    {pageNumber}
                  </button>
                ))}
                <button
                  type="button"
                  className="btn btn-sm join-item border-[var(--border-subtle)] bg-base-100"
                  onClick={() => onChangePage(Math.min(pageCount, currentPage + 1))}
                  disabled={currentPage >= pageCount}
                >
                  {t('repository.search.next')}
                </button>
              </div>
            </div>
          ) : null}
        </div>
      )}
    </section>
  )
}
