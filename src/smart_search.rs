use crate::browser::BrowserState;
use crate::fetch::{fetch_url, truncate_content};
use crate::search::{perform_google_search, SearchCache, SearchResult};
use anyhow::Result;
use futures_util::future::join_all;
use serde::Serialize;
use tracing::{info, warn};

#[derive(Debug, Clone, Serialize)]
pub struct SmartSearchItem {
    pub title: String,
    pub url: String,
    pub snippet: String,
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SmartSearchResult {
    pub query: String,
    pub total_found: usize,
    pub fetched_pages: usize,
    pub items: Vec<SmartSearchItem>,
}

pub async fn perform_smart_search(
    browser_state: &BrowserState,
    search_cache: &SearchCache,
    query: &str,
    max_pages: usize,
) -> Result<SmartSearchResult> {
    info!("Performing smart_search for query: '{}' (max_pages: {})", query, max_pages);

    let search_results = perform_google_search(browser_state, search_cache, query).await?;
    let total_found = search_results.len();

    let target_items: Vec<_> = search_results.into_iter().take(max_pages).collect();

    // fetch_url only holds the shared browser lock briefly (to open a
    // page), so these futures run truly concurrently under join_all
    // instead of serializing on a single browser mutex for the whole
    // page load.
    let fetch_futures = target_items
        .into_iter()
        .map(|item| fetch_one_item(browser_state.clone(), item));

    let items = join_all(fetch_futures).await;
    let fetched_pages = items.iter().filter(|i| i.content.is_some()).count();

    Ok(SmartSearchResult {
        query: query.to_string(),
        total_found,
        fetched_pages,
        items,
    })
}

/// Fetches Markdown content for a single search result. Failures don't
/// propagate to the caller -- the item just carries an `error` explanation
/// with a hint instead of `content`, so one bad page doesn't fail the
/// whole `smart_search` call.
async fn fetch_one_item(browser_state: BrowserState, item: SearchResult) -> SmartSearchItem {
    let SearchResult { title, url, snippet } = item;

    let (content, error) = match fetch_url(&browser_state, &url).await {
        Ok(res) => (Some(truncate_content(&res.content, 2500)), None),
        Err(e) => {
            // FetchError's message already contains its own hint, so it's
            // used verbatim rather than nested inside another hint.
            warn!("Failed to fetch content for {}: {}", url, e);
            (None, Some(format!("Content unavailable for this result: {e}")))
        }
    };

    SmartSearchItem { title, url, snippet, content, error }
}
