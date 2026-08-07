use super::*;

#[test]
fn test_search_cache_ttl() {
    let cache = SearchCache::new(Duration::from_millis(50), 10);
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
fn test_parse_bing_rss() {
    let xml = r#"<?xml version="1.0" encoding="utf-8" ?><rss version="2.0"><channel><title>Bing: example</title><item><title>Example Title</title><link>https://example.com</link><description>Example snippet text.</description><pubDate>Mon, 27 Jul 2026 00:00:00 GMT</pubDate></item></channel></rss>"#;
    let results = parse_bing_rss(xml).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].title, "Example Title");
    assert_eq!(results[0].url, "https://example.com");
    assert_eq!(results[0].snippet, "Example snippet text.");
}
