use crate::browser::BrowserState;
use scraper::{ElementRef, Html, Selector};
use dashmap::DashMap;
use std::time::{Duration, Instant};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Clone, Serialize)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub snippet: String,
}

/// Domain errors for `google_search`/`smart_search`. As in `FetchError`,
/// every variant carries an already-formatted message (no `#[source]`) so
/// propagating this through `?` into `anyhow::Result` and later printing it
/// with `{:#}` at the MCP boundary doesn't repeat the underlying cause.
#[derive(Debug, Error)]
pub enum SearchError {
    #[error("Failed to build HTTP client: {0}")]
    ClientBuild(String),

    #[error("Selector error: {0}")]
    Selector(String),

    #[error("Failed to retrieve search results: DuckDuckGo ({ddg}), Google ({google}). Hint: both search backends failed (likely rate-limited, network issue, or a block/CAPTCHA page) -- wait a moment and retry, try rephrasing the query, or use web_fetch directly if you already know a candidate URL.")]
    BothBackendsFailed { ddg: String, google: String },
}

pub struct SearchCache {
    cache: DashMap<String, (Vec<SearchResult>, Instant)>,
    ttl: Duration,
}

impl SearchCache {
    pub fn new(ttl: Duration) -> Self {
        Self {
            cache: DashMap::new(),
            ttl,
        }
    }

    pub fn get(&self, query: &str) -> Option<Vec<SearchResult>> {
        if let Some(entry) = self.cache.get(query) {
            let (results, expires_at) = entry.value();
            if *expires_at > Instant::now() {
                return Some(results.clone());
            }
        }
        None
    }

    pub fn set(&self, query: String, results: Vec<SearchResult>) {
        self.cache.insert(query, (results, Instant::now() + self.ttl));
    }
}

pub async fn perform_google_search(
    _browser_state: &BrowserState,
    cache: &SearchCache,
    query: &str,
) -> Result<Vec<SearchResult>, SearchError> {
    if let Some(cached) = cache.get(query) {
        return Ok(cached);
    }

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| SearchError::ClientBuild(e.to_string()))?;

    let results = search_with_fallback(&client, query).await?;
    cache.set(query.to_string(), results.clone());
    Ok(results)
}

/// Tries `DuckDuckGo`'s HTML search first (fast, rarely blocked); falls back
/// to a Google HTML search if `DuckDuckGo` fails or returns zero results.
async fn search_with_fallback(client: &reqwest::Client, query: &str) -> Result<Vec<SearchResult>, SearchError> {
    let ddg_url = format!("https://html.duckduckgo.com/html/?q={}", urlencoding::encode(query));
    match fetch_and_parse(client, &ddg_url, parse_ddg_results).await {
        Ok(results) => Ok(results),
        Err(ddg_reason) => {
            let google_url = format!("https://www.google.com/search?q={}", urlencoding::encode(query));
            fetch_and_parse(client, &google_url, parse_search_results)
                .await
                .map_err(|google_reason| SearchError::BothBackendsFailed { ddg: ddg_reason, google: google_reason })
        }
    }
}

/// Fetches `url` and runs `parser` over the body, returning a descriptive
/// error string (rather than swallowing it) so callers can report *why*
/// a search backend failed: network error, non-2xx status, or a parse
/// that produced zero results (most likely the page layout changed or we
/// were served a block/CAPTCHA page).
async fn fetch_and_parse(
    client: &reqwest::Client,
    url: &str,
    parser: fn(&str) -> Result<Vec<SearchResult>, SearchError>,
) -> Result<Vec<SearchResult>, String> {
    let response = client
        .get(url)
        .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36")
        .header("Accept-Language", "ja,en-US;q=0.9,en;q=0.8")
        .send()
        .await
        .map_err(|e| format!("request failed: {e}"))?;

    let status = response.status();
    if !status.is_success() {
        return Err(format!("HTTP {status}"));
    }

    let html = response
        .text()
        .await
        .map_err(|e| format!("failed to read response body: {e}"))?;

    let results = parser(&html).map_err(|e| format!("parse error: {e}"))?;
    if results.is_empty() {
        return Err("parsed 0 results (page layout changed or a block/CAPTCHA page was served)".to_string());
    }

    Ok(results)
}

fn parse_ddg_results(html_content: &str) -> Result<Vec<SearchResult>, SearchError> {
    let document = Html::parse_document(html_content);
    let result_selector = Selector::parse(".result").map_err(|e| SearchError::Selector(e.to_string()))?;
    let title_selector = Selector::parse(".result__a, .result__title").map_err(|e| SearchError::Selector(e.to_string()))?;
    let link_selector = Selector::parse("a.result__a, a.result__url").map_err(|e| SearchError::Selector(e.to_string()))?;
    let snippet_selector = Selector::parse(".result__snippet").map_err(|e| SearchError::Selector(e.to_string()))?;

    let mut results = Vec::new();
    for element in document.select(&result_selector) {
        let Some(result) = extract_ddg_result(&element, &title_selector, &link_selector, &snippet_selector) else {
            continue;
        };

        results.push(result);
        if results.len() >= 5 {
            break;
        }
    }

    Ok(results)
}

fn extract_ddg_result(
    element: &ElementRef,
    title_selector: &Selector,
    link_selector: &Selector,
    snippet_selector: &Selector,
) -> Option<SearchResult> {
    let title = element
        .select(title_selector)
        .next()
        .map(|el| el.text().collect::<Vec<_>>().join("").trim().to_string())
        .unwrap_or_default();

    let link = element
        .select(link_selector)
        .next()
        .and_then(|el| el.value().attr("href").map(std::string::ToString::to_string))
        .unwrap_or_default();

    let actual_link = resolve_ddg_redirect(link);

    let snippet = element
        .select(snippet_selector)
        .next()
        .map(|el| el.text().collect::<Vec<_>>().join("").trim().to_string())
        .unwrap_or_default();

    if title.is_empty() || !actual_link.starts_with("http") {
        return None;
    }

    Some(SearchResult { title, url: actual_link, snippet })
}

/// `DuckDuckGo`'s HTML search wraps result links in a `/l/?uddg=...`
/// redirect; unwrap it to the real destination URL when present.
fn resolve_ddg_redirect(link: String) -> String {
    if !link.contains("uddg=") {
        return link;
    }
    link.split("uddg=")
        .nth(1)
        .and_then(|s| s.split('&').next())
        .and_then(|s| urlencoding::decode(s).ok().map(std::borrow::Cow::into_owned))
        .unwrap_or(link)
}

fn parse_search_results(html_content: &str) -> Result<Vec<SearchResult>, SearchError> {
    let document = Html::parse_document(html_content);
    let mut results = Vec::new();

    let result_selector = Selector::parse("div.g").map_err(|e| SearchError::Selector(e.to_string()))?;
    let title_selector = Selector::parse("h3").map_err(|e| SearchError::Selector(e.to_string()))?;
    let link_selector = Selector::parse("a").map_err(|e| SearchError::Selector(e.to_string()))?;
    let snippet_selector = Selector::parse(".VwiAwd, .ST93db, .kb139e").map_err(|e| SearchError::Selector(e.to_string()))?;

    for element in document.select(&result_selector) {
        let title = element
            .select(&title_selector)
            .next().map_or_else(|| "No Title".to_string(), |el| el.text().collect::<Vec<_>>().join(""));

        let link = element
            .select(&link_selector)
            .next()
            .and_then(|el| el.value().attr("href").map(std::string::ToString::to_string))
            .unwrap_or_else(|| "No Link".to_string());

        let snippet = element
            .select(&snippet_selector)
            .next()
            .map(|el| el.text().collect::<Vec<_>>().join(""))
            .unwrap_or_default();

        if !title.is_empty() && link.starts_with("http") {
            results.push(SearchResult {
                title,
                url: link,
                snippet,
            });
        }
    }

    Ok(results)
}

#[cfg(test)]
mod tests;
