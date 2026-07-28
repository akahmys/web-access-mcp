use dashmap::DashMap;
use std::time::{Duration, Instant};

/// A simple string-keyed in-memory cache with a fixed TTL per entry.
/// Shared by `search::SearchCache` and `fetch::FetchCache`.
pub struct TtlCache<V: Clone> {
    entries: DashMap<String, (V, Instant)>,
    ttl: Duration,
}

impl<V: Clone> TtlCache<V> {
    pub fn new(ttl: Duration) -> Self {
        Self {
            entries: DashMap::new(),
            ttl,
        }
    }

    pub fn get(&self, key: &str) -> Option<V> {
        if let Some(entry) = self.entries.get(key) {
            let (value, expires_at) = entry.value();
            if *expires_at > Instant::now() {
                return Some(value.clone());
            }
        }
        None
    }

    pub fn set(&self, key: String, value: V) {
        self.entries.insert(key, (value, Instant::now() + self.ttl));
    }
}

#[cfg(test)]
mod tests;
