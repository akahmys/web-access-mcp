use super::{try_fetch_pdf, FetchError, WebFetchResult};
use crate::user_agent::user_agent;
use anyhow::Context;
use std::time::Duration;

/// Tries the browser-free fast paths (GitHub raw content, PDF text
/// extraction) in order. Returns `Ok(None)` if neither applies, so the
/// caller falls through to the browser.
pub(super) async fn try_fast_path(url: &str) -> Result<Option<WebFetchResult>, FetchError> {
    if let Some(raw_url) = get_github_raw_url(url) {
        match fetch_raw_content(&raw_url).await {
            Ok(content) => {
                return Ok(Some(WebFetchResult {
                    title: format!("GitHub Raw: {url}"),
                    content,
                }));
            }
            Err(e) => {
                eprintln!(
                    "Warning: Failed to fetch GitHub raw content: {e}. Falling back to browser."
                );
            }
        }
    }

    try_fetch_pdf(url).await
}

fn get_github_raw_url(url: &str) -> Option<String> {
    if url.contains("github.com") && url.contains("/blob/") {
        let mut raw_url = url
            .to_string()
            .replace("github.com", "raw.githubusercontent.com");
        raw_url = raw_url.replace("/blob/", "/");
        Some(raw_url)
    } else {
        None
    }
}

const MAX_RAW_DOWNLOAD_SIZE: usize = 10 * 1024 * 1024; // 10MB

async fn fetch_raw_content(url: &str) -> anyhow::Result<String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .context("Failed to build HTTP client")?;
    let res = client
        .get(url)
        .header("User-Agent", user_agent())
        .send()
        .await
        .context("Failed to send request")?;

    if let Some(len) = res.content_length() {
        if len > MAX_RAW_DOWNLOAD_SIZE as u64 {
            anyhow::bail!("Raw content size ({len} bytes) exceeds maximum limit of {MAX_RAW_DOWNLOAD_SIZE} bytes");
        }
    }

    let text = res.text().await.context("Failed to read response text")?;

    if text.len() > MAX_RAW_DOWNLOAD_SIZE {
        anyhow::bail!("Raw content size exceeds maximum limit of {MAX_RAW_DOWNLOAD_SIZE} bytes");
    }

    Ok(text)
}

#[cfg(test)]
mod tests;
