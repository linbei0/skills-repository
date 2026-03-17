import type { ReactNode } from 'react'
import type { HighlightRange } from '../../lib/repository-search'

interface HighlightedTextProps {
  text: string
  ranges?: HighlightRange[]
  className?: string
  highlightClassName?: string
}

const mergeRanges = (ranges: HighlightRange[]) => {
  if (ranges.length <= 1) {
    return ranges
  }

  const sorted = [...ranges].sort((left, right) => left.start - right.start || left.end - right.end)
  const merged: HighlightRange[] = [{ ...sorted[0] }]

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

export function HighlightedText({
  text,
  ranges = [],
  className,
  highlightClassName = 'rounded-sm bg-primary/15 px-0.5 text-primary',
}: HighlightedTextProps) {
  const mergedRanges = mergeRanges(
    ranges.filter((range) => range.start >= 0 && range.end > range.start && range.end <= text.length),
  )

  if (mergedRanges.length === 0) {
    return <span className={className}>{text}</span>
  }

  const nodes: ReactNode[] = []
  let cursor = 0

  mergedRanges.forEach((range, index) => {
    if (cursor < range.start) {
      nodes.push(
        <span key={`plain-${index}-${cursor}`}>{text.slice(cursor, range.start)}</span>,
      )
    }

    nodes.push(
      <mark key={`highlight-${index}-${range.start}`} className={highlightClassName}>
        {text.slice(range.start, range.end)}
      </mark>,
    )

    cursor = range.end
  })

  if (cursor < text.length) {
    nodes.push(<span key={`plain-tail-${cursor}`}>{text.slice(cursor)}</span>)
  }

  return <span className={className}>{nodes}</span>
}
