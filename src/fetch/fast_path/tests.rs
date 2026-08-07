use super::*;

#[test]
fn test_github_raw_url_resolver() {
    assert_eq!(
        get_github_raw_url("https://github.com/user/repo/blob/main/src/lib.rs"),
        Some("https://raw.githubusercontent.com/user/repo/main/src/lib.rs".to_string())
    );
    assert_eq!(get_github_raw_url("https://google.com"), None);
}
