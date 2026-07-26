use std::process::{Command, Stdio};
use std::io::{Write, BufReader, BufRead};

#[test]
fn test_mcp_list_tools() {
    // 1. Build the binary first
    let build_status = Command::new("cargo")
        .arg("build")
        .status()
        .expect("Failed to run cargo build");
    
    assert!(build_status.success(), "Failed to compile project");

    // 2. Spawn the compiled binary
    let mut child = Command::new("target/debug/web-access-mcp")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn web-access-mcp process");

    let mut stdin = child.stdin.take().expect("Failed to open stdin");
    let stdout = child.stdout.take().expect("Failed to open stdout");

    // 3. Write a list_tools request
    let request = r#"{"jsonrpc":"2.0","method":"list_tools","id":1}"#;
    writeln!(stdin, "{}", request).expect("Failed to write to stdin");

    // 4. Read the response
    let mut reader = BufReader::new(stdout);
    let mut response = String::new();
    reader.read_line(&mut response).expect("Failed to read line from stdout");

    // 5. Verify the response contains expected tools
    assert!(response.contains("tools"), "Response does not contain tools structure");
    assert!(response.contains("google_search"), "Response does not contain google_search tool");
    assert!(response.contains("web_fetch"), "Response does not contain web_fetch tool");

    // Clean up
    let _ = child.kill();
}

#[test]
fn test_mcp_standard_flow() {
    let mut child = Command::new("target/debug/web-access-mcp")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn web-access-mcp process");

    let mut stdin = child.stdin.take().expect("Failed to open stdin");
    let stdout = child.stdout.take().expect("Failed to open stdout");
    let mut reader = BufReader::new(stdout);

    // 1. Send initialize request
    let init_req = r#"{"jsonrpc":"2.0","method":"initialize","id":1,"params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}"#;
    writeln!(stdin, "{}", init_req).expect("Failed to write initialize request");

    let mut init_res = String::new();
    reader.read_line(&mut init_res).expect("Failed to read initialize response");
    assert!(init_res.contains("protocolVersion"), "Initialize response missing protocolVersion");
    assert!(init_res.contains("web-access-mcp"), "Initialize response missing serverInfo name");

    // 2. Send tools/list request
    let tools_req = r#"{"jsonrpc":"2.0","method":"tools/list","id":2}"#;
    writeln!(stdin, "{}", tools_req).expect("Failed to write tools/list request");

    let mut tools_res = String::new();
    reader.read_line(&mut tools_res).expect("Failed to read tools/list response");
    assert!(tools_res.contains("google_search"), "Response missing google_search");
    assert!(tools_res.contains("web_fetch"), "Response missing web_fetch");

    let _ = child.kill();
}
