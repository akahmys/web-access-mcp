use crate::cache::TtlCache;
use crate::user_agent::user_agent;
use serde::Deserialize;
use std::time::Duration;
use serde::Serialize;
use thiserror::Error;

pub type SearchCache = TtlCache<Vec<SearchResult>>;

#[derive(Debug, Clone, Serialize)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub snippet: String,
}

/// Domain errors for `web_search`. As in `FetchError`, every variant
/// carries an already-formatted message (no `#[source]`) so propagating
/// this through `?` into `anyhow::Result` and later printing it with
/// `{:#}` at the MCP boundary doesn't repeat the underlying cause.
#[derive(Debug, Error)]
pub enum SearchError {
    #[error("Failed to build HTTP client: {0}")]
    ClientBuild(String),

    #[error("Failed to retrieve search results from Bing: {0}. Hint: this may be a transient network/rate-limit issue -- wait a moment and retry, rephrase the query, or use web_fetch directly if you already know a candidate URL.")]
    RequestFailed(String),
}

/// Bing's RSS search feed. `format=rss` is a documented Bing output mode
/// that returns clean, structured XML instead of the JS-rendered HTML
/// `bing.com/search` normally serves -- far less brittle than scraping
/// CSS classes out of a search results page.
#[derive(Debug, Deserialize)]
struct BingRss {
    channel: BingChannel,
}

#[derive(Debug, Deserialize)]
struct BingChannel {
    #[serde(rename = "item", default)]
    items: Vec<BingItem>,
}

#[derive(Debug, Deserialize)]
struct BingItem {
    title: String,
    link: String,
    #[serde(default)]
    description: String,
}

pub trait SearchProvider: Send + Sync {
    fn search(&self, query: &str) -> impl std::future::Future<Output = Result<Vec<SearchResult>, SearchError>> + Send;
}

#[derive(Default, Debug, Clone)]
pub struct BingSearchProvider;

impl SearchProvider for BingSearchProvider {
    async fn search(&self, query: &str) -> Result<Vec<SearchResult>, SearchError> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .map_err(|e| SearchError::ClientBuild(e.to_string()))?;

        fetch_bing_results(&client, query).await
    }
}

pub async fn perform_web_search<P: SearchProvider + ?Sized>(
    provider: &P,
    cache: &SearchCache,
    query: &str,
) -> Result<Vec<SearchResult>, SearchError> {
    if let Some(cached) = cache.get(query) {
        return Ok(cached);
    }

    let results = provider.search(query).await?;
    cache.set(query.to_string(), results.clone());
    Ok(results)
}

async fn fetch_bing_results(client: &reqwest::Client, query: &str) -> Result<Vec<SearchResult>, SearchError> {
    let url = reqwest::Url::parse_with_params("https://www.bing.com/search", &[("q", query), ("format", "rss")])
        .map_err(|e| SearchError::RequestFailed(format!("failed to parse search URL: {e}")))?;
    let response = client
        .get(url)
        .header("User-Agent", user_agent())
        .header("Accept-Language", "ja,en-US;q=0.9,en;q=0.8")
        .send()
        .await
        .map_err(|e| SearchError::RequestFailed(format!("request failed: {e}")))?;

    let status = response.status();
    if !status.is_success() {
        return Err(SearchError::RequestFailed(format!("HTTP {status}")));
    }

    let xml = response
        .text()
        .await
        .map_err(|e| SearchError::RequestFailed(format!("failed to read response body: {e}")))?;

    parse_bing_rss(&xml)
}

fn parse_bing_rss(xml: &str) -> Result<Vec<SearchResult>, SearchError> {
    let feed: BingRss = quick_xml::de::from_str(xml)
        .map_err(|e| SearchError::RequestFailed(format!("parse error: {e} (page layout may have changed or a block page was served)")))?;

    Ok(feed
        .channel
        .items
        .into_iter()
        .map(|item| SearchResult {
            title: item.title,
            url: item.link,
            snippet: item.description,
        })
        .collect())
}

#[cfg(test)]
mod tests;
