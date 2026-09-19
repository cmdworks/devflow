use devflow_protocol::{LogEntry, LogFilter, LogLevel};
use std::collections::VecDeque;
use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct LogBuffer {
    capacity: usize,
    entries: Arc<RwLock<VecDeque<LogEntry>>>,
}

impl LogBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            entries: Arc::new(RwLock::new(VecDeque::with_capacity(capacity))),
        }
    }

    pub fn push(&self, entry: LogEntry) {
        let mut list = self.entries.write().unwrap();
        if list.len() >= self.capacity {
            list.pop_front();
        }
        list.push_back(entry);
    }

    pub fn query(&self, filter: &LogFilter) -> Vec<LogEntry> {
        let list = self.entries.read().unwrap();
        if let Some(limit) = filter.limit {
            let mut results: Vec<LogEntry> = list
                .iter()
                .rev()
                .filter(|e| filter.matches(e))
                .take(limit)
                .cloned()
                .collect();
            results.reverse();
            results
        } else {
            list.iter().filter(|e| filter.matches(e)).cloned().collect()
        }
    }

    /// Retrieve a window of logs matching filter with offset from the end for scrollback.
    /// Returns (entries in chronological order, total_matched_count).
    pub fn query_window(&self, filter: &LogFilter, offset: usize, limit: usize) -> (Vec<LogEntry>, usize) {
        let list = self.entries.read().unwrap();
        let matched: Vec<&LogEntry> = list.iter().filter(|e| filter.matches(e)).collect();
        let total = matched.len();
        if total == 0 {
            return (Vec::new(), 0);
        }

        let start = total.saturating_sub(offset + limit);
        let end = total.saturating_sub(offset);
        if start >= end {
            return (Vec::new(), total);
        }

        let window = matched[start..end].iter().copied().cloned().collect();
        (window, total)
    }

    pub fn get_recent_errors(&self, limit: usize) -> Vec<LogEntry> {
        let list = self.entries.read().unwrap();
        list.iter()
            .filter(|e| e.level == LogLevel::E)
            .rev()
            .take(limit)
            .cloned()
            .collect()
    }

    pub fn get_crash_clusters(&self) -> Vec<crate::grouping::CrashCluster> {
        let list = self.entries.read().unwrap();
        let entries_vec: Vec<LogEntry> = list.iter().cloned().collect();
        crate::grouping::CrashAggregator::group_crashes(&entries_vec)
    }

    pub fn clear(&self) {
        let mut list = self.entries.write().unwrap();
        list.clear();
    }


    pub fn len(&self) -> usize {
        self.entries.read().unwrap().len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Default for LogBuffer {
    fn default() -> Self {
        Self::new(2000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_buffer_push_and_query() {
        let buffer = LogBuffer::new(5);
        buffer.push(LogEntry::new(LogLevel::I, "Application started").with_tag("app"));
        buffer.push(LogEntry::new(LogLevel::W, "Slow network detected").with_tag("network"));
        buffer.push(LogEntry::new(LogLevel::E, "Fatal crash in main thread").with_tag("crash"));

        assert_eq!(buffer.len(), 3);

        let filter = LogFilter {
            levels: Some(vec![LogLevel::E]),
            ..Default::default()
        };
        let errors = buffer.query(&filter);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].message, "Fatal crash in main thread");

        let tag_filter = LogFilter {
            tags: Some(vec!["network".to_string()]),
            ..Default::default()
        };
        let net_logs = buffer.query(&tag_filter);
        assert_eq!(net_logs.len(), 1);
        assert_eq!(net_logs[0].level, LogLevel::W);
    }

    #[test]
    fn test_log_buffer_window_and_limit() {
        let buffer = LogBuffer::new(10);
        for i in 0..8 {
            buffer.push(LogEntry::new(LogLevel::I, format!("Log line {}", i)));
        }

        let limit_filter = LogFilter {
            limit: Some(3),
            ..Default::default()
        };
        let recent = buffer.query(&limit_filter);
        assert_eq!(recent.len(), 3);
        assert_eq!(recent[0].message, "Log line 5");
        assert_eq!(recent[1].message, "Log line 6");
        assert_eq!(recent[2].message, "Log line 7");

        // Test window: offset=2, limit=3
        let (window, total) = buffer.query_window(&LogFilter::default(), 2, 3);
        assert_eq!(total, 8);
        assert_eq!(window.len(), 3);
        assert_eq!(window[0].message, "Log line 3");
        assert_eq!(window[1].message, "Log line 4");
        assert_eq!(window[2].message, "Log line 5");
    }
}

