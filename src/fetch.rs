use anyhow::{anyhow, Context, Result};
use chromiumoxide::browser::Browser;
use chromiumoxide::page::Page;
use readabilityrs::Readability;
use html_to_markdown_rs::convert;
use serde::Serialize;
use std::time::Duration;

const MAX_CONTENT_LENGTH: usize = 10000;

#[derive(Debug, Clone, Serialize)]
pub struct WebFetchResult {
    pub title: String,
    pub content: String,
}

pub async fn fetch_url(browser: &Browser, url: &str) -> Result<WebFetchResult> {
    // 1. Check for GitHub Raw content first to bypass browser automation
    if let Some(raw_url) = get_github_raw_url(url) {
        match fetch_raw_content(&raw_url).await {
            Ok(content) => {
                return Ok(WebFetchResult {
                    title: format!("GitHub Raw: {}", url),
                    content,
                });
            }
            Err(e) => {
                eprintln!("Warning: Failed to fetch GitHub raw content: {}. Falling back to browser.", e);
            }
        }
    }

    // 2. Fallback to Chromium-based browser
    let page = browser
        .new_page(chromiumoxide::cdp::browser_protocol::target::CreateTargetParams::default())
        .await
        .context("Failed to create new page")?;

    // Wrap page navigation and waiting in a timeout
    let page_load_timeout = Duration::from_secs(15);
    tokio::time::timeout(page_load_timeout, async {
        page.goto(url).await.context("Failed to navigate to URL")?;
        wait_for_page_load(&page).await.context("Failed to wait for page load")?;
        anyhow::Ok(())
    }).await
    .map_err(|_| anyhow!("Page load timed out after {} seconds", page_load_timeout.as_secs()))??;

    let html_content = page.content().await.context("Failed to get page content")?;

    // Check for common CAPTCHA / Access Denied patterns
    if html_content.contains("cf-challenge") 
        || html_content.contains("g-recaptcha") 
        || html_content.contains("Access Denied") 
        || html_content.contains("Attention Required! | Cloudflare")
    {
        return Err(anyhow!("Access to the page was blocked by Cloudflare or CAPTCHA protection."));
    }

    let title = page
        .get_title()
        .await
        .map_err(|e| anyhow!("Failed to get title: {}", e))?
        .unwrap_or_else(|| "No Title".to_string());

    let markdown_content = html_to_markdown(&html_content)?;
    let content = truncate_content(&markdown_content, MAX_CONTENT_LENGTH);

    Ok(WebFetchResult {
        title,
        content,
    })
}

async fn wait_for_page_load(page: &Page) -> Result<()> {
    // 1. Wait for document.readyState to be 'complete'
    let _ = page.evaluate(r#"
        () => {
            return new Promise((resolve) => {
                if (document.readyState === 'complete') {
                    resolve('complete');
                } else {
                    window.addEventListener('load', () => resolve('complete'));
                }
            });
        }
    "#).await.context("Failed to evaluate load script")?;

    // 2. Stability check: Check if document body text length stabilizes
    let mut last_length: usize = 0;
    let mut stable_count = 0;
    let max_attempts = 5;

    for _ in 0..max_attempts {
        // Get length of body text
        let length_val: f64 = page
            .evaluate("document.body.innerText.length")
            .await?
            .into_value::<f64>()?;

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
        tokio::time::sleep(Duration::from_millis(1000)).await;
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

async fn fetch_raw_content(url: &str) -> Result<String> {
    let res = reqwest::get(url).await.context("Failed to send request")?;
    let text = res.text().await.context("Failed to read response text")?;
    Ok(text)
}

fn html_to_markdown(html_content: &str) -> Result<String> {
    let readability = Readability::new(html_content, None, None)
        .map_err(|e| anyhow!("Readability initialization error: {}", e))?;

    let article = readability
        .parse()
        .ok_or_else(|| anyhow!("Readability failed to parse the HTML content"))?;

    let clean_html = article
        .content
        .ok_or_else(|| anyhow!("Readability extracted content is empty"))?;

    let conversion_result = convert(&clean_html, None)
        .map_err(|e| anyhow!("Markdown conversion error: {}", e))?;

    let markdown = conversion_result
        .content
        .ok_or_else(|| anyhow!("Markdown conversion produced no content"))?;

    Ok(markdown.trim().to_string())
}

fn truncate_content(content: &str, max_len: usize) -> String {
    if content.len() <= max_len {
        return content.to_string();
    }

    let truncated = &content[..max_len];
    let last_newline = truncated.rfind('\n').unwrap_or(0);
    
    if last_newline > max_len / 2 {
        format!("{}...", &truncated[..last_newline])
    } else {
        format!("{}...", truncated)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_github_raw_url_resolver() {
        assert_eq!(
            get_github_raw_url("https://github.com/user/repo/blob/main/src/lib.rs"),
            Some("https://raw.githubusercontent.com/user/repo/main/src/lib.rs".to_string())
        );
        assert_eq!(
            get_github_raw_url("https://google.com"),
            None
        );
    }

    #[test]
    fn test_content_truncation() {
        let content = "line1\nline2\nline3";
        // truncate with max_len 8, should find last newline 'line1\n' (index 5)
        let truncated = truncate_content(content, 8);
        assert_eq!(truncated, "line1...");

        let content_no_newlines = "abcdefghijk";
        let truncated_no_newlines = truncate_content(content_no_newlines, 5);
        assert_eq!(truncated_no_newlines, "abcde...");
    }

    #[test]
    fn test_html_to_markdown_conversion() {
        let html = r#"
            <!DOCTYPE html>
            <html>
            <head><title>Test Article</title></head>
            <body>
                <div role="main">
                    <h1>Test Article Title</h1>
                    <p>This is a paragraph of the test article. It contains enough text to satisfy readability parser requirements, which look for content density and word count to identify main articles.</p>
                    <p>Here is another paragraph of text. We want to ensure that this is extracted cleanly without navigation bars or other noise.</p>
                </div>
            </body>
            </html>
        "#;
        let md = html_to_markdown(html).unwrap();
        assert!(md.contains("Test Article Title"));
        assert!(md.contains("paragraph of the test article"));
    }
}
