use super::*;

#[test]
fn test_ttl_cache_get_set() {
    let cache: TtlCache<String> = TtlCache::new(Duration::from_millis(50), 10);
    assert!(cache.get("key").is_none());

    cache.set("key".to_string(), "value".to_string());
    assert_eq!(cache.get("key"), Some("value".to_string()));

    std::thread::sleep(Duration::from_millis(60));
    assert!(cache.get("key").is_none());
}

#[test]
fn test_ttl_cache_evict_expired() {
    let cache: TtlCache<String> = TtlCache::new(Duration::from_millis(50), 10);
    cache.set("key1".to_string(), "val1".to_string());
    assert_eq!(cache.entries.len(), 1);

    std::thread::sleep(Duration::from_millis(60));
    // The key is expired but still present in entries until evicted
    cache.evict_expired();
    assert_eq!(cache.entries.len(), 0);
}

#[test]
#[allow(clippy::duration_suboptimal_units)]
fn test_ttl_cache_capacity_limit() {
    let cache: TtlCache<String> = TtlCache::new(Duration::from_secs(60), 2);
    cache.set("k1".to_string(), "v1".to_string());
    cache.set("k2".to_string(), "v2".to_string());
    assert_eq!(cache.entries.len(), 2);

    // Adding a 3rd key should evict one key to stay at capacity limit of 2
    cache.set("k3".to_string(), "v3".to_string());
    assert_eq!(cache.entries.len(), 2);
}
