use crate::mcp::JsonRpcMessage;
use anyhow::{Context, Result};
use serde_json;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::io::{stdin, stdout};

/// A transport layer for MCP communication over standard I/O.
/// 
/// `StdioTransport` reads JSON-RPC messages from `stdin` and writes
/// them to `stdout`, following the MCP protocol requirements.
pub struct StdioTransport {
    reader: BufReader<tokio::io::Stdin>,
    writer: tokio::io::Stdout,
}

impl StdioTransport {
/// Creates a new `StdioTransport` using standard input and output.
    pub fn new() -> Self {
        Self {
            reader: BufReader::new(stdin()),
            writer: stdout(),
        }
    }

    /// Reads the next `JsonRpcMessage` from `stdin`.
    /// 
    /// Returns `Ok(None)` when `stdin` is closed.
    pub async fn read_message(&mut self) -> Result<Option<JsonRpcMessage>> {
        let mut line = String::new();
        let bytes_read = self.reader.read_line(&mut line).await
            .context("Failed to read line from stdin")?;

        if bytes_read == 0 {
            return Ok(None);
        }

        let message: JsonRpcMessage = serde_json::from_str(&line)
            .context("Failed to parse JSON-RPC message")?;
        
        Ok(Some(message))
    }

    /// Writes the given `JsonRpcMessage` to `stdout`, followed by a newline.
    pub async fn write_message(&mut self, message: &JsonRpcMessage) -> Result<()> {
        let mut payload = serde_json::to_vec(message)?;
        payload.push(b'\n');
        self.writer.write_all(&payload).await
            .context("Failed to write message to stdout")?;
        self.writer.flush().await
            .context("Failed to flush stdout")?;
        Ok(())
    }
}
