use dashmap::DashMap;
use std::time::{Duration, Instant};

/// A simple string-keyed in-memory cache with a fixed TTL and maximum capacity per instance.
/// Shared by `search::SearchCache` and `fetch::FetchCache`.
#[derive(Clone)]
pub struct TtlCache<V: Clone> {
    entries: DashMap<String, (V, Instant)>,
    ttl: Duration,
    max_capacity: usize,
}

impl<V: Clone> TtlCache<V> {
    pub fn new(ttl: Duration, max_capacity: usize) -> Self {
        Self {
            entries: DashMap::new(),
            ttl,
            max_capacity,
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
        if self.entries.len() >= self.max_capacity {
            self.evict_expired();
            if self.entries.len() >= self.max_capacity {
                if let Some(oldest_key) = self
                    .entries
                    .iter()
                    .min_by_key(|entry| entry.value().1)
                    .map(|entry| entry.key().clone())
                {
                    self.entries.remove(&oldest_key);
                }
            }
        }
        self.entries.insert(key, (value, Instant::now() + self.ttl));
    }

    /// Retains only entries whose expiration timestamp is in the future.
    pub fn evict_expired(&self) {
        let now = Instant::now();
        self.entries.retain(|_, (_, expires_at)| *expires_at > now);
    }
}

#[cfg(test)]
mod tests;
