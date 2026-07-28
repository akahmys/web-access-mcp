use std::sync::OnceLock;

const CANDIDATES: &[&str] = &[
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36",
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.3 Safari/605.1.15",
];

static USER_AGENT: OnceLock<&'static str> = OnceLock::new();

/// Returns a realistic browser User-Agent string, picked once (from a
/// small fixed pool) the first time this is called and reused for the
/// rest of the process's lifetime. This varies which UA a given server
/// instance presents -- instead of every deployment sharing one
/// hardcoded literal -- while staying internally consistent for the
/// life of a session, the way a real browser would.
pub fn user_agent() -> &'static str {
    USER_AGENT.get_or_init(|| {
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.subsec_nanos() as usize);
        CANDIDATES[seed % CANDIDATES.len()]
    })
}

#[cfg(test)]
mod tests;
