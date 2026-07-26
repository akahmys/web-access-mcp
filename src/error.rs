use thiserror::Error;

#[derive(Error, Debug)]
pub enum BrowserError {
    #[error("Failed to launch browser: {0}")]
    Launch(String),
    #[error("Browser error: {0}")]
    Runtime(String),
}

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Browser error: {0}")]
    Browser(#[from] BrowserError),
}

pub type AppResult<T> = anyhow::Result<T>;
