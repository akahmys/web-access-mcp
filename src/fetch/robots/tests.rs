use texting_robots::Robot;

#[test]
fn test_robot_disallow_semantics_match_our_expectations() {
    // Sanity check on texting_robots' own semantics, since check_robots_txt
    // relies on being able to pass the full original URL (not just a path).
    let robot = Robot::new("web-access-mcp", b"User-agent: *\nDisallow: /private").unwrap();
    assert!(!robot.allowed("https://example.com/private/page"));
    assert!(robot.allowed("https://example.com/public/page"));
}

#[test]
fn test_robot_allows_everything_with_empty_rules() {
    let robot = Robot::new("web-access-mcp", b"").unwrap();
    assert!(robot.allowed("https://example.com/anything"));
}
