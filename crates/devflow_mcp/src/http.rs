use crate::handler::McpHandler;
use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse, Response,
    },
    routing::{get, post},
    Json, Router,
};
use devflow_core::event::EventBus;
use devflow_protocol::JsonRpcRequest;
use futures::stream::Stream;
use serde::Deserialize;
use serde_json::json;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tower_http::cors::CorsLayer;
use tracing::{info, warn};

#[derive(Clone)]
pub struct ServerState {
    pub handler: Arc<McpHandler>,
    pub auth_token: Option<String>,
    pub event_bus: Option<EventBus>,
}

pub struct HttpServer {
    state: ServerState,
    port: u16,
}

#[derive(Deserialize)]
struct AuthQuery {
    token: Option<String>,
}

impl HttpServer {
    pub fn new(handler: Arc<McpHandler>, port: u16) -> Self {
        Self {
            state: ServerState {
                handler,
                auth_token: None,
                event_bus: None,
            },
            port,
        }
    }

    pub fn with_auth(mut self, token: Option<String>) -> Self {
        self.state.auth_token = token;
        self
    }

    pub fn with_event_bus(mut self, event_bus: EventBus) -> Self {
        self.state.event_bus = Some(event_bus);
        self
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let app = Router::new()
            .route("/health", get(health_check))
            .route("/rpc", post(handle_rpc))
            .route("/", post(handle_rpc))
            .route("/events", get(handle_events_sse))
            .layer(CorsLayer::permissive())
            .with_state(self.state.clone());

        let addr = SocketAddr::from(([0, 0, 0, 0], self.port));
        if let Some(ref tok) = self.state.auth_token {
            let masked = if tok.len() > 4 {
                format!("{}...", &tok[..4])
            } else {
                "***".to_string()
            };
            info!(
                "DevFlow MCP HTTP server listening on http://{} (token auth enabled: {})",
                addr, masked
            );
        } else {
            info!(
                "DevFlow MCP HTTP server listening on http://{} (no auth)",
                addr
            );
        }

        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, app).await?;

        Ok(())
    }
}

fn check_auth(state: &ServerState, headers: &HeaderMap, query_token: Option<&str>) -> bool {
    let expected_token = match &state.auth_token {
        Some(t) if !t.is_empty() => t,
        _ => return true, // No token required
    };

    // 1. Check Authorization: Bearer <token>
    if let Some(auth_header) = headers.get("authorization").and_then(|v| v.to_str().ok()) {
        if let Some(token) = auth_header.strip_prefix("Bearer ") {
            if token.trim() == expected_token {
                return true;
            }
        }
    }

    // 2. Check query parameter ?token=<token>
    if let Some(tok) = query_token {
        if tok == expected_token {
            return true;
        }
    }

    false
}

async fn health_check(
    State(state): State<ServerState>,
    headers: HeaderMap,
    Query(query): Query<AuthQuery>,
) -> impl IntoResponse {
    let authenticated = check_auth(&state, &headers, query.token.as_deref());
    let resp = json!({
        "status": "ok",
        "service": "devflow-mcp",
        "version": "0.1.0",
        "auth_required": state.auth_token.is_some(),
        "authenticated": authenticated
    });
    (StatusCode::OK, Json(resp))
}

async fn handle_rpc(
    State(state): State<ServerState>,
    headers: HeaderMap,
    Query(query): Query<AuthQuery>,
    Json(payload): Json<JsonRpcRequest>,
) -> Response {
    if !check_auth(&state, &headers, query.token.as_deref()) {
        warn!("DevFlow MCP unauthorized RPC request attempt");
        let error_resp = devflow_protocol::JsonRpcResponse::error(
            payload.id,
            -32000,
            "Unauthorized: invalid or missing authentication token",
            None,
        );
        return (StatusCode::UNAUTHORIZED, Json(error_resp)).into_response();
    }

    let response = state.handler.handle_request(payload).await;
    (StatusCode::OK, Json(response)).into_response()
}

async fn handle_events_sse(
    State(state): State<ServerState>,
    headers: HeaderMap,
    Query(query): Query<AuthQuery>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, StatusCode> {
    if !check_auth(&state, &headers, query.token.as_deref()) {
        warn!("DevFlow MCP unauthorized SSE request attempt");
        return Err(StatusCode::UNAUTHORIZED);
    }

    let event_bus = state.event_bus.unwrap_or_default();
    let rx = event_bus.subscribe();

    let stream = futures::stream::unfold((rx, true), |(mut rx, is_first)| async move {
        if is_first {
            let conn_evt = Event::default()
                .event("connected")
                .data(json!({ "status": "connected" }).to_string());
            return Some((Ok(conn_evt), (rx, false)));
        }

        match rx.recv().await {
            Ok(evt) => {
                let serialized = serde_json::to_string(&evt).unwrap_or_default();
                let sse_evt = Event::default().event("devflow_event").data(serialized);
                Some((Ok(sse_evt), (rx, false)))
            }
            Err(_) => None,
        }
    });

    Ok(Sse::new(stream).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("keep-alive"),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    #[test]
    fn test_check_auth_no_token_required() {
        let state = ServerState {
            handler: Arc::new(McpHandler::new()),
            auth_token: None,
            event_bus: None,
        };
        let headers = HeaderMap::new();
        assert!(check_auth(&state, &headers, None));
    }

    #[test]
    fn test_check_auth_with_bearer_token() {
        let state = ServerState {
            handler: Arc::new(McpHandler::new()),
            auth_token: Some("secret123".to_string()),
            event_bus: None,
        };

        // No header -> fail
        let headers = HeaderMap::new();
        assert!(!check_auth(&state, &headers, None));

        // Wrong header -> fail
        let mut bad_headers = HeaderMap::new();
        bad_headers.insert("authorization", HeaderValue::from_static("Bearer wrong"));
        assert!(!check_auth(&state, &bad_headers, None));

        // Correct header -> pass
        let mut ok_headers = HeaderMap::new();
        ok_headers.insert(
            "authorization",
            HeaderValue::from_static("Bearer secret123"),
        );
        assert!(check_auth(&state, &ok_headers, None));

        // Correct query param -> pass
        assert!(check_auth(&state, &headers, Some("secret123")));
    }
}
