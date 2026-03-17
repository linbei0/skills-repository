# Repository Search Design

## Goal

Add a complete, responsive search experience to the repository page with:

- realtime filtering
- fuzzy matching
- keyword highlighting
- result count
- pagination
- friendly empty states
- bilingual Chinese and English matching support

## Constraints

- Keep the current backend contract unchanged for this task.
- Search only against fields already available in `RepositorySkillSummary`.
- "Language" and "tags" are not structured fields in the current repository payload, so this iteration can only match those concepts when they appear in existing text fields such as `name`, `slug`, or `description`.

## Recommended Approach

Create a dedicated frontend search adapter instead of embedding all logic in `RepositoryPage.tsx`.

### Why

- keeps the page component smaller
- makes search behavior testable as pure functions
- allows later expansion to structured filters without rewriting the page

## Search Model

Each repository item will be converted into a lightweight search document:

- original item
- display-ready source label
- display-ready status label
- normalized searchable text blob
- field-level normalized text for `name`, `slug`, `description`, and `source`

Normalization rules:

- use `NFKC` normalization
- lowercase latin text
- collapse repeated whitespace
- trim leading and trailing whitespace

This supports Chinese text directly and English text case-insensitively.

## Matching Strategy

Use token-based matching:

1. split query by whitespace
2. ignore empty tokens
3. require every token to match somewhere in the document
4. prefer direct substring matches
5. allow ordered subsequence matching as fuzzy fallback

Scoring priority:

1. exact or substring match in `name`
2. match in `slug`
3. match in `description`
4. match in `source` or status text
5. fuzzy subsequence fallback

## Highlighting

Highlight matched substrings in visible fields:

- repository name
- description
- source label

When a token only matches through fuzzy subsequence fallback, the item still remains visible even if no clean substring highlight is available.

## Pagination

- page size: 10
- reset to page 1 whenever query changes
- show current visible range and total result count
- provide previous/next buttons and numbered page buttons when needed

## UX Layout

Place a prominent search block between the header card and the repository table.

The block includes:

- large search input with icon
- helper text for searchable fields
- realtime result count
- clear button
- active query summary when searching

Empty states:

- repository truly empty: keep existing empty state
- search has no matches: show dedicated no-result panel with query echo and clear action

## Performance Plan

- precompute search documents with `useMemo`
- evaluate the query against the memoized documents only
- use lightweight debounced input state
- use `useDeferredValue` so typing stays responsive
- paginate after filtering and sorting

For the current repository-page data size, this should stay well under the requested 500 ms target on normal desktop usage.

## Testing Plan

Add unit tests for the pure search adapter covering:

- Chinese matching
- English case-insensitive matching
- multi-keyword matching
- fuzzy subsequence fallback
- highlight range extraction
- pagination boundaries

Then run project verification:

- `corepack pnpm lint`
- `corepack pnpm typecheck`
- `corepack pnpm build`

