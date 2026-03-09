use anyhow::{anyhow, Context, Result};
use serde_json::Value;

use crate::domain::types::{
    MarketSearchRequest, MarketSearchResponse, MarketSkillSummary, ProviderStatus,
};

pub const GITHUB_PROVIDER: &str = "github";

pub trait MarketProviderAdapter: Send + Sync {
    fn provider_id(&self) -> &'static str;
    fn search(&self, request: &MarketSearchRequest) -> Result<MarketSearchResponse>;
}

#[derive(Debug, Default, Clone)]
pub struct GithubMarketProvider;

impl GithubMarketProvider {
    fn build_search_url(&self, request: &MarketSearchRequest) -> String {
        let query = if request.query.trim().is_empty() {
            "skills".to_string()
        } else {
            format!("{} skills", request.query.trim())
        };

        format!(
            "https://api.github.com/search/repositories?q={}&sort=stars&order=desc&per_page={}&page={}",
            urlencoding::encode(&query),
            request.page_size.min(50),
            request.page.max(1)
        )
    }

    fn parse_repository(item: &Value) -> Option<MarketSkillSummary> {
        let source_url = item.get("html_url")?.as_str()?.to_string();
        let default_branch = item
            .get("default_branch")
            .and_then(Value::as_str)
            .unwrap_or("main")
            .to_string();

        Some(MarketSkillSummary {
            id: item.get("id")?.as_i64()?.to_string(),
            slug: item
                .get("full_name")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .replace('/', "-"),
            name: item.get("name")?.as_str()?.to_string(),
            description: item
                .get("description")
                .and_then(Value::as_str)
                .map(ToString::to_string),
            provider: GITHUB_PROVIDER.to_string(),
            source_url: source_url.clone(),
            download_url: Some(format!(
                "{}/archive/refs/heads/{}.zip",
                source_url, default_branch
            )),
            version: Some(default_branch),
            author: item
                .get("owner")
                .and_then(|owner| owner.get("login"))
                .and_then(Value::as_str)
                .map(ToString::to_string),
            tags: item
                .get("topics")
                .and_then(Value::as_array)
                .map(|topics| {
                    topics
                        .iter()
                        .filter_map(Value::as_str)
                        .map(ToString::to_string)
                        .collect()
                })
                .unwrap_or_default(),
        })
    }
}

impl MarketProviderAdapter for GithubMarketProvider {
    fn provider_id(&self) -> &'static str {
        GITHUB_PROVIDER
    }

    fn search(&self, request: &MarketSearchRequest) -> Result<MarketSearchResponse> {
        let response = ureq::get(&self.build_search_url(request))
            .set("User-Agent", "skills-manager/0.1.0")
            .set("Accept", "application/vnd.github+json")
            .call()
            .map_err(|error| anyhow!("github provider request failed: {}", error))?;

        let body = response
            .into_string()
            .context("failed to read github provider response body")?;
        let payload: Value =
            serde_json::from_str(&body).context("failed to parse github provider response")?;

        let results = payload
            .get("items")
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(Self::parse_repository)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        let total = payload
            .get("total_count")
            .and_then(Value::as_u64)
            .unwrap_or(results.len() as u64) as u32;

        Ok(MarketSearchResponse {
            results,
            providers: vec![ProviderStatus {
                provider: self.provider_id().to_string(),
                status: "ok".to_string(),
                message: None,
                cache_hit: false,
            }],
            page: request.page.max(1),
            page_size: request.page_size.min(50),
            total,
            cache_hit: false,
        })
    }
}
