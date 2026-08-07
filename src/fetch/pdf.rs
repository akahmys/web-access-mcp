use super::{truncate_content, FetchError, WebFetchResult, MAX_CONTENT_LENGTH};
use crate::user_agent::user_agent;
use std::time::Duration;

/// If `url` looks like a PDF -- a successful `HEAD` reporting
/// `Content-Type: application/pdf`, or (when `HEAD` is inconclusive) a
/// `.pdf` URL -- downloads and extracts its text directly. Returns
/// `Ok(None)` when it doesn't look like a PDF, so the caller falls
/// through to the normal browser-based fetch.
const MAX_DOWNLOAD_SIZE: usize = 10 * 1024 * 1024; // 10MB

pub(super) async fn try_fetch_pdf(url: &str) -> Result<Option<WebFetchResult>, FetchError> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .map_err(|e| FetchError::PdfExtraction(e.to_string()))?;

    if !looks_like_pdf(&client, url).await {
        return Ok(None);
    }

    let response = client
        .get(url)
        .header("User-Agent", user_agent())
        .send()
        .await
        .map_err(|e| FetchError::PdfExtraction(format!("request failed: {e}")))?;

    if let Some(len) = response.content_length() {
        if len > MAX_DOWNLOAD_SIZE as u64 {
            return Err(FetchError::PdfExtraction(format!(
                "PDF size ({len} bytes) exceeds maximum limit of {MAX_DOWNLOAD_SIZE} bytes"
            )));
        }
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|e| FetchError::PdfExtraction(format!("failed to read response body: {e}")))?;

    if bytes.len() > MAX_DOWNLOAD_SIZE {
        return Err(FetchError::PdfExtraction(format!(
            "PDF size exceeds maximum limit of {MAX_DOWNLOAD_SIZE} bytes"
        )));
    }

    let text = pdf_extract::extract_text_from_mem(&bytes)
        .map_err(|e| FetchError::PdfExtraction(e.to_string()))?;

    Ok(Some(WebFetchResult {
        title: format!("PDF: {url}"),
        content: truncate_content(&text, MAX_CONTENT_LENGTH),
    }))
}

async fn looks_like_pdf(client: &reqwest::Client, url: &str) -> bool {
    let content_type = client
        .head(url)
        .header("User-Agent", user_agent())
        .send()
        .await
        .ok()
        .filter(|res| res.status().is_success())
        .and_then(|res| res.headers().get(reqwest::header::CONTENT_TYPE).cloned())
        .and_then(|v| v.to_str().map(str::to_string).ok());

    match content_type {
        Some(ct) => ct.contains("application/pdf"),
        None => url.to_lowercase().ends_with(".pdf"),
    }
}
