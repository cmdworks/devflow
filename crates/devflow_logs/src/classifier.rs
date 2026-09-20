use devflow_protocol::{LogEntry, LogLevel};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CrashCategory {
    JavaException {
        exception_class: String,
        message: String,
    },
    RustPanic {
        location: Option<String>,
        message: String,
    },
    Segfault,
    OutOfMemory,
    GenericCrash(String),
}

pub struct LogClassifier;

impl LogClassifier {
    pub fn is_fatal_crash(entry: &LogEntry) -> Option<CrashCategory> {
        let msg = &entry.message;
        let lower = msg.to_lowercase();

        if lower.contains("fatal exception:") {
            let class = msg
                .split("FATAL EXCEPTION:")
                .nth(1)
                .unwrap_or("")
                .trim()
                .to_string();
            return Some(CrashCategory::JavaException {
                exception_class: class,
                message: msg.clone(),
            });
        }

        if lower.contains("panicked at")
            || lower.contains("thread '") && lower.contains("' panicked")
        {
            return Some(CrashCategory::RustPanic {
                location: None,
                message: msg.clone(),
            });
        }

        if lower.contains("segmentation fault") || lower.contains("sigsegv") {
            return Some(CrashCategory::Segfault);
        }

        if lower.contains("outofmemoryerror") || lower.contains("out of memory") {
            return Some(CrashCategory::OutOfMemory);
        }

        if entry.level == LogLevel::E && (lower.contains("crash") || lower.contains("abort")) {
            return Some(CrashCategory::GenericCrash(msg.clone()));
        }

        None
    }
}
