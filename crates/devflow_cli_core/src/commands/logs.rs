use colored::*;
use devflow_core::project::Project;
use devflow_devices::DeviceManager;
use devflow_logs::LogParser;
use devflow_platforms::{AndroidPlatformRunner, PlatformRegistry};
use devflow_protocol::{LogEntry, LogLevel};

pub async fn handle_logs(
    follow: bool,
    level: Option<char>,
    tag: Option<String>,
    query: Option<String>,
    limit: usize,
) -> anyhow::Result<()> {
    let current_dir = std::env::current_dir()?;
    let project_opt = Project::detect(&current_dir).ok();
    let platform_hint = project_opt.as_ref().map(|p| p.detected_platform);

    let device_opt = DeviceManager::find_best_match(platform_hint, None).await;

    let target_level = level.map(|c| match c.to_ascii_uppercase() {
        'E' => LogLevel::E,
        'W' => LogLevel::W,
        'D' => LogLevel::D,
        _ => LogLevel::I,
    });

    let format_entry = |entry: &LogEntry| {
        let badge = match entry.level {
            LogLevel::E => "[ERR]".red().bold(),
            LogLevel::W => "[WRN]".yellow().bold(),
            LogLevel::I => "[INF]".green(),
            LogLevel::D => "[DBG]".dimmed(),
        };
        let tag_str = entry
            .tag
            .as_deref()
            .map(|t| format!(" [{}]", t))
            .unwrap_or_default();
        let time_str = entry.timestamp.format("%H:%M:%S%.3f").to_string();
        println!(
            "{} {} {}{}",
            time_str.dimmed(),
            badge,
            entry.message,
            tag_str.cyan()
        );
    };

    let matches_filter = |entry: &LogEntry| -> bool {
        if let Some(ref lvl) = target_level {
            if &entry.level != lvl {
                return false;
            }
        }
        if let Some(ref t) = tag {
            if !entry
                .tag
                .as_deref()
                .map(|etag| etag.to_lowercase().contains(&t.to_lowercase()))
                .unwrap_or(false)
            {
                return false;
            }
        }
        if let Some(ref q) = query {
            if !entry.message.to_lowercase().contains(&q.to_lowercase()) {
                return false;
            }
        }
        true
    };

    if follow {
        let Some(device) = device_opt else {
            println!("{}", "No active device found to stream logs from. Connect a device or run 'devflow devices'.".yellow());
            return Ok(());
        };

        println!(
            "{} Streaming logs from '{}' [{}] (bounded buffer, Ctrl+C to stop)...",
            "⚡".cyan().bold(),
            device.name.bold(),
            device.platform
        );

        let (log_tx, mut log_rx) = tokio::sync::mpsc::channel::<LogEntry>(512);
        let runner = PlatformRegistry::get_runner(device.platform);

        let handle = runner.stream_logs(&current_dir, &device, log_tx).await?;

        tokio::select! {
            _ = async {
                while let Some(entry) = log_rx.recv().await {
                    if matches_filter(&entry) {
                        format_entry(&entry);
                    }
                }
            } => {}
            _ = tokio::signal::ctrl_c() => {
                println!("\n{}", "Log streaming stopped.".dimmed());
            }
        }

        handle.abort();
    } else {
        if let Some(ref device) = device_opt {
            if device.platform == devflow_protocol::Platform::Android {
                let fetch_count = if tag.is_some() || query.is_some() || target_level.is_some() {
                    (limit * 20).clamp(200, 2000)
                } else {
                    limit
                };
                let adb = AndroidPlatformRunner::resolve_adb();
                let output = tokio::process::Command::new(&adb)
                    .args([
                        "-s",
                        &device.id,
                        "logcat",
                        "-d",
                        "-t",
                        &fetch_count.to_string(),
                        "-v",
                        "time",
                    ])
                    .output()
                    .await;

                if let Ok(out) = output {
                    let text = String::from_utf8_lossy(&out.stdout);
                    let mut entries = Vec::new();
                    for line in text.lines().rev() {
                        let entry = LogParser::parse_line(line, Some("logcat"));
                        if matches_filter(&entry) {
                            entries.push(entry);
                            if entries.len() >= limit {
                                break;
                            }
                        }
                    }
                    if entries.is_empty() {
                        println!("{}", "No matching logs found in device buffer.".yellow());
                    } else {
                        entries.reverse();
                        for entry in &entries {
                            format_entry(entry);
                        }
                    }
                    return Ok(());
                }
            }
        }

        println!("{}", "No active session or readable device log buffer found. Use 'devflow dev' or 'devflow logs --follow'.".yellow());
    }

    Ok(())
}
