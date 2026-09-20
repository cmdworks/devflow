# Getting Started with DevFlow ⚡

Welcome to **DevFlow** — the universal, framework-aware development runner, interactive multi-target TUI hub, and Model Context Protocol (MCP) server for native mobile, desktop, and cross-platform applications.

---

## 📋 Prerequisites

Before installing DevFlow, ensure you have the appropriate toolchains installed for the platforms you are developing:

| Platform / Framework | Required Toolchain | Verification Command |
| :--- | :--- | :--- |
| **Rust / Core DevFlow** | Rust 1.74+ & Cargo | `cargo --version` |
| **Android / Kotlin** | Android SDK, `adb`, JDK 17+ | `adb --version` |
| **macOS / iOS / Swift** | Xcode or Command Line Tools, `xcrun` | `xcrun simctl list` |
| **Flutter** | Flutter SDK & Dart | `flutter doctor` |
| **React Native** | Node.js (18+), npm/yarn/pnpm, Metro | `node --version` |
| **Tauri** | Node.js, `cargo-tauri` or `@tauri-apps/cli` | `cargo tauri --version` |
| **Desktop Companion UI** | Node.js 18+ & npm (for GUI build) | `npm --version` |

> [!TIP]
> Run `devflow doctor` after installation to automatically check your installed SDKs, toolchains, simulator runtimes, and project health.

---

## 📦 Installation Options

### Option 1: 1-Line Remote Install (Zero Git / Rust Required)

For regular users who want to install pre-compiled binaries instantly:

```bash
# Complete install (CLI + Desktop GUI Companion)
curl -fsSL https://raw.githubusercontent.com/cmdworks/devflow/main/scripts/install.sh | bash

# CLI Only
curl -fsSL https://raw.githubusercontent.com/cmdworks/devflow/main/scripts/install-cli.sh | bash

# Desktop GUI Companion Only
curl -fsSL https://raw.githubusercontent.com/cmdworks/devflow/main/scripts/install-gui.sh | bash
```

The installer detects your OS/architecture, downloads the latest binary into `~/.local/bin`, and automatically clears macOS Gatekeeper quarantine flags.

---

### Option 2: Direct Download from GitHub Releases

Pre-compiled binaries are published on the **[GitHub Releases Page](https://github.com/cmdworks/devflow/releases)**:

| Platform | Architecture | 🌟 Desktop GUI Companion (Recommended) | Terminal CLI Download |
| :--- | :--- | :--- | :--- |
| **macOS** | Apple Silicon (`arm64` / M1–M4) | [devflow-gui-macos-arm64.tar.gz](https://github.com/cmdworks/devflow/releases/latest/download/devflow-gui-macos-arm64.tar.gz) | [devflow-cli-macos-arm64.tar.gz](https://github.com/cmdworks/devflow/releases/latest/download/devflow-cli-macos-arm64.tar.gz) |
| **macOS** | Intel 64-bit (`x86_64`) | [devflow-gui-macos-x86_64.tar.gz](https://github.com/cmdworks/devflow/releases/latest/download/devflow-gui-macos-x86_64.tar.gz) | [devflow-cli-macos-x86_64.tar.gz](https://github.com/cmdworks/devflow/releases/latest/download/devflow-cli-macos-x86_64.tar.gz) |
| **Linux** | x86_64 (`amd64`) | [devflow-gui-linux-x86_64.tar.gz](https://github.com/cmdworks/devflow/releases/latest/download/devflow-gui-linux-x86_64.tar.gz) | [devflow-cli-linux-x86_64.tar.gz](https://github.com/cmdworks/devflow/releases/latest/download/devflow-cli-linux-x86_64.tar.gz) |
| **Linux** | ARM64 (`aarch64`) | *(GUI coming soon)* | [devflow-cli-linux-aarch64.tar.gz](https://github.com/cmdworks/devflow/releases/latest/download/devflow-cli-linux-aarch64.tar.gz) |

---

### Option 3: Build from Source (Contributors)

If you are developing or contributing to DevFlow:

```bash
git clone https://github.com/cmdworks/devflow.git
cd devflow

# Build and link locally into ~/.local/bin
./scripts/install.sh

# Or build CLI directly with Cargo
cargo install --path crates/devflow_cli

# Run fast incremental development wrappers (<0.1s)
./bin/cli-dev [command]
./bin/gui-dev
```

---

## 🍎 macOS Gatekeeper & Quarantine Bypass (`xattr -cr`)

> [!IMPORTANT]
> Because DevFlow release binaries are not yet notarized with an Apple Developer ID, macOS Gatekeeper may display a security dialog stating that *"DevFlow cannot be opened because the developer cannot be verified"*.

To clear the quarantine flag and allow immediate execution:

```bash
# For installed binaries in ~/.local/bin
xattr -cr ~/.local/bin/devflow
xattr -cr ~/.local/bin/devflow-gui

# For binaries in your current directory or Downloads
xattr -cr ./devflow
xattr -cr ./devflow-gui
xattr -cr ~/Downloads/devflow*

# If running directly from a source clone
xattr -cr ./target/release/devflow
xattr -cr ./clients/tauri_app/src-tauri/target/release/devflow-desktop
```

---

## 🎯 Using the Interactive TUI Hub

Running `devflow` with zero arguments inside any workspace opens the **Interactive TUI Hub**:

```bash
devflow
```

```
┌ DevFlow ⚡ Hub ────────────────────────────────────────────────────────┐
│ [1] Targets     [2] Active Sessions (1)    [3] Devices (3)            │
├─────────────────────────────────────────────────────────────────────────┤
│ ▶ [1] mobile-android  (Kotlin / Gradle)                                 │
│   [2] desktop-macos   (Swift / SwiftPM)                                 │
│   [3] web-client      (Vite / React)                                    │
│                                                                         │
│ Target Details: mobile-android                                          │
│   Platform : Android (JVM 17)                                           │
│   Device   : Samsung Galaxy A35 (Physical USB)                          │
│   State    : IDLE (Press 's' to start live runner)                      │
├─────────────────────────────────────────────────────────────────────────┤
│ [1-9] Switch Tab | [s] Start/Stop | [r] Reload | [R] Restart | [q] Quit │
└─────────────────────────────────────────────────────────────────────────┘
```

### Keyboard Shortcuts

| Key | Action |
| :--- | :--- |
| `1` – `9` / `Tab` | Switch between tabs and targets |
| `↑` / `↓` (`k` / `j`) | Navigate list items (targets, sessions, devices) |
| `s` / `Enter` | Start or stop the selected development target runner |
| `a` | Start all discovered targets simultaneously |
| `r` | Trigger framework-aware hot reload (HMR / VM reload) |
| `R` | Trigger full application restart (kill + rebuild + relaunch) |
| `f` | Toggle auto-follow log streaming |
| `Esc` / `q` | Return to Hub / Quit |

---

## 🖥️ Using the Desktop Companion App

DevFlow includes a lightweight native desktop companion app powered by Tauri and Axum:

```bash
# Launch companion app
devflow-gui
# Or in development mode:
./bin/gui-dev
```

### Companion Features:
1. **Interactive Target Dashboard**: 1-click build, launch, reload, and restart for all discovered targets.
2. **Terminal & Split Log Panes**: Live $O(1)$ ring-buffered log streams with log-level color filters (`E`, `W`, `I`, `D`) and tag searching.
3. **Model Context Protocol (MCP) Inspector**:
   - Live AI agent connection traces.
   - Branded agent filter chips (Claude Desktop, Cursor, Google Antigravity, VS Code, CLI).
   - Rich session dropdowns with call counts and last-used tools.
   - Dual-payload inspector for input arguments (`📥`) and output responses (`📤`).
   - 2-step inline clear confirmation with 4-second safety timer.

---

## 🛠️ CLI Quick Reference

```bash
# Scaffold devflow.toml in the current directory
devflow init

# Run system and project diagnostics
devflow doctor

# List connected physical devices, emulators, and desktop hosts
devflow devices

# Start live runner for default target (or specify target name)
devflow dev [target_name]

# One-off build with optional release flag
devflow build [target_name] --release

# Stream live filtered logs
devflow logs --follow --level E,W --tag AndroidRuntime

# Signal active session to hot-reload from any terminal
devflow reload

# Signal active session to restart from any terminal
devflow restart

# Query MCP traces and access logs
devflow mcp logs --agent cursor --limit 10 --full
```

---

## ⚙️ Workspace Configuration (`devflow.toml`)

While DevFlow detects targets automatically with zero configuration, you can customize behavior using `devflow.toml` at your workspace root:

```toml
[workspace]
name = "my-multiplatform-app"

[[target]]
name = "android-app"
path = "apps/android"
platform = "android"
framework = "kotlin"
default_device = "emulator-5554"

[[target]]
name = "macos-companion"
path = "apps/desktop"
platform = "macos"
framework = "swift"

[watcher]
debounce_ms = 300
ignore_paths = [".git", "build", "target", "node_modules", ".gradle"]
extensions = ["kt", "swift", "dart", "tsx", "rs"]

[logs]
max_lines = 10000
filter_levels = ["I", "W", "E"]
```

---

## 🤖 Next Steps: AI Coding Agents

To enable AI assistants (Cursor, Claude Desktop, Antigravity, VS Code) to build, test, and debug your application automatically through DevFlow:

👉 **[Read the MCP Integration Guide](file:///Users/as/Dev/Projects/opensources/devflow/docs/ai/mcp-integration.md)**  
👉 **[Read the AI Agent Workflow Guide](file:///Users/as/Dev/Projects/opensources/devflow/docs/ai/agent-workflow.md)**
