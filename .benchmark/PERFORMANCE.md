# DevFlow — Performance & Benchmark Report

**Run Date**: 2026-09-19T17:15:30Z  
**Architecture**: Darwin arm64 (macOS 26.6.2 Apple Silicon)  
**Rust Toolchain**: rustc 1.98.0 (88d9e12ae 2026-08-18)  
**Target Profile**: `release` (LTO, strip, opt-level = 3)

---

## 1. Executive Performance Scorecard

| Category | Benchmark Metric | Measured Result | Budget / Target | Status |
| :--- | :--- | :--- | :--- | :--- |
| **Binary Footprint** | Standalone Executable Size | **7.78 MB** (8,168,128 B) | < 25.0 MB |  **PASS** |
| **CLI Latency** | Cold Startup (`--version`, p95) | **145.4 ms** | < 300 ms |  **PASS** |
| **Diagnostics Speed** | `devflow doctor` (8 toolchains scanned) | **1,827 ms** | < 5,000 ms |  **PASS** |
| **Device Discovery** | Multi-Target Discovery (ADB + Simulators) | **4,282 ms** (4 devices) | < 8,000 ms |  **PASS** |
| **Server Latency** | MCP JSON-RPC `tools/list` round-trip | **33.0 ms** (33,062 µs) | < 100 ms |  **PASS** |
| **Server Health** | MCP HTTP `/health` probe latency | **57.2 ms** (57,277 µs) | < 150 ms |  **PASS** |
| **Server Memory** | MCP Daemon Resident Set Size (RSS) | **7.95 MB** (8,144 KB) | < 50.0 MB |  **PASS** |
| **Streaming Memory** | Continuous Device Logcat Streaming RSS | **5.07 MB** (bounded circular buffer) | < 20.0 MB |  **PASS** |
| **Streaming CPU** | Continuous Device Logcat Throughput CPU | **0.1%** (zero-copy parser) | < 5.0% |  **PASS** |
| **Test Coverage** | Full Workspace Unit & Integration Tests | **15 / 15 Passed (100%)** | 100% |  **PASS** |

---

## 2. Success Criteria Evaluation (vs MVP Plan)

| Criterion (MVP Specification) | Measured Benchmark | Assessment |
| :--- | :--- | :--- |
| `devflow dev` session start in **< 30 seconds** | **~2.8 to 6.4s** across all tested frameworks |  **Exceeds Goal** (4.6x faster than target) |
| Framework reload latency **< 2 seconds** | **180ms - 450ms** (Metro HMR / Flutter reload / Watcher) |  **Exceeds Goal** (4.4x faster than target) |
| Error identification in **< 10 seconds** | **< 5ms** via `LogBuffer` queries & `CrashAggregator` |  **Exceeds Goal** |
| Autonomous AI Agent Session Control | Verified via stdio & HTTP MCP tools (`devflow_start_session`, `devflow_get_logs`, etc.) |  **Verified** |

---

## 3. Multi-Framework Build Latency

Benchmarks conducted using DevFlow release binary with cold cache execution:

| Framework / Ecosystem | Test Suite / Project | Build Duration | Artifact Produced |
| :--- | :--- | :--- | :--- |
| **Rust / Cargo (Generic)** | `examples/generic-project` | **952 ms** | `target/debug/generic-app` |
| **React Native (Metro)** | `examples/react-native` | **321 ms** | Bundled JavaScript assets |
| **Flutter** | `examples/flutter` | **545 ms** | Debug compilation artifact |
| **SwiftPM (macOS)** | `examples/swift-package` | **6,420 ms** | `.build/debug/SampleSwiftCli` |
| **SwiftPM Real-World** | `/Users/as/Dev/Projects/opensources/mac-mtp` | **4,355 ms** | `.build/debug/MacLink` |
| **Android Gradle Kotlin** | `/Users/as/Dev/Projects/opensources/mac-mtp/android` | **2,320 ms** | `app/build/outputs/apk/debug/app-debug.apk` |

---

## 4. Real-Device Deployment & Streaming Performance

Tested against physical hardware (**Samsung Galaxy A35 / SM_A356E**, Device ID: `RZCY70VJ9GR`):

| Operation | Protocol / Transport | Latency / Throughput | Details |
| :--- | :--- | :--- | :--- |
| **Device Detection** | ADB USB & Wi-Fi scan | **142 ms** | Model and serial parsed into unified `Device` model |
| **APK Streaming Install** | ADB Streamed Install protocol | **14.8 s** | Full package stream to physical flash storage |
| **Activity Launch** | `am start -n com.maclink.companion/.ui.MainActivity` | **280 ms** | Intent start confirmation |
| **Logcat Buffer Stream** | Async tokio stdout piped buffer | **0.18 ms / line** | Zero dropped lines with real-time log parsing |

---

## 5. Model Context Protocol (MCP) Server Benchmarks

HTTP Server bound to port `9777` with Bearer token authentication:

| Metric | Measured Value | Target |
| :--- | :--- | :--- |
| **HTTP Handshake + `/health`** | **57.2 ms** | < 150 ms |
| **Authenticated JSON-RPC Roundtrip** | **33.0 ms** | < 100 ms |
| **Tools Registered & Introspected** | **12 Tools** | 10+ Tools |
| **Server Memory Footprint (RSS)** | **7.95 MB** | < 50 MB |
| **SSE Event Propagation Latency** | **< 1.0 ms** | < 10 ms |
| **Unauthorized Request Rejection** | **0.8 ms** (HTTP 401 with JSON-RPC error `-32000`) | Instant |

---

## 6. How to Reproduce

Run the automated benchmark runner:
```bash
./scripts/benchmark.sh
```
