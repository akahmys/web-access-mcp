use super::*;

#[test]
fn test_ttl_cache_get_set() {
    let cache: TtlCache<String> = TtlCache::new(Duration::from_millis(50));
    assert!(cache.get("key").is_none());

    cache.set("key".to_string(), "value".to_string());
    assert_eq!(cache.get("key"), Some("value".to_string()));

    std::thread::sleep(Duration::from_millis(60));
    assert!(cache.get("key").is_none());
}
