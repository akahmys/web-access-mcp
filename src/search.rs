use crate::browser::BrowserState;
use anyhow::{anyhow, Result};
use scraper::{Html, Selector};
use dashmap::DashMap;
use std::time::{Duration, Instant};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub snippet: String,
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
    browser_state: &BrowserState,
    cache: &SearchCache,
    query: &str,
) -> Result<Vec<SearchResult>> {
    if let Some(cached) = cache.get(query) {
        return Ok(cached);
    }

    let browser = browser_state
        .get_browser()
        .await
        .ok_or_else(|| anyhow!("Browser is not running"))?;

    let page = browser.lock().await.new_page(chromiumoxide::cdp::browser_protocol::target::CreateTargetParams::default()).await?;
    let url = format!("https://www.google.com/search?q={}", urlencoding::encode(query));
    
    page.goto(&url).await?;
    
    // Wait for the search results to be present in the DOM.
    tokio::time::sleep(Duration::from_secs(2)).await;

    let html_content: String = page.content().await?;
    
    // CAPTCHA / Block Detection
    if html_content.contains("google.com/sorry/") 
        || html_content.contains("unusual traffic from your computer network")
        || html_content.contains("captcha-form")
    {
        return Err(anyhow!("Google Search blocked by CAPTCHA/unusual traffic detection."));
    }

    let results = parse_search_results(&html_content)?;

    if !results.is_empty() {
        cache.set(query.to_string(), results.clone());
    }

    Ok(results)
}

fn parse_search_results(html_content: &str) -> Result<Vec<SearchResult>> {
    let document = Html::parse_document(html_content);
    let mut results = Vec::new();

    let result_selector = Selector::parse("div.g").map_err(|e| anyhow!("Selector error: {}", e))?;
    let title_selector = Selector::parse("h3").map_err(|e| anyhow!("Selector error: {}", e))?;
    let link_selector = Selector::parse("a").map_err(|e| anyhow!("Selector error: {}", e))?;
    let snippet_selector = Selector::parse(".VwiAwd, .ST93db, .kb139e").map_err(|e| anyhow!("Selector error: {}", e))?;

    for element in document.select(&result_selector) {
        let title = element
            .select(&title_selector)
            .next()
            .map(|el| el.text().collect::<Vec<_>>().join(""))
            .unwrap_or_else(|| "No Title".to_string());

        let link = element
            .select(&link_selector)
            .next()
            .and_then(|el| el.value().attr("href").map(|s| s.to_string()))
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
mod tests {
    use super::*;

    #[test]
    fn test_search_cache_ttl() {
        let cache = SearchCache::new(Duration::from_millis(50));
        let results = vec![SearchResult {
            title: "Test".to_string(),
            url: "https://example.com".to_string(),
            snippet: "Snippet".to_string(),
        }];

        cache.set("query".to_string(), results.clone());
        assert!(cache.get("query").is_some());

        // Wait for TTL expiration
        std::thread::sleep(Duration::from_millis(60));
        assert!(cache.get("query").is_none());
    }

    #[test]
    fn test_parse_search_results() {
        let html = r#"
            <div class="g">
                <h3><a href="https://example.com">Example Title</a></h3>
                <div class="VwiAwd">Example snippet text.</div>
            </div>
        "#;
        let results = parse_search_results(html).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Example Title");
        assert_eq!(results[0].url, "https://example.com");
        assert_eq!(results[0].snippet, "Example snippet text.");
    }
}
