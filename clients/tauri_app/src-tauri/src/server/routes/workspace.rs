use axum::{extract::Query, http::StatusCode, Json};
use devflow_core::project::Project;
use devflow_core::registry::{GlobalRegistry, KnownProject};
use devflow_protocol::{Device, Platform};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::devices::get_or_refresh_devices;

#[derive(Deserialize)]
pub struct WorkspaceQuery {
    pub dir: Option<String>,
}

#[derive(Serialize)]
pub struct WorkspaceResponse {
    pub workspace_name: String,
    pub workspace_path: String,
    pub targets: Vec<devflow_core::project::ProjectTarget>,
    pub devices: Vec<Device>,
    pub active_sessions: Vec<devflow_core::registry::ActiveSessionInfo>,
}

#[derive(Deserialize)]
pub struct WorkspacePathRequest {
    pub path: String,
}

pub fn resolve_workspace_path(input: &str) -> Option<PathBuf> {
    let raw = input.trim().trim_matches('"').trim_matches('\'');
    let raw = raw.strip_prefix("file://").unwrap_or(raw);
    let raw = raw.trim_end_matches('/');
    if raw.is_empty() {
        return None;
    }

    // 1. Tilde expansion (~/Dev/...)
    let expanded = if raw.starts_with("~/") || raw == "~" {
        if let Some(home) = dirs::home_dir() {
            if raw == "~" {
                home
            } else {
                home.join(&raw[2..])
            }
        } else {
            PathBuf::from(raw)
        }
    } else {
        PathBuf::from(raw)
    };

    if expanded.exists() && expanded.is_dir() {
        return Some(expanded.canonicalize().unwrap_or(expanded));
    }

    // 2. Relative to current working dir
    if let Ok(cwd) = std::env::current_dir() {
        let joined = cwd.join(raw);
        if joined.exists() && joined.is_dir() {
            return Some(joined.canonicalize().unwrap_or(joined));
        }
    }

    // 3. Match against known projects by name or path (case-insensitive)
    let known = GlobalRegistry::list_projects();
    for p in known {
        let p_clean = p.path.trim_end_matches('/');
        if p.name.eq_ignore_ascii_case(raw) || p_clean.eq_ignore_ascii_case(raw) {
            let pb = PathBuf::from(&p.path);
            if pb.exists() && pb.is_dir() {
                return Some(pb.canonicalize().unwrap_or(pb));
            }
        }
    }

    None
}

pub async fn handle_list_workspaces() -> Json<Vec<KnownProject>> {
    Json(GlobalRegistry::list_projects())
}

pub async fn handle_add_workspace(
    Json(req): Json<WorkspacePathRequest>,
) -> Result<Json<WorkspaceResponse>, (StatusCode, String)> {
    let canonical = match resolve_workspace_path(&req.path) {
        Some(p) => p,
        None => {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("Directory does not exist or cannot be resolved: {}", req.path),
            ));
        }
    };
    let name = canonical
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "Workspace".to_string());
    let targets = Project::discover_workspace_targets(&canonical);
    let devices = get_or_refresh_devices().await;
    let active_sessions = GlobalRegistry::list_active_sessions();

    let primary_platform = targets
        .first()
        .map(|t| t.platform)
        .unwrap_or(Platform::Generic);
    let primary_framework = targets
        .first()
        .map(|t| t.framework.clone())
        .unwrap_or_else(|| "generic".to_string());

    GlobalRegistry::record_project(&canonical, &name, primary_platform, &primary_framework);

    Ok(Json(WorkspaceResponse {
        workspace_name: name,
        workspace_path: canonical.display().to_string(),
        targets,
        devices,
        active_sessions,
    }))
}

pub async fn handle_remove_workspace(
    Json(req): Json<WorkspacePathRequest>,
) -> Json<serde_json::Value> {
    let clean = req.path.trim().trim_matches('"').trim_matches('\'');
    let clean = clean.strip_prefix("file://").unwrap_or(clean);
    let p = PathBuf::from(clean);
    GlobalRegistry::remove_project(&p);
    Json(serde_json::json!({ "success": true }))
}

pub async fn handle_workspace(
    Query(query): Query<WorkspaceQuery>,
) -> Json<WorkspaceResponse> {
    let canonical = query
        .dir
        .as_deref()
        .and_then(resolve_workspace_path)
        .unwrap_or_else(|| {
            std::env::current_dir()
                .unwrap_or_else(|_| PathBuf::from("."))
                .canonicalize()
                .unwrap_or_else(|_| PathBuf::from("."))
        });

    let workspace_name = canonical
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "Workspace".to_string());

    let targets = Project::discover_workspace_targets(&canonical);
    let devices = get_or_refresh_devices().await;
    let active_sessions = GlobalRegistry::list_active_sessions();

    Json(WorkspaceResponse {
        workspace_name,
        workspace_path: canonical.display().to_string(),
        targets,
        devices,
        active_sessions,
    })
}
