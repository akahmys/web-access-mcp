use super::*;

#[test]
fn test_blocks_loopback() {
    assert!(is_blocked("127.0.0.1".parse().unwrap()));
    assert!(is_blocked("::1".parse().unwrap()));
}

#[test]
fn test_blocks_private_ranges() {
    assert!(is_blocked("10.1.2.3".parse().unwrap()));
    assert!(is_blocked("172.16.0.5".parse().unwrap()));
    assert!(is_blocked("192.168.1.1".parse().unwrap()));
}

#[test]
fn test_blocks_cloud_metadata_endpoint() {
    // 169.254.169.254 is the AWS/GCP/Azure instance metadata endpoint.
    assert!(is_blocked("169.254.169.254".parse().unwrap()));
}

#[test]
fn test_blocks_ipv4_mapped_ipv6() {
    // A DNS-rebinding-style bypass attempt: an IPv4-mapped IPv6 literal
    // wrapping a blocked address must still be caught.
    assert!(is_blocked("::ffff:169.254.169.254".parse().unwrap()));
}

#[test]
fn test_blocks_ipv6_unique_local() {
    assert!(is_blocked("fc00::1".parse().unwrap()));
    assert!(is_blocked("fe80::1".parse().unwrap()));
}

#[test]
fn test_allows_public_addresses() {
    assert!(!is_blocked("8.8.8.8".parse().unwrap()));
    assert!(!is_blocked("1.1.1.1".parse().unwrap()));
    assert!(!is_blocked("2606:4700:4700::1111".parse().unwrap()));
}
