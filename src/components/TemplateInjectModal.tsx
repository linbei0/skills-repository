import { open as openDialog } from '@tauri-apps/plugin-dialog'
import { useEffect, useMemo, useState } from 'react'
import { useTranslation } from 'react-i18next'
import type { SkillsTargetOption } from '../lib/skills-targets'
import type {
  InjectTemplateRequest,
  InjectTemplateResult,
  TemplateRecord,
} from '../types/app'

interface TemplateInjectModalProps {
  open: boolean
  template: TemplateRecord | null
  targets: SkillsTargetOption[]
  injecting: boolean
  result: InjectTemplateResult | null
  validSkillCount: number
  missingSkillCount: number
  onClose: () => void
  onSubmit: (request: InjectTemplateRequest) => Promise<void>
}

export function TemplateInjectModal({
  open,
  template,
  targets,
  injecting,
  result,
  validSkillCount,
  missingSkillCount,
  onClose,
  onSubmit,
}: TemplateInjectModalProps) {
  const { t } = useTranslation()
  const [projectRoot, setProjectRoot] = useState('')
  const [targetType, setTargetType] = useState<'tag' | 'custom'>('tag')
  const [targetAgentId, setTargetAgentId] = useState<string>('')
  const [customRelativePath, setCustomRelativePath] = useState('')
  const [installMode, setInstallMode] = useState<'symlink' | 'copy'>('symlink')

  useEffect(() => {
    if (!open) return
    setProjectRoot('')
    setTargetType('tag')
    setTargetAgentId(targets[0]?.id ?? '')
    setCustomRelativePath('')
    setInstallMode('symlink')
  }, [open, targets])

  const resolvedTargetPath = useMemo(() => {
    if (!projectRoot.trim()) return ''

    if (targetType === 'custom') {
      return customRelativePath.trim()
        ? `${projectRoot.replace(/[\\/]+$/, '')}/${customRelativePath.replace(/^[\\/]+/, '')}`
        : ''
    }

    const target = targets.find((item) => item.id === targetAgentId)
    return target ? `${projectRoot.replace(/[\\/]+$/, '')}/${target.relativePath}` : ''
  }, [customRelativePath, projectRoot, targetAgentId, targetType, targets])

  if (!open || !template) return null

  const canSubmit =
    !injecting &&
    projectRoot.trim().length > 0 &&
    validSkillCount > 0 &&
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

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-base-content/45 p-4 backdrop-blur-sm md:p-6">
      <div className="flex max-h-[92vh] w-full max-w-4xl flex-col overflow-hidden rounded-box border border-base-300 bg-base-100 shadow-2xl">
        <div className="flex items-start justify-between gap-4 border-b border-base-300 px-6 py-5">
          <div>
            <h3 className="text-2xl font-semibold">{t('templates.inject.title')}</h3>
            <p className="mt-2 text-sm text-base-content/60">
              {t('templates.inject.subtitle', { name: template.name })}
            </p>
          </div>
          <button className="btn btn-ghost btn-circle" aria-label={t('common.close')} onClick={onClose}>
            <span className="text-xl font-semibold leading-none">x</span>
          </button>
        </div>

        <div className="space-y-6 overflow-y-auto p-6">
          <section className="grid gap-4 md:grid-cols-[1.3fr_0.7fr]">
            <label className="form-control">
              <span className="label-text">{t('templates.inject.projectRoot')}</span>
              <input
                className="input input-bordered"
                value={projectRoot}
                onChange={(event) => setProjectRoot(event.target.value)}
                placeholder={t('templates.inject.projectRootPlaceholder')}
              />
            </label>
            <div className="form-control">
              <span className="label-text">{t('templates.inject.projectPicker')}</span>
              <button className="btn btn-outline" onClick={() => void chooseProjectDirectory()}>
                {t('templates.inject.chooseDirectory')}
              </button>
            </div>
          </section>

          <section className="rounded-box border border-base-300 bg-base-200/50 p-4">
            <h4 className="font-semibold">{t('templates.inject.targetTitle')}</h4>
            <div className="mt-4 flex flex-wrap gap-3">
              <label className="label cursor-pointer gap-2">
                <input
                  type="radio"
                  className="radio radio-sm"
                  checked={targetType === 'tag'}
                  onChange={() => setTargetType('tag')}
                />
                <span>{t('templates.inject.targetTag')}</span>
              </label>
              <label className="label cursor-pointer gap-2">
                <input
                  type="radio"
                  className="radio radio-sm"
                  checked={targetType === 'custom'}
                  onChange={() => setTargetType('custom')}
                />
                <span>{t('templates.inject.targetCustom')}</span>
              </label>
            </div>

            {targetType === 'tag' ? (
              <div className="mt-4 flex flex-wrap gap-3">
                {targets.map((target) => (
                  <button
                    key={target.id}
                    className={targetAgentId === target.id ? 'btn btn-sm btn-primary' : 'btn btn-sm btn-outline'}
                    onClick={() => setTargetAgentId(target.id)}
                  >
                    {target.label}
                  </button>
                ))}
              </div>
            ) : (
              <label className="form-control mt-4">
                <span className="label-text">{t('templates.inject.customRelativePath')}</span>
                <input
                  className="input input-bordered"
                  value={customRelativePath}
                  onChange={(event) => setCustomRelativePath(event.target.value)}
                  placeholder={t('templates.inject.customRelativePathPlaceholder')}
                />
              </label>
            )}
          </section>

          <section className="rounded-box border border-base-300 bg-base-200/50 p-4">
            <h4 className="font-semibold">{t('templates.inject.modeTitle')}</h4>
            <div className="mt-4 flex flex-wrap gap-3">
              {(['symlink', 'copy'] as const).map((mode) => (
                <button
                  key={mode}
                  className={installMode === mode ? 'btn btn-sm btn-primary' : 'btn btn-sm btn-outline'}
                  onClick={() => setInstallMode(mode)}
                >
                  {t(`templates.inject.modes.${mode}`)}
                </button>
              ))}
            </div>
          </section>

          <section className="rounded-box border border-base-300 bg-base-200/50 p-4">
            <div className="flex flex-wrap items-center justify-between gap-3">
              <h4 className="font-semibold">{t('templates.inject.previewTitle')}</h4>
              <div className="flex flex-wrap gap-2 text-sm text-base-content/60">
                <span>{t('templates.inject.validSkills', { count: validSkillCount })}</span>
                <span>{t('templates.inject.missingSkills', { count: missingSkillCount })}</span>
              </div>
            </div>
            <p className="mt-3 break-all text-sm text-base-content/60">
              {resolvedTargetPath || t('templates.inject.noTargetPreview')}
            </p>
          </section>

          {result ? (
            <section className="rounded-box border border-base-300 bg-base-200/50 p-4">
              <div className="flex flex-wrap gap-4 text-sm">
                <span>{t('templates.inject.result.installed', { count: result.installed.length })}</span>
                <span>{t('templates.inject.result.skipped', { count: result.skipped.length })}</span>
                <span>{t('templates.inject.result.failed', { count: result.failed.length })}</span>
              </div>
              <div className="mt-4 space-y-3">
                {[...result.installed, ...result.skipped, ...result.failed].map((item) => (
                  <article key={`${item.skillId}-${item.targetPath}`} className="rounded-box border border-base-300 bg-base-100 p-3 text-sm">
                    <p className="font-medium">{item.skillName}</p>
                    <p className="mt-1 break-all text-xs text-base-content/55">{item.targetPath}</p>
                    {item.reason ? <p className="mt-2 text-xs text-warning">{item.reason}</p> : null}
                  </article>
                ))}
              </div>
            </section>
          ) : null}
        </div>

        <div className="flex justify-end gap-3 border-t border-base-300 px-6 py-5">
          <button className="btn btn-ghost" onClick={onClose}>
            {t('common.close')}
          </button>
          <button
            className="btn btn-primary"
            disabled={!canSubmit}
            onClick={() =>
              void onSubmit({
                templateId: template.id,
                projectRoot: projectRoot.trim(),
                targetType,
                targetAgentId: targetType === 'tag' ? targetAgentId : null,
                customRelativePath: targetType === 'custom' ? customRelativePath.trim() : null,
                installMode,
              })
            }
          >
            {injecting ? t('templates.inject.injecting') : t('templates.inject.confirm')}
          </button>
        </div>
      </div>
    </div>
  )
}
