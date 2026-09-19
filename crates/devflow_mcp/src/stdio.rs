use crate::handler::McpHandler;
use devflow_protocol::{JsonRpcRequest, JsonRpcResponse};
use std::io::{self, BufRead, Write};
use std::sync::Arc;
use tracing::{debug, error};

pub struct StdioServer {
    handler: Arc<McpHandler>,
}

impl StdioServer {
    pub fn new(handler: Arc<McpHandler>) -> Self {
        Self { handler }
    }

    pub async fn run(&self) -> io::Result<()> {
        let stdin = io::stdin();
        let mut stdout = io::stdout();

        for line in stdin.lock().lines() {
            let line = match line {
                Ok(l) => l,
                Err(e) => {
                    error!("Error reading line from stdin: {}", e);
                    break;
                }
            };

            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            debug!("Received MCP stdio line: {}", trimmed);

            let req: JsonRpcRequest = match serde_json::from_str(trimmed) {
                Ok(r) => r,
                Err(e) => {
                    let err_resp = JsonRpcResponse::error(None, -32700, format!("Parse error: {}", e), None);
                    let out = serde_json::to_string(&err_resp).unwrap();
                    writeln!(stdout, "{}", out)?;
                    stdout.flush()?;
                    continue;
                }
            };

            let response = self.handler.handle_request(req).await;
            let out = serde_json::to_string(&response).unwrap();
            writeln!(stdout, "{}", out)?;
            stdout.flush()?;
        }

        Ok(())
    }
}
