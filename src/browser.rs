use chromiumoxide::browser::{Browser, BrowserConfig};
use chromiumoxide::cdp::browser_protocol::target::CreateTargetParams;
use chromiumoxide::page::Page;
use std::sync::Arc;
use std::env;
use std::path::PathBuf;
use tokio::sync::{RwLock, Mutex};
use crate::error::{AppError, BrowserError};
use crate::user_agent::user_agent;
use futures_util::StreamExt;
use tracing::{info, warn};

#[derive(Clone, Default)]
pub struct BrowserState {
    browser: Arc<RwLock<Option<Arc<Mutex<Browser>>>>>,
    user_data_dir: Arc<RwLock<Option<PathBuf>>>,
}

impl BrowserState {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn get_or_start_browser(&self) -> Result<Arc<Mutex<Browser>>, AppError> {
        // Fast read lock check
        {
            let lock = self.browser.read().await;
            if let Some(b) = lock.as_ref() {
                return Ok(Arc::clone(b));
            }
        }

        // Write lock to initialize browser lazily
        let mut lock = self.browser.write().await;
        if let Some(b) = lock.as_ref() {
            return Ok(Arc::clone(b));
        }

        let user_data_dir = env::temp_dir().join(format!("web-access-mcp-{}", std::process::id()));
        let mut builder = BrowserConfig::builder()
            .arg("--no-sandbox")
            .arg("--disable-dev-shm-usage")
            .arg("--disable-gpu")
            .arg(format!("--user-agent={}", user_agent()))
            .user_data_dir(&user_data_dir);

        if let Ok(chrome_path) = env::var("CHROME_PATH") {
            info!("Using custom CHROME_PATH: {}", chrome_path);
            builder = builder.chrome_executable(chrome_path);
        }

        // Chromium doesn't read HTTP_PROXY/HTTPS_PROXY itself (unlike
        // reqwest, which honors them by default); pass one through
        // explicitly via --proxy-server if the operator set one.
        if let Some(proxy) = proxy_from_env() {
            info!("Using proxy: {}", proxy);
            builder = builder.arg(format!("--proxy-server={proxy}"));
        }

        let config = builder
            .build()
            .map_err(|e| BrowserError::Launch(e.clone()))?;

        let (browser, mut handler) = Browser::launch(config)
            .await
            .map_err(|e| {
                warn!("Failed to launch browser: {}", e);
                BrowserError::Launch(e.to_string())
            })?;

        tokio::spawn(async move {
            while let Some(_event) = handler.next().await {
                // Keep the handler running
            }
        });

        let browser_arc = Arc::new(Mutex::new(browser));
        *lock = Some(Arc::clone(&browser_arc));
        *self.user_data_dir.write().await = Some(user_data_dir);
        info!("Chromium browser initialized successfully.");
        Ok(browser_arc)
    }

    /// Creates a new browser page/tab. The browser mutex is only held for the
    /// brief CDP call that opens the page, not for whatever the caller does
    /// with the returned `Page` afterwards. This lets multiple pages be
    /// fetched concurrently instead of serializing on a single lock.
    pub async fn new_page(&self) -> Result<Page, AppError> {
        let browser_arc = self.get_or_start_browser().await?;
        let browser = browser_arc.lock().await;
        browser
            .new_page(CreateTargetParams::default())
            .await
            .map_err(|e| AppError::Browser(BrowserError::Runtime(e.to_string())))
    }

    pub async fn stop(&self) -> Result<(), AppError> {
        let mut lock = self.browser.write().await;
        if let Some(browser_mutex_arc) = lock.take() {
            let mut browser = browser_mutex_arc.lock().await;
            browser.close().await.map_err(|e| AppError::Browser(BrowserError::Runtime(e.to_string())))?;
        }
        drop(lock);

        // Clean up the process-isolated profile directory so it doesn't
        // accumulate in the OS temp dir across server restarts.
        if let Some(dir) = self.user_data_dir.write().await.take() {
            if let Err(e) = tokio::fs::remove_dir_all(&dir).await {
                warn!("Failed to clean up browser profile dir {:?}: {}", dir, e);
            }
        }
        Ok(())
    }
}

/// Checks the standard proxy env vars, preferring an HTTPS-specific proxy
/// over a generic one, and upper-case (the conventional form) over
/// lower-case (some tools/shells only set the latter).
fn proxy_from_env() -> Option<String> {
    ["HTTPS_PROXY", "https_proxy", "HTTP_PROXY", "http_proxy"]
        .into_iter()
        .find_map(|var| env::var(var).ok())
}
