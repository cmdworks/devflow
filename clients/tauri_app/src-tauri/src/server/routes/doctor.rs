use axum::{extract::Query, Json};
use devflow_core::doctor::DoctorEngine;
use devflow_protocol::DoctorReport;
use std::path::PathBuf;

use super::workspace::WorkspaceQuery;

pub async fn handle_doctor(
    Query(query): Query<WorkspaceQuery>,
) -> Json<DoctorReport> {
    let dir = query.dir.map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."));
    let report = DoctorEngine::run_diagnostics(&dir).await;
    Json(report)
}
