use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum LogLevel {
    D,
    I,
    W,
    E,
}

impl LogLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::D => "DEBUG",
            LogLevel::I => "INFO",
            LogLevel::W => "WARN",
            LogLevel::E => "ERROR",
        }
    }

    pub fn parse_char(c: char) -> Option<Self> {
        match c.to_ascii_uppercase() {
            'D' | 'V' => Some(LogLevel::D),
            'I' => Some(LogLevel::I),
            'W' => Some(LogLevel::W),
            'E' | 'F' => Some(LogLevel::E),
            _ => None,
        }
    }
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LogEntry {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub level: LogLevel,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub process_id: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

impl LogEntry {
    pub fn new(level: LogLevel, message: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            level,
            tag: None,
            message: message.into(),
            process_id: None,
            source: None,
        }
    }

    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tag = Some(tag.into());
        self
    }

    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LogFilter {
    pub levels: Option<Vec<LogLevel>>,
    pub tags: Option<Vec<String>>,
    pub query: Option<String>,
    pub limit: Option<usize>,
    pub since: Option<DateTime<Utc>>,
}

impl LogFilter {
    pub fn matches(&self, entry: &LogEntry) -> bool {
        if let Some(since) = self.since {
            if entry.timestamp < since {
                return false;
            }
        }

        if let Some(levels) = &self.levels {
            if !levels.is_empty() && !levels.contains(&entry.level) {
                return false;
            }
        }

        if let Some(tags) = &self.tags {
            if !tags.is_empty() {
                match &entry.tag {
                    Some(tag) => {
                        if !tags.iter().any(|t| t.eq_ignore_ascii_case(tag)) {
                            return false;
                        }
                    }
                    None => return false,
                }
            }
        }

        if let Some(query) = &self.query {
            if !query.is_empty() {
                let q_lower = query.to_lowercase();
                let msg_match = entry.message.to_lowercase().contains(&q_lower);
                let tag_match = entry.tag.as_ref().map(|t| t.to_lowercase().contains(&q_lower)).unwrap_or(false);
                if !msg_match && !tag_match {
                    return false;
                }
            }
        }

        true
    }
}
