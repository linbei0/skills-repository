import { useEffect, useMemo, useState } from 'react'
import { useTranslation } from 'react-i18next'
import {
  BUILTIN_SKILLS_TARGETS,
  createCustomSkillsTargetId,
  hasSkillsTarget,
  normalizeRelativeSkillsPath,
  resolveSkillsTargets,
} from '../lib/skills-targets'
import { useSettingsStore } from '../stores/use-settings-store'
import { useSkillsStore } from '../stores/use-skills-store'

export function SkillsPage() {
  const { t } = useTranslation()
  const settings = useSettingsStore((state) => state.settings)
  const saveSettings = useSettingsStore((state) => state.save)
  const toggleVisibleSkillsTarget = useSettingsStore((state) => state.toggleVisibleSkillsTarget)
  const addCustomSkillsTarget = useSettingsStore((state) => state.addCustomSkillsTarget)
  const removeCustomSkillsTarget = useSettingsStore((state) => state.removeCustomSkillsTarget)
  const settingsSaving = useSettingsStore((state) => state.saving)
  const selectedAgentId = useSkillsStore((state) => state.selectedAgentId)
  const loading = useSkillsStore((state) => state.loading)
  const loaded = useSkillsStore((state) => state.loaded)
  const error = useSkillsStore((state) => state.error)
  const rootPath = useSkillsStore((state) => state.rootPath)
  const entries = useSkillsStore((state) => state.entries)
  const setSelectedAgentId = useSkillsStore((state) => state.setSelectedAgentId)
  const scanAgentGlobalSkills = useSkillsStore((state) => state.scanAgentGlobalSkills)
  const [managerOpen, setManagerOpen] = useState(false)
  const [configError, setConfigError] = useState<string | null>(null)
  const [customLabel, setCustomLabel] = useState('')
  const [customRelativePath, setCustomRelativePath] = useState('')

  const allTargets = useMemo(() => resolveSkillsTargets(settings), [settings])
  const visibleTargets = useMemo(
    () =>
      allTargets.filter((target) => settings.visibleSkillsTargetIds.includes(target.id)),
    [allTargets, settings.visibleSkillsTargetIds],
  )
  const selectedTarget =
    visibleTargets.find((target) => target.id === selectedAgentId) ?? visibleTargets[0] ?? null

  useEffect(() => {
    if (!selectedTarget) return
    if (selectedAgentId !== selectedTarget.id) {
      setSelectedAgentId(selectedTarget.id)
      return
    }

    void scanAgentGlobalSkills({
      agentId: selectedTarget.id,
      agentLabel: selectedTarget.label,
      relativePath: selectedTarget.relativePath,
    })
  }, [scanAgentGlobalSkills, selectedAgentId, selectedTarget, setSelectedAgentId])

  const persistSettings = async () => {
    try {
      setConfigError(null)
      await saveSettings()
    } catch (saveError) {
      setConfigError(
        saveError instanceof Error ? saveError.message : String(saveError),
      )
    }
  }

  const handleToggleTarget = async (targetId: string) => {
    toggleVisibleSkillsTarget(targetId)
    await persistSettings()
  }

  const handleAddCustomTarget = async () => {
    const label = customLabel.trim()
    const relativePath = normalizeRelativeSkillsPath(customRelativePath)

    if (!label) {
      setConfigError(t('skills.manager.validation.nameRequired'))
      return
    }
    if (!relativePath) {
      setConfigError(t('skills.manager.validation.pathRequired'))
      return
    }
    if (relativePath.startsWith('/')) {
      setConfigError(t('skills.manager.validation.relativePathOnly'))
      return
    }

    const id = createCustomSkillsTargetId(label)
    if (hasSkillsTarget(settings, id)) {
      setConfigError(t('skills.manager.validation.duplicateName'))
      return
    }

    addCustomSkillsTarget({
      id,
      label,
      relativePath,
    })
    setCustomLabel('')
    setCustomRelativePath('')
    await persistSettings()
  }

  const handleRemoveCustomTarget = async (targetId: string) => {
    removeCustomSkillsTarget(targetId)
    await persistSettings()
  }

  return (
    <div className="space-y-6">
      <section className="rounded-box border border-base-300 bg-base-100 p-6">
        <div className="flex flex-wrap items-start justify-between gap-4">
          <div>
            <h2 className="text-3xl font-semibold">{t('skills.title')}</h2>
            <p className="mt-3 max-w-3xl text-sm text-base-content/65">
              {t('skills.description')}
            </p>
          </div>
          <button className="btn btn-outline btn-sm" onClick={() => setManagerOpen(true)}>
            {t('skills.manager.open')}
          </button>
        </div>
      </section>

      <section className="rounded-box border border-base-300 bg-base-100 p-6">
        {visibleTargets.length === 0 ? (
          <div className="rounded-box border border-dashed border-base-300 bg-base-200/60 p-4 text-sm text-base-content/60">
            {t('skills.noVisibleTabs')}
          </div>
        ) : (
          <div className="flex flex-wrap gap-3">
            {visibleTargets.map((target) => (
              <button
                key={target.id}
                className={
                  selectedTarget?.id === target.id
                    ? 'btn btn-sm btn-primary'
                    : 'btn btn-sm btn-outline'
                }
                onClick={() => setSelectedAgentId(target.id)}
              >
                {target.label}
              </button>
            ))}
          </div>
        )}

        {rootPath ? (
          <p className="mt-4 break-all text-sm text-base-content/60">{rootPath}</p>
        ) : null}
      </section>

      {configError ? (
        <section className="rounded-box border border-error/30 bg-error/5 p-4 text-sm text-error">
          {configError}
        </section>
      ) : null}

      <section className="space-y-4">
        {!selectedTarget && visibleTargets.length === 0 ? (
          <div className="rounded-box border border-dashed border-base-300 bg-base-100 p-6 text-sm text-base-content/60">
            {t('skills.noVisibleTabs')}
          </div>
        ) : loading ? (
          <div className="rounded-box border border-base-300 bg-base-100 p-6 text-sm text-base-content/60">
            {t('skills.loading')}
          </div>
        ) : error ? (
          <div className="rounded-box border border-error/30 bg-base-100 p-6 text-sm text-error">
            {error}
          </div>
        ) : loaded && entries.length === 0 ? (
          <div className="rounded-box border border-dashed border-base-300 bg-base-100 p-6 text-sm text-base-content/60">
            {t('skills.empty')}
          </div>
        ) : (
          entries.map((entry) => (
            <article key={entry.id} className="rounded-box border border-base-300 bg-base-100 p-5">
              <div className="flex items-start justify-between gap-4">
                <div className="min-w-0">
                  <h3 className="truncate text-xl font-semibold">{entry.name}</h3>
                  <p className="mt-2 text-sm text-base-content/60">
                    {selectedTarget?.label} · {t(`skills.relationshipValues.${entry.relationship}`)}
                  </p>
                  <p className="mt-3 break-all text-sm text-base-content/50">{entry.path}</p>
                </div>
              </div>
            </article>
          ))
        )}
      </section>

      {managerOpen ? (
        <dialog className="modal modal-open">
          <div className="modal-box max-w-4xl">
            <div className="flex items-start justify-between gap-4">
              <div>
                <h3 className="text-xl font-semibold">{t('skills.manager.title')}</h3>
                <p className="mt-2 text-sm text-base-content/60">
                  {t('skills.manager.description')}
                </p>
              </div>
              <button className="btn btn-ghost btn-sm" onClick={() => setManagerOpen(false)}>
                {t('common.close')}
              </button>
            </div>

            <div className="mt-6 grid gap-6 lg:grid-cols-[1.4fr_1fr]">
              <section className="space-y-4">
                <div>
                  <p className="text-sm font-semibold">{t('skills.manager.supportedTitle')}</p>
                  <p className="mt-1 text-sm text-base-content/60">
                    {t('skills.manager.supportedDescription')}
                  </p>
                </div>
                <div className="grid gap-3 sm:grid-cols-2">
                  {BUILTIN_SKILLS_TARGETS.map((target) => (
                    <label
                      key={target.id}
                      className="flex items-start gap-3 rounded-box border border-base-300 bg-base-200/40 p-4"
                    >
                      <input
                        type="checkbox"
                        className="checkbox checkbox-sm mt-0.5"
                        checked={settings.visibleSkillsTargetIds.includes(target.id)}
                        onChange={() => void handleToggleTarget(target.id)}
                        disabled={settingsSaving}
                      />
                      <div className="min-w-0">
                        <p className="font-medium">{target.label}</p>
                        <p className="mt-1 break-all text-xs text-base-content/55">
                          {target.relativePath}
                        </p>
                      </div>
                    </label>
                  ))}
                </div>
              </section>

              <section className="space-y-4">
                <div>
                  <p className="text-sm font-semibold">{t('skills.manager.customTitle')}</p>
                  <p className="mt-1 text-sm text-base-content/60">
                    {t('skills.manager.customDescription')}
                  </p>
                </div>

                <div className="space-y-3">
                  {settings.customSkillsTargets.length === 0 ? (
                    <div className="rounded-box border border-dashed border-base-300 bg-base-200/40 p-4 text-sm text-base-content/60">
                      {t('skills.manager.customEmpty')}
                    </div>
                  ) : (
                    settings.customSkillsTargets.map((target) => (
                      <article
                        key={target.id}
                        className="rounded-box border border-base-300 bg-base-200/40 p-4"
                      >
                        <div className="flex items-start justify-between gap-3">
                          <label className="flex items-start gap-3">
                            <input
                              type="checkbox"
                              className="checkbox checkbox-sm mt-0.5"
                              checked={settings.visibleSkillsTargetIds.includes(target.id)}
                              onChange={() => void handleToggleTarget(target.id)}
                              disabled={settingsSaving}
                            />
                            <div className="min-w-0">
                              <p className="font-medium">{target.label}</p>
                              <p className="mt-1 break-all text-xs text-base-content/55">
                                {target.relativePath}
                              </p>
                            </div>
                          </label>
                          <button
                            className="btn btn-ghost btn-xs text-error"
                            onClick={() => void handleRemoveCustomTarget(target.id)}
                            disabled={settingsSaving}
                          >
                            {t('skills.manager.remove')}
                          </button>
                        </div>
                      </article>
                    ))
                  )}
                </div>

                <div className="rounded-box border border-base-300 bg-base-200/40 p-4">
                  <p className="text-sm font-semibold">{t('skills.manager.addTitle')}</p>
                  <div className="mt-4 space-y-3">
                    <label className="form-control">
                      <span className="mb-2 text-sm font-medium">{t('skills.manager.name')}</span>
                      <input
                        className="input input-bordered"
                        value={customLabel}
                        onChange={(event) => setCustomLabel(event.target.value)}
                        placeholder={t('skills.manager.namePlaceholder')}
                      />
                    </label>
                    <label className="form-control">
                      <span className="mb-2 text-sm font-medium">{t('skills.manager.path')}</span>
                      <input
                        className="input input-bordered"
                        value={customRelativePath}
                        onChange={(event) => setCustomRelativePath(event.target.value)}
                        placeholder={t('skills.manager.pathPlaceholder')}
                      />
                    </label>
                    <button
                      className="btn btn-primary btn-sm"
                      onClick={() => void handleAddCustomTarget()}
                      disabled={settingsSaving}
                    >
                      {t('skills.manager.add')}
                    </button>
                  </div>
                </div>
              </section>
            </div>
          </div>
          <form method="dialog" className="modal-backdrop">
            <button onClick={() => setManagerOpen(false)}>{t('common.close')}</button>
          </form>
        </dialog>
      ) : null}
    </div>
  )
}
