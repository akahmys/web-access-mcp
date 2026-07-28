use super::{fetch_url, truncate_content, FetchCache};
use crate::browser::BrowserState;

/// Fetches `url` and truncates its content to `max_len`, returning
/// `(Some(content), None)` on success or `(None, Some(explanation))` on
/// failure. Shared by `smart_search` and `batch_fetch`, which both fetch
/// several URLs concurrently and can't let one bad URL fail the batch.
pub async fn fetch_content_or_error(
    browser_state: &BrowserState,
    fetch_cache: &FetchCache,
    url: &str,
    max_len: usize,
) -> (Option<String>, Option<String>) {
    match fetch_url(browser_state, fetch_cache, url, &[]).await {
        Ok(res) => (Some(truncate_content(&res.content, max_len)), None),
        Err(e) => {
            tracing::warn!("Failed to fetch content for {}: {}", url, e);
            (None, Some(format!("Content unavailable for this result: {e}")))
        }
    }
}
