use thiserror::Error;

/// Internal errors produced by the Chromium browser manager.
#[derive(Error, Debug)]
pub enum BrowserError {
    #[error("Failed to launch browser: {0}")]
    Launch(String),
    #[error("Browser error: {0}")]
    Runtime(String),
}

/// Generic error wrapper for browser management errors.
#[derive(Error, Debug)]
pub enum AppError {
    #[error("Browser error: {0}")]
    Browser(#[from] BrowserError),
}

/// Top-level application result type using anyhow context propagation.
pub type AppResult<T> = anyhow::Result<T>;
