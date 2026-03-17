# Repository Search Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a full repository-page search experience with realtime filtering, fuzzy matching, highlight rendering, result counts, pagination, and no-result feedback.

**Architecture:** Extract repository search into a pure utility module that normalizes text, scores matches, and produces highlight metadata. Keep `RepositoryPage.tsx` focused on state wiring and rendering. Add a small presentational highlighter component for consistent UI output.

**Tech Stack:** React 19, TypeScript, Zustand, i18next, Vite, Tailwind CSS, daisyUI, Vitest

---

### Task 1: Add test support for the pure search adapter

**Files:**
- Modify: `package.json`
- Modify: `pnpm-lock.yaml`

**Step 1: Add the failing test toolchain dependency**

- Add `vitest` as a dev dependency.

**Step 2: Add the test script**

- Add `"test": "vitest run"` to `package.json`.

**Step 3: Run the test command before tests exist**

Run: `corepack pnpm test`
Expected: no matching tests or failing baseline.

### Task 2: Write failing unit tests for repository search behavior

**Files:**
- Create: `src/lib/repository-search.test.ts`

**Step 1: Write tests for normalization and matching**

- Cover Chinese query match.
- Cover English case-insensitive match.
- Cover multi-token all-match behavior.
- Cover fuzzy subsequence fallback.

**Step 2: Write tests for highlight and pagination**

- Cover highlight extraction.
- Cover page slicing and page count.

**Step 3: Run the tests to verify failure**

Run: `corepack pnpm test`
Expected: failing imports or failing assertions because the implementation does not exist yet.

### Task 3: Implement the pure repository search adapter

**Files:**
- Create: `src/lib/repository-search.ts`

**Step 1: Add types and normalization helpers**

- Search document type
- match result type
- normalization helper
- token extraction helper

**Step 2: Add scoring and fuzzy match logic**

- field-level substring match
- subsequence fallback
- score ordering

**Step 3: Add highlight metadata generation**

- produce visible field highlight ranges

**Step 4: Add pagination helper**

- compute page count
- clamp active page
- slice page items

**Step 5: Run the tests to verify they pass**

Run: `corepack pnpm test`
Expected: passing search adapter tests.

### Task 4: Add a reusable highlighter renderer

**Files:**
- Create: `src/components/common/HighlightedText.tsx`

**Step 1: Render plain text with merged highlight ranges**

- accept text and ranges
- merge overlapping ranges
- render semantic `<mark>` nodes

**Step 2: Keep styling compatible with the current repository table design**

- subtle highlight background
- readable foreground in dark theme

### Task 5: Integrate search UX into the repository page

**Files:**
- Modify: `src/pages/RepositoryPage.tsx`

**Step 1: Add local search state**

- raw query
- debounced query
- active page
- memoized search dataset

**Step 2: Replace direct `items.map(...)` rendering with paginated search results**

- show full list when query is empty
- show result count
- show current range

**Step 3: Add the top search panel**

- prominent input
- clear action
- helper copy
- active-search summary

**Step 4: Render highlight output**

- name
- description
- source label

**Step 5: Add no-result and pagination UI**

- empty-search panel
- previous/next buttons
- page number buttons

**Step 6: Run tests after integration**

Run: `corepack pnpm test`
Expected: still green.

### Task 6: Add translation strings

**Files:**
- Modify: `src/locales/zh-CN/common.json`
- Modify: `src/locales/en-US/common.json`
- Modify: `src/locales/ja-JP/common.json`

**Step 1: Add repository search strings**

- placeholder
- helper
- result count
- page summary
- no result title/body
- clear action
- pagination labels

**Step 2: Run typecheck and build-oriented validation**

Run: `corepack pnpm typecheck`
Expected: pass.

### Task 7: Verify the whole feature before claiming completion

**Files:**
- Modify: `src/pages/RepositoryPage.tsx`
- Modify: `src/lib/repository-search.ts`
- Modify: `src/components/common/HighlightedText.tsx`
- Modify: `src/locales/zh-CN/common.json`
- Modify: `src/locales/en-US/common.json`
- Modify: `src/locales/ja-JP/common.json`
- Modify: `package.json`
- Modify: `pnpm-lock.yaml`

**Step 1: Run lint**

Run: `corepack pnpm lint`
Expected: pass.

**Step 2: Run typecheck**

Run: `corepack pnpm typecheck`
Expected: pass.

**Step 3: Run build**

Run: `corepack pnpm build`
Expected: pass.

**Step 4: Report the result with command evidence**

- Include which files changed
- Include the verification commands run
- Include any remaining caveat about `language` and `tags` being limited by current backend data

