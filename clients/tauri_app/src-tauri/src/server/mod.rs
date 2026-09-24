pub mod assets;
pub mod routes;

use devflow_core::event::EventBus;
use devflow_mcp::McpHandler;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, AtomicU16, Ordering};
use std::sync::mpsc::Sender;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::info;

use crate::state::AppState;

pub fn start_server_thread(
    base_port: u16,
    ready_tx: Sender<Result<u16, String>>,
) {
    std::thread::spawn(move || {
        let rt = match tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
        {
            Ok(runtime) => runtime,
            Err(e) => {
                let _ = ready_tx.send(Err(format!("Failed to create Tokio runtime: {}", e)));
                return;
            }
        };

        rt.block_on(async move {
            let event_bus = EventBus::new(2000);
            let mcp_handler = Arc::new(McpHandler::new().with_event_bus(event_bus.clone()));
            let mcp_enabled = Arc::new(AtomicBool::new(true));
            let actual_port_holder = Arc::new(AtomicU16::new(base_port));

            let state = AppState {
                event_bus,
                active_sessions: Arc::new(Mutex::new(HashMap::new())),
                mcp_handler,
                mcp_enabled,
                actual_port: actual_port_holder.clone(),
            };

            let app = routes::build_router(state);

            // Attempt to bind to base_port, or fallback to auto-incrementing free port
            let port = base_port;
            let mut listener_opt = None;

            for offset in 0..50 {
                let candidate_port = port + offset;
                let addr = SocketAddr::from(([127, 0, 0, 1], candidate_port));
                match tokio::net::TcpListener::bind(addr).await {
                    Ok(l) => {
                        listener_opt = Some((l, candidate_port));
                        break;
                    }
                    Err(_) => continue,
                }
            }

            match listener_opt {
                Some((listener, bound_port)) => {
                    actual_port_holder.store(bound_port, Ordering::Relaxed);
                    let addr = SocketAddr::from(([127, 0, 0, 1], bound_port));
                    info!(target: "devflow_desktop", "DevFlow backend server listening on http://{}", addr);

                    let _ = ready_tx.send(Ok(bound_port));

                    if let Err(e) = axum::serve(listener, app).await {
                        tracing::error!("Server error: {}", e);
                    }
                }
                None => {
                    let err_msg = format!("Could not bind to any port in range {}-{}", port, port + 50);
                    eprintln!("{}", err_msg);
                    let _ = ready_tx.send(Err(err_msg));
                }
            }
        });
    });
}
