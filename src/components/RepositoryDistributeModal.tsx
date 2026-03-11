import { open as openDialog } from '@tauri-apps/plugin-dialog'
import { useEffect, useMemo, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { ProjectDistributionFields } from './ProjectDistributionFields'
import { ProjectDistributionResultPanel } from './ProjectDistributionResultPanel'
import type { SkillsTargetOption } from '../lib/skills-targets'
import type {
  BatchDistributeRepositorySkillsRequest,
  BatchDistributeResult,
  RepositorySkillSummary,
} from '../types/app'

interface RepositoryDistributeModalProps {
  open: boolean
  repositorySkills: RepositorySkillSummary[]
  targets: SkillsTargetOption[]
  distributing: boolean
  error: string | null
  result: BatchDistributeResult | null
  onClose: () => void
  onSubmit: (request: BatchDistributeRepositorySkillsRequest) => Promise<void>
}

export function RepositoryDistributeModal({
  open,
  repositorySkills,
  targets,
  distributing,
  error,
  result,
  onClose,
  onSubmit,
}: RepositoryDistributeModalProps) {
  const { t } = useTranslation()
  const [query, setQuery] = useState('')
  const [targetScope, setTargetScope] = useState<'global' | 'project'>('project')
  const [selectedSkillIds, setSelectedSkillIds] = useState<string[]>([])
  const [projectRoot, setProjectRoot] = useState('')
  const [targetType, setTargetType] = useState<'tag' | 'custom'>('tag')
  const [targetAgentId, setTargetAgentId] = useState('')
  const [customRelativePath, setCustomRelativePath] = useState('')
  const [installMode, setInstallMode] = useState<'symlink' | 'copy'>('symlink')

  useEffect(() => {
    if (!open) return
    setQuery('')
    setTargetScope('project')
    setSelectedSkillIds([])
    setProjectRoot('')
    setTargetType('tag')
    setTargetAgentId(targets[0]?.id ?? '')
    setCustomRelativePath('')
    setInstallMode('symlink')
  }, [open, targets])

  const filteredSkills = useMemo(() => {
    const normalizedQuery = query.trim().toLowerCase()
    return repositorySkills.filter((skill) => {
      if (!normalizedQuery) return true
      return (
        skill.name.toLowerCase().includes(normalizedQuery) ||
        skill.slug.toLowerCase().includes(normalizedQuery) ||
        (skill.sourceMarket ?? '').toLowerCase().includes(normalizedQuery)
      )
    })
  }, [query, repositorySkills])

  const selectedSkills = useMemo(
    () => repositorySkills.filter((skill) => selectedSkillIds.includes(skill.id)),
    [repositorySkills, selectedSkillIds],
  )

  const resolvedTargetPath = useMemo(() => {
    if (targetScope === 'project' && !projectRoot.trim()) return ''
    const rootPrefix = targetScope === 'global' ? '<home>' : projectRoot.replace(/[\\/]+$/, '')
    if (targetType === 'custom') {
      return customRelativePath.trim()
        ? `${rootPrefix}/${customRelativePath.replace(/^[\\/]+/, '')}`
        : ''
    }

    const target = targets.find((item) => item.id === targetAgentId)
    return target ? `${rootPrefix}/${target.relativePath}` : ''
  }, [customRelativePath, projectRoot, targetAgentId, targetScope, targetType, targets])

  if (!open) return null

  const canSubmit =
    !distributing &&
    selectedSkillIds.length > 0 &&
    (targetScope === 'global' || projectRoot.trim().length > 0) &&
    ((targetType === 'tag' && targetAgentId.length > 0) ||
      (targetType === 'custom' && customRelativePath.trim().length > 0))

  const chooseProjectDirectory = async () => {
    const selected = await openDialog({
      directory: true,
      multiple: false,
    })

    if (typeof selected === 'string') {
      setProjectRoot(selected)
    }
  }

  const toggleSkill = (skillId: string) => {
    setSelectedSkillIds((current) =>
      current.includes(skillId) ? current.filter((item) => item !== skillId) : [...current, skillId],
    )
  }

  const selectAllVisible = () => {
    setSelectedSkillIds(Array.from(new Set([...selectedSkillIds, ...filteredSkills.map((skill) => skill.id)])))
  }

  const clearSelection = () => {
    setSelectedSkillIds([])
  }

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-base-content/45 p-4 backdrop-blur-sm md:p-6">
      <div className="flex max-h-[92vh] w-full max-w-6xl flex-col overflow-hidden rounded-[28px] border border-base-300 bg-base-100 shadow-2xl">
        <div className="flex items-start justify-between gap-4 border-b border-base-300 px-6 py-5 md:px-7">
          <div>
            <h3 className="text-2xl font-semibold">{t('repository.distribute.title')}</h3>
            <p className="mt-2 max-w-3xl text-sm leading-6 text-base-content/60">
              {t('repository.distribute.subtitle')}
            </p>
          </div>
          <button className="btn btn-ghost btn-circle" aria-label={t('common.close')} onClick={onClose}>
            <span className="text-xl font-semibold leading-none">x</span>
          </button>
        </div>

        <div className="space-y-6 overflow-y-auto p-6 md:p-7">
          <section className="rounded-[24px] border border-base-300 bg-base-200/50 p-5">
            <div className="flex flex-wrap items-start justify-between gap-4">
              <div>
                <h4 className="text-lg font-semibold">{t('repository.distribute.skillsTitle')}</h4>
                <p className="mt-1 text-sm text-base-content/60">
                  {t('repository.distribute.selectedCount', { count: selectedSkillIds.length })}
                </p>
              </div>
              <div className="flex flex-wrap gap-2">
                <button className="btn btn-sm btn-outline" onClick={selectAllVisible}>
                  {t('repository.distribute.selectAll')}
                </button>
                <button className="btn btn-sm btn-ghost" onClick={clearSelection}>
                  {t('repository.distribute.clearSelection')}
                </button>
              </div>
            </div>

            <label className="input input-bordered mt-5 flex items-center gap-2">
              <i className="hn hn-search text-base-content/50" aria-hidden />
              <input
                className="grow"
                value={query}
                onChange={(event) => setQuery(event.target.value)}
                placeholder={t('repository.distribute.searchPlaceholder')}
              />
            </label>

            <div className="mt-5 max-h-[18rem] space-y-3 overflow-y-auto pr-1">
              {filteredSkills.length === 0 ? (
                <div className="rounded-[20px] border border-dashed border-base-300 bg-base-100 p-5 text-sm text-base-content/60">
                  {t('repository.distribute.emptySearch')}
                </div>
              ) : (
                filteredSkills.map((skill) => (
                  <label
                    key={skill.id}
                    className="flex cursor-pointer items-start gap-4 rounded-[20px] border border-base-300 bg-base-100 p-4"
                  >
                    <input
                      type="checkbox"
                      className="checkbox checkbox-sm mt-1"
                      checked={selectedSkillIds.includes(skill.id)}
                      onChange={() => toggleSkill(skill.id)}
                    />
                    <div className="min-w-0">
                      <div className="flex flex-wrap items-center gap-2">
                        <p className="font-semibold">{skill.name}</p>
                        <span className="badge badge-outline">{skill.slug}</span>
                      </div>
                      <p className="mt-1 text-xs text-base-content/60">
                        {skill.sourceMarket ?? t('repository.sourceUnknown')}
                      </p>
                    </div>
                  </label>
                ))
              )}
            </div>
          </section>

          <section className="rounded-box border border-base-300 bg-base-200/50 p-4">
            <h4 className="font-semibold">{t('repository.distribute.scopeTitle')}</h4>
            <div className="mt-4 flex flex-wrap gap-3">
              <label className="label cursor-pointer gap-2">
                <input
                  type="radio"
                  className="radio radio-sm"
                  checked={targetScope === 'project'}
                  onChange={() => setTargetScope('project')}
                />
                <span>{t('repository.distribute.scopeProject')}</span>
              </label>
              <label className="label cursor-pointer gap-2">
                <input
                  type="radio"
                  className="radio radio-sm"
                  checked={targetScope === 'global'}
                  onChange={() => setTargetScope('global')}
                />
                <span>{t('repository.distribute.scopeGlobal')}</span>
              </label>
            </div>
          </section>

          <ProjectDistributionFields
            projectRoot={projectRoot}
            targetType={targetType}
            targetAgentId={targetAgentId}
            customRelativePath={customRelativePath}
            installMode={installMode}
            targets={targets}
            resolvedTargetPath={resolvedTargetPath}
            titleTarget={t('repository.distribute.targetTitle')}
            titleMode={t('repository.distribute.modeTitle')}
            titlePreview={t('repository.distribute.previewTitle')}
            labelProjectRoot={t('repository.distribute.projectRoot')}
            labelProjectPicker={t('repository.distribute.projectPicker')}
            labelChooseDirectory={t('repository.distribute.chooseDirectory')}
            labelTargetTag={t('repository.distribute.targetTag')}
            labelTargetCustom={t('repository.distribute.targetCustom')}
            labelCustomRelativePath={t('repository.distribute.customRelativePath')}
            placeholderProjectRoot={t('repository.distribute.projectRootPlaceholder')}
            placeholderCustomRelativePath={t('repository.distribute.customRelativePathPlaceholder')}
            noTargetPreviewText={t('repository.distribute.noTargetPreview')}
            showProjectRoot={targetScope === 'project'}
            previewMeta={
              <div className="flex flex-wrap gap-2 text-sm text-base-content/60">
                <span>{t('repository.distribute.selectedCount', { count: selectedSkillIds.length })}</span>
              </div>
            }
            onProjectRootChange={setProjectRoot}
            onTargetTypeChange={setTargetType}
            onTargetAgentIdChange={setTargetAgentId}
            onCustomRelativePathChange={setCustomRelativePath}
            onInstallModeChange={setInstallMode}
            onChooseProjectDirectory={() => void chooseProjectDirectory()}
            renderModeLabel={(mode) => t(`repository.distribute.modes.${mode}`)}
          />

          <section className="rounded-box border border-base-300 bg-base-200/50 p-4">
            <h4 className="font-semibold">{t('repository.distribute.previewListTitle')}</h4>
            {selectedSkills.length === 0 ? (
              <div className="mt-4 rounded-box border border-dashed border-base-300 bg-base-100 p-4 text-sm text-base-content/60">
                {t('repository.distribute.noSkillsSelected')}
              </div>
            ) : (
              <div className="mt-4 flex flex-wrap gap-2">
                {selectedSkills.map((skill) => (
                  <span key={skill.id} className="badge badge-outline">
                    {skill.name}
                  </span>
                ))}
              </div>
            )}
          </section>

          {error ? (
            <section className="rounded-box border border-error/30 bg-error/5 p-4 text-sm leading-6 text-error">
              {error}
            </section>
          ) : null}

          <ProjectDistributionResultPanel
            result={result}
            titleInstalled={t('repository.distribute.result.installed', { count: result?.installed.length ?? 0 })}
            titleSkipped={t('repository.distribute.result.skipped', { count: result?.skipped.length ?? 0 })}
            titleFailed={t('repository.distribute.result.failed', { count: result?.failed.length ?? 0 })}
          />
        </div>

        <div className="flex justify-end gap-3 border-t border-base-300 px-6 py-5 md:px-7">
          <button className="btn btn-ghost" onClick={onClose}>
            {t('common.close')}
          </button>
          <button
            className="btn btn-primary"
            disabled={!canSubmit}
            onClick={() =>
              void onSubmit({
                targetScope,
                skillIds: selectedSkillIds,
                projectRoot: targetScope === 'project' ? projectRoot.trim() : null,
                targetType,
                targetAgentId: targetType === 'tag' ? targetAgentId : null,
                customRelativePath: targetType === 'custom' ? customRelativePath.trim() : null,
                installMode,
              }).catch(() => undefined)
            }
          >
            {distributing ? t('repository.distribute.distributing') : t('repository.distribute.confirm')}
          </button>
        </div>
      </div>
    </div>
  )
}
