use crate::classifier::{ChangeAction, ChangeClassifier};
use devflow_core::config::WatchConfig;
use devflow_core::error::{DevflowError, Result};
use notify_debouncer_mini::new_debouncer;
use std::path::{Path, PathBuf};
use std::sync::mpsc as std_mpsc;
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{debug, info, warn};

pub struct FileWatcher {
    root_dir: PathBuf,
    config: WatchConfig,
}

#[derive(Debug, Clone)]
pub struct FileChangeEvent {
    pub paths: Vec<PathBuf>,
    pub action: ChangeAction,
}

impl FileWatcher {
    pub fn new(root_dir: impl AsRef<Path>, config: WatchConfig) -> Self {
        Self {
            root_dir: root_dir.as_ref().to_path_buf(),
            config,
        }
    }

    pub fn start(&self, tx: mpsc::Sender<FileChangeEvent>) -> Result<tokio::task::JoinHandle<()>> {
        let (std_tx, std_rx) = std_mpsc::channel();
        let debounce_ms = self.config.debounce_ms;
        let mut debouncer = new_debouncer(Duration::from_millis(debounce_ms), std_tx)
            .map_err(|e| DevflowError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;

        let watch_paths = if self.config.paths.is_empty() {
            vec![self.root_dir.clone()]
        } else {
            self.config
                .paths
                .iter()
                .map(|p| self.root_dir.join(p))
                .filter(|p| p.exists())
                .collect()
        };

        if watch_paths.is_empty() {
            // fallback to root dir
            debouncer
                .watcher()
                .watch(&self.root_dir, notify::RecursiveMode::Recursive)
                .map_err(|e| DevflowError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;
        } else {
            for path in &watch_paths {
                info!("Watching directory: {}", path.display());
                debouncer
                    .watcher()
                    .watch(path, notify::RecursiveMode::Recursive)
                    .map_err(|e| DevflowError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;
            }
        }

        let config = self.config.clone();

        let handle = tokio::task::spawn_blocking(move || {
            let _debouncer = debouncer;

            while let Ok(res) = std_rx.recv() {
                match res {
                    Ok(events) => {
                        let mut changed_paths = Vec::new();
                        let mut chosen_action = ChangeAction::Ignore;

                        for event in events {
                            let action = ChangeClassifier::classify(&event.path, &config);
                            if action != ChangeAction::Ignore {
                                changed_paths.push(event.path);
                                if action == ChangeAction::Restart || chosen_action == ChangeAction::Ignore {
                                    chosen_action = action;
                                }
                            }
                        }

                        if !changed_paths.is_empty() && chosen_action != ChangeAction::Ignore {
                            debug!("Detected relevant changes in {} files", changed_paths.len());
                            let _ = tx.blocking_send(FileChangeEvent {
                                paths: changed_paths,
                                action: chosen_action,
                            });
                        }
                    }
                    Err(e) => {
                        warn!("Watch error: {:?}", e);
                    }
                }
            }
        });

        Ok(handle)
    }
}
