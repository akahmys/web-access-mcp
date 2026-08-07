use crate::browser::BrowserState;
use crate::fetch::FetchCache;
use crate::search::{BingSearchProvider, SearchCache};
use std::sync::Arc;
use std::time::Duration;

/// Encapsulates all thread-safe shared application state:
/// the lazy Chromium browser controller, search provider, search result TTL cache,
/// and web fetch result TTL cache.
#[derive(Clone)]
pub struct AppContext {
    pub browser: BrowserState,
    pub search_provider: Arc<BingSearchProvider>,
    pub search_cache: SearchCache,
    pub fetch_cache: FetchCache,
}

const MAX_CACHE_ENTRIES: usize = 500;

impl AppContext {
    pub fn new(search_ttl: Duration, fetch_ttl: Duration) -> Self {
        Self {
            browser: BrowserState::new(),
            search_provider: Arc::new(BingSearchProvider),
            search_cache: SearchCache::new(search_ttl, MAX_CACHE_ENTRIES),
            fetch_cache: FetchCache::new(fetch_ttl, MAX_CACHE_ENTRIES),
        }
    }

    /// Evicts expired entries from both search and fetch caches.
    pub fn evict_expired_caches(&self) {
        self.search_cache.evict_expired();
        self.fetch_cache.evict_expired();
    }
}
