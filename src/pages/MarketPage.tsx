import { useTranslation } from 'react-i18next'
import { useMarketStore } from '../stores/use-market-store'

const resolveInstallStateTone = (status: 'installed' | 'blocked' | 'failed') => {
  if (status === 'installed') return 'border-success/30 bg-success/5 text-success'
  if (status === 'blocked') return 'border-error/30 bg-error/5 text-error'
  return 'border-warning/30 bg-warning/5 text-warning'
}

export function MarketPage() {
  const { t } = useTranslation()
  const query = useMarketStore((state) => state.query)
  const loading = useMarketStore((state) => state.loading)
  const searched = useMarketStore((state) => state.searched)
  const error = useMarketStore((state) => state.error)
  const results = useMarketStore((state) => state.results)
  const providers = useMarketStore((state) => state.providers)
  const cacheHit = useMarketStore((state) => state.cacheHit)
  const total = useMarketStore((state) => state.total)
  const installStates = useMarketStore((state) => state.installStates)
  const setQuery = useMarketStore((state) => state.setQuery)
  const search = useMarketStore((state) => state.search)
  const install = useMarketStore((state) => state.install)

  return (
    <div className="space-y-6">
      <section className="rounded-box border border-base-300 bg-base-100 p-6">
        <h2 className="text-3xl font-semibold">{t('market.title')}</h2>
        <p className="mt-3 max-w-3xl text-sm text-base-content/65">{t('market.description')}</p>
      </section>

      <section className="rounded-box border border-base-300 bg-base-100 p-6">
        <form
          className="flex flex-col gap-4 md:flex-row"
          onSubmit={(event) => {
            event.preventDefault()
            void search()
          }}
        >
          <label className="input input-bordered flex flex-1 items-center gap-2">
            <i className="hn hn-search text-base-content/50" aria-hidden />
            <input
              type="text"
              className="grow"
              value={query}
              onChange={(event) => setQuery(event.target.value)}
              placeholder={t('market.searchPlaceholder')}
              aria-label={t('market.searchPlaceholder')}
            />
          </label>
          <button className="btn btn-primary md:min-w-32" type="submit" disabled={loading}>
            {loading ? t('market.searching') : t('market.search')}
          </button>
        </form>

        <p className="mt-3 text-sm text-base-content/60">{t('market.helper')}</p>
        <p className="mt-2 text-xs text-base-content/50">{t('market.defaultInstallHint')}</p>
      </section>

      <section className="grid gap-4 xl:grid-cols-[1.1fr_1.9fr]">
        <div className="rounded-box border border-base-300 bg-base-100 p-6">
          <div className="flex items-center justify-between gap-3">
            <h3 className="text-lg font-semibold">{t('market.providersTitle')}</h3>
            {cacheHit ? <span className="badge badge-info">{t('market.cacheHit')}</span> : null}
          </div>
          <div className="mt-4 space-y-3">
            {providers.length === 0 ? (
              <div className="rounded-box border border-dashed border-base-300 bg-base-200/60 p-4 text-sm text-base-content/60">
                {t('market.providersEmpty')}
              </div>
            ) : (
              providers.map((provider) => (
                <article key={provider.provider} className="rounded-box border border-base-300 bg-base-200/60 p-4">
                  <div className="flex items-center justify-between gap-3">
                    <p className="font-medium">{provider.provider}</p>
                    <span className="badge badge-outline">{t(`market.providerStatuses.${provider.status}`)}</span>
                  </div>
                  <p className="mt-2 text-sm text-base-content/60">
                    {provider.message ?? t('market.providerReady')}
                  </p>
                </article>
              ))
            )}
          </div>
        </div>

        <div className="rounded-box border border-base-300 bg-base-100 p-6">
          <div className="flex items-center justify-between gap-3">
            <h3 className="text-lg font-semibold">{t('market.resultsTitle')}</h3>
            {searched ? (
              <span className="text-sm text-base-content/60">
                {t('market.resultsCount', { count: total })}
              </span>
            ) : null}
          </div>

          {error ? (
            <div className="mt-4 rounded-box border border-error/30 bg-error/5 p-4 text-sm text-error">
              {error}
            </div>
          ) : null}

          {!searched && !loading ? (
            <div className="mt-4 rounded-box border border-dashed border-base-300 bg-base-200/60 p-4 text-sm text-base-content/60">
              {t('market.idle')}
            </div>
          ) : null}

          {searched && results.length === 0 && !error ? (
            <div className="mt-4 rounded-box border border-dashed border-base-300 bg-base-200/60 p-4 text-sm text-base-content/60">
              {t('market.empty')}
            </div>
          ) : null}

          <div className="mt-4 space-y-4">
            {results.map((item) => {
              const installState = installStates[item.id]
              const isInstalling = installState?.status === 'installing'
              const isInstalled = installState?.status === 'installed'
              const installDisabled = isInstalling || isInstalled || !item.installable

              return (
                <article key={item.id} className="rounded-box border border-base-300 bg-base-200/60 p-4">
                  <div className="flex items-start justify-between gap-3">
                    <div>
                      <div className="flex flex-wrap items-center gap-2">
                        <p className="text-lg font-semibold">{item.name}</p>
                        <span className="badge badge-outline">{item.provider}</span>
                      </div>
                      <p className="mt-2 text-sm text-base-content/60">
                        {item.description ?? t('market.noDescription')}
                      </p>
                    </div>
                    <a
                      className="btn btn-sm btn-outline"
                      href={item.sourceUrl}
                      target="_blank"
                      rel="noreferrer"
                    >
                      {t('market.openSource')}
                    </a>
                  </div>

                  <div className="mt-4 flex flex-wrap gap-2 text-xs text-base-content/55">
                    {item.author ? <span>{t('market.author', { author: item.author })}</span> : null}
                    {item.version ? <span>{t('market.version', { version: item.version })}</span> : null}
                    {item.skillRoot ? <span>{t('market.skillRoot', { path: item.skillRoot })}</span> : null}
                  </div>

                  <div className="mt-4 flex flex-wrap items-center gap-3">
                    <button
                      className="btn btn-sm btn-primary"
                      disabled={installDisabled}
                      onClick={() => void install(item)}
                    >
                      {isInstalling
                        ? t('market.installing')
                        : isInstalled
                          ? t('market.installed')
                          : t('market.install')}
                    </button>
                    <span className="text-xs text-base-content/55">{t('market.installHint')}</span>
                  </div>

                  {item.packageRef ? (
                    <p className="mt-3 break-all text-xs text-base-content/55">
                      {t('market.packageRef', { ref: item.packageRef })}
                    </p>
                  ) : null}

                  {installState && installState.status !== 'installing' ? (
                    <div
                      className={`mt-4 rounded-box border p-3 text-sm ${resolveInstallStateTone(
                        installState.status,
                      )}`}
                    >
                      {installState.status === 'installed' ? (
                        <div className="space-y-1">
                          <p>{t('market.installSuccess')}</p>
                          {installState.canonicalPath ? (
                            <p className="break-all text-xs opacity-80">{installState.canonicalPath}</p>
                          ) : null}
                        </div>
                      ) : installState.status === 'blocked' ? (
                        <p>
                          {t('market.installBlocked', {
                            level: installState.securityLevel
                              ? t(`security.levels.${installState.securityLevel}`)
                              : installState.message,
                          })}
                        </p>
                      ) : (
                        <p>{t('market.installFailed', { message: installState.message })}</p>
                      )}
                    </div>
                  ) : null}
                </article>
              )
            })}
          </div>
        </div>
      </section>
    </div>
  )
}
