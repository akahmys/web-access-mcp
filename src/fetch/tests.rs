use super::*;

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
