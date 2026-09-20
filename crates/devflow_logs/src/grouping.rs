use chrono::{DateTime, Utc};
use devflow_protocol::{LogEntry, LogLevel};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CrashCluster {
    pub id: String,
    pub exception_type: String,
    pub message: String,
    pub top_frame: Option<String>,
    pub stack_trace: Vec<String>,
    pub occurrences: usize,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
}

pub struct CrashAggregator;

impl CrashAggregator {
    pub fn group_crashes(entries: &[LogEntry]) -> Vec<CrashCluster> {
        let mut clusters: HashMap<String, CrashCluster> = HashMap::new();
        let mut i = 0;

        while i < entries.len() {
            let entry = &entries[i];

            // Check if this line marks the start of a crash or exception
            let is_crash_start = entry.level == LogLevel::E
                && (entry.message.contains("FATAL EXCEPTION")
                    || entry.message.contains("Exception in thread")
                    || entry.message.contains("fatal error:")
                    || entry.message.contains("panicked at")
                    || entry.message.contains("Exception:")
                    || entry.message.contains("Error:")
                    || entry.message.contains("SIGSEGV")
                    || entry.message.contains("SIGABRT"));

            if is_crash_start {
                let mut stack_trace = vec![entry.message.clone()];
                let mut top_frame = None;
                let first_seen = entry.timestamp;
                let mut last_seen = entry.timestamp;

                let mut exc_type = Self::extract_exception_type(&entry.message);

                // Collect subsequent lines that look like part of this crash report
                let mut j = i + 1;
                while j < entries.len() {
                    let next = &entries[j];
                    let next_msg = next.message.trim();

                    let is_stack_continuation = next_msg.starts_with("at ")
                        || next_msg.starts_with("Caused by:")
                        || next_msg.starts_with("... ")
                        || next_msg.starts_with("Process:")
                        || next_msg.starts_with("PID:")
                        || next_msg.contains("Exception:")
                        || next_msg.contains("stack backtrace:")
                        || (next.level == LogLevel::E && next_msg.starts_with("java."));

                    if is_stack_continuation {
                        if next_msg.contains("Exception:")
                            && (exc_type == "UnknownException" || exc_type == "main")
                        {
                            exc_type = Self::extract_exception_type(next_msg);
                        }
                        if top_frame.is_none() && next_msg.starts_with("at ") {
                            top_frame = Some(next_msg.to_string());
                        }
                        stack_trace.push(next.message.clone());
                        last_seen = next.timestamp;
                        j += 1;
                    } else {
                        break;
                    }
                }

                let key = format!("{}:{}", exc_type, top_frame.as_deref().unwrap_or(""));

                clusters
                    .entry(key)
                    .and_modify(|c| {
                        c.occurrences += 1;
                        c.last_seen = last_seen;
                    })
                    .or_insert_with(|| CrashCluster {
                        id: uuid::Uuid::new_v4().to_string(),
                        exception_type: exc_type,
                        message: entry.message.clone(),
                        top_frame,
                        stack_trace,
                        occurrences: 1,
                        first_seen,
                        last_seen,
                    });

                i = j;
            } else {
                i += 1;
            }
        }

        let mut result: Vec<CrashCluster> = clusters.into_values().collect();
        result.sort_by_key(|a| std::cmp::Reverse(a.occurrences));
        result
    }

    fn extract_exception_type(line: &str) -> String {
        if let Some(pos) = line.find("Exception:") {
            let before = &line[..pos + "Exception".len()];
            let words: Vec<&str> = before.split_whitespace().collect();
            if let Some(last) = words.last() {
                return last.to_string();
            }
        }

        if let Some(pos) = line.find("FATAL EXCEPTION:") {
            let rest = &line[pos + "FATAL EXCEPTION:".len()..];
            return rest.trim().to_string();
        }

        if line.contains("panicked at") {
            return "RustPanic".to_string();
        }

        "UnknownException".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_group_android_crashes() {
        let entries = vec![
            LogEntry::new(LogLevel::E, "FATAL EXCEPTION: main").with_tag("AndroidRuntime"),
            LogEntry::new(
                LogLevel::E,
                "java.lang.NullPointerException: Attempt to invoke virtual method on null object",
            )
            .with_tag("AndroidRuntime"),
            LogEntry::new(
                LogLevel::E,
                "    at com.example.MainActivity.onCreate(MainActivity.kt:42)",
            )
            .with_tag("AndroidRuntime"),
            LogEntry::new(
                LogLevel::E,
                "    at android.app.Activity.performCreate(Activity.java:8051)",
            )
            .with_tag("AndroidRuntime"),
        ];

        let clusters = CrashAggregator::group_crashes(&entries);
        assert_eq!(clusters.len(), 1);
        assert_eq!(clusters[0].exception_type, "java.lang.NullPointerException");
        assert_eq!(
            clusters[0].top_frame.as_deref(),
            Some("at com.example.MainActivity.onCreate(MainActivity.kt:42)")
        );
        assert_eq!(clusters[0].stack_trace.len(), 4);
    }
}
