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
