use super::FetchError;
use crate::user_agent::user_agent;
use std::time::Duration;
use texting_robots::{get_robots_url, Robot};

const ROBOTS_FETCH_TIMEOUT: Duration = Duration::from_secs(5);

/// Checks `url` against its host's `robots.txt`, failing *open* (i.e.
/// allowing the fetch) whenever `robots.txt` is missing, unreachable, or
/// unparseable -- matching standard crawler convention, where an absent
/// `robots.txt` means "no restrictions" rather than "deny everything".
/// Set `WEB_FETCH_IGNORE_ROBOTS=1` on the server to skip this check
/// entirely: `robots.txt` is a policy signal aimed at bulk crawlers, and
/// some operators may judge it doesn't apply to a single agent-directed
/// fetch.
pub(super) async fn check_robots_txt(url: &str) -> Result<(), FetchError> {
    if std::env::var_os("WEB_FETCH_IGNORE_ROBOTS").is_some() {
        return Ok(());
    }

    let Some(body) = fetch_robots_txt(url).await else {
        return Ok(());
    };

    let Ok(robot) = Robot::new(user_agent(), &body) else {
        return Ok(());
    };

    if robot.allowed(url) {
        Ok(())
    } else {
        Err(FetchError::RobotsDisallowed(url.to_string()))
    }
}

/// Fetches the raw `robots.txt` body for `url`'s host, returning `None`
/// on any failure (missing file, network error, timeout) so the caller
/// treats it as "no restrictions" rather than propagating an error.
async fn fetch_robots_txt(url: &str) -> Option<Vec<u8>> {
    let robots_url = get_robots_url(url).ok()?;
    let client = reqwest::Client::builder().timeout(ROBOTS_FETCH_TIMEOUT).build().ok()?;
    let response = client.get(&robots_url).header("User-Agent", user_agent()).send().await.ok()?;

    if !response.status().is_success() {
        return None;
    }

    response.bytes().await.ok().map(|b| b.to_vec())
}

#[cfg(test)]
mod tests;
