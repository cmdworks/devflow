use crate::handler::McpHandler;
use devflow_protocol::{JsonRpcRequest, JsonRpcResponse};
use std::io::{self, IsTerminal};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tracing::{debug, error};

pub struct StdioServer {
    handler: Arc<McpHandler>,
}

impl StdioServer {
    pub fn new(handler: Arc<McpHandler>) -> Self {
        Self { handler }
    }

    pub async fn run(&self) -> io::Result<()> {
        if std::io::stdin().is_terminal() {
            eprintln!("⚡ DevFlow Model Context Protocol (MCP) Stdio Server");
            eprintln!("   Status: Listening for JSON-RPC messages on stdin/stdout...");
            eprintln!(
                "   Host Integration: Ready for Claude Desktop, Cursor, Antigravity, or VS Code."
            );
            eprintln!("   (Type JSON-RPC 2.0 requests or press Ctrl+C to exit)\n");
        }

        let stdin = tokio::io::stdin();
        let mut stdout = tokio::io::stdout();
        let mut reader = BufReader::new(stdin);
        let mut line = String::new();

        loop {
            line.clear();
            match reader.read_line(&mut line).await {
                Ok(0) => break, // EOF reached
                Ok(_) => {
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        continue;
                    }

                    debug!("Received MCP stdio line: {}", trimmed);

                    let req: JsonRpcRequest = match serde_json::from_str(trimmed) {
                        Ok(r) => r,
                        Err(e) => {
                            let err_resp = JsonRpcResponse::error(
                                None,
                                -32700,
                                format!("Parse error: {}", e),
                                None,
                            );
                            if let Ok(out) = serde_json::to_string(&err_resp) {
                                let _ = stdout.write_all(out.as_bytes()).await;
                                let _ = stdout.write_all(b"\n").await;
                                let _ = stdout.flush().await;
                            }
                            continue;
                        }
                    };

                    let response = self.handler.handle_request(req).await;
                    if let Ok(out) = serde_json::to_string(&response) {
                        let _ = stdout.write_all(out.as_bytes()).await;
                        let _ = stdout.write_all(b"\n").await;
                        let _ = stdout.flush().await;
                    }
                }
                Err(e) => {
                    error!("Error reading line from stdin: {}", e);
                    break;
                }
            }
        }

        Ok(())
    }
}
