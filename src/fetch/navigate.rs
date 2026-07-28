use super::FetchError;
use crate::browser::BrowserState;
use anyhow::Context;
use chromiumoxide::page::Page;
use std::time::Duration;

/// Retries `open_and_load_page` once, after a short backoff, on
/// failures that look transient (timeout, navigation, or page-creation
/// errors) -- but not on failures where retrying the identical request
/// is known to be pointless, like a CAPTCHA block.
pub(super) async fn open_and_load_page_with_retry(browser_state: &BrowserState, url: &str) -> Result<Page, FetchError> {
    match open_and_load_page(browser_state, url).await {
        Err(FetchError::Timeout(_) | FetchError::Navigation(_) | FetchError::PageCreation(_)) => {
            tokio::time::sleep(Duration::from_secs(2)).await;
            open_and_load_page(browser_state, url).await
        }
        result => result,
    }
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
