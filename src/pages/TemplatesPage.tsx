import { useEffect, useMemo, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { useTemplatesStore } from '../stores/use-templates-store'
import type {
  SaveTemplateRequest,
  TemplateInjectionRequest,
  TemplateRecord,
} from '../types/app'

const createEmptyDraft = (): SaveTemplateRequest => ({
  id: null,
  name: '',
  description: '',
  tags: [],
  targetAgents: ['Claude Code'],
  scope: 'user',
  items: [],
})

const toDraft = (template: TemplateRecord): SaveTemplateRequest => ({
  id: template.id,
  name: template.name,
  description: template.description ?? '',
  tags: template.tags,
  targetAgents: template.targetAgents,
  scope: template.scope,
  items: template.items.map((item, index) => ({
    ...item,
    orderIndex: index,
  })),
})

const parseCommaSeparated = (value: string) =>
  value
    .split(',')
    .map((item) => item.trim())
    .filter(Boolean)

export function TemplatesPage() {
  const { t } = useTranslation()
  const templates = useTemplatesStore((state) => state.templates)
  const selectedTemplateId = useTemplatesStore((state) => state.selectedTemplateId)
  const selectedTemplate = useTemplatesStore((state) => state.selectedTemplate)
  const loading = useTemplatesStore((state) => state.loading)
  const loaded = useTemplatesStore((state) => state.loaded)
  const saving = useTemplatesStore((state) => state.saving)
  const deleting = useTemplatesStore((state) => state.deleting)
  const injecting = useTemplatesStore((state) => state.injecting)
  const error = useTemplatesStore((state) => state.error)
  const injectResult = useTemplatesStore((state) => state.injectResult)
  const refresh = useTemplatesStore((state) => state.refresh)
  const selectTemplate = useTemplatesStore((state) => state.selectTemplate)
  const saveTemplate = useTemplatesStore((state) => state.saveTemplate)
  const deleteTemplate = useTemplatesStore((state) => state.deleteTemplate)
  const injectTemplate = useTemplatesStore((state) => state.injectTemplate)

  const [draft, setDraft] = useState<SaveTemplateRequest>(createEmptyDraft)
  const [tagsInput, setTagsInput] = useState('')
  const [targetAgentsInput, setTargetAgentsInput] = useState('Claude Code')
  const [injectRequest, setInjectRequest] = useState<TemplateInjectionRequest>({
    templateId: '',
    targetProjectPath: '',
    overwriteStrategy: 'skip_existing',
  })

  useEffect(() => {
    if (!loaded) {
      void refresh()
    }
  }, [loaded, refresh])

  const saveDisabled = saving || draft.name.trim().length === 0
  const injectionSummary = useMemo(
    () =>
      injectResult
        ? [
            t('templates.injection.summary.installed', { count: injectResult.installedCount }),
            t('templates.injection.summary.skipped', { count: injectResult.skippedCount }),
            t('templates.injection.summary.failed', { count: injectResult.failedCount }),
          ]
        : [],
    [injectResult, t],
  )

  const updateDraft = (patch: Partial<SaveTemplateRequest>) =>
    setDraft((state) => ({ ...state, ...patch }))

  const updateItem = (
    index: number,
    updater: (item: SaveTemplateRequest['items'][number]) => SaveTemplateRequest['items'][number],
  ) => {
    setDraft((state) => ({
      ...state,
      items: state.items.map((item, itemIndex) =>
        itemIndex === index ? updater(item) : item,
      ),
    }))
  }

  const removeItem = (index: number) => {
    setDraft((state) => ({
      ...state,
      items: state.items
        .filter((_, itemIndex) => itemIndex !== index)
        .map((item, itemIndex) => ({ ...item, orderIndex: itemIndex })),
    }))
  }

  const resetToCreate = () => {
    void selectTemplate(null)
    setDraft(createEmptyDraft())
    setTagsInput('')
    setTargetAgentsInput('Claude Code')
    setInjectRequest({
      templateId: '',
      targetProjectPath: '',
      overwriteStrategy: 'skip_existing',
    })
  }

  const handleSave = async () => {
    const payload: SaveTemplateRequest = {
      ...draft,
      description: draft.description?.trim() ? draft.description.trim() : null,
      tags: parseCommaSeparated(tagsInput),
      targetAgents: parseCommaSeparated(targetAgentsInput),
      items: draft.items.map((item, index) => ({
        ...item,
        id: item.id ?? '',
        displayName: item.displayName?.trim() ? item.displayName.trim() : null,
        orderIndex: index,
      })),
    }

    const saved = await saveTemplate(payload)
    setDraft(toDraft(saved))
    setTagsInput(saved.tags.join(', '))
    setTargetAgentsInput(saved.targetAgents.join(', '))
    setInjectRequest((state) => ({
      ...state,
      templateId: saved.id,
    }))
  }

  const handleDelete = async () => {
    if (!selectedTemplateId) return
    await deleteTemplate(selectedTemplateId)
    resetToCreate()
  }

  const handleInject = async () => {
    if (!selectedTemplateId) return
    await injectTemplate({
      ...injectRequest,
      templateId: selectedTemplateId,
    })
  }

  return (
    <div className="space-y-6">
      <section className="rounded-box border border-base-300 bg-base-100 p-6">
        <div className="flex items-start justify-between gap-4">
          <div>
            <h2 className="text-3xl font-semibold">{t('templates.title')}</h2>
            <p className="mt-3 max-w-3xl text-sm text-base-content/65">
              {t('templates.description')}
            </p>
          </div>
          <button className="btn btn-outline" onClick={resetToCreate}>
            {t('templates.create')}
          </button>
        </div>
      </section>

      {error ? (
        <section className="rounded-box border border-error/30 bg-error/5 p-5 text-sm leading-6 text-error">
          {error}
        </section>
      ) : null}

      <section className="grid gap-6 xl:grid-cols-[1fr_1.6fr]">
        <div className="rounded-box border border-base-300 bg-base-100 p-6">
          <div className="flex items-center justify-between gap-3">
            <h3 className="text-lg font-semibold">{t('templates.listTitle')}</h3>
            <span className="text-sm text-base-content/55">
              {t('templates.count', { count: templates.length })}
            </span>
          </div>

          {templates.length === 0 ? (
            <div className="mt-4 rounded-box border border-dashed border-base-300 bg-base-200/60 p-4 text-sm text-base-content/60">
              {loading ? t('templates.loading') : t('templates.empty')}
            </div>
          ) : (
            <div className="mt-4 space-y-3">
              {templates.map((template) => (
                <button
                  key={template.id}
                  className={`w-full rounded-box border p-4 text-left transition-colors ${
                    selectedTemplateId === template.id
                      ? 'border-primary bg-primary/5'
                      : 'border-base-300 bg-base-200/60 hover:bg-base-200'
                  }`}
                  onClick={() => {
                    void selectTemplate(template.id).then((loadedTemplate) => {
                      if (!loadedTemplate) return
                      setDraft(toDraft(loadedTemplate))
                      setTagsInput(loadedTemplate.tags.join(', '))
                      setTargetAgentsInput(loadedTemplate.targetAgents.join(', '))
                      setInjectRequest((state) => ({
                        ...state,
                        templateId: loadedTemplate.id,
                      }))
                    })
                  }}
                >
                  <div className="flex items-start justify-between gap-3">
                    <div>
                      <p className="font-semibold">{template.name}</p>
                      <p className="mt-1 text-sm text-base-content/60">
                        {template.description ?? t('templates.noDescription')}
                      </p>
                    </div>
                    <span className="badge badge-outline">{template.items.length}</span>
                  </div>
                  <div className="mt-3 flex flex-wrap gap-2 text-xs text-base-content/55">
                    <span>{t(`common.scopeValues.${template.scope}`)}</span>
                    {template.targetAgents.map((targetAgent) => (
                      <span key={`${template.id}-${targetAgent}`} className="badge badge-ghost">
                        {targetAgent}
                      </span>
                    ))}
                  </div>
                </button>
              ))}
            </div>
          )}
        </div>

        <div className="space-y-6">
          <section className="rounded-box border border-base-300 bg-base-100 p-6">
            <div className="flex items-center justify-between gap-3">
              <h3 className="text-lg font-semibold">
                {selectedTemplateId ? t('templates.editTitle') : t('templates.createTitle')}
              </h3>
              {selectedTemplate?.isBuiltin ? (
                <span className="badge badge-secondary">{t('templates.builtin')}</span>
              ) : null}
            </div>

            <div className="mt-4 grid gap-4 md:grid-cols-2">
              <label className="form-control">
                <span className="label-text">{t('templates.fields.name')}</span>
                <input
                  className="input input-bordered"
                  value={draft.name}
                  onChange={(event) => updateDraft({ name: event.target.value })}
                  placeholder={t('templates.placeholders.name')}
                />
              </label>

              <label className="form-control">
                <span className="label-text">{t('templates.fields.scope')}</span>
                <select
                  className="select select-bordered"
                  value={draft.scope}
                  onChange={(event) => updateDraft({ scope: event.target.value })}
                >
                  <option value="user">{t('templates.scopeOptions.user')}</option>
                  <option value="system">{t('templates.scopeOptions.system')}</option>
                </select>
              </label>

              <label className="form-control md:col-span-2">
                <span className="label-text">{t('templates.fields.description')}</span>
                <textarea
                  className="textarea textarea-bordered min-h-24"
                  value={draft.description ?? ''}
                  onChange={(event) => updateDraft({ description: event.target.value })}
                  placeholder={t('templates.placeholders.description')}
                />
              </label>

              <label className="form-control">
                <span className="label-text">{t('templates.fields.tags')}</span>
                <input
                  className="input input-bordered"
                  value={tagsInput}
                  onChange={(event) => setTagsInput(event.target.value)}
                  placeholder={t('templates.placeholders.tags')}
                />
              </label>

              <label className="form-control">
                <span className="label-text">{t('templates.fields.targetAgents')}</span>
                <input
                  className="input input-bordered"
                  value={targetAgentsInput}
                  onChange={(event) => setTargetAgentsInput(event.target.value)}
                  placeholder={t('templates.placeholders.targetAgents')}
                />
              </label>
            </div>

            <div className="mt-6">
              <div className="flex items-center justify-between gap-3">
                <h4 className="text-base font-semibold">{t('templates.itemsTitle')}</h4>
                <button
                  className="btn btn-sm btn-outline"
                  onClick={() =>
                    setDraft((state) => ({
                      ...state,
                      items: [
                        ...state.items,
                        {
                          id: '',
                          skillRefType: 'skill_id',
                          skillRef: '',
                          displayName: '',
                          required: true,
                          orderIndex: state.items.length,
                        },
                      ],
                    }))
                  }
                >
                  {t('templates.addItem')}
                </button>
              </div>

              {draft.items.length === 0 ? (
                <div className="mt-4 rounded-box border border-dashed border-base-300 bg-base-200/60 p-4 text-sm text-base-content/60">
                  {t('templates.itemsEmpty')}
                </div>
              ) : (
                <div className="mt-4 space-y-4">
                  {draft.items.map((item, index) => (
                    <article key={`${item.id || 'new'}-${index}`} className="rounded-box border border-base-300 bg-base-200/60 p-4">
                      <div className="grid gap-4 md:grid-cols-2">
                        <label className="form-control">
                          <span className="label-text">{t('templates.fields.itemType')}</span>
                          <select
                            className="select select-bordered"
                            value={item.skillRefType}
                            onChange={(event) =>
                              updateItem(index, (current) => ({
                                ...current,
                                skillRefType: event.target.value,
                              }))
                            }
                          >
                            <option value="skill_id">skill_id</option>
                            <option value="source_url">source_url</option>
                            <option value="market_ref">market_ref</option>
                          </select>
                        </label>

                        <label className="form-control">
                          <span className="label-text">{t('templates.fields.skillRef')}</span>
                          <input
                            className="input input-bordered"
                            value={item.skillRef}
                            onChange={(event) =>
                              updateItem(index, (current) => ({
                                ...current,
                                skillRef: event.target.value,
                              }))
                            }
                            placeholder={t('templates.placeholders.skillRef')}
                          />
                        </label>

                        <label className="form-control">
                          <span className="label-text">{t('templates.fields.displayName')}</span>
                          <input
                            className="input input-bordered"
                            value={item.displayName ?? ''}
                            onChange={(event) =>
                              updateItem(index, (current) => ({
                                ...current,
                                displayName: event.target.value,
                              }))
                            }
                            placeholder={t('templates.placeholders.displayName')}
                          />
                        </label>

                        <label className="label mt-6 cursor-pointer justify-start gap-3">
                          <input
                            type="checkbox"
                            className="checkbox checkbox-primary"
                            checked={item.required}
                            onChange={(event) =>
                              updateItem(index, (current) => ({
                                ...current,
                                required: event.target.checked,
                              }))
                            }
                          />
                          <span className="label-text">{t('templates.fields.required')}</span>
                        </label>
                      </div>

                      <div className="mt-4 flex justify-end">
                        <button className="btn btn-sm btn-ghost text-error" onClick={() => removeItem(index)}>
                          {t('templates.removeItem')}
                        </button>
                      </div>
                    </article>
                  ))}
                </div>
              )}
            </div>

            <div className="mt-6 flex flex-wrap justify-end gap-3">
              {selectedTemplateId ? (
                <button className="btn btn-ghost text-error" onClick={() => void handleDelete()} disabled={deleting}>
                  {deleting ? t('templates.deleting') : t('templates.delete')}
                </button>
              ) : null}
              <button className="btn btn-primary" onClick={() => void handleSave()} disabled={saveDisabled}>
                {saving ? t('templates.saving') : t('templates.save')}
              </button>
            </div>
          </section>

          <section className="rounded-box border border-base-300 bg-base-100 p-6">
            <div className="flex items-center justify-between gap-3">
              <div>
                <h3 className="text-lg font-semibold">{t('templates.injection.title')}</h3>
                <p className="mt-2 text-sm text-base-content/60">
                  {t('templates.injection.description')}
                </p>
              </div>
              {selectedTemplateId ? (
                <span className="badge badge-outline">{selectedTemplate?.name}</span>
              ) : null}
            </div>

            {!selectedTemplateId ? (
              <div className="mt-4 rounded-box border border-dashed border-base-300 bg-base-200/60 p-4 text-sm text-base-content/60">
                {t('templates.injection.selectTemplateFirst')}
              </div>
            ) : (
              <>
                <div className="mt-4 grid gap-4 md:grid-cols-2">
                  <label className="form-control md:col-span-2">
                    <span className="label-text">{t('templates.injection.projectPath')}</span>
                    <input
                      className="input input-bordered"
                      value={injectRequest.targetProjectPath}
                      onChange={(event) =>
                        setInjectRequest((state) => ({
                          ...state,
                          targetProjectPath: event.target.value,
                        }))
                      }
                      placeholder={t('templates.injection.projectPlaceholder')}
                    />
                  </label>

                  <label className="form-control">
                    <span className="label-text">{t('templates.injection.overwriteStrategy')}</span>
                    <select
                      className="select select-bordered"
                      value={injectRequest.overwriteStrategy}
                      onChange={(event) =>
                        setInjectRequest((state) => ({
                          ...state,
                          overwriteStrategy: event.target.value,
                        }))
                      }
                    >
                      <option value="skip_existing">{t('templates.injection.overwriteOptions.skipExisting')}</option>
                      <option value="overwrite">{t('templates.injection.overwriteOptions.overwrite')}</option>
                    </select>
                  </label>
                </div>

                <div className="mt-4 flex justify-end">
                  <button
                    className="btn btn-primary"
                    onClick={() => void handleInject()}
                    disabled={injecting || injectRequest.targetProjectPath.trim().length === 0}
                  >
                    {injecting ? t('templates.injection.injecting') : t('templates.injection.inject')}
                  </button>
                </div>
              </>
            )}

            {injectResult ? (
              <div className="mt-6 space-y-4">
                <div className="rounded-box border border-base-300 bg-base-200/60 p-4">
                  <div className="flex flex-wrap items-center justify-between gap-3">
                    <p className="font-semibold">{t(`templates.injection.statuses.${injectResult.status}`)}</p>
                    <span className="badge badge-outline">{injectResult.targetProjectPath}</span>
                  </div>
                  <p className="mt-3 text-sm text-base-content/60">{injectionSummary.join(' · ')}</p>
                </div>

                <div className="space-y-3">
                  {injectResult.results.map((item) => (
                    <article key={`${injectResult.templateId}-${item.skillRef}`} className="rounded-box border border-base-300 bg-base-200/60 p-4">
                      <div className="flex flex-wrap items-start justify-between gap-3">
                        <div>
                          <p className="font-medium">{item.skillRef}</p>
                          {item.message ? (
                            <p className="mt-2 text-sm text-base-content/60">{item.message}</p>
                          ) : null}
                        </div>
                        <span className="badge badge-outline">
                          {t(`templates.injection.itemStatuses.${item.status}`)}
                        </span>
                      </div>
                    </article>
                  ))}
                </div>
              </div>
            ) : null}
          </section>
        </div>
      </section>
    </div>
  )
}
