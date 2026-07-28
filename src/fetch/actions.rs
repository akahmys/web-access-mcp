use super::FetchError;
use chromiumoxide::page::Page;
use serde::Deserialize;
use std::time::Duration;

/// A single, whitelisted browser interaction `web_fetch` can perform
/// before extracting content. Deliberately *not* arbitrary JS `eval` --
/// exposing that to the calling model would turn `web_fetch` into a
/// remote-code-execution primitive against whatever the browser can
/// reach, compounding the SSRF risk `ssrf.rs` closes off. No form-fill/
/// login support: out of scope for this minimal action set.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum PageAction {
    Click { selector: String },
    Scroll { target: ScrollTarget },
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ScrollTarget {
    Top,
    Bottom,
}

/// Applies `actions` to `page`, in order, before extraction. Each action
/// pauses briefly afterward to let the resulting DOM change (content
/// load, animation) settle.
pub(super) async fn run_actions(page: &Page, actions: &[PageAction]) -> Result<(), FetchError> {
    for action in actions {
        apply_one(page, action).await?;
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    Ok(())
}

async fn apply_one(page: &Page, action: &PageAction) -> Result<(), FetchError> {
    match action {
        PageAction::Click { selector } => {
            let element = page
                .find_element(selector.as_str())
                .await
                .map_err(|e| FetchError::ActionFailed(format!("click '{selector}': no matching element ({e})")))?;
            element
                .click()
                .await
                .map_err(|e| FetchError::ActionFailed(format!("click '{selector}': {e}")))?;
        }
        PageAction::Scroll { target } => {
            let script = match target {
                ScrollTarget::Top => "window.scrollTo(0, 0)",
                ScrollTarget::Bottom => "window.scrollTo(0, document.body.scrollHeight)",
            };
            page.evaluate(script).await.map_err(|e| FetchError::ActionFailed(format!("scroll: {e}")))?;
        }
    }
    Ok(())
}
