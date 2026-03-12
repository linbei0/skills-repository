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
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-[var(--bg-modal-overlay)] p-4 backdrop-blur-sm transition-all duration-300 md:p-6">
      <div className="flex max-h-[92vh] w-full max-w-6xl flex-col overflow-hidden rounded-xl border border-[var(--border-subtle)] bg-[var(--bg-modal-panel)] shadow-[0_0_50px_rgba(0,0,0,0.5)]">
        
        {/* Header */}
        <div className="flex items-start justify-between gap-4 border-b border-[var(--border-subtle)] bg-base-100/50 px-8 py-6 backdrop-blur-md md:px-8">
          <div>
            <h3 className="text-2xl font-bold text-base-content">{t('repository.distribute.title')}</h3>
            <p className="mt-2 max-w-3xl text-sm leading-relaxed text-base-content/60">
              {t('repository.distribute.subtitle')}
            </p>
          </div>
          <button 
            className="btn btn-circle btn-ghost btn-sm text-base-content/50 hover:bg-base-content/10 hover:text-base-content" 
            aria-label={t('common.close')} 
            onClick={onClose}
          >
            <i className="hn hn-times text-lg"></i>
          </button>
        </div>

        <div className="space-y-6 overflow-y-auto p-8 md:p-8 custom-scrollbar">
          
          {/* Skill Selection Section */}
          <section className="rounded-lg border border-[var(--border-subtle)] bg-base-200/20 p-6">
            <div className="flex flex-wrap items-center justify-between gap-4">
              <div>
                <h4 className="text-lg font-bold text-base-content">{t('repository.distribute.skillsTitle')}</h4>
                <p className="mt-1 text-sm text-base-content/50">
                  {t('repository.distribute.selectedCount', { count: selectedSkillIds.length })}
                </p>
              </div>
              <div className="flex flex-wrap gap-2">
                <button className="btn btn-xs btn-ghost text-primary hover:bg-primary/10" onClick={selectAllVisible}>
                  {t('repository.distribute.selectAll')}
                </button>
                <button className="btn btn-xs btn-ghost text-base-content/50 hover:text-base-content" onClick={clearSelection}>
                  {t('repository.distribute.clearSelection')}
                </button>
              </div>
            </div>

            <div className="relative mt-6">
              <div className="pointer-events-none absolute inset-y-0 left-0 flex items-center pl-4">
                <i className="hn hn-search text-base-content/40"></i>
              </div>
              <input
                className="input input-bordered h-12 w-full border-[var(--border-subtle)] bg-[var(--bg-input)] pl-11 text-base-content placeholder:text-base-content/30 focus:border-primary/50 focus:bg-[var(--bg-input-focus)] focus:outline-none"
                value={query}
                onChange={(event) => setQuery(event.target.value)}
                placeholder={t('repository.distribute.searchPlaceholder')}
              />
            </div>

            <div className="mt-6 max-h-[18rem] space-y-3 overflow-y-auto pr-1 custom-scrollbar">
              {filteredSkills.length === 0 ? (
                <div className="flex flex-col items-center justify-center rounded-lg border border-dashed border-[var(--border-subtle)] bg-base-100/50 p-8 text-center">
                  <div className="mb-3 rounded-full bg-base-200 p-3 text-base-content/20">
                    <i className="hn hn-search text-2xl"></i>
                  </div>
                  <p className="text-sm text-base-content/40">
                    {t('repository.distribute.emptySearch')}
                  </p>
                </div>
              ) : (
                filteredSkills.map((skill) => (
                  <label
                    key={skill.id}
                    className={`group flex cursor-pointer items-center gap-4 rounded-lg border p-4 transition-all duration-200 ${
                      selectedSkillIds.includes(skill.id)
                        ? 'border-primary/50 bg-primary/5 shadow-[inset_0_0_10px_rgba(var(--color-primary),0.05)]' 
                        : 'border-[var(--border-subtle)] bg-base-100 hover:border-[var(--border-subtle)] hover:bg-base-100/80'
                    }`}
                  >
                    <input
                      type="checkbox"
                      className="checkbox checkbox-sm checkbox-primary border-[var(--border-subtle)] bg-base-100"
                      checked={selectedSkillIds.includes(skill.id)}
                      onChange={() => toggleSkill(skill.id)}
                    />
                    <div className="min-w-0 flex-1">
                      <div className="flex flex-wrap items-center gap-2">
                        <p className={`font-bold ${selectedSkillIds.includes(skill.id) ? 'text-primary' : 'text-base-content/90'}`}>
                          {skill.name}
                        </p>
                        <span className="badge badge-outline border-[var(--border-subtle)] text-xs text-base-content/50">{skill.slug}</span>
                      </div>
                      <p className="mt-1 text-xs text-base-content/40">
                        {skill.sourceMarket ?? t('repository.sourceUnknown')}
                      </p>
                    </div>
                  </label>
                ))
              )}
            </div>
          </section>

          {/* Scope Selection */}
          <section className="rounded-lg border border-[var(--border-subtle)] bg-base-200/20 p-6">
            <h4 className="font-bold text-base-content mb-4">{t('repository.distribute.scopeTitle')}</h4>
            <div className="flex flex-wrap gap-4">
              <label className={`flex cursor-pointer items-center gap-3 rounded-lg border px-5 py-3 transition-all ${
                targetScope === 'project' 
                  ? 'border-primary/50 bg-primary/5' 
                  : 'border-[var(--border-subtle)] bg-base-100 hover:border-[var(--border-subtle)]'
              }`}>
                <input
                  type="radio"
                  className="radio radio-sm radio-primary border-[var(--border-subtle)]"
                  checked={targetScope === 'project'}
                  onChange={() => setTargetScope('project')}
                />
                <span className={targetScope === 'project' ? 'text-base-content' : 'text-base-content/70'}>
                  {t('repository.distribute.scopeProject')}
                </span>
              </label>
              <label className={`flex cursor-pointer items-center gap-3 rounded-lg border px-5 py-3 transition-all ${
                targetScope === 'global' 
                  ? 'border-primary/50 bg-primary/5' 
                  : 'border-[var(--border-subtle)] bg-base-100 hover:border-[var(--border-subtle)]'
              }`}>
                <input
                  type="radio"
                  className="radio radio-sm radio-primary border-[var(--border-subtle)]"
                  checked={targetScope === 'global'}
                  onChange={() => setTargetScope('global')}
                />
                <span className={targetScope === 'global' ? 'text-base-content' : 'text-base-content/70'}>
                  {t('repository.distribute.scopeGlobal')}
                </span>
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

          <section className="rounded-lg border border-[var(--border-subtle)] bg-base-200/20 p-6">
            <h4 className="font-bold text-base-content mb-4">{t('repository.distribute.previewListTitle')}</h4>
            {selectedSkills.length === 0 ? (
              <div className="rounded-lg border border-dashed border-[var(--border-subtle)] bg-base-100/50 p-6 text-center text-sm text-base-content/40">
                {t('repository.distribute.noSkillsSelected')}
              </div>
            ) : (
              <div className="flex flex-wrap gap-2">
                {selectedSkills.map((skill) => (
                  <span key={skill.id} className="badge badge-lg border-[var(--border-subtle)] bg-base-100 text-base-content/80 pl-3 pr-3 h-8">
                    {skill.name}
                  </span>
                ))}
              </div>
            )}
          </section>

          {error ? (
            <section className="rounded-lg border border-error/30 bg-error/10 p-4 text-sm leading-6 text-error shadow-[0_0_15px_rgba(255,0,0,0.1)]">
              <div className="flex items-center gap-3">
                <i className="hn hn-exclaimation text-lg"></i>
                {error}
              </div>
            </section>
          ) : null}

          <ProjectDistributionResultPanel
            result={result}
            titleInstalled={t('repository.distribute.result.installed', { count: result?.installed.length ?? 0 })}
            titleSkipped={t('repository.distribute.result.skipped', { count: result?.skipped.length ?? 0 })}
            titleFailed={t('repository.distribute.result.failed', { count: result?.failed.length ?? 0 })}
          />
        </div>

        <div className="flex justify-end gap-3 border-t border-[var(--border-subtle)] bg-base-100/50 px-8 py-6 backdrop-blur-md md:px-8">
          <button 
            className="btn btn-ghost text-base-content/60 hover:bg-base-content/10 hover:text-base-content" 
            onClick={onClose}
          >
            {t('common.close')}
          </button>
          <button
            className="btn btn-primary h-10 min-h-[2.5rem] border-none bg-primary px-8 text-[var(--text-inverse)] shadow-[var(--shadow-neon-primary)] transition-all duration-300 hover:bg-primary hover:brightness-110 hover:shadow-[0_0_30px_rgba(var(--color-primary),0.6)] disabled:bg-base-300 disabled:text-base-content/30 disabled:shadow-none"
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
            {distributing ? (
              <>
                <span className="loading loading-spinner loading-sm"></span>
                {t('repository.distribute.distributing')}
              </>
            ) : (
              <>
                <i className="hn hn-share mr-2"></i>
                {t('repository.distribute.confirm')}
              </>
            )}
          </button>
        </div>
      </div>
    </div>
  )
}
