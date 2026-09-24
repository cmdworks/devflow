pub mod cli;
pub mod commands;
pub mod mcp_connect;
pub mod shell;

use clap::Parser;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub use cli::{Cli, CliAction, Commands, McpCommands, ShellCommands};
pub use devflow_core::init_environment;

pub async fn run_cli_args<I, T>(args: I) -> anyhow::Result<CliAction>
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
{
    init_environment();
    let cli = Cli::parse_from(args);

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

    commands::dispatch(cli).await
}
