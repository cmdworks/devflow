# DevFlow Architecture & Engine Design 🏛️

DevFlow is engineered as a modular, high-throughput Rust workspace designed to bridge heterogeneous native and cross-platform runtimes into a unified developer control plane.

---

## 🧱 Workspace Crates Overview

```
devflow/
├── crates/
│   ├── devflow_protocol/    # IPC schemas, JSON-RPC, device descriptors, log structures
│   ├── devflow_core/        # Configuration, project model, doctor diagnostics, session registry
│   ├── devflow_devices/     # Hardware discovery (adb, simctl, macOS desktop host)
│   ├── devflow_logs/        # Bounded O(1) ring buffers, logcat/console regex parsers, crash clustering
│   ├── devflow_watcher/     # Kernel-level file watcher (kqueue/FSEvents/inotify) + classifier
│   ├── devflow_platforms/   # Platform runners (Android, Apple, Desktop) & lifecycle managers
│   ├── devflow_frameworks/  # Framework adapters (Kotlin, SwiftPM, Flutter, React Native, Tauri)
│   ├── devflow_mcp/         # Model Context Protocol (MCP) server, SQLite WAL logger, REST API
│   ├── devflow_tui/         # Ratatui-powered interactive terminal hub & session UI
│   ├── devflow_cli_core/    # Core CLI commands, shell symlinks, and MCP connectors
│   └── devflow_cli/         # Top-level executable binary & CLI entrypoint
├── clients/
│   └── tauri_app/           # Lightweight native desktop companion app (Tauri + Axum + React)
└── docs/
    └── ai/                  # AI agent context, MCP documentation, and repository map
```

---

## ⚡ Key Architectural Pillars

### 1. Zero-Polling, Bounded Memory Model
Traditional dev tools and Python polling scripts frequently suffer from unbounded log growth, memory leaks (140MB+ to 500MB+), and continuous CPU polling churn.

DevFlow solves this with three core primitives:
- **$O(1)$ Circular Ring Buffers** (`crates/devflow_logs/src/buffer.rs`): Stores log lines in fixed-capacity ring buffers (e.g. 10,000 lines), maintaining sustained memory footprints under **6.5 MB RSS**.
- **Kernel-Level File Events** (`crates/devflow_watcher/src/watcher.rs`): Utilizes operating system event queues (`kqueue` on macOS, `inotify` on Linux) with millisecond debouncing, ensuring **0% idle CPU**.
- **Non-blocking Tokio Channels**: Multi-producer single-consumer (`mpsc`) event distribution prevents log producers from blocking execution threads.

### 2. Dual-Tier Session Tracking & Cross-Process IPC
DevFlow registers running sessions in `~/.devflow/` with Unix domain sockets and PID liveness validation (`crates/devflow_core/src/registry.rs`).

- **Target Runner Session**: Represents a live process running on a hardware device or emulator (e.g., `android-target-1`).
- **Agent Connection Session**: Represents an AI assistant connection over MCP stdio/HTTP (e.g., `claude-088d30c7`, `cursor-9bda175f`).
- **Cross-Terminal Sync**: Running `devflow reload` or `devflow restart` in any separate terminal sends an IPC command across the socket to hot-reload the running app instantly.
- **Smart Context Resolution**: When an AI agent calls tools without specifying a `session_id`, DevFlow automatically resolves and injects the active target runner session.

### 3. Model Context Protocol (MCP) & SQLite WAL Engine
DevFlow exposes native MCP tools compliant with the `2024-11-05` specification (`crates/devflow_mcp`):
- **SQLite WAL Storage (`~/.devflow/mcp_logs.db`)**:
  - Thread-safe embedded SQLite with Write-Ahead Logging (`WAL`) and `NORMAL` synchronous mode.
  - Dedicated indexes on `timestamp`, `session_id`, `client`, `tool_name`, and `status`.
- **Dual Payload Capture**: Automatically serializes and stores formatted JSON input arguments (`arguments`) and output responses (`response`) for all tool calls.
- **Auto-Detection**: Handshakes automatically extract `clientInfo.name` to detect Claude Desktop, Cursor, Antigravity, VS Code, and CLI clients.

### 4. Framework Adapter Lifecycle
Every framework implements the `FrameworkAdapter` trait (`crates/devflow_frameworks/src/adapter.rs`):

```rust
#[async_trait]
pub trait FrameworkAdapter: Send + Sync {
    fn framework_type(&self) -> FrameworkType;
    fn can_handle(&self, path: &Path) -> bool;
    async fn build(&self, ctx: &BuildContext) -> Result<BuildArtifact>;
    async fn run(&self, ctx: &RunContext) -> Result<Box<dyn ProcessHandle>>;
    async fn reload(&self, handle: &mut dyn ProcessHandle, changes: &[PathBuf]) -> Result<ReloadResult>;
    async fn restart(&self, handle: &mut dyn ProcessHandle) -> Result<()>;
}
```

This guarantees modular extensibility for new languages, build systems, or cross-platform toolchains.
