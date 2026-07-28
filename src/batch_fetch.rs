use crate::browser::BrowserState;
use crate::fetch::{fetch_content_or_error, FetchCache};
use futures_util::future::join_all;
use serde::Serialize;

pub const MAX_URLS: usize = 10;
const PER_ITEM_CONTENT_LIMIT: usize = 2500;

#[derive(Debug, Clone, Serialize)]
pub struct BatchFetchItem {
    pub url: String,
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Fetches `urls` concurrently (capped at `MAX_URLS`), the same
/// "one bad URL doesn't fail the batch" pattern `smart_search` uses.
/// Unlike `smart_search`, there's no search step -- the caller already
/// has the URLs it wants.
pub async fn fetch_many(browser_state: &BrowserState, fetch_cache: &FetchCache, urls: &[String]) -> Vec<BatchFetchItem> {
    let fetch_futures = urls
        .iter()
        .take(MAX_URLS)
        .map(|url| fetch_one(browser_state.clone(), fetch_cache, url.clone()));

    join_all(fetch_futures).await
}

async fn fetch_one(browser_state: BrowserState, fetch_cache: &FetchCache, url: String) -> BatchFetchItem {
    let (content, error) = fetch_content_or_error(&browser_state, fetch_cache, &url, PER_ITEM_CONTENT_LIMIT).await;
    BatchFetchItem { url, content, error }
}
