use devflow_protocol::{LogEntry, LogLevel};
use regex::Regex;
use std::sync::LazyLock;

static LOGCAT_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(?:[0-9\-]+\s+[0-9:\.]+\s+)?(?:[0-9]+\s+[0-9]+\s+)?([VDIWEF])(?:\s+|/)([^\(:]+?)(?:\(\s*([0-9]+)\))?:\s*(.*)$").unwrap()
});

pub struct LogParser;

impl LogParser {
    pub fn parse_line(line: &str, default_source: Option<&str>) -> LogEntry {
        let trimmed = line.trim_end();

        // 1. Try Logcat parsing
        if let Some(caps) = LOGCAT_RE.captures(trimmed) {
            if let Some(level_char) = caps.get(1) {
                let lvl = LogLevel::parse_char(level_char.as_str().chars().next().unwrap_or('I'))
                    .unwrap_or(LogLevel::I);
                let tag = caps.get(2).map(|m| m.as_str().trim().to_string());
                let msg = caps
                    .get(4)
                    .map(|m| m.as_str().to_string())
                    .unwrap_or_default();

                let mut entry = LogEntry::new(lvl, msg);
                entry.tag = tag;
                entry.source = default_source.map(|s| s.to_string());
                return entry;
            }
        }

        // 2. Fallback heuristic detection for errors/warnings in stdout/stderr
        let lower = trimmed.to_lowercase();
        let level = if lower.contains("error:")
            || lower.contains("fatal")
            || lower.contains("panic:")
            || lower.contains("exception")
            || lower.starts_with("e/")
        {
            LogLevel::E
        } else if lower.contains("warn:") || lower.contains("warning:") || lower.starts_with("w/") {
            LogLevel::W
        } else if lower.contains("debug:") || lower.starts_with("d/") {
            LogLevel::D
        } else {
            LogLevel::I
        };

        let mut entry = LogEntry::new(level, trimmed);
        entry.source = default_source.map(|s| s.to_string());
        entry
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_logcat_formats() {
        // -v time format
        let e1 = LogParser::parse_line(
            "09-19 23:13:01.551 I/View    (26435): Frame rendered in 12ms",
            Some("logcat"),
        );
        assert_eq!(e1.level, LogLevel::I);
        assert_eq!(e1.tag.as_deref(), Some("View"));
        assert_eq!(e1.message, "Frame rendered in 12ms");

        // -v threadtime format
        let e2 = LogParser::parse_line(
            "09-19 23:13:01.551  1511  1520 W ActivityManager: Slow dispatch",
            Some("logcat"),
        );
        assert_eq!(e2.level, LogLevel::W);
        assert_eq!(e2.tag.as_deref(), Some("ActivityManager"));
        assert_eq!(e2.message, "Slow dispatch");

        // brief format
        let e3 = LogParser::parse_line("E/AndroidRuntime(1234): FATAL EXCEPTION", Some("logcat"));
        assert_eq!(e3.level, LogLevel::E);
        assert_eq!(e3.tag.as_deref(), Some("AndroidRuntime"));
        assert_eq!(e3.message, "FATAL EXCEPTION");
    }
}
