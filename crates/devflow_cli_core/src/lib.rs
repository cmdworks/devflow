pub mod mcp_connect;
pub mod shell;

use clap::{Parser, Subcommand};
use colored::*;
use devflow_core::config::DevflowConfig;
use devflow_core::doctor::DoctorEngine;
use devflow_core::event::EventBus;
use devflow_core::project::Project;
use devflow_devices::DeviceManager;
use devflow_frameworks::adapter::BuildContext;
use devflow_frameworks::registry::FrameworkRegistry;
use devflow_frameworks::session::SessionManager;
use devflow_logs::LogParser;
use devflow_mcp::{HttpServer, McpHandler, StdioServer};
use devflow_platforms::{AndroidPlatformRunner, PlatformRegistry};
use devflow_protocol::{CheckStatus, LogEntry, LogLevel};
use devflow_tui::TuiRunner;
use std::path::PathBuf;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub use devflow_core::init_environment;

#[derive(Debug, PartialEq, Eq)]
pub enum CliAction {
    Executed,
    LaunchGui {
        dir: PathBuf,
        port: u16,
        open_browser: bool,
    },
    LaunchTui {
        dir: PathBuf,
    },
}

#[derive(Parser, Debug)]
#[command(name = "devflow", author, version, about = "Universal, framework-aware development runner, desktop companion, and MCP server", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Change working directory before executing
    #[arg(short = 'C', long = "dir", global = true)]
    pub dir: Option<PathBuf>,

    /// Launch GUI desktop companion app
    #[arg(long = "gui", global = true)]
    pub gui: bool,

    /// Open in external web browser
    #[arg(long = "web", global = true)]
    pub web: bool,

    /// Set verbose output level
    #[arg(short, long, global = true)]
    pub verbose: bool,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Scaffold devflow.toml for the current project
    Init {
        /// Target platform (android, apple, macos, desktop, generic)
        #[arg(short, long)]
        platform: Option<String>,

        /// Target framework (kotlin, swift, react-native, flutter, generic)
        #[arg(short, long)]
        framework: Option<String>,
    },

    /// Check development tooling (adb, xcodebuild, simctl, gradle, swift, etc.) and project setup
    Doctor {
        /// Project directory path
        #[arg(short, long, default_value = ".")]
        project_path: PathBuf,

        /// Output findings as machine-readable JSON
        #[arg(long)]
        json: bool,
    },

    /// List available connected devices and emulators
    Devices {
        /// Boot an Android Virtual Device (AVD) or simulator by name
        #[arg(short, long)]
        boot: Option<String>,

        /// Output devices as machine-readable JSON
        #[arg(long)]
        json: bool,
    },

    /// Start a live development session (detect -> build -> install -> launch -> logs -> watch)
    Dev {
        /// Target device ID or name
        #[arg(short, long)]
        target: Option<String>,

        /// Override framework adapter
        #[arg(short, long)]
        framework: Option<String>,

        /// Run in headless CLI mode without interactive TUI
        #[arg(long)]
        no_tui: bool,

        /// Output status events as machine-readable JSON
        #[arg(long)]
        json: bool,
    },

    /// Perform a one-off build
    Build {
        /// Build in release mode
        #[arg(short, long)]
        release: bool,

        /// Target device ID or platform
        #[arg(short, long)]
        target: Option<String>,

        /// Output build result as JSON
        #[arg(long)]
        json: bool,
    },

    /// Stream logs for current session or device
    Logs {
        /// Stream live logs continuously
        #[arg(short = 'f', long)]
        follow: bool,

        /// Filter by minimum log level (D, I, W, E)
        #[arg(short, long)]
        level: Option<char>,

        /// Filter by tag name
        #[arg(short, long)]
        tag: Option<String>,

        /// Filter search query
        #[arg(short, long)]
        query: Option<String>,

        /// Limit number of lines
        #[arg(short = 'n', long, default_value = "100")]
        limit: usize,
    },

    /// Trigger a framework reload for an active project session
    Reload,

    /// Trigger a full app restart for an active project session
    Restart,

    /// Open terminal preview (TUI)
    Preview,

    /// Launch Desktop & Web GUI companion
    Gui {
        /// Workspace directory to open
        #[arg(default_value = ".")]
        path: PathBuf,

        /// Server port
        #[arg(short, long, default_value = "9292")]
        port: u16,

        /// Open web companion in external browser
        #[arg(long)]
        web: bool,
    },

    /// Open workspace in DevFlow GUI (alias for 'gui')
    Open {
        /// Workspace directory to open
        #[arg(default_value = ".")]
        path: PathBuf,

        /// Server port
        #[arg(short, long, default_value = "9292")]
        port: u16,

        /// Open web companion in external browser
        #[arg(long)]
        web: bool,
    },

    /// Manage Shell integration, PATH symlinks, and VS Code terminal bindings
    Shell {
        #[command(subcommand)]
        command: ShellCommands,
    },

    /// Generate shell tab completions
    Completions {
        /// Target shell (bash, zsh, fish, powershell, elvish)
        #[arg(default_value = "zsh")]
        shell: String,
    },

    /// Start the Model Context Protocol (MCP) server for AI agents
    Mcp {
        #[command(subcommand)]
        command: McpCommands,
    },
}

#[derive(Subcommand, Debug)]
pub enum ShellCommands {
    /// Install 'devflow' command symlink into system PATH (e.g. /usr/local/bin or ~/.local/bin)
    Install {
        /// Custom destination directory
        #[arg(short, long)]
        dest: Option<PathBuf>,
    },

    /// Remove 'devflow' symlink from system PATH
    Uninstall,

    /// Output shell integration script with aliases (dfr, dfrs, dflog, dfopen)
    Hook {
        /// Target shell (zsh, bash, fish)
        #[arg(default_value = "zsh")]
        shell: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum McpCommands {
    /// Serve MCP tools (stdio by default or HTTP with --http)
    Serve {
        /// Run HTTP server instead of stdio
        #[arg(long)]
        http: bool,

        /// Port for HTTP server
        #[arg(short, long, default_value = "9292")]
        port: u16,

        /// Optional bearer authentication token for HTTP mode (or DEVFLOW_AUTH_TOKEN env var)
        #[arg(short, long, env = "DEVFLOW_AUTH_TOKEN")]
        token: Option<String>,

        /// Explicitly specify calling AI agent client name (e.g. 'Claude Desktop', 'Cursor')
        #[arg(long)]
        agent: Option<String>,
    },

    /// Start MCP tools server (alias for 'serve')
    Start {
        /// Run HTTP server instead of stdio
        #[arg(long)]
        http: bool,

        /// Port for HTTP server
        #[arg(short, long, default_value = "9292")]
        port: u16,

        /// Optional bearer authentication token for HTTP mode (or DEVFLOW_AUTH_TOKEN env var)
        #[arg(short, long, env = "DEVFLOW_AUTH_TOKEN")]
        token: Option<String>,

        /// Explicitly specify calling AI agent client name (e.g. 'Claude Desktop', 'Cursor')
        #[arg(long)]
        agent: Option<String>,
    },

    /// Auto-connect/install DevFlow MCP server into Claude Desktop, Cursor, Antigravity, or VS Code
    Connect {
        /// Target AI host: 'claude', 'cursor', 'antigravity', 'vscode', or 'all' (default: all)
        #[arg(default_value = "all")]
        agent: String,

        /// Force overwrite of existing devflow entry
        #[arg(short, long)]
        force: bool,
    },

    /// Alias for 'connect'
    Install {
        /// Target AI host: 'claude', 'cursor', 'antigravity', 'vscode', or 'all' (default: all)
        #[arg(default_value = "all")]
        agent: String,

        /// Force overwrite of existing devflow entry
        #[arg(short, long)]
        force: bool,
    },

    /// Check DevFlow MCP server status, health, and available tools
    Status {
        /// Output status as JSON
        #[arg(long)]
        json: bool,
    },

    /// List all 11 MCP tools and parameter schemas
    Tools {
        /// Output tools catalog as JSON
        #[arg(long)]
        json: bool,
    },

    /// List all 11 MCP tools (alias for 'tools')
    List {
        /// Output tools catalog as JSON
        #[arg(long)]
        json: bool,
    },

    /// View or manage MCP tool execution & access logs
    Logs {
        /// Number of recent access log entries to show (default: 20)
        #[arg(short, long, default_value = "20")]
        limit: usize,

        /// Filter logs by active session ID
        #[arg(short, long)]
        session: Option<String>,

        /// Filter logs by calling AI agent (e.g. 'Cursor', 'Claude', 'Antigravity', 'vscode')
        #[arg(short, long)]
        agent: Option<String>,

        /// Filter logs by tool name (e.g. devflow_doctor)
        #[arg(short, long)]
        tool: Option<String>,

        /// Filter logs by status ('success' or 'error')
        #[arg(long)]
        status: Option<String>,

        /// Show full input arguments and output response payload JSON
        #[arg(long)]
        full: bool,

        /// Clear logs (all logs, or session logs if --session is specified)
        #[arg(long)]
        clear: bool,

        /// Delete a single log entry by its ID
        #[arg(long)]
        delete: Option<String>,
    },
}

pub async fn run_cli_args<I, T>(args: I) -> anyhow::Result<CliAction>
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
{
    init_environment();
    let cli = Cli::parse_from(args);

    if let Some(ref dir) = cli.dir {
        std::env::set_current_dir(dir)?;
    }

    if cli.gui || cli.web {
        let current_dir = std::env::current_dir()?;
        return Ok(CliAction::LaunchGui {
            dir: current_dir,
            port: 9292,
            open_browser: cli.web,
        });
    }

    let filter = if cli.verbose {
        EnvFilter::new("debug")
    } else {
        EnvFilter::new("warn")
    };

    let is_mcp_stdio = matches!(
        &cli.command,
        Some(Commands::Mcp {
            command: McpCommands::Serve { http: false, .. }
        })
    );

    if !is_mcp_stdio {
        let _ = tracing_subscriber::registry()
            .with(filter)
            .with(tracing_subscriber::fmt::layer().compact())
            .try_init();
    }

    if let Some(cmd) = cli.command {
        match cmd {
            Commands::Init {
                platform,
                framework,
            } => {
                handle_init(platform, framework).await?;
                Ok(CliAction::Executed)
            }
            Commands::Doctor { project_path, json } => {
                handle_doctor(project_path, json).await?;
                Ok(CliAction::Executed)
            }
            Commands::Devices { boot, json } => {
                handle_devices(boot, json).await?;
                Ok(CliAction::Executed)
            }
            Commands::Dev {
                target,
                framework,
                no_tui,
                json,
            } => {
                handle_dev(target, framework, no_tui, json).await?;
                Ok(CliAction::Executed)
            }
            Commands::Build {
                release,
                target,
                json,
            } => {
                handle_build(release, target, json).await?;
                Ok(CliAction::Executed)
            }
            Commands::Logs {
                follow,
                level,
                tag,
                query,
                limit,
            } => {
                handle_logs(follow, level, tag, query, limit).await?;
                Ok(CliAction::Executed)
            }
            Commands::Reload => {
                handle_reload().await?;
                Ok(CliAction::Executed)
            }
            Commands::Restart => {
                handle_restart().await?;
                Ok(CliAction::Executed)
            }
            Commands::Preview => {
                handle_preview().await?;
                Ok(CliAction::Executed)
            }
            Commands::Gui { path, port, web } | Commands::Open { path, port, web } => {
                let resolved = path.canonicalize().unwrap_or(path);
                Ok(CliAction::LaunchGui {
                    dir: resolved,
                    port,
                    open_browser: web || cli.web,
                })
            }
            Commands::Shell { command } => match command {
                ShellCommands::Install { dest } => {
                    let msg = shell::install_cli_symlink(dest)?;
                    println!("{} {}", "✓".green().bold(), msg);
                    println!(
                        "Tip: Add '{}' to your shell rc file for shortcuts & completions.",
                        "eval \"$(devflow shell hook zsh)\"".cyan()
                    );
                    Ok(CliAction::Executed)
                }
                ShellCommands::Uninstall => {
                    let msg = shell::uninstall_cli_symlink()?;
                    println!("{} {}", "✓".green().bold(), msg);
                    Ok(CliAction::Executed)
                }
                ShellCommands::Hook { shell } => {
                    let script = shell::generate_shell_hook(&shell);
                    println!("{}", script);
                    Ok(CliAction::Executed)
                }
            },
            Commands::Completions { shell } => {
                let script = shell::generate_completions::<Cli>(&shell)?;
                println!("{}", script);
                Ok(CliAction::Executed)
            }
            Commands::Mcp { command } => match command {
                McpCommands::Serve {
                    http,
                    port,
                    token,
                    agent,
                }
                | McpCommands::Start {
                    http,
                    port,
                    token,
                    agent,
                } => {
                    handle_mcp_serve(http, port, token, agent).await?;
                    Ok(CliAction::Executed)
                }
                McpCommands::Connect { agent, force } | McpCommands::Install { agent, force } => {
                    handle_mcp_connect(&agent, force).await?;
                    Ok(CliAction::Executed)
                }
                McpCommands::Status { json } => {
                    handle_mcp_status(json).await?;
                    Ok(CliAction::Executed)
                }
                McpCommands::Tools { json } | McpCommands::List { json } => {
                    handle_mcp_tools(json).await?;
                    Ok(CliAction::Executed)
                }
                McpCommands::Logs {
                    limit,
                    session,
                    agent,
                    tool,
                    status,
                    full,
                    clear,
                    delete,
                } => {
                    handle_mcp_logs(limit, session, agent, tool, status, full, clear, delete)
                        .await?;
                    Ok(CliAction::Executed)
                }
            },
        }
    } else {
        let current_dir = std::env::current_dir()?;
        use std::io::IsTerminal;
        if !std::io::stdout().is_terminal() || !std::io::stdin().is_terminal() {
            handle_hub_summary(&current_dir).await?;
            Ok(CliAction::Executed)
        } else {
            Ok(CliAction::LaunchTui { dir: current_dir })
        }
    }
}

pub async fn handle_init(
    _platform: Option<String>,
    framework: Option<String>,
) -> anyhow::Result<()> {
    let current_dir = std::env::current_dir()?;
    let config_path = current_dir.join("devflow.toml");

    if config_path.exists() {
        println!(
            "{}",
            "devflow.toml already exists in current directory.".yellow()
        );
        return Ok(());
    }

    let project_name = current_dir
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "my-app".to_string());

    let fw_str = framework
        .unwrap_or_else(|| "generic".to_string())
        .to_lowercase();
    let content = match fw_str.as_str() {
        "kotlin" | "android" => DevflowConfig::template_android(&project_name),
        "swift" | "swiftpm" | "macos" => DevflowConfig::template_swift(&project_name),
        "react-native" | "rn" => DevflowConfig::template_react_native(&project_name),
        "flutter" => DevflowConfig::template_flutter(&project_name),
        "tauri" => DevflowConfig::template_tauri(&project_name),
        _ => DevflowConfig::template_generic(&project_name),
    };

    std::fs::write(&config_path, content)?;
    println!(
        "{} Created {}",
        "✓".green().bold(),
        "devflow.toml".cyan().bold()
    );
    println!("Edit devflow.toml to customize build, install, launch, and watch settings.");
    Ok(())
}

pub async fn handle_doctor(project_path: PathBuf, json: bool) -> anyhow::Result<()> {
    let report = DoctorEngine::run_diagnostics(&project_path).await;

    if json {
        println!("{}", serde_json::to_string_pretty(&report)?);
        return Ok(());
    }

    println!("\n{}", "═══ DevFlow Doctor Diagnostics ═══".cyan().bold());
    println!("Project path: {}\n", report.project_path.dimmed());

    for check in &report.checks {
        let (icon, status_str) = match check.status {
            CheckStatus::Passed => ("✓".green().bold(), "PASS".green()),
            CheckStatus::Warning => ("!".yellow().bold(), "WARN".yellow()),
            CheckStatus::Failed => ("✗".red().bold(), "FAIL".red()),
            CheckStatus::Skipped => ("-".dimmed(), "SKIP".dimmed()),
        };

        println!(
            " {} [{}] {} — {}",
            icon,
            status_str,
            check.name.bold(),
            check.message
        );
        if let Some(ref hint) = check.fix_hint {
            println!("     {} {}", "Fix hint:".magenta(), hint.dimmed());
        }
    }

    println!(
        "\nSummary: {} passed, {} warnings, {} failed",
        report.passed_count.to_string().green(),
        report.warning_count.to_string().yellow(),
        report.failure_count.to_string().red()
    );

    if report.is_healthy() {
        println!(
            "{}\n",
            "✓ Your development environment is ready!".green().bold()
        );
    } else {
        println!(
            "{}\n",
            "! Some required tools or configs are missing."
                .yellow()
                .bold()
        );
    }

    Ok(())
}

pub async fn handle_devices(boot: Option<String>, json: bool) -> anyhow::Result<()> {
    if let Some(avd_name) = boot {
        println!("{} Booting emulator '{}'...", "🚀".cyan(), avd_name.bold());
        match DeviceManager::boot_emulator(&avd_name).await {
            Ok(msg) => println!("{} {}", "✓".green().bold(), msg),
            Err(e) => println!("{} Failed to boot: {}", "✗".red().bold(), e),
        }
        return Ok(());
    }

    let devices = DeviceManager::discover_all().await;

    if json {
        println!("{}", serde_json::to_string_pretty(&devices)?);
        return Ok(());
    }

    println!(
        "\n{}",
        "═══ Discovered Devices & Emulators ═══".cyan().bold()
    );
    if devices.is_empty() {
        println!("{}", "No devices or emulators found.".yellow());
        return Ok(());
    }

    for (i, d) in devices.iter().enumerate() {
        let default_badge = if d.is_default {
            " (default)".cyan()
        } else {
            "".normal()
        };
        let emu_badge = if d.is_emulator {
            " [emulator/avd]".magenta()
        } else {
            "".normal()
        };
        let state_badge = match d.state {
            devflow_protocol::DeviceState::Connected | devflow_protocol::DeviceState::Booted => {
                "Connected".green()
            }
            devflow_protocol::DeviceState::Shutdown => "Shutdown (bootable)".yellow(),
            _ => format!("{}", d.state).dimmed(),
        };

        println!(
            " {}. {} [ID: {}]{}{}",
            (i + 1).to_string().bold(),
            d.name.bold(),
            d.id.dimmed(),
            default_badge,
            emu_badge
        );
        println!("    Platform: {} | Status: {}", d.platform, state_badge);
    }
    println!();
    Ok(())
}

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

pub async fn handle_build(release: bool, target: Option<String>, json: bool) -> anyhow::Result<()> {
    let current_dir = std::env::current_dir()?;
    let project = Project::detect(&current_dir)?;
    let registry = FrameworkRegistry::new();
    let adapter = registry.select_adapter(&project);

    let device =
        DeviceManager::find_best_match(Some(project.detected_platform), target.as_deref()).await;

    let ctx = BuildContext {
        project_dir: project.root_dir.clone(),
        config: project.effective_config(),
        target_device: device,
        is_release: release,
    };

    println!(
        "{} Building project '{}' using {} adapter...",
        "🔨".cyan(),
        project.name.bold(),
        adapter.name().magenta()
    );
    let res = adapter.build(&ctx).await?;

    if json {
        println!("{}", serde_json::to_string_pretty(&res)?);
        return Ok(());
    }

    if res.success {
        println!(
            "{} Build succeeded in {}ms",
            "✓".green().bold(),
            res.duration_ms
        );
        if let Some(ref art) = res.artifact {
            println!("  {} {}", "Artifact:".cyan(), art.path.bold());
        }
    } else {
        println!("{} Build failed in {}ms", "✗".red().bold(), res.duration_ms);
        if let Some(ref err) = res.error_message {
            println!("{}\n", err.red());
        }
    }

    Ok(())
}

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

pub async fn handle_reload() -> anyhow::Result<()> {
    if let Some(session) = devflow_core::registry::GlobalRegistry::find_active_session(None) {
        println!(
            "{} Sending reload signal to active session '{}' (PID {})...",
            "⚡".cyan(),
            session.project_name.bold(),
            session.pid
        );
        let resp = devflow_core::ipc::IpcClient::send_command(
            &session.socket_path,
            devflow_core::ipc::IpcRequest::Reload { files: vec![] },
        )
        .await?;
        if resp.success {
            println!("{} {}", "✓".green().bold(), resp.message);
        } else {
            println!("{} {}", "✗".red().bold(), resp.message);
        }
    } else {
        println!("{}", "No active DevFlow session found on system to reload. Run 'devflow' or 'devflow dev' first.".yellow());
    }
    Ok(())
}

pub async fn handle_restart() -> anyhow::Result<()> {
    if let Some(session) = devflow_core::registry::GlobalRegistry::find_active_session(None) {
        println!(
            "{} Sending restart signal to active session '{}' (PID {})...",
            "⚡".cyan(),
            session.project_name.bold(),
            session.pid
        );
        let resp = devflow_core::ipc::IpcClient::send_command(
            &session.socket_path,
            devflow_core::ipc::IpcRequest::Restart,
        )
        .await?;
        if resp.success {
            println!("{} {}", "✓".green().bold(), resp.message);
        } else {
            println!("{} {}", "✗".red().bold(), resp.message);
        }
    } else {
        println!("{}", "No active DevFlow session found on system to restart. Run 'devflow' or 'devflow dev' first.".yellow());
    }
    Ok(())
}

pub async fn handle_preview() -> anyhow::Result<()> {
    let current_dir = std::env::current_dir()?;
    TuiRunner::run_hub(current_dir)
        .await
        .map_err(|e| anyhow::anyhow!("{}", e))
}

pub async fn handle_mcp_serve(
    http: bool,
    port: u16,
    token: Option<String>,
    agent_hint: Option<String>,
) -> anyhow::Result<()> {
    let handler = Arc::new(McpHandler::new().with_client_hint(agent_hint));

    if http {
        let server = HttpServer::new(handler, port).with_auth(token);
        server.run().await.map_err(|e| anyhow::anyhow!("{}", e))?;
    } else {
        let server = StdioServer::new(handler);
        server.run().await?;
    }

    Ok(())
}

pub async fn handle_mcp_connect(agent: &str, force: bool) -> anyhow::Result<()> {
    println!(
        "\n{}",
        "═══ DevFlow MCP AI Host Auto-Connector ═══".purple().bold()
    );
    let targets = mcp_connect::get_known_host_targets();

    let filter = agent.to_lowercase();
    let mut configured_count = 0;

    for target in &targets {
        let matches = match filter.as_str() {
            "all" => true,
            "claude" => target.name.to_lowercase().contains("claude"),
            "cursor" => target.name.to_lowercase().contains("cursor"),
            "antigravity" | "agy" => target.name.to_lowercase().contains("antigravity"),
            "vscode" | "code" => target.name.to_lowercase().contains("vs code"),
            other => target.name.to_lowercase().contains(other),
        };

        if matches {
            match mcp_connect::configure_agent_target(target, force) {
                Ok(msg) => {
                    println!("  {} {}", "✓".green().bold(), msg);
                    configured_count += 1;
                }
                Err(e) => {
                    println!("  {} Failed for {}: {}", "✗".red().bold(), target.name, e);
                }
            }
        }
    }

    if configured_count == 0 {
        println!("  {}", format!("No matching AI hosts found for '{}'. Available: claude, cursor, antigravity, vscode, all", agent).yellow());
    } else {
        println!("\n{} Connected {} configuration file(s). Restart your AI agent to activate DevFlow MCP tools.", "⚡".cyan().bold(), configured_count);
    }
    println!();

    // Persist to MCP log
    let entry = devflow_mcp::McpAccessLogEntry::new(
        "devflow-cli",
        "cli/connect",
        None,
        None,
        None,
        Some(serde_json::json!({ "agent": agent, "force": force })),
        Some(serde_json::json!({ "status": "ok", "configured_count": configured_count })),
        1,
        "success",
        format!(
            "Connected {} AI host configurations for '{}'",
            configured_count, agent
        ),
        None,
    );
    devflow_mcp::append_entry_to_disk(&entry);

    Ok(())
}

pub async fn handle_mcp_status(json_mode: bool) -> anyhow::Result<()> {
    let tools = devflow_mcp::get_tool_definitions();
    let active_sessions = devflow_core::registry::GlobalRegistry::list_active_sessions();

    // Persist to MCP log
    let entry = devflow_mcp::McpAccessLogEntry::new(
        "devflow-cli",
        "cli/status",
        None,
        None,
        None,
        None,
        Some(serde_json::json!({
            "tools_count": tools.len(),
            "active_sessions_count": active_sessions.len()
        })),
        1,
        "success",
        format!(
            "Checked MCP status: {} tools registered, {} active sessions",
            tools.len(),
            active_sessions.len()
        ),
        None,
    );
    devflow_mcp::append_entry_to_disk(&entry);

    if json_mode {
        let status = serde_json::json!({
            "protocol_version": "2024-11-05",
            "server_name": "devflow",
            "version": env!("CARGO_PKG_VERSION"),
            "tools_count": tools.len(),
            "tools": tools,
            "active_sessions_count": active_sessions.len(),
            "active_sessions": active_sessions,
        });
        println!("{}", serde_json::to_string_pretty(&status)?);
        return Ok(());
    }

    println!(
        "\n{}",
        "═══ DevFlow Model Context Protocol (MCP) Status ═══"
            .purple()
            .bold()
    );
    println!("  Protocol Version : {}", "2024-11-05".cyan());
    println!("  Server Binary    : {}", "devflow mcp serve".green());
    println!(
        "  Available Tools  : {}",
        format!("{} tools registered", tools.len()).bold()
    );
    println!(
        "  Active Sessions  : {} running project target(s)",
        active_sessions.len()
    );
    println!("\n{}", "Registered MCP Tools:".bold());
    for t in &tools {
        let name = t.get("name").and_then(|n| n.as_str()).unwrap_or("");
        let desc = t.get("description").and_then(|d| d.as_str()).unwrap_or("");
        println!("  • {} : {}", name.cyan().bold(), desc.dimmed());
    }
    println!(
        "\nTip: Run '{}' to auto-configure Claude, Cursor, Antigravity, or VS Code.",
        "devflow mcp connect all".bold()
    );
    println!(
        "     Run '{}' to start the stdio server.\n",
        "devflow mcp serve".bold()
    );
    Ok(())
}

pub async fn handle_mcp_tools(json_mode: bool) -> anyhow::Result<()> {
    let tools = devflow_mcp::get_tool_definitions();

    // Persist to MCP log
    let entry = devflow_mcp::McpAccessLogEntry::new(
        "devflow-cli",
        "cli/tools",
        None,
        None,
        None,
        None,
        Some(serde_json::json!({ "tools_count": tools.len() })),
        1,
        "success",
        format!("Listed {} MCP tool definitions", tools.len()),
        None,
    );
    devflow_mcp::append_entry_to_disk(&entry);

    if json_mode {
        println!("{}", serde_json::to_string_pretty(&tools)?);
        return Ok(());
    }

    println!(
        "\n{}",
        "═══ DevFlow MCP Tool Catalog (11 Tools) ═══".cyan().bold()
    );
    for (i, t) in tools.iter().enumerate() {
        let name = t.get("name").and_then(|n| n.as_str()).unwrap_or("");
        let desc = t.get("description").and_then(|d| d.as_str()).unwrap_or("");
        let schema = t.get("inputSchema");
        let props = schema
            .and_then(|s| s.get("properties"))
            .and_then(|p| p.as_object());
        let required = schema
            .and_then(|s| s.get("required"))
            .and_then(|r| r.as_array());

        println!(
            "\n[{}] {}",
            (i + 1).to_string().bold(),
            name.purple().bold()
        );
        println!("    {}", desc);
        if let Some(props_map) = props {
            if !props_map.is_empty() {
                println!("    Parameters:");
                for (pname, pinfo) in props_map {
                    let ptype = pinfo.get("type").and_then(|v| v.as_str()).unwrap_or("any");
                    let pdesc = pinfo
                        .get("description")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    let is_req =
                        required.is_some_and(|reqs| reqs.iter().any(|r| r.as_str() == Some(pname)));
                    let req_tag = if is_req {
                        "[required]".red()
                    } else {
                        "[optional]".dimmed()
                    };
                    println!(
                        "      - {} ({}) {}: {}",
                        pname.cyan(),
                        ptype,
                        req_tag,
                        pdesc
                    );
                }
            } else {
                println!("    No input parameters required");
            }
        }
    }
    println!();
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub async fn handle_mcp_logs(
    limit: usize,
    session: Option<String>,
    agent: Option<String>,
    tool: Option<String>,
    status: Option<String>,
    full: bool,
    clear: bool,
    delete: Option<String>,
) -> anyhow::Result<()> {
    if clear {
        if let Some(ref sess) = session {
            devflow_mcp::delete_access_logs_by_session(sess);
            println!(
                "{} Cleared all MCP logs for session '{}'.",
                "✓".green().bold(),
                sess.cyan()
            );
        } else {
            devflow_mcp::clear_all_access_logs();
            println!(
                "{} Cleared all MCP logs from SQLite store.",
                "✓".green().bold()
            );
        }
        return Ok(());
    }

    if let Some(ref entry_id) = delete {
        let deleted = devflow_mcp::delete_access_log(entry_id);
        if deleted {
            println!(
                "{} Successfully deleted MCP log entry '{}'.",
                "✓".green().bold(),
                entry_id.cyan()
            );
        } else {
            println!(
                "{} MCP log entry '{}' not found.",
                "!".yellow().bold(),
                entry_id
            );
        }
        return Ok(());
    }

    let filter = devflow_mcp::db::McpLogFilter {
        limit: Some(limit),
        session_id: session.clone(),
        client: agent.clone(),
        tool_name: tool.clone(),
        status: status.clone(),
        ..Default::default()
    };

    let entries = devflow_mcp::query_access_logs(&filter);

    let mut filter_parts = Vec::new();
    if let Some(ref s) = session {
        filter_parts.push(format!("session: {}", s));
    }
    if let Some(ref a) = agent {
        filter_parts.push(format!("agent: {}", a));
    }
    if let Some(ref t) = tool {
        filter_parts.push(format!("tool: {}", t));
    }
    let filter_desc = if filter_parts.is_empty() {
        String::new()
    } else {
        format!(" | {}", filter_parts.join(", "))
    };
    println!(
        "\n{}",
        format!(
            "═══ DevFlow MCP Access & Trace Logs ({} entries{}) ═══",
            entries.len(),
            filter_desc
        )
        .purple()
        .bold()
    );
    if entries.is_empty() {
        println!(
            "  {}",
            "No matching MCP tool calls or CLI activities found.".dimmed()
        );
        println!("  Tip: Trigger tools via an AI host (Cursor, Claude, Antigravity) or run 'devflow mcp status'.\n");
        return Ok(());
    }

    for (i, entry) in entries.iter().enumerate() {
        let status_colored = if entry.status == "success" {
            "✓ SUCCESS".green().bold()
        } else {
            "✗ FAILED".red().bold()
        };
        let target_label = entry.tool_name.as_deref().unwrap_or(&entry.method);
        let session_label = entry.session_id.as_deref().unwrap_or("standalone");
        let short_id = if entry.id.len() > 8 {
            &entry.id[..8]
        } else {
            &entry.id
        };

        println!(
            "[{}] {} | {} | {} ({}ms) [{}]",
            (i + 1).to_string().dimmed(),
            entry.timestamp.dimmed(),
            status_colored,
            target_label.cyan().bold(),
            entry.duration_ms,
            short_id.dimmed()
        );
        let client_colored = match entry.client.to_lowercase().as_str() {
            c if c.contains("cursor") => entry.client.cyan().bold(),
            c if c.contains("claude") => entry.client.yellow().bold(),
            c if c.contains("antigravity") => entry.client.purple().bold(),
            c if c.contains("code") => entry.client.blue().bold(),
            _ => entry.client.green(),
        };
        println!(
            "     Agent: {} | Session: {} | Path: {}",
            client_colored,
            session_label.bold(),
            entry.project_path.as_deref().unwrap_or("-").dimmed()
        );
        println!("     Summary: {}", entry.summary);
        if let Some(err) = &entry.error_message {
            println!("     Error: {}", err.red());
        }

        if full {
            if let Some(ref args) = entry.arguments {
                if let Ok(formatted) = serde_json::to_string_pretty(args) {
                    println!(
                        "     Input Arguments:\n{}",
                        formatted
                            .lines()
                            .map(|l| format!("       {}", l))
                            .collect::<Vec<_>>()
                            .join("\n")
                            .dimmed()
                    );
                }
            }
            if let Some(ref resp) = entry.response {
                if let Ok(formatted) = serde_json::to_string_pretty(resp) {
                    println!(
                        "     Output Response:\n{}",
                        formatted
                            .lines()
                            .map(|l| format!("       {}", l))
                            .collect::<Vec<_>>()
                            .join("\n")
                            .dimmed()
                    );
                }
            }
        }
    }

    println!();
    if !full {
        println!(
            "  Tip: Add {} to view full input arguments & response payloads.",
            "--full".cyan().bold()
        );
    }
    println!(
        "  Tip: Filter by session: {}, clear logs: {}, delete entry: {}\n",
        "devflow mcp logs --session <id>".bold(),
        "devflow mcp logs --clear".bold(),
        "devflow mcp logs --delete <id>".bold()
    );
    Ok(())
}

pub async fn handle_hub_summary(current_dir: &std::path::Path) -> anyhow::Result<()> {
    println!(
        "\n{}",
        "═══ DevFlow Multi-Target Workspace Hub ═══".cyan().bold()
    );
    let folder_name = current_dir
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "workspace".to_string());
    println!(
        "Workspace: {} ({})",
        folder_name.bold(),
        current_dir.display().to_string().dimmed()
    );

    let targets = Project::discover_workspace_targets(current_dir);
    println!("\n{}", "Runnable Targets:".bold());
    if targets.is_empty() {
        println!(
            "  {}",
            "No recognized framework targets found in this workspace.".yellow()
        );
    } else {
        for (i, target) in targets.iter().enumerate() {
            println!(
                "  [{}] {} [{:?}] ({}) -> {}",
                i + 1,
                target.name.bold(),
                target.platform,
                target.framework.green(),
                target.path.display().to_string().dimmed()
            );
        }
    }

    let sessions = devflow_core::registry::GlobalRegistry::list_active_sessions();
    println!("\n{}", "Active Sessions Across Terminals:".bold());
    if sessions.is_empty() {
        println!(
            "  {}",
            "No other active DevFlow sessions running on system.".dimmed()
        );
    } else {
        for s in &sessions {
            println!(
                "  ● PID {}: {} [{:?}] (socket: {})",
                s.pid,
                s.project_name.bold(),
                s.platform,
                s.socket_path.dimmed()
            );
        }
    }

    let devices = DeviceManager::discover_all().await;
    println!("\n{}", "Available Devices:".bold());
    if devices.is_empty() {
        println!("  {}", "No connected devices found.".dimmed());
    } else {
        for d in &devices {
            let state_str = match d.state {
                devflow_protocol::DeviceState::Connected
                | devflow_protocol::DeviceState::Booted => "ONLINE".green(),
                _ => "OFFLINE".dimmed(),
            };
            let emu_str = if d.is_emulator { " [emulator]" } else { "" };
            println!(
                "  ○ {} [{:?}]{} ({})",
                d.name.bold(),
                d.platform,
                emu_str,
                state_str
            );
        }
    }

    println!(
        "\n{}",
        "Tip: Run 'devflow' in an interactive terminal to launch the interactive TUI Hub.".dimmed()
    );
    println!(
        "     Run 'devflow dev' to start live session, or 'devflow --help' for CLI commands.\n"
    );
    Ok(())
}
