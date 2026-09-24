pub mod build;
pub mod dev;
pub mod devices;
pub mod doctor;
pub mod hub;
pub mod init;
pub mod logs;
pub mod mcp;
pub mod session;

use colored::*;
use std::io::IsTerminal;

use crate::cli::{Cli, CliAction, Commands, McpCommands, ShellCommands};
use crate::shell;

pub async fn dispatch(cli: Cli) -> anyhow::Result<CliAction> {
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

    if let Some(cmd) = cli.command {
        match cmd {
            Commands::Init {
                platform,
                framework,
            } => {
                init::handle_init(platform, framework).await?;
                Ok(CliAction::Executed)
            }
            Commands::Doctor { project_path, json } => {
                doctor::handle_doctor(project_path, json).await?;
                Ok(CliAction::Executed)
            }
            Commands::Devices { boot, json } => {
                devices::handle_devices(boot, json).await?;
                Ok(CliAction::Executed)
            }
            Commands::Dev {
                target,
                framework,
                no_tui,
                json,
            } => {
                dev::handle_dev(target, framework, no_tui, json).await?;
                Ok(CliAction::Executed)
            }
            Commands::Build {
                release,
                target,
                json,
            } => {
                build::handle_build(release, target, json).await?;
                Ok(CliAction::Executed)
            }
            Commands::Logs {
                follow,
                level,
                tag,
                query,
                limit,
            } => {
                logs::handle_logs(follow, level, tag, query, limit).await?;
                Ok(CliAction::Executed)
            }
            Commands::Reload => {
                session::handle_reload().await?;
                Ok(CliAction::Executed)
            }
            Commands::Restart => {
                session::handle_restart().await?;
                Ok(CliAction::Executed)
            }
            Commands::Preview => {
                session::handle_preview().await?;
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
                    mcp::handle_mcp_serve(http, port, token, agent).await?;
                    Ok(CliAction::Executed)
                }
                McpCommands::Connect { agent, force } | McpCommands::Install { agent, force } => {
                    mcp::handle_mcp_connect(&agent, force).await?;
                    Ok(CliAction::Executed)
                }
                McpCommands::Status { json } => {
                    mcp::handle_mcp_status(json).await?;
                    Ok(CliAction::Executed)
                }
                McpCommands::Tools { json } | McpCommands::List { json } => {
                    mcp::handle_mcp_tools(json).await?;
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
                    mcp::handle_mcp_logs(limit, session, agent, tool, status, full, clear, delete)
                        .await?;
                    Ok(CliAction::Executed)
                }
            },
        }
    } else {
        let current_dir = std::env::current_dir()?;
        if !std::io::stdout().is_terminal() || !std::io::stdin().is_terminal() {
            hub::handle_hub_summary(&current_dir).await?;
            Ok(CliAction::Executed)
        } else {
            Ok(CliAction::LaunchTui { dir: current_dir })
        }
    }
}
