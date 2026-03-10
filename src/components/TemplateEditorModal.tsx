import { useEffect, useMemo, useState } from 'react'
import { useTranslation } from 'react-i18next'
import type {
  RepositorySkillSummary,
  SaveTemplateItemRequest,
  SaveTemplateRequest,
} from '../types/app'

interface TemplateEditorModalProps {
  open: boolean
  draft: SaveTemplateRequest
  tagsInput: string
  repositorySkills: RepositorySkillSummary[]
  saving: boolean
  onClose: () => void
  onDraftChange: (draft: SaveTemplateRequest) => void
  onTagsInputChange: (value: string) => void
  onSave: () => Promise<void>
}

const selectedSkillIds = (items: SaveTemplateItemRequest[]) =>
  new Set(items.map((item) => item.skillRef))

export function TemplateEditorModal({
  open,
  draft,
  tagsInput,
  repositorySkills,
  saving,
  onClose,
  onDraftChange,
  onTagsInputChange,
  onSave,
}: TemplateEditorModalProps) {
  const { t } = useTranslation()
  const [skillQuery, setSkillQuery] = useState('')

  useEffect(() => {
    if (!open) {
      setSkillQuery('')
    }
  }, [open])

  const chosenSkillIds = useMemo(() => selectedSkillIds(draft.items), [draft.items])
  const repositorySkillMap = useMemo(
    () => new Map(repositorySkills.map((skill) => [skill.id, skill])),
    [repositorySkills],
  )

  const filteredSkills = useMemo(() => {
    const query = skillQuery.trim().toLowerCase()
    return repositorySkills.filter((skill) => {
      if (chosenSkillIds.has(skill.id)) return false
      if (!query) return true
      return (
        skill.name.toLowerCase().includes(query) ||
        skill.id.toLowerCase().includes(query) ||
        (skill.sourceMarket ?? '').toLowerCase().includes(query)
      )
    })
  }, [chosenSkillIds, repositorySkills, skillQuery])

  if (!open) return null

  const addSkill = (skill: RepositorySkillSummary) => {
    onDraftChange({
      ...draft,
      items: [
        ...draft.items,
        {
          skillRefType: 'repository_skill',
          skillRef: skill.id,
          displayName: skill.name,
          orderIndex: draft.items.length,
        },
      ],
    })
  }

  const removeSkill = (skillId: string) => {
    onDraftChange({
      ...draft,
      items: draft.items
        .filter((item) => item.skillRef !== skillId)
        .map((item, index) => ({ ...item, orderIndex: index })),
    })
  }

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-base-content/45 p-4 backdrop-blur-sm md:p-6">
      <div className="flex max-h-[92vh] w-full max-w-6xl flex-col overflow-hidden rounded-[28px] border border-base-300 bg-base-100 shadow-2xl">
        <div className="border-b border-base-300 px-6 py-5 md:px-7">
          <div className="flex items-start justify-between gap-4">
            <div className="space-y-2">
              <h3 className="text-2xl font-semibold md:text-3xl">
                {draft.id ? t('templates.editor.editTitle') : t('templates.editor.createTitle')}
              </h3>
              <p className="max-w-3xl text-sm leading-6 text-base-content/60">
                {t('templates.editor.subtitle')}
              </p>
            </div>
            <button className="btn btn-ghost btn-circle" aria-label={t('common.close')} onClick={onClose}>
              <span className="text-xl font-semibold leading-none">x</span>
            </button>
          </div>

          <div className="mt-4 flex flex-wrap gap-2 text-sm text-base-content/65">
            <span className="badge badge-outline">
              {t('templates.editor.selectedCount', { count: draft.items.length })}
            </span>
            <span className="badge badge-outline">
              {t('templates.editor.repositoryCount', { count: repositorySkills.length })}
            </span>
          </div>
        </div>

        <div className="grid gap-6 overflow-y-auto p-6 lg:grid-cols-[minmax(0,0.95fr)_minmax(0,1.05fr)] md:p-7">
          <section className="space-y-6">
            <article className="rounded-[24px] border border-base-300 bg-base-200/50 p-5">
              <div className="mb-5 flex items-center justify-between gap-3">
                <h4 className="text-lg font-semibold">{t('templates.fields.name')}</h4>
                <span className="text-xs text-base-content/55">ID: {draft.id ?? 'new'}</span>
              </div>

              <div className="space-y-4">
                <label className="block space-y-2">
                  <span className="text-sm font-medium text-base-content/80">
                    {t('templates.fields.name')}
                  </span>
                  <input
                    className="input input-bordered w-full"
                    value={draft.name}
                    onChange={(event) => onDraftChange({ ...draft, name: event.target.value })}
                    placeholder={t('templates.placeholders.name')}
                  />
                </label>

                <label className="block space-y-2">
                  <span className="text-sm font-medium text-base-content/80">
                    {t('templates.fields.description')}
                  </span>
                  <textarea
                    className="textarea textarea-bordered min-h-28 w-full"
                    value={draft.description ?? ''}
                    onChange={(event) =>
                      onDraftChange({ ...draft, description: event.target.value })
                    }
                    placeholder={t('templates.placeholders.description')}
                  />
                </label>

                <label className="block space-y-2">
                  <span className="text-sm font-medium text-base-content/80">
                    {t('templates.fields.tags')}
                  </span>
                  <input
                    className="input input-bordered w-full"
                    value={tagsInput}
                    onChange={(event) => onTagsInputChange(event.target.value)}
                    placeholder={t('templates.placeholders.tags')}
                  />
                </label>
              </div>
            </article>

            <article className="rounded-[24px] border border-base-300 bg-base-200/50 p-5">
              <div className="flex items-center justify-between gap-3">
                <h4 className="text-lg font-semibold">{t('templates.editor.selectedSkills')}</h4>
                <span className="shrink-0 text-sm text-base-content/55">
                  {t('templates.editor.selectedCount', { count: draft.items.length })}
                </span>
              </div>

              {draft.items.length === 0 ? (
                <div className="mt-5 rounded-[20px] border border-dashed border-base-300 bg-base-100 p-5 text-sm leading-6 text-base-content/60">
                  {t('templates.editor.emptySelected')}
                </div>
              ) : (
                <div className="mt-5 max-h-[24rem] space-y-3 overflow-y-auto pr-1">
                  {draft.items.map((item, index) => {
                    const matchedSkill = repositorySkillMap.get(item.skillRef)
                    const missing = !matchedSkill

                    return (
                      <article
                        key={item.skillRef}
                        className="rounded-[20px] border border-base-300 bg-base-100 p-4"
                      >
                        <div className="flex items-start justify-between gap-4">
                          <div className="min-w-0 space-y-2">
                            <div className="flex flex-wrap items-center gap-2">
                              <span className="badge badge-outline">#{index + 1}</span>
                              <p className="truncate font-semibold">
                                {item.displayName ?? matchedSkill?.name ?? item.skillRef}
                              </p>
                            </div>
                            <p className="break-all text-xs text-base-content/55">{item.skillRef}</p>
                            <p className="text-xs text-base-content/60">
                              {matchedSkill?.sourceMarket ?? t('templates.editor.repositorySource')}
                            </p>
                            {missing ? (
                              <span className="badge badge-warning">
                                {t('templates.editor.missingSkill')}
                              </span>
                            ) : null}
                          </div>
                          <button
                            className="btn btn-sm btn-ghost text-error"
                            onClick={() => removeSkill(item.skillRef)}
                          >
                            {t('templates.editor.removeSkill')}
                          </button>
                        </div>
                      </article>
                    )
                  })}
                </div>
              )}
            </article>
          </section>

          <section>
            <article className="rounded-[24px] border border-base-300 bg-base-200/50 p-5">
              <div className="flex flex-wrap items-start justify-between gap-4">
                <div>
                  <h4 className="text-lg font-semibold">{t('templates.editor.repositorySkills')}</h4>
                  <p className="mt-1 text-sm leading-6 text-base-content/60">
                    {t('templates.editor.searchSkills')}
                  </p>
                </div>
                <span className="shrink-0 text-sm text-base-content/55">
                  {t('templates.editor.repositoryCount', { count: repositorySkills.length })}
                </span>
              </div>

              <label className="input input-bordered mt-5 flex items-center gap-2">
                <i className="hn hn-search text-base-content/50" aria-hidden />
                <input
                  className="grow"
                  value={skillQuery}
                  onChange={(event) => setSkillQuery(event.target.value)}
                  placeholder={t('templates.editor.searchSkills')}
                />
              </label>

              <div className="mt-5 max-h-[38rem] space-y-3 overflow-y-auto pr-1">
                {filteredSkills.length === 0 ? (
                  <div className="rounded-[20px] border border-dashed border-base-300 bg-base-100 p-5 text-sm leading-6 text-base-content/60">
                    {t('templates.editor.noRepositorySkills')}
                  </div>
                ) : (
                  filteredSkills.map((skill) => (
                    <article
                      key={skill.id}
                      className="rounded-[20px] border border-base-300 bg-base-100 p-4"
                    >
                      <div className="flex items-start justify-between gap-4">
                        <div className="min-w-0 space-y-2">
                          <p className="font-semibold">{skill.name}</p>
                          <p className="break-all text-xs text-base-content/55">{skill.id}</p>
                          <p className="text-sm text-base-content/60">
                            {skill.sourceMarket ?? t('templates.editor.repositorySource')}
                          </p>
                        </div>
                        <button className="btn btn-sm btn-outline" onClick={() => addSkill(skill)}>
                          {t('templates.editor.addSkill')}
                        </button>
                      </div>
                    </article>
                  ))
                )}
              </div>
            </article>
          </section>
        </div>

        <div className="flex justify-end gap-3 border-t border-base-300 px-6 py-5 md:px-7">
          <button className="btn btn-ghost" onClick={onClose}>
            {t('common.close')}
          </button>
          <button
            className="btn btn-primary"
            onClick={() => void onSave()}
            disabled={saving || draft.name.trim().length === 0}
          >
            {saving ? t('templates.saving') : t('templates.save')}
          </button>
        </div>
      </div>
    </div>
  )
}
