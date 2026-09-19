# DevFlow — Benchmark Suite & Performance Baselines

This directory contains the formal performance benchmarks, latency measurements, and resource footprint tracking for **DevFlow**.

---

## Benchmark Reports

- [**`PERFORMANCE.md`**](./PERFORMANCE.md) — Comprehensive performance scorecards, latency benchmarks, build times, physical device metrics, and MVP success criteria evaluation.
- [**`RESOURCES.md`**](./RESOURCES.md) — System resource and memory utilization report, CPU & RSS profiling during sustained log streaming, and architectural comparison against legacy Python runner scripts.

---

## Methodology & Reproducibility

Benchmarks are executed against the optimized release binary (`target/release/devflow`) under cold and warm caches.

### Benchmark Harness
The complete test suite is automated via the harness script:
```bash
./scripts/benchmark.sh
```

### Metrics Tracked
1. **Binary Footprint**: Standalone executable size (target: `< 25 MB`).
2. **Cold CLI Startup**: Startup time to execute `--version` and `--help` across 10 iterations (p95 target: `< 300 ms`).
3. **Multi-Target Device Discovery**: Latency to scan and parse ADB (USB + Wi-Fi), Apple simulators via `simctl`, and Desktop host (target: `< 8,000 ms`).
4. **Environment Diagnostics (`doctor`)**: Complete health check across 8 development toolchains (target: `< 5,000 ms`).
5. **Multi-Framework Build Speed**: Cold-cache execution for Rust Cargo, React Native, Flutter, SwiftPM, and Gradle Kotlin.
6. **Model Context Protocol (MCP) Server**:
   - HTTP health check latency (`/health`)
   - Authenticated JSON-RPC roundtrip (`tools/list`)
   - Daemon memory footprint (Resident Set Size / RSS in MB)
7. **Physical Device Deployment**: Real-world install, launch, and logcat streaming latency tested on physical hardware (Samsung Galaxy A35).
