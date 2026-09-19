use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Platform {
    Android,
    Apple,
    Ios,
    Macos,
    Desktop,
    Generic,
}

impl std::fmt::Display for Platform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Platform::Android => write!(f, "Android"),
            Platform::Apple => write!(f, "Apple"),
            Platform::Ios => write!(f, "iOS"),
            Platform::Macos => write!(f, "macOS"),
            Platform::Desktop => write!(f, "Desktop"),
            Platform::Generic => write!(f, "Generic"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeviceState {
    Connected,
    Booted,
    Shutdown,
    Unavailable,
    Busy,
}

impl std::fmt::Display for DeviceState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeviceState::Connected => write!(f, "Connected"),
            DeviceState::Booted => write!(f, "Booted"),
            DeviceState::Shutdown => write!(f, "Shutdown"),
            DeviceState::Unavailable => write!(f, "Unavailable"),
            DeviceState::Busy => write!(f, "Busy"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Device {
    pub id: String,
    pub name: String,
    pub platform: Platform,
    pub state: DeviceState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_arch: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub os_version: Option<String>,
    #[serde(default)]
    pub is_emulator: bool,
    #[serde(default)]
    pub is_default: bool,
}

impl Device {
    pub fn new(id: impl Into<String>, name: impl Into<String>, platform: Platform, state: DeviceState) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            platform,
            state,
            target_arch: None,
            os_version: None,
            is_emulator: false,
            is_default: false,
        }
    }

    pub fn host_desktop() -> Self {
        Self {
            id: "desktop-host".to_string(),
            name: "Local Desktop Host".to_string(),
            platform: Platform::Desktop,
            state: DeviceState::Connected,
            target_arch: Some(std::env::consts::ARCH.to_string()),
            os_version: Some(std::env::consts::OS.to_string()),
            is_emulator: false,
            is_default: true,
        }
    }
}
