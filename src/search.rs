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
    _browser_state: &BrowserState,
    cache: &SearchCache,
    query: &str,
) -> Result<Vec<SearchResult>> {
    if let Some(cached) = cache.get(query) {
        return Ok(cached);
    }

    // Try fast HTTP DuckDuckGo search first
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;

    let ddg_url = format!("https://html.duckduckgo.com/html/?q={}", urlencoding::encode(query));
    let res = client
        .get(&ddg_url)
        .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36")
        .header("Accept-Language", "ja,en-US;q=0.9,en;q=0.8")
        .send()
        .await;

    if let Ok(response) = res {
        if response.status().is_success() {
            if let Ok(html) = response.text().await {
                let results = parse_ddg_results(&html)?;
                if !results.is_empty() {
                    cache.set(query.to_string(), results.clone());
                    return Ok(results);
                }
            }
        }
    }

    // Fallback to Google HTTP search
    let google_url = format!("https://www.google.com/search?q={}", urlencoding::encode(query));
    let res = client
        .get(&google_url)
        .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36")
        .header("Accept-Language", "ja,en-US;q=0.9,en;q=0.8")
        .send()
        .await;

    if let Ok(response) = res {
        if response.status().is_success() {
            if let Ok(html) = response.text().await {
                let results = parse_search_results(&html)?;
                if !results.is_empty() {
                    cache.set(query.to_string(), results.clone());
                    return Ok(results);
                }
            }
        }
    }

    Err(anyhow!("Failed to retrieve search results"))
}

fn parse_ddg_results(html_content: &str) -> Result<Vec<SearchResult>> {
    let document = Html::parse_document(html_content);
    let mut results = Vec::new();

    let result_selector = Selector::parse(".result").map_err(|e| anyhow!("Selector error: {}", e))?;
    let title_selector = Selector::parse(".result__a, .result__title").map_err(|e| anyhow!("Selector error: {}", e))?;
    let link_selector = Selector::parse("a.result__a, a.result__url").map_err(|e| anyhow!("Selector error: {}", e))?;
    let snippet_selector = Selector::parse(".result__snippet").map_err(|e| anyhow!("Selector error: {}", e))?;

    for element in document.select(&result_selector) {
        let title = element
            .select(&title_selector)
            .next()
            .map(|el| el.text().collect::<Vec<_>>().join("").trim().to_string())
            .unwrap_or_default();

        let link = element
            .select(&link_selector)
            .next()
            .and_then(|el| el.value().attr("href").map(|s| s.to_string()))
            .unwrap_or_default();

        // Extract actual URL if wrapped in DDG redirect (/l/?uddg=...)
        let actual_link = if link.contains("uddg=") {
            link.split("uddg=")
                .nth(1)
                .and_then(|s| s.split('&').next())
                .and_then(|s| urlencoding::decode(s).ok().map(|c| c.into_owned()))
                .unwrap_or(link)
        } else {
            link
        };

        let snippet = element
            .select(&snippet_selector)
            .next()
            .map(|el| el.text().collect::<Vec<_>>().join("").trim().to_string())
            .unwrap_or_default();

        if !title.is_empty() && actual_link.starts_with("http") {
            results.push(SearchResult {
                title,
                url: actual_link,
                snippet,
            });
        }

        if results.len() >= 5 {
            break;
        }
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
