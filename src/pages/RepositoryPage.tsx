import { useDeferredValue, useEffect, useMemo, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { RepositoryDistributeModal } from '../components/RepositoryDistributeModal'
import { RepositoryImportModal } from '../components/RepositoryImportModal'
import {
  buildRepositorySearchIndex,
  paginateRepositorySearchResults,
  searchRepositoryIndex,
} from '../lib/repository-search'
import { resolveSkillsTargets } from '../lib/skills-targets'
import { useAppStore } from '../stores/use-app-store'
import { useRepositoryStore } from '../stores/use-repository-store'
import { useSettingsStore } from '../stores/use-settings-store'
import type {
  BatchDistributeRepositorySkillsRequest,
  ImportRepositorySkillRequest,
  RepositoryImportSourceKind,
} from '../types/app'
import { RepositoryDeletePreviewModal } from './repository/RepositoryDeletePreviewModal'
import { RepositoryDetailModal } from './repository/RepositoryDetailModal'
import { RepositoryPageHeader } from './repository/RepositoryPageHeader'
import { RepositorySkillsSection } from './repository/RepositorySkillsSection'
import { RepositoryUpdateFeedback } from './repository/RepositoryUpdateFeedback'
import {
  buildPaginationWindow,
  buildPlainRepositoryRows,
  buildRepositorySearchKeywords,
  REPOSITORY_SEARCH_DEBOUNCE_MS,
  REPOSITORY_SEARCH_PAGE_SIZE,
  resolveDescription,
  resolveSourceLabel,
  resolveStatusKey,
} from './repository/repository-page-helpers'

export function RepositoryPage() {
  const { t, i18n } = useTranslation()
  const [importOpen, setImportOpen] = useState(false)
  const [searchExpanded, setSearchExpanded] = useState(false)
  const [searchQuery, setSearchQuery] = useState('')
  const [debouncedSearchQuery, setDebouncedSearchQuery] = useState('')
  const [currentPage, setCurrentPage] = useState(1)
  const deferredSearchQuery = useDeferredValue(debouncedSearchQuery)

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

  useEffect(() => {
    const timer = window.setTimeout(() => {
      setDebouncedSearchQuery(searchQuery)
    }, REPOSITORY_SEARCH_DEBOUNCE_MS)

    return () => window.clearTimeout(timer)
  }, [searchQuery])

  const visibleTargets = resolveSkillsTargets(builtinSkillsTargets, settings).filter((target) =>
    settings.visibleSkillsTargetIds.includes(target.id),
  )
  const updatableCount = items.filter((item) => item.canUpdate).length

  const searchIndex = useMemo(
    () =>
      buildRepositorySearchIndex(
        items.map((item) => {
          const statusKey = resolveStatusKey(
            item.securityLevel,
            item.blocked,
            item.riskOverrideApplied,
          )

          return {
            ...item,
            description: resolveDescription(item.description, t),
            sourceLabel: resolveSourceLabel(item.sourceType, item.sourceMarket, t),
            statusLabel: t(`repository.statusValues.${statusKey}`),
            keywords: buildRepositorySearchKeywords(item.sourceType, statusKey),
          }
        }),
      ),
    [items, t],
  )

  const activeSearchQuery = searchExpanded ? deferredSearchQuery : ''
  const searchResults = useMemo(
    () => searchRepositoryIndex(searchIndex, activeSearchQuery),
    [searchIndex, activeSearchQuery],
  )
  const paginatedSearchResults = useMemo(
    () => paginateRepositorySearchResults(searchResults, currentPage, REPOSITORY_SEARCH_PAGE_SIZE),
    [currentPage, searchResults],
  )
  const searchPageNumbers = useMemo(
    () => buildPaginationWindow(paginatedSearchResults.page, paginatedSearchResults.pageCount),
    [paginatedSearchResults.page, paginatedSearchResults.pageCount],
  )

  const isSearching = searchExpanded && activeSearchQuery.trim().length > 0
  const hasSearchResults = paginatedSearchResults.total > 0
  const visibleRangeStart =
    paginatedSearchResults.total === 0 ? 0 : paginatedSearchResults.startIndex + 1
  const rows = searchExpanded ? paginatedSearchResults.items : buildPlainRepositoryRows(items)

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

  const handleSearchQueryChange = (value: string) => {
    setSearchQuery(value)
    setCurrentPage(1)
  }

  const handleClearSearch = () => {
    setSearchQuery('')
    setCurrentPage(1)
  }

  const handleResolveImport = async (sourceKind: RepositoryImportSourceKind, input: string) => {
    await resolveImport({ sourceKind, input })
  }

  const handleImportSkill = (request: ImportRepositorySkillRequest) => importSkill(request)
  const handleBatchDistributeSkills = async (request: BatchDistributeRepositorySkillsRequest) => {
    await batchDistributeSkills(request)
  }

  return (
    <div className="space-y-8 p-8">
      <RepositoryPageHeader
        loading={loading}
        loaded={loaded}
        hasItems={items.length > 0}
        searchExpanded={searchExpanded}
        searchQuery={searchQuery}
        isSearching={isSearching}
        paginatedTotal={paginatedSearchResults.total}
        visibleRangeStart={visibleRangeStart}
        visibleRangeEnd={paginatedSearchResults.endIndex}
        updatableCount={updatableCount}
        batchUpdating={batchUpdating}
        updatingSkillId={updatingSkillId}
        onToggleSearch={() => setSearchExpanded((value) => !value)}
        onSearchQueryChange={handleSearchQueryChange}
        onClearSearch={handleClearSearch}
        onUpdateGithubSkills={() => void updateGithubSkills()}
        onOpenImportModal={openImportModal}
        onOpenDistribution={handleOpenDistribution}
      />

      <RepositoryUpdateFeedback
        updateError={updateError}
        lastUpdateResult={lastUpdateResult}
        lastBatchUpdateResult={lastBatchUpdateResult}
        onClear={clearUpdateState}
      />

      <RepositorySkillsSection
        loading={loading}
        loaded={loaded}
        error={error}
        items={items}
        rows={rows}
        searchExpanded={searchExpanded}
        isSearching={isSearching}
        hasSearchResults={hasSearchResults}
        searchQueryDisplay={activeSearchQuery.trim()}
        visibleRangeStart={visibleRangeStart}
        visibleRangeEnd={paginatedSearchResults.endIndex}
        paginatedTotal={paginatedSearchResults.total}
        currentPage={paginatedSearchResults.page}
        pageCount={paginatedSearchResults.pageCount}
        searchPageNumbers={searchPageNumbers}
        locale={i18n.language}
        batchUpdating={batchUpdating}
        updatingSkillId={updatingSkillId}
        uninstallingSkillId={uninstallingSkillId}
        onClearSearch={handleClearSearch}
        onUpdateSkill={(skillId) => void updateSkill(skillId)}
        onLoadDetail={(skillId) => void loadDetail(skillId)}
        onOpenDeletePreview={(skillId) => void loadDeletePreview(skillId)}
        onChangePage={setCurrentPage}
      />

      <RepositoryDetailModal
        selectedDetail={selectedDetail}
        detailLoading={detailLoading}
        detailError={detailError}
        locale={i18n.language}
        batchUpdating={batchUpdating}
        updatingSkillId={updatingSkillId}
        onClose={closeDetail}
        onUpdateSkill={(skillId) => void updateSkill(skillId)}
      />

      <RepositoryDeletePreviewModal
        deletePreview={deletePreview}
        deletePreviewLoading={deletePreviewLoading}
        deletePreviewError={deletePreviewError}
        uninstallingSkillId={uninstallingSkillId}
        onClose={clearDeletePreview}
        onUninstall={(skillId) => void uninstall(skillId)}
      />

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
