use anyhow::Result;
use std::path::Path;

use crate::{
    adapters::market::{GithubMarketProvider, MarketProviderAdapter, GITHUB_PROVIDER},
    domain::types::{MarketSearchRequest, MarketSearchResponse, ProviderStatus},
    repositories::market as market_repository,
};

fn enabled_provider_ids(request: &MarketSearchRequest) -> Vec<String> {
    if request.enabled_providers.is_empty() {
        return vec![GITHUB_PROVIDER.to_string()];
    }

    request.enabled_providers.clone()
}

fn search_with_providers(
    db_path: &Path,
    request: &MarketSearchRequest,
    providers: Vec<Box<dyn MarketProviderAdapter>>,
) -> Result<MarketSearchResponse> {
    let mut aggregated_results = Vec::new();
    let mut provider_statuses = Vec::new();
    let mut response_cache_hit = false;

    for provider in providers {
        if let Some(mut cached) = market_repository::load_cached_search(
            db_path,
            provider.provider_id(),
            &request.query,
            request.page.max(1),
            request.page_size.min(50),
        )? {
            response_cache_hit = true;
            aggregated_results.append(&mut cached.results);
            provider_statuses.push(ProviderStatus {
                provider: provider.provider_id().to_string(),
                status: "cached".to_string(),
                message: Some("served from SQLite cache".to_string()),
                cache_hit: true,
            });
            continue;
        }

        match provider.search(request) {
            Ok(response) => {
                market_repository::save_cached_search(
                    db_path,
                    provider.provider_id(),
                    &request.query,
                    request.page.max(1),
                    request.page_size.min(50),
                    &response,
                )?;
                aggregated_results.extend(response.results);
                provider_statuses.extend(response.providers);
            }
            Err(error) => {
                provider_statuses.push(ProviderStatus {
                    provider: provider.provider_id().to_string(),
                    status: "error".to_string(),
                    message: Some(error.to_string()),
                    cache_hit: false,
                });
            }
        }
    }

    let total = aggregated_results.len() as u32;
    Ok(MarketSearchResponse {
        results: aggregated_results,
        providers: provider_statuses,
        page: request.page.max(1),
        page_size: request.page_size.min(50),
        total,
        cache_hit: response_cache_hit,
    })
}

pub fn search_market_skills(
    db_path: &Path,
    request: &MarketSearchRequest,
) -> Result<MarketSearchResponse> {
    let mut providers: Vec<Box<dyn MarketProviderAdapter>> = Vec::new();
    let enabled = enabled_provider_ids(request);

    for provider in enabled {
        if provider == GITHUB_PROVIDER {
            providers.push(Box::<GithubMarketProvider>::default());
        } else {
            return Ok(MarketSearchResponse {
                results: Vec::new(),
                providers: vec![ProviderStatus {
                    provider,
                    status: "unavailable".to_string(),
                    message: Some("provider is not configured in this build".to_string()),
                    cache_hit: false,
                }],
                page: request.page.max(1),
                page_size: request.page_size.min(50),
                total: 0,
                cache_hit: false,
            });
        }
    }

    search_with_providers(db_path, request, providers)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::types::MarketSkillSummary;
    use crate::repositories::db::run_migrations;
    use tempfile::tempdir;

    #[derive(Default)]
    struct SuccessProvider;

    impl MarketProviderAdapter for SuccessProvider {
        fn provider_id(&self) -> &'static str {
            "success"
        }

        fn search(&self, request: &MarketSearchRequest) -> Result<MarketSearchResponse> {
            Ok(MarketSearchResponse {
                results: vec![MarketSkillSummary {
                    id: "1".into(),
                    slug: "demo-skill".into(),
                    name: format!("{} Skill", request.query),
                    description: Some("demo".into()),
                    provider: "success".into(),
                    source_type: "catalog".into(),
                    source_url: "https://example.com/demo".into(),
                    repo_url: Some("https://example.com/demo".into()),
                    download_url: Some("https://example.com/demo.git".into()),
                    package_ref: Some("demo/catalog@skill".into()),
                    manifest_path: Some("skills/demo/SKILL.md".into()),
                    skill_root: Some("skills/demo".into()),
                    version: Some("main".into()),
                    author: Some("tester".into()),
                    tags: vec!["skills".into()],
                    installable: true,
                    resolver_status: "resolved".into(),
                }],
                providers: vec![ProviderStatus {
                    provider: "success".into(),
                    status: "ok".into(),
                    message: None,
                    cache_hit: false,
                }],
                page: request.page,
                page_size: request.page_size,
                total: 1,
                cache_hit: false,
            })
        }
    }

    #[derive(Default)]
    struct FailingProvider;

    impl MarketProviderAdapter for FailingProvider {
        fn provider_id(&self) -> &'static str {
            "failure"
        }

        fn search(&self, _request: &MarketSearchRequest) -> Result<MarketSearchResponse> {
            anyhow::bail!("provider unavailable")
        }
    }

    fn request() -> MarketSearchRequest {
        MarketSearchRequest {
            query: "python".into(),
            page: 1,
            page_size: 10,
            enabled_providers: vec![],
        }
    }

    #[test]
    fn persists_provider_success_to_cache() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("market.db");
        run_migrations(&db_path).unwrap();

        let response = SuccessProvider.search(&request()).unwrap();
        market_repository::save_cached_search(&db_path, "success", "python", 1, 10, &response)
            .unwrap();

        let cached =
            market_repository::load_cached_search(&db_path, "success", "python", 1, 10).unwrap();

        assert!(cached.is_some());
        assert_eq!(cached.unwrap().results.len(), 1);
    }

    #[test]
    fn reports_provider_failure_without_cache() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("market.db");
        run_migrations(&db_path).unwrap();
        let response = search_with_providers(
            &db_path,
            &request(),
            vec![
                Box::<SuccessProvider>::default(),
                Box::<FailingProvider>::default(),
            ],
        )
        .unwrap();

        assert_eq!(response.results.len(), 1);
        assert_eq!(response.providers.len(), 2);
        assert_eq!(response.providers[1].status, "error");
        assert!(response.providers[1]
            .message
            .as_deref()
            .unwrap_or_default()
            .contains("provider unavailable"));
    }

    #[test]
    fn returns_cached_results_when_cache_hits() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("market.db");
        run_migrations(&db_path).unwrap();

        let cached_response = MarketSearchResponse {
            results: vec![MarketSkillSummary {
                id: "cached".into(),
                slug: "cached-skill".into(),
                name: "Cached Skill".into(),
                description: Some("from cache".into()),
                provider: GITHUB_PROVIDER.into(),
                source_type: "github-resolved-skill".into(),
                source_url: "https://example.com/cached".into(),
                repo_url: Some("https://example.com/cached".into()),
                download_url: None,
                package_ref: Some("cached/repo@skills/cached".into()),
                manifest_path: Some("skills/cached/SKILL.md".into()),
                skill_root: Some("skills/cached".into()),
                version: None,
                author: None,
                tags: vec!["cached".into()],
                installable: true,
                resolver_status: "resolved".into(),
            }],
            providers: vec![ProviderStatus {
                provider: GITHUB_PROVIDER.into(),
                status: "ok".into(),
                message: None,
                cache_hit: false,
            }],
            page: 1,
            page_size: 10,
            total: 1,
            cache_hit: false,
        };
        market_repository::save_cached_search(
            &db_path,
            GITHUB_PROVIDER,
            "python",
            1,
            10,
            &cached_response,
        )
        .unwrap();

        let response = search_with_providers(
            &db_path,
            &MarketSearchRequest {
                query: "python".into(),
                page: 1,
                page_size: 10,
                enabled_providers: vec![GITHUB_PROVIDER.into()],
            },
            vec![Box::<GithubMarketProvider>::default()],
        )
        .unwrap();

        assert_eq!(response.results.len(), 1);
        assert!(response.cache_hit);
        assert_eq!(response.providers[0].status, "cached");
    }
}
