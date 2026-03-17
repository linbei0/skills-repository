import { describe, expect, it } from 'vitest'
import {
  buildRepositorySearchIndex,
  paginateRepositorySearchResults,
  searchRepositoryIndex,
} from './repository-search'

const fixtures = buildRepositorySearchIndex([
  {
    id: '1',
    name: '中文技能助手',
    slug: 'zh-helper',
    description: '支持中文和 English prompts',
    sourceLabel: 'GitHub Import',
    statusLabel: 'Safe',
  },
  {
    id: '2',
    name: 'React Workflow',
    slug: 'react-workflow',
    description: 'Frontend helper for UI systems',
    sourceLabel: 'Local Import',
    statusLabel: 'Low Risk',
  },
  {
    id: '3',
    name: 'writing-plans',
    slug: 'writing-plans',
    description: 'Use when you have a spec or requirements',
    sourceLabel: 'GitHub Import',
    statusLabel: 'Safe',
  },
])

describe('repository search', () => {
  it('matches Chinese queries', () => {
    const results = searchRepositoryIndex(fixtures, '中文')

    expect(results).toHaveLength(1)
    expect(results[0]?.item.id).toBe('1')
  })

  it('matches English queries case-insensitively', () => {
    const results = searchRepositoryIndex(fixtures, 'react')

    expect(results).toHaveLength(1)
    expect(results[0]?.item.id).toBe('2')
  })

  it('requires all keywords to match somewhere in the document', () => {
    const results = searchRepositoryIndex(fixtures, 'english prompts')

    expect(results).toHaveLength(1)
    expect(results[0]?.item.id).toBe('1')
  })

  it('falls back to fuzzy subsequence matching', () => {
    const results = searchRepositoryIndex(fixtures, 'wrpl')

    expect(results).toHaveLength(1)
    expect(results[0]?.item.id).toBe('3')
  })

  it('returns highlight ranges for direct substring matches', () => {
    const results = searchRepositoryIndex(fixtures, 'react')

    expect(results[0]?.highlights.name).toEqual([{ start: 0, end: 5 }])
  })

  it('paginates filtered results with clamped page numbers', () => {
    const results = searchRepositoryIndex(fixtures, '')
    const page = paginateRepositorySearchResults(results, 3, 2)

    expect(page.total).toBe(3)
    expect(page.pageCount).toBe(2)
    expect(page.page).toBe(2)
    expect(page.items).toHaveLength(1)
    expect(page.items[0]?.item.id).toBe('3')
  })
})
