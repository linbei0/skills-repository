import type {
  BatchRepositorySkillUpdateResult,
  RepositorySkillSummary,
  RepositorySkillUpdateItemResult,
} from '../../types/app'

export type TranslateFn = (key: string, options?: Record<string, unknown>) => string

export const REPOSITORY_SEARCH_PAGE_SIZE = 10
export const REPOSITORY_SEARCH_DEBOUNCE_MS = 120

export const formatInstalledAt = (value: number, locale: string) =>
  new Intl.DateTimeFormat(locale, {
    year: 'numeric',
    month: 'long',
    day: 'numeric',
  }).format(new Date(value * 1000))

export const resolveSourceLabel = (
  sourceType: string,
  sourceMarket: string | null | undefined,
  t: TranslateFn,
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

export const resolveStatusKey = (
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

export const resolveDescription = (value: string | null | undefined, t: TranslateFn) =>
  value?.trim() ? value : t('repository.descriptionMissing')

export const logSourceOpenFailure = (error: unknown) => {
  console.error('Failed to open source reference:', error)
}

export const shouldShowSingleUpdateFeedback = (
  result: RepositorySkillUpdateItemResult | null,
) => Boolean(result && (result.status !== 'updated' || result.copyDistributionCount > 0))

export const batchResultCount = (result: BatchRepositorySkillUpdateResult | null) =>
  result ? result.updated.length + result.skipped.length + result.failed.length : 0

const asRecord = (value: Record<string, unknown> | null | undefined) =>
  value && typeof value === 'object' ? value : null

const detailString = (details: Record<string, unknown> | null | undefined, key: string) => {
  const value = asRecord(details)?.[key]
  return typeof value === 'string' && value.trim() ? value : null
}

export const formatUpdateMessage = (
  result: RepositorySkillUpdateItemResult,
  t: TranslateFn,
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

export const buildPaginationWindow = (page: number, pageCount: number) => {
  if (pageCount <= 1) {
    return []
  }

  const end = Math.min(pageCount, Math.max(5, page + 2))
  const start = Math.max(1, end - 4)

  return Array.from({ length: end - start + 1 }, (_, index) => start + index)
}

export const buildRepositorySearchKeywords = (sourceType: string, statusKey: string) => {
  const sourceKeywords =
    sourceType === 'github'
      ? ['github', 'github import', 'github 导入']
      : sourceType === 'local'
        ? ['local', 'local import', '本地', '本地导入']
        : sourceType === 'market'
          ? ['market', 'marketplace', '市场']
          : ['unknown', '未知']

  const statusKeywords =
    statusKey === 'safe'
      ? ['safe', 'security', '安全']
      : statusKey === 'low'
        ? ['low risk', 'low', '低风险']
        : statusKey === 'medium'
          ? ['medium risk', 'medium', '中风险']
          : statusKey === 'blocked'
            ? ['blocked', 'high risk', '阻断', '高风险']
            : statusKey === 'overridden'
              ? ['override', 'overridden', 'risk override', '忽略风险']
              : ['unknown', '未知']

  return [...sourceKeywords, ...statusKeywords]
}

export const buildPlainRepositoryRows = (items: RepositorySkillSummary[]) =>
  items.map((item) => ({
    item,
    highlights: { name: [], description: [], source: [] },
  }))
