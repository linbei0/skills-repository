import { useTranslation } from 'react-i18next'

interface RepositoryPageHeaderProps {
  loading: boolean
  loaded: boolean
  hasItems: boolean
  searchExpanded: boolean
  searchQuery: string
  isSearching: boolean
  paginatedTotal: number
  visibleRangeStart: number
  visibleRangeEnd: number
  updatableCount: number
  batchUpdating: boolean
  updatingSkillId: string | null
  onToggleSearch: () => void
  onSearchQueryChange: (value: string) => void
  onClearSearch: () => void
  onUpdateGithubSkills: () => void
  onOpenImportModal: () => void
  onOpenDistribution: () => void
}

export function RepositoryPageHeader({
  loading,
  loaded,
  hasItems,
  searchExpanded,
  searchQuery,
  isSearching,
  paginatedTotal,
  visibleRangeStart,
  visibleRangeEnd,
  updatableCount,
  batchUpdating,
  updatingSkillId,
  onToggleSearch,
  onSearchQueryChange,
  onClearSearch,
  onUpdateGithubSkills,
  onOpenImportModal,
  onOpenDistribution,
}: RepositoryPageHeaderProps) {
  const { t } = useTranslation()
  const headerActionButtonClass =
    'btn btn-square h-12 w-12 min-h-[3rem] rounded-xl border text-base transition-all duration-200'
  const headerOutlineButtonClass = `${headerActionButtonClass} border-[var(--border-subtle)] bg-base-100 text-base-content/78 hover:border-primary hover:bg-primary/10 hover:text-primary`
  const headerPrimaryButtonClass = `${headerActionButtonClass} border-none bg-primary text-[var(--text-inverse)] hover:bg-primary hover:shadow-[var(--shadow-neon-primary)]`
  const searchTooltip = searchExpanded ? t('repository.search.close') : t('repository.search.open')
  const searchDisabled = loading || (loaded && !hasItems)

  return (
    <section className="relative overflow-hidden rounded-lg border border-[var(--border-subtle)] bg-base-100 p-8 shadow-[inset_0_0_20px_rgba(var(--color-primary),0.05)]">
      <div className="absolute top-0 left-0 h-1 w-full bg-gradient-to-r from-primary/0 via-primary/50 to-primary/0 opacity-20"></div>
      <div className="flex flex-col gap-6 md:flex-row md:items-center md:justify-between">
        <div className="flex-1">
          <h2 className="text-3xl font-bold tracking-tight text-base-content">{t('repository.title')}</h2>
          <p className="mt-3 max-w-3xl text-sm leading-relaxed text-base-content/70">
            {t('repository.description')}
          </p>
        </div>
        <div className="flex flex-wrap items-center gap-3 md:justify-end">
          <div className="flex flex-wrap items-center gap-2 rounded-2xl border border-[var(--border-subtle)] bg-base-200/25 p-2 shadow-[inset_0_0_16px_rgba(var(--color-primary),0.02)]">
            <div className="tooltip tooltip-bottom" data-tip={searchTooltip}>
              <button
                type="button"
                className={
                  searchExpanded
                    ? `${headerActionButtonClass} border border-primary/30 bg-primary/10 text-primary hover:bg-primary/14`
                    : headerOutlineButtonClass
                }
                onClick={onToggleSearch}
                disabled={searchDisabled}
                aria-label={searchTooltip}
              >
                <i className="hn hn-search text-lg"></i>
              </button>
            </div>

            <div className="tooltip tooltip-bottom" data-tip={t('repository.update.open', { count: updatableCount })}>
              <button
                className={headerOutlineButtonClass}
                disabled={updatableCount === 0 || batchUpdating || Boolean(updatingSkillId)}
                onClick={onUpdateGithubSkills}
                aria-label={t('repository.update.open', { count: updatableCount })}
              >
                {batchUpdating ? (
                  <span className="loading loading-spinner loading-sm"></span>
                ) : (
                  <i className="hn hn-refresh text-lg"></i>
                )}
              </button>
            </div>

            <div className="tooltip tooltip-bottom" data-tip={t('repository.import.open')}>
              <button
                className={headerPrimaryButtonClass}
                onClick={onOpenImportModal}
                aria-label={t('repository.import.open')}
              >
                <i className="hn hn-download-alt text-lg"></i>
              </button>
            </div>

            <div className="tooltip tooltip-bottom" data-tip={t('repository.distribute.open')}>
              <button
                className={headerOutlineButtonClass}
                disabled={!hasItems}
                onClick={onOpenDistribution}
                aria-label={t('repository.distribute.open')}
              >
                <i className="hn hn-share text-lg"></i>
              </button>
            </div>
          </div>
        </div>
      </div>
      {searchExpanded ? (
        <div className="mt-6 rounded-xl border border-[var(--border-subtle)] bg-base-200/25 p-4 shadow-[inset_0_0_18px_rgba(var(--color-primary),0.02)]">
          <div className="flex flex-col gap-3">
            <div className="flex flex-col gap-3 lg:flex-row lg:items-center">
              <label className="input input-bordered flex h-12 flex-1 items-center gap-3 border-[var(--border-subtle)] bg-base-100 px-4 focus-within:border-primary focus-within:shadow-[0_0_0_3px_rgba(var(--color-primary),0.08)]">
                <i className="hn hn-search text-base text-primary" aria-hidden />
                <input
                  type="text"
                  className="grow bg-transparent text-sm text-base-content outline-none"
                  value={searchQuery}
                  onChange={(event) => onSearchQueryChange(event.target.value)}
                  placeholder={t('repository.search.placeholder')}
                  aria-label={t('repository.search.placeholder')}
                  disabled={searchDisabled}
                />
                {searchQuery ? (
                  <button
                    type="button"
                    className="btn btn-circle btn-ghost btn-xs text-base-content/50 hover:bg-base-content/10 hover:text-base-content"
                    onClick={onClearSearch}
                    title={t('repository.search.clear')}
                  >
                    <i className="hn hn-times text-xs"></i>
                  </button>
                ) : null}
              </label>

              {isSearching ? (
                <button
                  type="button"
                  className="btn btn-ghost h-12 border border-[var(--border-subtle)] px-4 text-sm text-base-content/65 hover:border-primary hover:bg-primary/5 hover:text-primary"
                  onClick={onClearSearch}
                >
                  {t('repository.search.clear')}
                </button>
              ) : null}
            </div>

            {isSearching ? (
              <div className="flex flex-wrap items-center gap-2 text-xs">
                <span className="badge border-0 bg-primary/10 text-primary">
                  {t('repository.search.resultsCount', { count: paginatedTotal })}
                </span>
                <span className="text-base-content/50">
                  {t('repository.search.pageSummary', {
                    start: visibleRangeStart,
                    end: visibleRangeEnd,
                    total: paginatedTotal,
                  })}
                </span>
                <span className="text-base-content/45">
                  {t('repository.search.activeQuery', { query: searchQuery.trim() })}
                </span>
              </div>
            ) : null}
          </div>
        </div>
      ) : null}
    </section>
  )
}
