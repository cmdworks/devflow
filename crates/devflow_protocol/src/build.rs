use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactInfo {
    pub path: String,
    pub size_bytes: Option<u64>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildResult {
    pub success: bool,
    pub duration_ms: u64,
    pub artifact: Option<ArtifactInfo>,
    pub stdout: String,
    pub stderr: String,
    pub error_message: Option<String>,
}

impl BuildResult {
    pub fn ok(
        duration_ms: u64,
        artifact_path: Option<String>,
        stdout: String,
        stderr: String,
    ) -> Self {
        Self {
            success: true,
            duration_ms,
            artifact: artifact_path.map(|p| ArtifactInfo {
                path: p,
                size_bytes: None,
                created_at: Some(chrono::Utc::now()),
            }),
            stdout,
            stderr,
            error_message: None,
        }
    }

    pub fn failed(duration_ms: u64, stdout: String, stderr: String, error_message: String) -> Self {
        Self {
            success: false,
            duration_ms,
            artifact: None,
            stdout,
            stderr,
            error_message: Some(error_message),
        }
    }
}
