import type { ReactNode } from 'react'
import type { SkillsTargetOption } from '../lib/skills-targets'

interface ProjectDistributionFieldsProps {
  projectRoot: string
  showProjectRoot?: boolean
  targetType: 'tag' | 'custom'
  targetAgentId: string
  customRelativePath: string
  installMode: 'symlink' | 'copy'
  targets: SkillsTargetOption[]
  resolvedTargetPath: string
  titleTarget: string
  titleMode: string
  titlePreview: string
  labelProjectRoot: string
  labelProjectPicker: string
  labelChooseDirectory: string
  labelTargetTag: string
  labelTargetCustom: string
  labelCustomRelativePath: string
  placeholderProjectRoot: string
  placeholderCustomRelativePath: string
  noTargetPreviewText: string
  previewMeta?: ReactNode
  onProjectRootChange: (value: string) => void
  onTargetTypeChange: (value: 'tag' | 'custom') => void
  onTargetAgentIdChange: (value: string) => void
  onCustomRelativePathChange: (value: string) => void
  onInstallModeChange: (value: 'symlink' | 'copy') => void
  onChooseProjectDirectory: () => void
  renderModeLabel: (mode: 'symlink' | 'copy') => string
}

export function ProjectDistributionFields({
  projectRoot,
  showProjectRoot = true,
  targetType,
  targetAgentId,
  customRelativePath,
  installMode,
  targets,
  resolvedTargetPath,
  titleTarget,
  titleMode,
  titlePreview,
  labelProjectRoot,
  labelProjectPicker,
  labelChooseDirectory,
  labelTargetTag,
  labelTargetCustom,
  labelCustomRelativePath,
  placeholderProjectRoot,
  placeholderCustomRelativePath,
  noTargetPreviewText,
  previewMeta,
  onProjectRootChange,
  onTargetTypeChange,
  onTargetAgentIdChange,
  onCustomRelativePathChange,
  onInstallModeChange,
  onChooseProjectDirectory,
  renderModeLabel,
}: ProjectDistributionFieldsProps) {
  return (
    <>
      {showProjectRoot ? (
        <section className="grid gap-4 md:grid-cols-[1fr_auto] items-end">
          <label className="form-control w-full">
            <span className="label-text">{labelProjectRoot}</span>
            <input
              className="input input-bordered"
              value={projectRoot}
              onChange={(event) => onProjectRootChange(event.target.value)}
              placeholder={placeholderProjectRoot}
            />
          </label>
          <div className="form-control">
            <span className="label-text mb-2 block opacity-0 select-none">.</span>
            <button 
              className="btn h-12 min-h-[3rem] border-[var(--border-subtle)] bg-base-200/50 text-base-content hover:bg-base-200 hover:border-[var(--border-subtle)]" 
              onClick={onChooseProjectDirectory}
            >
              <i className="hn hn-folder-open mr-2"></i>
              {labelChooseDirectory}
            </button>
          </div>
        </section>
      ) : null}

      <section className="rounded-box border border-base-300 bg-base-200/50 p-4">
        <h4 className="font-semibold">{titleTarget}</h4>
        <div className="mt-4 flex flex-wrap gap-3">
          <label className="label cursor-pointer gap-2">
            <input
              type="radio"
              className="radio radio-sm"
              checked={targetType === 'tag'}
              onChange={() => onTargetTypeChange('tag')}
            />
            <span>{labelTargetTag}</span>
          </label>
          <label className="label cursor-pointer gap-2">
            <input
              type="radio"
              className="radio radio-sm"
              checked={targetType === 'custom'}
              onChange={() => onTargetTypeChange('custom')}
            />
            <span>{labelTargetCustom}</span>
          </label>
        </div>

        {targetType === 'tag' ? (
          <div className="mt-4 flex flex-wrap gap-3">
            {targets.map((target) => (
              <button
                key={target.id}
                className={targetAgentId === target.id ? 'btn btn-sm btn-primary' : 'btn btn-sm btn-outline'}
                onClick={() => onTargetAgentIdChange(target.id)}
              >
                {target.label}
              </button>
            ))}
          </div>
        ) : (
          <label className="form-control mt-4">
            <span className="label-text">{labelCustomRelativePath}</span>
            <input
              className="input input-bordered"
              value={customRelativePath}
              onChange={(event) => onCustomRelativePathChange(event.target.value)}
              placeholder={placeholderCustomRelativePath}
            />
          </label>
        )}
      </section>

      <section className="rounded-box border border-base-300 bg-base-200/50 p-4">
        <h4 className="font-semibold">{titleMode}</h4>
        <div className="mt-4 flex flex-wrap gap-3">
          {(['symlink', 'copy'] as const).map((mode) => (
            <button
              key={mode}
              className={installMode === mode ? 'btn btn-sm btn-primary' : 'btn btn-sm btn-outline'}
              onClick={() => onInstallModeChange(mode)}
            >
              {renderModeLabel(mode)}
            </button>
          ))}
        </div>
      </section>

      <section className="rounded-box border border-base-300 bg-base-200/50 p-4">
        <div className="flex flex-wrap items-center justify-between gap-3">
          <h4 className="font-semibold">{titlePreview}</h4>
          {previewMeta}
        </div>
        <p className="mt-3 break-all text-sm text-base-content/60">
          {resolvedTargetPath || noTargetPreviewText}
        </p>
      </section>
    </>
  )
}
