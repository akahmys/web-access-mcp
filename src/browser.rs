use chromiumoxide::browser::{Browser, BrowserConfig};
use std::sync::Arc;
use tokio::sync::{RwLock, Mutex};
use crate::error::{AppError, BrowserError};
use futures_util::StreamExt;

#[derive(Clone, Default)]
pub struct BrowserState {
    browser: Arc<RwLock<Option<Arc<Mutex<Browser>>>>>,
}

impl BrowserState {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn start(&self) -> Result<(), AppError> {
        let config = BrowserConfig::builder()
            .build()
            .map_err(|e| BrowserError::Launch(e.to_string()))?;
        
        let (browser, mut handler) = Browser::launch(config)
            .await
            .map_err(|e| BrowserError::Launch(e.to_string()))?;
        
        tokio::spawn(async move {
            while let Some(_event) = handler.next().await {
                // Keep the handler running
            }
        });

        let mut lock = self.browser.write().await;
        *lock = Some(Arc::new(Mutex::new(browser)));
        Ok(())
    }

    pub async fn get_browser(&self) -> Option<Arc<Mutex<Browser>>> {
        let lock = self.browser.read().await;
        lock.as_ref().map(Arc::clone)
    }

    pub async fn stop(&self) -> Result<(), AppError> {
        let mut lock = self.browser.write().await;
        if let Some(browser_mutex_arc) = lock.take() {
            let mut browser = browser_mutex_arc.lock().await;
            browser.close().await.map_err(|e| AppError::Browser(BrowserError::Runtime(e.to_string())))?;
        }
        Ok(())
    }
}
