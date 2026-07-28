use super::FetchError;
use ipnet::{Ipv4Net, Ipv6Net};
use reqwest::Url;
use std::net::{IpAddr, Ipv4Addr};

/// Rejects URLs whose host resolves to a non-public IP range (loopback,
/// private, link-local, and other reserved ranges -- including cloud
/// metadata endpoints like `169.254.169.254`) before any network/browser
/// call is made against it. Resolves the hostname via DNS and checks the
/// *resolved* IPs, not just literal-IP URLs, so a DNS-rebinding attack (a
/// public-looking hostname that resolves to an internal address) is
/// caught too.
pub(super) async fn validate_public_url(url: &str) -> Result<(), FetchError> {
    let parsed = Url::parse(url).map_err(|e| FetchError::InvalidUrl(e.to_string()))?;

    if parsed.scheme() != "http" && parsed.scheme() != "https" {
        return Err(FetchError::InvalidUrl(format!(
            "unsupported scheme '{}' -- only http/https are allowed",
            parsed.scheme()
        )));
    }

    let host = parsed.host_str().ok_or_else(|| FetchError::InvalidUrl("URL has no host".to_string()))?;
    let port = parsed.port_or_known_default().unwrap_or(443);

    let addrs: Vec<IpAddr> = tokio::net::lookup_host((host, port))
        .await
        .map_err(|e| FetchError::InvalidUrl(format!("failed to resolve host '{host}': {e}")))?
        .map(|addr| addr.ip())
        .collect();

    if addrs.is_empty() {
        return Err(FetchError::InvalidUrl(format!("host '{host}' did not resolve to any address")));
    }

    if let Some(ip) = addrs.into_iter().find(|ip| is_blocked(*ip)) {
        return Err(FetchError::SsrfBlocked(format!("{host} resolves to non-public address {ip}")));
    }

    Ok(())
}

fn is_blocked(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => is_blocked_v4(v4),
        IpAddr::V6(v6) => v6.to_ipv4_mapped().is_some_and(is_blocked_v4) || blocked_ipv6_nets().iter().any(|net| net.contains(&v6)),
    }
}

fn is_blocked_v4(v4: Ipv4Addr) -> bool {
    blocked_ipv4_nets().iter().any(|net| net.contains(&v4))
}

/// "This network" (0.0.0.0/8), loopback, RFC1918 private ranges, RFC6598
/// carrier-grade NAT, and link-local (which also covers the
/// 169.254.169.254 cloud metadata endpoint used by AWS/GCP/Azure).
fn blocked_ipv4_nets() -> Vec<Ipv4Net> {
    [
        "0.0.0.0/8",
        "10.0.0.0/8",
        "100.64.0.0/10",
        "127.0.0.0/8",
        "169.254.0.0/16",
        "172.16.0.0/12",
        "192.168.0.0/16",
    ]
    .into_iter()
    .filter_map(|s| s.parse().ok())
    .collect()
}

/// Unspecified, loopback, unique-local (RFC4193, the IPv6 equivalent of
/// RFC1918 private space), and link-local.
fn blocked_ipv6_nets() -> Vec<Ipv6Net> {
    ["::/128", "::1/128", "fc00::/7", "fe80::/10"]
        .into_iter()
        .filter_map(|s| s.parse().ok())
        .collect()
}

#[cfg(test)]
mod tests;
