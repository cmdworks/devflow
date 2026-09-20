# DevFlow Repository Map & Codebase Conventions 🗺️

This document provides a concise symbol, crate, and convention index for AI assistants maintaining, extending, or fixing DevFlow.

---

## 📂 Crate Taxonomy

| Crate Path | Responsibility | Key Structs / Traits |
| :--- | :--- | :--- |
| `crates/devflow_protocol` | Shared data types, IPC messages, device models, RPC schemas | `Device`, `SessionState`, `LogEntry`, `DevflowEvent` |
| `crates/devflow_core` | Config parsing, doctor diagnostics, project model, session registry | `Config`, `DoctorCheck`, `Project`, `ActiveSessionRegistry` |
| `crates/devflow_devices` | ADB, Apple simctl, and desktop host discovery & process spawn | `AdbDeviceManager`, `SimctlManager`, `DeviceDiscovery` |
| `crates/devflow_logs` | Ring buffer, logcat/console regex parsing, crash clustering | `LogRingBuffer`, `LogParser`, `CrashClusterer` |
| `crates/devflow_watcher` | Kernel file watching (kqueue/inotify), debouncing, path classifier | `FileWatcher`, `ChangeClassifier`, `ChangeAction` |
| `crates/devflow_platforms` | Platform execution lifecycle (Android, Apple, Desktop) | `PlatformRunner`, `AndroidRunner`, `AppleRunner` |
| `crates/devflow_frameworks` | Framework adapters (Kotlin, SwiftPM, Flutter, RN, Tauri) | `FrameworkAdapter`, `KotlinAdapter`, `SwiftAdapter` |
| `crates/devflow_mcp` | Stdio/HTTP MCP server, SQLite WAL access logger, REST API | `McpServer`, `McpHandler`, `McpSqliteStore` |
| `crates/devflow_tui` | Ratatui-based interactive terminal UI & hub views | `TuiApp`, `TuiRunner`, `HubView`, `SessionView` |
| `crates/devflow_cli_core` | CLI command handlers, shell installer, MCP connectors | `handle_dev`, `handle_doctor`, `handle_mcp_logs` |
| `crates/devflow_cli` | CLI executable binary entrypoint | `main` |
| `clients/tauri_app` | Native desktop companion GUI (Tauri + Axum + React) | `App.tsx`, `McpView.tsx`, `TargetsView.tsx` |

---

## 📐 Key Codebase Conventions

1. **Rust Edition & Toolchain**:
   - Edition 2021, targeting modern stable Rust (1.74+).
   - Zero compiler warnings: all code must compile with `cargo clippy --workspace --all-targets -- -D warnings`.

2. **Error Handling**:
   - Use `anyhow::Result` for application and CLI layer boundaries.
   - Use `thiserror::Error` for domain-specific error types in core crates (`devflow_core::error::DevflowError`).

3. **Memory & Concurrency**:
   - Use `tokio::sync::mpsc` or `broadcast` channels for async event passing.
   - Guard shared state with `Arc<tokio::sync::Mutex<T>>` or `Arc<std::sync::Mutex<T>>` for sync storage.
   - Never use unbounded `Vec<String>` for log storage; always wrap in `LogRingBuffer`.

4. **MCP Tool Implementation**:
   - Registered tools live in `crates/devflow_mcp/src/tools.rs` and handlers in `crates/devflow_mcp/src/handler.rs`.
   - Always record tool calls via `record_access_log` in `McpSqliteStore`.
   - Never allow `session_id` to be null in SQLite logs; fall back to the calling agent's connection session ID.

5. **Testing**:
   - Every crate includes unit tests in `tests` module or integration tests in `tests/`.
   - Run tests using `cargo test --workspace`.
