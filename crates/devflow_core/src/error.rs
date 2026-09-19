use thiserror::Error;

#[derive(Error, Debug)]
pub enum DevflowError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Config error: {0}")]
    Config(String),

    #[error("Project detection error: {0}")]
    ProjectDetection(String),

    #[error("Device error: {0}")]
    Device(String),

    #[error("Build error: {0}")]
    Build(String),

    #[error("Install error: {0}")]
    Install(String),

    #[error("Launch error: {0}")]
    Launch(String),

    #[error("Reload error: {0}")]
    Reload(String),

    #[error("Session error: {0}")]
    Session(String),

    #[error("IPC error: {0}")]
    Ipc(String),

    #[error("Tool execution failed: {0}")]
    ToolExecution(String),

    #[error("Unknown framework: {0}")]
    UnknownFramework(String),

    #[error("Unknown platform: {0}")]
    UnknownPlatform(String),

    #[error("Serialization error: {0}")]
    Serialization(String),
}

pub type Result<T> = std::result::Result<T, DevflowError>;
