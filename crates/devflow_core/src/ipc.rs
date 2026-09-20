use crate::error::{DevflowError, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tracing::{debug, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", content = "payload")]
pub enum IpcRequest {
    Reload { files: Vec<String> },
    Restart,
    Status,
    Stop,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcResponse {
    pub success: bool,
    pub message: String,
}

impl IpcResponse {
    pub fn ok(msg: impl Into<String>) -> Self {
        Self {
            success: true,
            message: msg.into(),
        }
    }

    pub fn err(msg: impl Into<String>) -> Self {
        Self {
            success: false,
            message: msg.into(),
        }
    }
}

pub type IpcCommandHandler =
    Arc<dyn Fn(IpcRequest) -> tokio::sync::oneshot::Receiver<IpcResponse> + Send + Sync>;

pub struct IpcServer {
    pub socket_path: PathBuf,
}

impl IpcServer {
    pub fn new(socket_path: impl AsRef<Path>) -> Self {
        Self {
            socket_path: socket_path.as_ref().to_path_buf(),
        }
    }

    pub async fn start<F, Fut>(&self, handler: F) -> Result<tokio::task::JoinHandle<()>>
    where
        F: Fn(IpcRequest) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = IpcResponse> + Send + 'static,
    {
        if self.socket_path.exists() {
            let _ = std::fs::remove_file(&self.socket_path);
        }

        let listener = UnixListener::bind(&self.socket_path).map_err(|e| {
            DevflowError::Ipc(format!(
                "Failed to bind IPC socket {}: {}",
                self.socket_path.display(),
                e
            ))
        })?;

        let socket_path_clone = self.socket_path.clone();
        let handler_arc = Arc::new(handler);

        let handle = tokio::spawn(async move {
            debug!("IPC Server listening on {}", socket_path_clone.display());
            loop {
                match listener.accept().await {
                    Ok((stream, _)) => {
                        let h = handler_arc.clone();
                        tokio::spawn(async move {
                            if let Err(e) = Self::handle_connection(stream, h).await {
                                debug!("IPC connection ended: {}", e);
                            }
                        });
                    }
                    Err(e) => {
                        warn!("IPC accept error: {}", e);
                        break;
                    }
                }
            }
            let _ = std::fs::remove_file(&socket_path_clone);
        });

        Ok(handle)
    }

    async fn handle_connection<F, Fut>(mut stream: UnixStream, handler: Arc<F>) -> Result<()>
    where
        F: Fn(IpcRequest) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = IpcResponse> + Send + 'static,
    {
        let (reader, mut writer) = stream.split();
        let mut lines = BufReader::new(reader).lines();

        if let Ok(Some(line)) = lines.next_line().await {
            let resp = match serde_json::from_str::<IpcRequest>(&line) {
                Ok(req) => handler(req).await,
                Err(e) => IpcResponse::err(format!("Invalid IPC request JSON: {}", e)),
            };

            let mut out = serde_json::to_string(&resp).unwrap_or_default();
            out.push('\n');
            let _ = writer.write_all(out.as_bytes()).await;
            let _ = writer.flush().await;
        }

        Ok(())
    }
}

pub struct IpcClient;

impl IpcClient {
    pub async fn send_command(
        socket_path: impl AsRef<Path>,
        req: IpcRequest,
    ) -> Result<IpcResponse> {
        let path = socket_path.as_ref();
        if !path.exists() {
            return Err(DevflowError::Ipc(format!(
                "IPC socket not found at {}",
                path.display()
            )));
        }

        let mut stream = UnixStream::connect(path)
            .await
            .map_err(|e| DevflowError::Ipc(format!("Failed to connect to IPC socket: {}", e)))?;

        let (reader, mut writer) = stream.split();
        let mut msg =
            serde_json::to_string(&req).map_err(|e| DevflowError::Serialization(e.to_string()))?;
        msg.push('\n');

        writer
            .write_all(msg.as_bytes())
            .await
            .map_err(DevflowError::Io)?;
        writer.flush().await.map_err(DevflowError::Io)?;

        let mut lines = BufReader::new(reader).lines();
        let resp_line = tokio::time::timeout(std::time::Duration::from_secs(5), lines.next_line())
            .await
            .map_err(|_| DevflowError::Ipc("IPC response timed out".to_string()))?
            .map_err(DevflowError::Io)?
            .ok_or_else(|| DevflowError::Ipc("Empty response from session IPC".to_string()))?;

        let resp = serde_json::from_str::<IpcResponse>(&resp_line)
            .map_err(|e| DevflowError::Ipc(format!("Failed to parse IPC response: {}", e)))?;

        Ok(resp)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ipc_roundtrip() {
        let sock_path =
            std::env::temp_dir().join(format!("test_devflow_{}.sock", std::process::id()));
        let _ = std::fs::remove_file(&sock_path);

        let server = IpcServer::new(&sock_path);
        let _server = server
            .start(|req| async move {
                match req {
                    IpcRequest::Reload { .. } => IpcResponse::ok("Reload dispatched successfully"),
                    IpcRequest::Restart => IpcResponse::ok("Restart dispatched successfully"),
                    IpcRequest::Status => IpcResponse::ok("Running"),
                    _ => IpcResponse::err("Unknown command"),
                }
            })
            .await
            .unwrap();

        // Give server a few ms to bind
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        let reload_resp = IpcClient::send_command(&sock_path, IpcRequest::Reload { files: vec![] })
            .await
            .unwrap();
        assert!(reload_resp.success);
        assert_eq!(reload_resp.message, "Reload dispatched successfully");

        let restart_resp = IpcClient::send_command(&sock_path, IpcRequest::Restart)
            .await
            .unwrap();
        assert!(restart_resp.success);
        assert_eq!(restart_resp.message, "Restart dispatched successfully");
    }
}
