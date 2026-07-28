use super::{truncate_content, FetchError, WebFetchResult, MAX_CONTENT_LENGTH};
use std::time::Duration;

/// If `url` looks like a PDF -- a successful `HEAD` reporting
/// `Content-Type: application/pdf`, or (when `HEAD` is inconclusive) a
/// `.pdf` URL -- downloads and extracts its text directly. Returns
/// `Ok(None)` when it doesn't look like a PDF, so the caller falls
/// through to the normal browser-based fetch.
pub(super) async fn try_fetch_pdf(url: &str) -> Result<Option<WebFetchResult>, FetchError> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .map_err(|e| FetchError::PdfExtraction(e.to_string()))?;

    if !looks_like_pdf(&client, url).await {
        return Ok(None);
    }

    let bytes = client
        .get(url)
        .send()
        .await
        .map_err(|e| FetchError::PdfExtraction(format!("request failed: {e}")))?
        .bytes()
        .await
        .map_err(|e| FetchError::PdfExtraction(format!("failed to read response body: {e}")))?;

    let text = pdf_extract::extract_text_from_mem(&bytes).map_err(|e| FetchError::PdfExtraction(e.to_string()))?;

    Ok(Some(WebFetchResult {
        title: format!("PDF: {url}"),
        content: truncate_content(&text, MAX_CONTENT_LENGTH),
    }))
}

async fn looks_like_pdf(client: &reqwest::Client, url: &str) -> bool {
    let content_type = client
        .head(url)
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
