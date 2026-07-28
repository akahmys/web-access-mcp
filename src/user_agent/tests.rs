use super::*;

#[test]
fn test_user_agent_is_stable_and_nonempty() {
    let first = user_agent();
    assert!(!first.is_empty());
    // Picked once and cached: repeated calls return the exact same value.
    assert_eq!(first, user_agent());
}
