use clap::{Parser, Subcommand};
use std::path::PathBuf;

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
#[command(
    name = "devflow",
    author,
    version,
    about = "Universal, framework-aware development runner, desktop companion, and MCP server",
    long_about = None
)]
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
