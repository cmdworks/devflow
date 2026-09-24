use axum::http::StatusCode;
use axum::response::IntoResponse;
use tracing::warn;

#[derive(rust_embed::Embed)]
#[folder = "../dist"]
pub struct FrontendAssets;

pub async fn index_handler() -> impl IntoResponse {
    serve_asset("index.html")
}

pub async fn fallback_handler(req: axum::extract::Request) -> axum::response::Response {
    let path = req.uri().path().to_string();
    let method = req.method().clone();

    // If an API or RPC endpoint was requested but not matched, return 404 JSON (never HTML!)
    if path.starts_with("/api/") || path == "/rpc" {
        warn!("API route not found: {} {}", method, path);
        return (
            StatusCode::NOT_FOUND,
            [(axum::http::header::CONTENT_TYPE, "application/json")],
            serde_json::json!({
                "error": format!("API route not found: {} {}", method, path)
            })
            .to_string(),
        )
            .into_response();
    }

    // Static assets only respond to GET and HEAD
    if method != axum::http::Method::GET && method != axum::http::Method::HEAD {
        warn!("Method not allowed for static asset: {} {}", method, path);
        return (StatusCode::METHOD_NOT_ALLOWED, "Method Not Allowed").into_response();
    }

    let clean_path = path.trim_start_matches('/');
    serve_asset(clean_path)
}

pub fn serve_asset(path: &str) -> axum::response::Response {
    let asset_path = if path.is_empty() { "index.html" } else { path };

    match FrontendAssets::get(asset_path) {
        Some(content) => {
            let mime = mime_guess::from_path(asset_path).first_or_octet_stream();
            (
                [(axum::http::header::CONTENT_TYPE, mime.as_ref())],
                content.data.into_owned(),
            )
                .into_response()
        }
        None => {
            if let Some(index_content) = FrontendAssets::get("index.html") {
                (
                    [(axum::http::header::CONTENT_TYPE, "text/html")],
                    index_content.data.into_owned(),
                )
                    .into_response()
            } else {
                (StatusCode::NOT_FOUND, "404 Not Found").into_response()
            }
        }
    }
}
