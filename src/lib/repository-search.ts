export interface HighlightRange {
  start: number
  end: number
}

export interface RepositorySearchSourceItem {
  id: string
  name: string
  slug: string
  description?: string | null
  sourceLabel: string
  statusLabel: string
  keywords?: string[]
}

interface IndexedField {
  value: string
  normalized: string
  map: Array<{ start: number; end: number }>
}

export interface RepositorySearchIndexItem<TItem extends RepositorySearchSourceItem> {
  item: TItem
  fields: {
    name: IndexedField
    slug: IndexedField
    description: IndexedField
    source: IndexedField
    status: IndexedField
  }
  searchableText: string
}

export interface RepositorySearchMatch<TItem extends RepositorySearchSourceItem> {
  item: TItem
  score: number
  highlights: {
    name: HighlightRange[]
    description: HighlightRange[]
    source: HighlightRange[]
  }
}

export interface RepositorySearchPage<T> {
  items: T[]
  page: number
  pageCount: number
  pageSize: number
  total: number
  startIndex: number
  endIndex: number
}

const normalizeSearchValue = (value: string) =>
  value.normalize('NFKC').toLocaleLowerCase()

const normalizeSearchQuery = (value: string) =>
  normalizeSearchValue(value)
    .replace(/\s+/g, ' ')
    .trim()

const buildNormalizedMap = (value: string): IndexedField => {
  const map: Array<{ start: number; end: number }> = []
  let normalized = ''
  let offset = 0

  for (const character of value) {
    const start = offset
    offset += character.length
    const end = offset
    const normalizedCharacter = normalizeSearchValue(character)

    for (const normalizedUnit of normalizedCharacter) {
      normalized += normalizedUnit
      map.push({ start, end })
    }
  }

  return {
    value,
    normalized,
    map,
  }
}

const buildHighlightRange = (field: IndexedField, start: number, end: number): HighlightRange | null => {
  const first = field.map[start]
  const last = field.map[end - 1]
  if (!first || !last) {
    return null
  }

  return {
    start: first.start,
    end: last.end,
  }
}

const mergeHighlightRanges = (ranges: HighlightRange[]) => {
  if (ranges.length <= 1) {
    return ranges
  }

  const sorted = [...ranges].sort((left, right) => left.start - right.start || left.end - right.end)
  const merged: HighlightRange[] = [sorted[0]]

  for (const range of sorted.slice(1)) {
    const previous = merged[merged.length - 1]
    if (range.start <= previous.end) {
      previous.end = Math.max(previous.end, range.end)
      continue
    }

    merged.push({ ...range })
  }

  return merged
}

const tokenMatchesSubstring = (field: IndexedField, token: string) => {
  const index = field.normalized.indexOf(token)
  if (index === -1) {
    return null
  }

  return {
    score: 1,
    range: buildHighlightRange(field, index, index + token.length),
  }
}

const tokenMatchesSubsequence = (field: IndexedField, token: string) => {
  const positions: number[] = []
  let cursor = 0

  for (const character of token) {
    const position = field.normalized.indexOf(character, cursor)
    if (position === -1) {
      return null
    }
    positions.push(position)
    cursor = position + 1
  }

  const ranges = positions
    .map((position) => buildHighlightRange(field, position, position + 1))
    .filter((range): range is HighlightRange => Boolean(range))

  return {
    score: 0.35,
    ranges,
  }
}

const scoreFieldMatch = (
  field: IndexedField,
  token: string,
  weight: number,
) => {
  const substringMatch = tokenMatchesSubstring(field, token)
  if (substringMatch) {
    return {
      score: substringMatch.score * weight,
      ranges: substringMatch.range ? [substringMatch.range] : [],
      fuzzy: false,
    }
  }

  const subsequenceMatch = tokenMatchesSubsequence(field, token)
  if (subsequenceMatch) {
    return {
      score: subsequenceMatch.score * weight,
      ranges: subsequenceMatch.ranges,
      fuzzy: true,
    }
  }

  return null
}

const extractTokens = (query: string) => normalizeSearchQuery(query).split(' ').filter(Boolean)

export const buildRepositorySearchIndex = <TItem extends RepositorySearchSourceItem>(
  items: TItem[],
): RepositorySearchIndexItem<TItem>[] =>
  items.map((item) => {
    const description = item.description ?? ''
    const fields = {
      name: buildNormalizedMap(item.name),
      slug: buildNormalizedMap(item.slug),
      description: buildNormalizedMap(description),
      source: buildNormalizedMap(item.sourceLabel),
      status: buildNormalizedMap(item.statusLabel),
    }

    return {
      item,
      fields,
      searchableText: normalizeSearchQuery(
        [
          item.name,
          item.slug,
          description,
          item.sourceLabel,
          item.statusLabel,
          ...(item.keywords ?? []),
        ].join(' '),
      ),
    }
  })

export const searchRepositoryIndex = <TItem extends RepositorySearchSourceItem>(
  index: RepositorySearchIndexItem<TItem>[],
  query: string,
): RepositorySearchMatch<TItem>[] => {
  const tokens = extractTokens(query)

  if (tokens.length === 0) {
    return index.map((entry) => ({
      item: entry.item,
      score: 0,
      highlights: {
        name: [],
        description: [],
        source: [],
      },
    }))
  }

  return index
    .flatMap((entry) => {
      const highlightBuckets = {
        name: [] as HighlightRange[],
        description: [] as HighlightRange[],
        source: [] as HighlightRange[],
      }
      let score = 0

      for (const token of tokens) {
        if (!entry.searchableText.includes(token)) {
          const hasSubsequenceFallback = [
            entry.fields.name,
            entry.fields.slug,
            entry.fields.description,
            entry.fields.source,
            entry.fields.status,
          ].some((field) => tokenMatchesSubsequence(field, token))

          if (!hasSubsequenceFallback) {
            return []
          }
        }

        const rankedMatches = [
          { key: 'name', match: scoreFieldMatch(entry.fields.name, token, 5) },
          { key: 'slug', match: scoreFieldMatch(entry.fields.slug, token, 4) },
          { key: 'description', match: scoreFieldMatch(entry.fields.description, token, 3) },
          { key: 'source', match: scoreFieldMatch(entry.fields.source, token, 2) },
          { key: 'status', match: scoreFieldMatch(entry.fields.status, token, 1) },
        ]
          .filter(
            (
              value,
            ): value is {
              key: 'name' | 'slug' | 'description' | 'source' | 'status'
              match: NonNullable<ReturnType<typeof scoreFieldMatch>>
            } => Boolean(value.match),
          )
          .sort((left, right) => right.match.score - left.match.score)

        const bestMatch = rankedMatches[0]
        if (!bestMatch) {
          return []
        }

        score += bestMatch.match.score

        if (bestMatch.key === 'name') {
          highlightBuckets.name.push(...bestMatch.match.ranges)
        } else if (bestMatch.key === 'description') {
          highlightBuckets.description.push(...bestMatch.match.ranges)
        } else if (bestMatch.key === 'source') {
          highlightBuckets.source.push(...bestMatch.match.ranges)
        }
      }

      return [
        {
          item: entry.item,
          score,
          highlights: {
            name: mergeHighlightRanges(highlightBuckets.name),
            description: mergeHighlightRanges(highlightBuckets.description),
            source: mergeHighlightRanges(highlightBuckets.source),
          },
        },
      ]
    })
    .sort((left, right) => right.score - left.score || left.item.name.localeCompare(right.item.name))
}

export const paginateRepositorySearchResults = <T>(
  results: T[],
  page: number,
  pageSize: number,
): RepositorySearchPage<T> => {
  const safePageSize = Math.max(1, pageSize)
  const total = results.length
  const pageCount = Math.max(1, Math.ceil(total / safePageSize))
  const safePage = Math.min(Math.max(1, page), pageCount)
  const startIndex = total === 0 ? 0 : (safePage - 1) * safePageSize
  const endIndex = total === 0 ? 0 : Math.min(total, startIndex + safePageSize)

  return {
    items: results.slice(startIndex, endIndex),
    page: safePage,
    pageCount,
    pageSize: safePageSize,
    total,
    startIndex,
    endIndex,
  }
}
