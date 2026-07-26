use crate::browser::BrowserState;
use crate::fetch::{fetch_url, truncate_content};
use crate::search::{perform_google_search, SearchCache};
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

    let fetch_futures = target_items.into_iter().map(|item| {
        let browser_state = browser_state.clone();
        async move {
            let title = item.title;
            let url = item.url;
            let snippet = item.snippet;

            // Attempt to fetch content. fetch_url only holds the shared
            // browser lock briefly (to open a page), so these futures run
            // truly concurrently under join_all instead of serializing on
            // a single browser mutex for the whole page load.
            let fetched_content = match fetch_url(&browser_state, &url).await {
                Ok(res) => Some(truncate_content(&res.content, 2500)),
                Err(e) => {
                    warn!("Failed to fetch content for {}: {:?}", url, e);
                    None
                }
            };

            SmartSearchItem {
                title,
                url,
                snippet,
                content: fetched_content,
            }
        }
    });

    let items = join_all(fetch_futures).await;
    let fetched_pages = items.iter().filter(|i| i.content.is_some()).count();

    Ok(SmartSearchResult {
        query: query.to_string(),
        total_found,
        fetched_pages,
        items,
    })
}
