use thiserror::Error;

#[derive(Error, Debug)]
pub enum BrowserError {
    #[error("Failed to launch browser: {0}")]
    Launch(String),
    #[error("Failed to connect to browser: {0}")]
    Connection(String),
    #[error("Browser error: {0}")]
    Runtime(String),
    #[error("Browser closed: {0}")]
    Closed(String),
}

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Internal error: {0}")]
    Internal(String),
    #[error("Network error: {0}")]
    Network(String),
    #[error("Browser error: {0}")]
    Browser(#[from] BrowserError),
    #[error("Parsing error: {0}")]
    Parse(String),
    #[error("Web fetch error: {0}")]
    WebFetch(String),
}

pub type AppResult<T> = anyhow::Result<T>;
