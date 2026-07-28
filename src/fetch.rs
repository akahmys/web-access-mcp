use anyhow::Context;
use chromiumoxide::page::Page;
use crate::browser::BrowserState;
use readabilityrs::Readability;
use html_to_markdown_rs::convert;
use serde::Serialize;
use std::time::Duration;
use thiserror::Error;

mod pdf;
use pdf::try_fetch_pdf;

mod ssrf;
use ssrf::validate_public_url;

const MAX_CONTENT_LENGTH: usize = 10000;

#[derive(Debug, Clone, Serialize)]
pub struct WebFetchResult {
    pub title: String,
    pub content: String,
}

/// Domain errors for `web_fetch`. Every variant's message already includes
/// a `Hint:` clause for the calling model, so none of them use thiserror's
/// `#[source]`/`#[from]` (which would make anyhow's `{:#}` chain-printer at
/// the MCP boundary print the same underlying cause a second time, since
/// it's already interpolated into the message text here).
#[derive(Debug, Error)]
pub enum FetchError {
    #[error("Failed to create new page: {0}. Hint: this is usually a transient browser/environment issue -- retry the call once; if it keeps failing, the server's Chromium may be unavailable and web_fetch can't be used this session.")]
    PageCreation(String),

    #[error("Page load timed out after {0} seconds. Hint: the site may be slow, unreachable, or blocking automated browsers -- retry once, or try a different URL if it keeps timing out.")]
    Timeout(u64),

    #[error("{0}. Hint: the URL may be invalid, offline, or require interaction beyond a simple page load -- double-check the URL and retry once, or try a different source.")]
    Navigation(String),

    #[error("Failed to get page content: {0}. Hint: the page may have crashed or navigated away -- retry the web_fetch call once.")]
    PageContent(String),

    #[error("Access to the page was blocked by Cloudflare or CAPTCHA protection. Hint: don't retry this exact URL, it will likely block again -- use web_search or smart_search to find an alternative source, or ask the user for a different link.")]
    Blocked,

    #[error("Failed to get title: {0}. Hint: transient browser error -- retry the web_fetch call once.")]
    Title(String),

    #[error("Readability initialization error: {0}. Hint: the page's HTML could not be parsed -- retry once; if it persists, this URL isn't supported by web_fetch.")]
    ReadabilityInit(String),

    #[error("Readability failed to parse the HTML content. Hint: this page likely isn't a plain article (e.g. an app-like SPA, login wall, or mostly non-text UI) -- try a different URL, or ask the user for the specific content needed.")]
    ReadabilityParse,

    #[error("Readability extracted content is empty. Hint: the page has no identifiable main content (may require login or JavaScript interaction) -- try a different URL.")]
    EmptyArticle,

    #[error("Markdown conversion error: {0}. Hint: internal conversion issue -- retry once; if it persists, this page's content can't be converted by web_fetch.")]
    MarkdownConversion(String),

    #[error("Markdown conversion produced no content. Hint: the extracted article was empty after conversion -- try a different URL.")]
    EmptyMarkdown,

    #[error("Failed to extract text from PDF: {0}. Hint: the PDF may be a scanned image with no text layer (OCR isn't supported), encrypted, or corrupted -- try a different URL, or ask the user for the specific text needed.")]
    PdfExtraction(String),

    #[error("Invalid URL: {0}. Hint: check that this is a well-formed, absolute http/https URL.")]
    InvalidUrl(String),

    #[error("Blocked for security: {0}. Hint: web_fetch refuses to access private/internal network addresses (localhost, RFC1918 ranges, link-local, cloud metadata endpoints) -- this isn't a transient failure, don't retry; use a public URL instead.")]
    SsrfBlocked(String),
}

pub async fn fetch_url(browser_state: &BrowserState, url: &str) -> Result<WebFetchResult, FetchError> {
    // 0. Reject URLs that resolve to loopback/private/link-local/metadata
    // addresses before making any request against them, closing off
    // web_fetch as an SSRF vector into the host's internal network.
    validate_public_url(url).await?;

    // 1. Check for GitHub Raw content first to bypass browser automation
    if let Some(raw_url) = get_github_raw_url(url) {
        match fetch_raw_content(&raw_url).await {
            Ok(content) => {
                return Ok(WebFetchResult {
                    title: format!("GitHub Raw: {url}"),
                    content,
                });
            }
            Err(e) => {
                eprintln!("Warning: Failed to fetch GitHub raw content: {e}. Falling back to browser.");
            }
        }
    }

    // 2. Check for PDF content and extract it directly, bypassing the
    // browser (Chromium's built-in PDF viewer renders a viewer UI, not
    // text this pipeline can read).
    if let Some(result) = try_fetch_pdf(url).await? {
        return Ok(result);
    }

    // 3. Fallback to Chromium-based browser.
    let page = open_and_load_page(browser_state, url).await?;
    let html_content = get_verified_html(&page).await?;
    let title = get_page_title(&page).await?;

    let markdown_content = html_to_markdown(&html_content)?;
    let content = truncate_content(&markdown_content, MAX_CONTENT_LENGTH);

    Ok(WebFetchResult { title, content })
}

/// Opens a new page in the shared browser and navigates it to `url`,
/// waiting for the DOM to load and stabilize under a fixed timeout. Only
/// the page-creation call is made under the shared browser lock; the
/// navigation below runs against this page's own CDP session, so
/// concurrent fetches don't serialize on each other.
async fn open_and_load_page(browser_state: &BrowserState, url: &str) -> Result<Page, FetchError> {
    let page = browser_state
        .new_page()
        .await
        .map_err(|e| FetchError::PageCreation(e.to_string()))?;

    let page_load_timeout = Duration::from_secs(15);
    let nav_result = tokio::time::timeout(page_load_timeout, async {
        page.goto(url).await.context("Failed to navigate to URL")?;
        wait_for_page_load(&page).await.context("Failed to wait for page load")?;
        anyhow::Ok(())
    }).await;

    match nav_result {
        Err(_) => Err(FetchError::Timeout(page_load_timeout.as_secs())),
        Ok(Err(e)) => Err(FetchError::Navigation(format!("{e:#}"))),
        Ok(Ok(())) => Ok(page),
    }
}

/// Fetches the page's rendered HTML and rejects known CAPTCHA/block pages.
async fn get_verified_html(page: &Page) -> Result<String, FetchError> {
    let html_content = page.content().await.map_err(|e| FetchError::PageContent(e.to_string()))?;

    if html_content.contains("cf-challenge")
        || html_content.contains("g-recaptcha")
        || html_content.contains("Access Denied")
        || html_content.contains("Attention Required! | Cloudflare")
    {
        return Err(FetchError::Blocked);
    }

    Ok(html_content)
}

async fn get_page_title(page: &Page) -> Result<String, FetchError> {
    let title = page
        .get_title()
        .await
        .map_err(|e| FetchError::Title(e.to_string()))?
        .unwrap_or_else(|| "No Title".to_string());
    Ok(title)
}

async fn wait_for_page_load(page: &Page) -> anyhow::Result<()> {
    wait_for_document_ready(page).await?;
    wait_for_content_stable(page).await
}

/// Waits for `document.readyState` to become 'complete'.
async fn wait_for_document_ready(page: &Page) -> anyhow::Result<()> {
    let _ = page.evaluate(r"
        () => {
            return new Promise((resolve) => {
                if (document.readyState === 'complete') {
                    resolve('complete');
                } else {
                    window.addEventListener('load', () => resolve('complete'));
                }
            });
        }
    ").await.context("Failed to evaluate load script")?;
    Ok(())
}

/// Polls `document.body.innerText.length` until it stops growing for two
/// consecutive checks, giving client-rendered (SPA) content a chance to
/// finish painting before extraction runs.
async fn wait_for_content_stable(page: &Page) -> anyhow::Result<()> {
    let mut last_length: usize = 0;
    let mut stable_count = 0;
    let max_attempts = 5;

    for _ in 0..max_attempts {
        let length_val: f64 = page
            .evaluate("document.body.innerText.length")
            .await?
            .into_value::<f64>()?;

        // `innerText.length` is a JS string length: always a non-negative
        // integer far below usize::MAX, so this narrowing cast can't
        // truncate or lose sign in practice.
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        let current_length = length_val as usize;

        if current_length > 0 && current_length == last_length {
            stable_count += 1;
        } else {
            stable_count = 0;
        }

        if stable_count >= 2 {
            break;
        }

        last_length = current_length;
        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    Ok(())
}

fn get_github_raw_url(url: &str) -> Option<String> {
    if url.contains("github.com") && url.contains("/blob/") {
        let mut raw_url = url.to_string().replace("github.com", "raw.githubusercontent.com");
        raw_url = raw_url.replace("/blob/", "/");
        Some(raw_url)
    } else {
        None
    }
}

async fn fetch_raw_content(url: &str) -> anyhow::Result<String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .context("Failed to build HTTP client")?;
    let res = client.get(url).send().await.context("Failed to send request")?;
    let text = res.text().await.context("Failed to read response text")?;
    Ok(text)
}

fn html_to_markdown(html_content: &str) -> Result<String, FetchError> {
    let readability = Readability::new(html_content, None, None)
        .map_err(|e| FetchError::ReadabilityInit(e.to_string()))?;

    let article = readability.parse().ok_or(FetchError::ReadabilityParse)?;

    let clean_html = article.content.ok_or(FetchError::EmptyArticle)?;

    let conversion_result = convert(&clean_html, None)
        .map_err(|e| FetchError::MarkdownConversion(e.to_string()))?;

    let markdown = conversion_result.content.ok_or(FetchError::EmptyMarkdown)?;

    Ok(markdown.trim().to_string())
}

pub fn truncate_content(content: &str, max_len: usize) -> String {
    if content.len() <= max_len {
        return content.to_string();
    }

    let truncated = &content[..max_len];
    let last_newline = truncated.rfind('\n').unwrap_or(0);

    if last_newline > max_len / 2 {
        format!("{}...", &truncated[..last_newline])
    } else {
        format!("{truncated}...")
    }
}

#[cfg(test)]
mod tests;
