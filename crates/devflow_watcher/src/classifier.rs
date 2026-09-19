use devflow_core::config::WatchConfig;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChangeAction {
    Reload,
    Restart,
    Ignore,
}

pub struct ChangeClassifier;

impl ChangeClassifier {
    pub fn classify(path: &Path, config: &WatchConfig) -> ChangeAction {
        // Ignore hidden files / directories (like .git, .build, target, etc.)
        for component in path.components() {
            let str = component.as_os_str().to_string_lossy();
            if str.starts_with('.') && str != "." && str != ".." {
                return ChangeAction::Ignore;
            }
            if str == "target" || str == "build" || str == ".build" || str == "node_modules" {
                return ChangeAction::Ignore;
            }
        }

        // Check extension match
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        if !config.extensions.is_empty() && !config.extensions.iter().any(|e| e.eq_ignore_ascii_case(&ext)) {
            return ChangeAction::Ignore;
        }

        // Check configured default action
        match config.action.to_lowercase().as_str() {
            "reload" => ChangeAction::Reload,
            "restart" => ChangeAction::Restart,
            _ => ChangeAction::Restart,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_classify_changes() {
        let config = WatchConfig {
            paths: vec!["src".to_string()],
            extensions: vec!["rs".to_string(), "toml".to_string()],
            action: "reload".to_string(),
            debounce_ms: 300,
        };

        // Matching file
        assert_eq!(
            ChangeClassifier::classify(&PathBuf::from("src/main.rs"), &config),
            ChangeAction::Reload
        );

        // Ignored extension
        assert_eq!(
            ChangeClassifier::classify(&PathBuf::from("src/image.png"), &config),
            ChangeAction::Ignore
        );

        // Ignored build folder
        assert_eq!(
            ChangeClassifier::classify(&PathBuf::from("target/debug/build.rs"), &config),
            ChangeAction::Ignore
        );

        // Ignored hidden git folder
        assert_eq!(
            ChangeClassifier::classify(&PathBuf::from(".git/HEAD"), &config),
            ChangeAction::Ignore
        );
    }
}

