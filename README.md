# DevFlow ⚡

> **Universal, framework-aware development runner, interactive multi-target TUI hub, and MCP server for native & cross-platform applications.**

DevFlow bridges native mobile, desktop, and hybrid frameworks into a unified, high-performance developer experience built in Rust. It eliminates slow polling scripts, unbounded memory leaks, and fragmented CLI workflows.

---

## ✨ Features

- **🎯 Zero-Argument Multi-Target TUI Hub**: Simply run `devflow` in any workspace. It automatically discovers all runnable targets (e.g., macOS SwiftPM app + Android Gradle companion), connected devices, and active sessions.
- **⚡ Blazing Performance & Bounded Memory**: Sustained device log streaming consumes **< 6.5 MB RSS** memory and **~0.1% CPU** using circular $O(1)$ ring buffers and non-blocking channels (compared to 140MB+ in Python scripts).
- **🔄 Cross-Terminal Session Sync & IPC**: Active sessions register in `~/.devflow/`. Run `devflow reload` or `devflow restart` from any terminal window to instantly hot-reload your active build across processes.
- **🤖 Built-in MCP Server**: Exposes Model Context Protocol tools over Stdio and HTTP with Bearer Auth, enabling AI coding agents (Antigravity, Cursor, Claude) to discover devices, run diagnostics, build, and inspect logs.
- **📱 Broad Platform Support**: Native support for **Android** (Kotlin/Java/Gradle), **macOS / iOS** (Swift / SwiftPM / Xcode), **React Native**, **Flutter**, and **Tauri**.
- **🩺 DevFlow Doctor**: Instant diagnostics for SDKs, toolchains (`adb`, `xcodebuild`, `simctl`, `swift`, `gradle`), and project configuration.

---

## 🚀 Quick Start

### Interactive TUI Hub
Run `devflow` with zero arguments inside any project directory:
```bash
devflow
```
- Use `[Tab]` to navigate between Runnable Targets, Active Sessions, and Devices.
- Press `[Enter]` to launch a live development session.
- Press `[r]` to hot-reload, `[R]` to restart app, or `[q]` to quit.

---

## 🛠️ CLI Usage

| Command | Description |
| :--- | :--- |
| `devflow` | Opens the interactive multi-target TUI Hub (or prints summary in scripts) |
| `devflow init` | Scaffold `devflow.toml` for the current workspace |
| `devflow doctor` | Check environment, SDKs, toolchains, and project health |
| `devflow devices` | Discover connected hardware devices and bootable emulators |
| `devflow dev` | Start live development session (build → install → launch → logs → watch) |
| `devflow build` | One-off build with optional `--release` and `--json` |
| `devflow logs` | Stream or filter device logs (`--follow`, `--limit`, `--tag`, `--level`, `--query`) |
| `devflow reload` | Signal active session to hot-reload from any terminal |
| `devflow restart` | Signal active session to perform a full app restart |
| `devflow mcp serve` | Start Model Context Protocol server (stdio or `--http --port 9090`) |

---

## 📊 Benchmark Comparison

Empirical tests sampled during live log streaming on physical hardware (**Samsung Galaxy A35**):

| Metric | DevFlow (Rust) | Legacy Python Dev Scripts | Advantage |
| :--- | :--- | :--- | :--- |
| **Memory Footprint (RSS)** | **5.07 MB – 6.22 MB** (flat line) | **140 MB – 300 MB** | **22.5x to 27x leaner** |
| **CPU Utilization** | **0.0% – 0.1%** | **3.5% – 8.0%** idle; 4–12% active | **~40x lower CPU churn** |
| **Log Buffer Strategy** | $O(1)$ Bounded Ring Buffer | Unbounded string list | Zero heap exhaustion |
| **File Watching** | Kernel `kqueue` / `FSEvents` (0% CPU) | `os.walk` polling loop every 1s | Instant, zero overhead |

---

## 🏗️ Architecture

DevFlow is organized as a modular Rust workspace:

```
devflow/
├── crates/
│   ├── devflow_core/        # Configuration, doctor, project model, global registry & IPC
│   ├── devflow_protocol/    # IPC types, device models, log schemas, JSON-RPC
│   ├── devflow_devices/     # adb, Apple simctl, desktop host device discovery & controls
│   ├── devflow_platforms/   # Platform runners (Android, macOS, Apple, Desktop)
│   ├── devflow_frameworks/  # Framework adapters (Kotlin, SwiftPM, RN, Flutter, Tauri, Generic)
│   ├── devflow_watcher/     # Kernel-level file watcher with debounce & classification
│   ├── devflow_logs/        # High-throughput ring buffer & logcat/console parser
│   ├── devflow_tui/         # Ratatui-powered interactive terminal hub & session UI
│   ├── devflow_mcp/         # Stdio & HTTP Model Context Protocol (MCP) server
│   └── devflow_cli/         # CLI binary entry point & subcommand dispatch
```

---

## 📄 License

Apache-2.0 or MIT.
