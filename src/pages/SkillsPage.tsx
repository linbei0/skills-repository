import { useEffect } from 'react'
import { useTranslation } from 'react-i18next'
import { useSkillsStore } from '../stores/use-skills-store'

const agentTabs = [
  { id: 'antigravity', label: 'Antigravity' },
  { id: 'claude-code', label: 'Claude Code' },
  { id: 'codebuddy', label: 'CodeBuddy' },
  { id: 'codex', label: 'Codex' },
  { id: 'cursor', label: 'Cursor' },
  { id: 'kiro', label: 'Kiro' },
  { id: 'openclaw', label: 'OpenClaw' },
  { id: 'opencode', label: 'OpenCode' },
  { id: 'qoder', label: 'Qoder' },
  { id: 'trae', label: 'Trae' },
  { id: 'vscode', label: 'VSCode' },
  { id: 'windsurf', label: 'Windsurf' },
] as const

export function SkillsPage() {
  const { t } = useTranslation()
  const selectedAgentId = useSkillsStore((state) => state.selectedAgentId)
  const loading = useSkillsStore((state) => state.loading)
  const loaded = useSkillsStore((state) => state.loaded)
  const error = useSkillsStore((state) => state.error)
  const rootPath = useSkillsStore((state) => state.rootPath)
  const entries = useSkillsStore((state) => state.entries)
  const setSelectedAgentId = useSkillsStore((state) => state.setSelectedAgentId)
  const scanAgentGlobalSkills = useSkillsStore((state) => state.scanAgentGlobalSkills)

  useEffect(() => {
    void scanAgentGlobalSkills(selectedAgentId)
  }, [scanAgentGlobalSkills, selectedAgentId])

  return (
    <div className="space-y-6">
      <section className="rounded-box border border-base-300 bg-base-100 p-6">
        <h2 className="text-3xl font-semibold">{t('skills.title')}</h2>
        <p className="mt-3 max-w-3xl text-sm text-base-content/65">{t('skills.description')}</p>
      </section>

      <section className="rounded-box border border-base-300 bg-base-100 p-6">
        <div className="flex flex-wrap gap-3">
          {agentTabs.map((tab) => (
            <button
              key={tab.id}
              className={selectedAgentId === tab.id ? 'btn btn-sm btn-primary' : 'btn btn-sm btn-outline'}
              onClick={() => setSelectedAgentId(tab.id)}
            >
              {tab.label}
            </button>
          ))}
        </div>

        {rootPath ? (
          <p className="mt-4 break-all text-sm text-base-content/60">{rootPath}</p>
        ) : null}
      </section>

      <section className="space-y-4">
        {loading ? (
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
                    {agentTabs.find((tab) => tab.id === selectedAgentId)?.label} · {t(`skills.relationshipValues.${entry.relationship}`)}
                  </p>
                  <p className="mt-3 break-all text-sm text-base-content/50">{entry.path}</p>
                </div>
              </div>
            </article>
          ))
        )}
      </section>
    </div>
  )
}
