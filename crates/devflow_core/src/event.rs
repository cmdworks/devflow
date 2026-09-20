use devflow_protocol::{BuildResult, Device, LogEntry, SessionState, SessionStatus};
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum DevflowEvent {
    SessionStateChanged {
        session_id: String,
        status: SessionStatus,
        state: Box<SessionState>,
    },
    BuildStarted {
        session_id: String,
        project_name: String,
    },
    BuildCompleted {
        session_id: String,
        result: BuildResult,
    },
    LogAppended {
        session_id: String,
        entry: LogEntry,
    },
    WatcherTriggered {
        session_id: String,
        paths: Vec<String>,
        action: String,
    },
    DeviceDiscovered {
        device: Device,
    },
    ErrorOccurred {
        session_id: Option<String>,
        message: String,
    },
    McpAccessLog {
        entry: serde_json::Value,
    },
}

#[derive(Clone)]
pub struct EventBus {
    sender: broadcast::Sender<DevflowEvent>,
}

impl EventBus {
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender }
    }

    pub fn publish(&self, event: DevflowEvent) {
        let _ = self.sender.send(event);
    }

    pub fn subscribe(&self) -> broadcast::Receiver<DevflowEvent> {
        self.sender.subscribe()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new(512)
    }
}
