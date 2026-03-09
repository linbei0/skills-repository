import { useTranslation } from 'react-i18next'

export function SecurityPage() {
  const { t } = useTranslation()
  const statusCards = [
    { key: 'safe', accent: 'border-success/30 bg-success/5 text-success' },
    { key: 'medium', accent: 'border-warning/30 bg-warning/5 text-warning' },
    { key: 'blocked', accent: 'border-error/30 bg-error/5 text-error' },
  ] as const

  return (
    <div className="space-y-6">
      <section className="rounded-box border border-base-300 bg-base-100 p-6">
        <h2 className="text-3xl font-semibold">{t('security.title')}</h2>
        <p className="mt-3 max-w-3xl text-sm text-base-content/65">{t('security.description')}</p>
      </section>

      <section className="grid gap-4 md:grid-cols-3">
        {statusCards.map((card) => (
          <div key={card.key} className="rounded-box border border-base-300 bg-base-100 p-5">
            <div className="flex items-start justify-between gap-3">
              <p className="text-sm uppercase tracking-[0.2em] text-base-content/50">
                {t(`security.cards.${card.key}.label`)}
              </p>
              <span className={`badge badge-outline ${card.accent}`}>
                {t('common.comingSoon')}
              </span>
            </div>
            <p className="mt-4 text-lg font-semibold">
              {t(`security.cards.${card.key}.title`)}
            </p>
            <p className="mt-2 text-sm leading-6 text-base-content/60">
              {t(`security.cards.${card.key}.description`)}
            </p>
          </div>
        ))}
      </section>

      <section className="rounded-box border border-dashed border-warning/40 bg-warning/5 p-5 text-sm leading-6 text-base-content/70">
        <p className="font-medium text-base-content">{t('security.notConnectedTitle')}</p>
        <p className="mt-2">{t('security.notConnectedDescription')}</p>
      </section>
    </div>
  )
}
