interface ProjectDistributionResultItem {
  skillId: string
  skillName: string
  targetPath: string
  reason?: string | null
}

interface ProjectDistributionResultLike {
  installed: ProjectDistributionResultItem[]
  skipped: ProjectDistributionResultItem[]
  failed: ProjectDistributionResultItem[]
}

interface ProjectDistributionResultPanelProps {
  result: ProjectDistributionResultLike | null
  titleInstalled: string
  titleSkipped: string
  titleFailed: string
}

export function ProjectDistributionResultPanel({
  result,
  titleInstalled,
  titleSkipped,
  titleFailed,
}: ProjectDistributionResultPanelProps) {
  if (!result) return null

  return (
    <section className="rounded-box border border-base-300 bg-base-200/50 p-4">
      <div className="flex flex-wrap gap-4 text-sm">
        <span>{titleInstalled}</span>
        <span>{titleSkipped}</span>
        <span>{titleFailed}</span>
      </div>
      <div className="mt-4 space-y-3">
        {[...result.installed, ...result.skipped, ...result.failed].map((item) => (
          <article
            key={`${item.skillId}-${item.targetPath}`}
            className="rounded-box border border-base-300 bg-base-100 p-3 text-sm"
          >
            <p className="font-medium">{item.skillName}</p>
            <p className="mt-1 break-all text-xs text-base-content/55">{item.targetPath}</p>
            {item.reason ? <p className="mt-2 text-xs text-warning">{item.reason}</p> : null}
          </article>
        ))}
      </div>
    </section>
  )
}
