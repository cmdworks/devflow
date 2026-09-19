use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckStatus {
    Passed,
    Warning,
    Failed,
    Skipped,
}

impl std::fmt::Display for CheckStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CheckStatus::Passed => write!(f, "PASSED"),
            CheckStatus::Warning => write!(f, "WARNING"),
            CheckStatus::Failed => write!(f, "FAILED"),
            CheckStatus::Skipped => write!(f, "SKIPPED"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoctorCheck {
    pub name: String,
    pub status: CheckStatus,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detected_version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fix_hint: Option<String>,
}

impl DoctorCheck {
    pub fn pass(name: impl Into<String>, message: impl Into<String>, version: Option<String>) -> Self {
        Self {
            name: name.into(),
            status: CheckStatus::Passed,
            message: message.into(),
            detected_version: version,
            fix_hint: None,
        }
    }

    pub fn warn(name: impl Into<String>, message: impl Into<String>, fix_hint: Option<String>) -> Self {
        Self {
            name: name.into(),
            status: CheckStatus::Warning,
            message: message.into(),
            detected_version: None,
            fix_hint,
        }
    }

    pub fn fail(name: impl Into<String>, message: impl Into<String>, fix_hint: Option<String>) -> Self {
        Self {
            name: name.into(),
            status: CheckStatus::Failed,
            message: message.into(),
            detected_version: None,
            fix_hint,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoctorReport {
    pub project_path: String,
    pub checks: Vec<DoctorCheck>,
    pub passed_count: usize,
    pub warning_count: usize,
    pub failure_count: usize,
}

impl DoctorReport {
    pub fn new(project_path: impl Into<String>, checks: Vec<DoctorCheck>) -> Self {
        let mut passed_count = 0;
        let mut warning_count = 0;
        let mut failure_count = 0;

        for check in &checks {
            match check.status {
                CheckStatus::Passed => passed_count += 1,
                CheckStatus::Warning => warning_count += 1,
                CheckStatus::Failed => failure_count += 1,
                CheckStatus::Skipped => {}
            }
        }

        Self {
            project_path: project_path.into(),
            checks,
            passed_count,
            warning_count,
            failure_count,
        }
    }

    pub fn is_healthy(&self) -> bool {
        self.failure_count == 0
    }
}
