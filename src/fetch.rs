use chromiumoxide::page::Page;
use crate::browser::BrowserState;
use crate::cache::TtlCache;
use readabilityrs::Readability;
use html_to_markdown_rs::convert;
use serde::Serialize;
use thiserror::Error;

mod pdf;
use pdf::try_fetch_pdf;

mod fast_path;
use fast_path::try_fast_path;

mod ssrf;
use ssrf::validate_public_url;

mod robots;
use robots::check_robots_txt;

mod navigate;
use navigate::open_and_load_page_with_retry;

mod multi;
pub use multi::fetch_content_or_error;

mod actions;
pub use actions::PageAction;
use actions::run_actions;

const MAX_CONTENT_LENGTH: usize = 10000;

pub type FetchCache = TtlCache<WebFetchResult>;

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

    #[error("Action failed: {0}. Hint: check the selector is correct and the element exists at that point in the action sequence (e.g. after a prior click/scroll) -- retry with a corrected action list.")]
    ActionFailed(String),

    #[error("Blocked by robots.txt: {0}. Hint: this host's robots.txt disallows automated access to this path. Don't retry -- try a different URL or ask the user for the specific content. (Server operators who've decided robots.txt shouldn't apply to single agent-directed fetches can set WEB_FETCH_IGNORE_ROBOTS=1.)")]
    RobotsDisallowed(String),
}

/// Fetches and extracts `url`, serving a cached result if one was fetched
/// within the last `FetchCache` TTL. Pages change more often than search
/// rankings, so this TTL is much shorter than `SearchCache`'s. Caching is
/// skipped entirely when `actions` is non-empty: the same URL can produce
/// different content depending on what actions were applied, and this
/// module doesn't key the cache on the action list.
pub async fn fetch_url(
    browser_state: &BrowserState,
    fetch_cache: &FetchCache,
    url: &str,
    actions: &[PageAction],
) -> Result<WebFetchResult, FetchError> {
    if actions.is_empty() {
        if let Some(cached) = fetch_cache.get(url) {
            return Ok(cached);
        }
    }

    let result = fetch_url_uncached(browser_state, url, actions).await?;

    if actions.is_empty() {
        fetch_cache.set(url.to_string(), result.clone());
    }

    Ok(result)
}

async fn fetch_url_uncached(browser_state: &BrowserState, url: &str, actions: &[PageAction]) -> Result<WebFetchResult, FetchError> {
    // 0. Reject URLs that resolve to loopback/private/link-local/metadata
    // addresses before making any request against them, closing off
    // web_fetch as an SSRF vector into the host's internal network.
    validate_public_url(url).await?;

    // 0.5. Respect the host's robots.txt (fails open if it's missing,
    // unreachable, or unparseable -- see robots.rs).
    check_robots_txt(url).await?;

    // 1. Try the browser-free fast paths (GitHub raw, PDF). Skipped when
    // actions are requested: neither path touches a browser, so neither
    // can honor them.
    if actions.is_empty() {
        if let Some(result) = try_fast_path(url).await? {
            return Ok(result);
        }
    }

    // 2. Fallback to Chromium-based browser.
    let page = open_and_load_page_with_retry(browser_state, url).await?;
    run_actions(&page, actions).await?;
    let html_content = get_verified_html(&page).await?;
    let title = get_page_title(&page).await?;

    let markdown_content = html_to_markdown(&html_content)?;
    let content = truncate_content(&markdown_content, MAX_CONTENT_LENGTH);

    Ok(WebFetchResult { title, content })
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
