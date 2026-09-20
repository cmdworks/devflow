pub mod config;
pub mod doctor;
pub mod env;
pub mod error;
pub mod event;
pub mod explainer;
pub mod ipc;
pub mod project;
pub mod registry;

pub use config::DevflowConfig;
pub use doctor::DoctorEngine;
pub use env::init_environment;
pub use error::{DevflowError, Result};
pub use event::{DevflowEvent, EventBus};
pub use explainer::{ErrorExplanation, ToolchainExplainer};
pub use ipc::{IpcClient, IpcRequest, IpcResponse, IpcServer};
pub use project::{DetectedFramework, Project, ProjectTarget};
pub use registry::{ActiveSessionInfo, GlobalRegistry, KnownProject};
