use colored::*;
use devflow_core::event::EventBus;
use devflow_frameworks::session::SessionManager;
use devflow_protocol::LogLevel;
use devflow_tui::TuiRunner;
use std::sync::Arc;

pub async fn handle_dev(
    target: Option<String>,
    framework: Option<String>,
    no_tui: bool,
    json: bool,
) -> anyhow::Result<()> {
    let current_dir = std::env::current_dir()?;
    let event_bus = EventBus::default();

    let session = SessionManager::create(
        &current_dir,
        target.as_deref(),
        framework.as_deref(),
        event_bus.clone(),
    )
    .await?;

    let session_arc = Arc::new(session);

    if no_tui || json {
        println!(
            "{} Starting session in CLI mode for '{}'...",
            "⚡".cyan().bold(),
            session_arc.project.name.bold()
        );
        let mut event_rx = event_bus.subscribe();

        tokio::spawn(async move {
            loop {
                match event_rx.recv().await {
                    Ok(evt) => {
                        if json {
                            if let Ok(js) = serde_json::to_string(&evt) {
                                println!("{}", js);
                            }
                        } else {
                            match evt {
                                devflow_core::event::DevflowEvent::LogAppended {
                                    entry, ..
                                } => {
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
                                    println!("{} {}{}", badge, entry.message, tag_str.cyan());
                                }
                                devflow_core::event::DevflowEvent::SessionStateChanged {
                                    status,
                                    ..
                                } => {
                                    println!(
                                        "{} State: {}",
                                        "⚡".cyan(),
                                        status.to_string().bold()
                                    );
                                }
                                devflow_core::event::DevflowEvent::WatcherTriggered {
                                    action,
                                    paths,
                                    ..
                                } => {
                                    println!(
                                        "{} File changed ({}) -> {:?}",
                                        "👁".yellow(),
                                        action,
                                        paths
                                    );
                                }
                                _ => {}
                            }
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {}
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                }
            }
        });

        session_arc.start_session().await?;
        tokio::signal::ctrl_c().await?;
        session_arc.stop().await?;
    } else {
        let session_for_start = session_arc.clone();
        let session_mut = session_for_start;
        tokio::spawn(async move {
            let _ = session_mut.start_session().await;
        });

        TuiRunner::run(session_arc.clone()).await?;
        session_arc.stop().await?;
    }

    Ok(())
}
