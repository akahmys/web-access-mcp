use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Internal error: {0}")]
    Internal(String),
    #[error("Network error: {0}")]
    Network(String),
    #[error("Browser error: {0}")]
    Browser(String),
    #[error("Parsing error: {0}")]
    Parse(String),
}

pub type AppResult<T> = anyhow::Result<T>;
