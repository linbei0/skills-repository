import { useTranslation } from 'react-i18next'
import { useMemo } from 'react'
import { useAppStore } from '../stores/use-app-store'
import { useSkillsStore } from '../stores/use-skills-store'

export function SkillsPage() {
  const { t } = useTranslation()
  const skills = useSkillsStore((state) => state.skills)
  const distributions = useSkillsStore((state) => state.distributions)
  const distributeSkill = useSkillsStore((state) => state.distributeSkill)
  const agents = useAppStore((state) => state.agents)

  const distributionsBySkillId = useMemo(
    () =>
      distributions.reduce<Record<string, typeof distributions>>((accumulator, distribution) => {
        accumulator[distribution.skillId] = accumulator[distribution.skillId] ?? []
        accumulator[distribution.skillId].push(distribution)
        return accumulator
      }, {}),
    [distributions],
  )

  return (
    <div className="space-y-6">
      <section className="rounded-box border border-base-300 bg-base-100 p-6">
        <h2 className="text-3xl font-semibold">{t('skills.title')}</h2>
        <p className="mt-3 max-w-3xl text-sm text-base-content/65">{t('skills.description')}</p>
      </section>

      <section className="overflow-hidden rounded-box border border-base-300 bg-base-100">
        {skills.length === 0 ? (
          <div className="p-6 text-sm text-base-content/60">{t('skills.empty')}</div>
        ) : (
          <div className="overflow-x-auto">
            <table className="table">
              <thead>
                <tr>
                  <th>{t('common.agent')}</th>
                  <th>{t('common.name')}</th>
                  <th>{t('common.scope')}</th>
                  <th>{t('skills.distributionStatus')}</th>
                  <th>{t('common.path')}</th>
                  <th>{t('skills.actions')}</th>
                </tr>
              </thead>
              <tbody>
                {skills.map((skill) => {
                  const skillDistributions = distributionsBySkillId[skill.id] ?? []
                  const distributionStatus = skillDistributions.some(
                    (distribution) => distribution.status === 'failed',
                  )
                    ? 'failed'
                    : skillDistributions.some((distribution) => distribution.status === 'active')
                      ? 'active'
                      : 'notDistributed'
                  const preferredAgent = agents.find(
                    (agent) => agent.label === skill.agent.primary,
                  )
                  const targetKind = skill.scope === 'project' ? 'project' : 'global'
                  const installMode =
                    targetKind === 'project'
                      ? preferredAgent?.defaultProjectMode ?? 'copy'
                      : preferredAgent?.defaultGlobalMode ?? 'symlink'
                  const canDistribute = skill.scope !== 'custom'

                  return (
                  <tr key={skill.id}>
                    <td className="min-w-[12rem]">
                      <p className="font-medium">{skill.agent.primary}</p>
                      {skill.agent.compatibleAgents.length > 0 ? (
                        <p className="mt-1 text-xs text-base-content/55">
                          {t('skills.sharedPath', {
                            agents: skill.agent.compatibleAgents.join(', '),
                          })}
                        </p>
                      ) : null}
                      {skill.agent.aliases.length > 0 ? (
                        <p className="mt-1 text-xs text-base-content/55">
                          {t('skills.aliases', {
                            agents: skill.agent.aliases.join(', '),
                          })}
                        </p>
                      ) : null}
                      <p className="mt-1 text-xs text-base-content/45">
                        {t('skills.priority', { value: skill.agent.priority })}
                      </p>
                    </td>
                    <td>{skill.name}</td>
                    <td>{t(`common.scopeValues.${skill.scope}`)}</td>
                    <td className="min-w-[12rem]">
                      <p>{t(`skills.statusValues.${distributionStatus}`)}</p>
                      <p className="mt-1 text-xs text-base-content/55">
                        {skillDistributions.length > 0
                          ? t('skills.distributionCount', { count: skillDistributions.length })
                          : t('skills.distributionEmpty')}
                      </p>
                      {skillDistributions.slice(0, 2).map((distribution) => (
                        <p key={distribution.id} className="mt-1 text-xs text-base-content/45">
                          {distribution.targetAgent}: {distribution.targetPath}
                        </p>
                      ))}
                    </td>
                    <td className="max-w-[20rem] truncate font-mono text-xs">{skill.path}</td>
                    <td>
                      <button
                        className="btn btn-sm btn-outline"
                        disabled={!canDistribute}
                        title={
                          canDistribute ? t('skills.distribute') : t('skills.distributionUnavailable')
                        }
                        onClick={() =>
                          void distributeSkill({
                            skillId: skill.id,
                            targetKind,
                            targetAgent: skill.agent.primary,
                            installMode,
                            projectRoot: skill.projectRoot ?? null,
                            customTargetPath: null,
                          })
                        }
                      >
                        {t('skills.distribute')}
                      </button>
                    </td>
                  </tr>
                  )
                })}
              </tbody>
            </table>
          </div>
        )}
      </section>
    </div>
  )
}
