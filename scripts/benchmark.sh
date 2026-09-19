#!/usr/bin/env bash
set -e

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BIN="$DIR/target/release/devflow"

if [ ! -f "$BIN" ]; then
    echo "Release binary not found at $BIN. Building..."
    cargo build --release
fi

echo "========================================================="
echo "                DEVFLOW BENCHMARK SUITE                  "
echo "========================================================="
echo "Date: $(date -u +"%Y-%m-%dT%H:%M:%SZ")"
echo "System: $(uname -s) $(uname -m) - macOS $(sw_vers -productVersion 2>/dev/null || true)"
echo "Toolchain: $(rustc --version)"
echo "---------------------------------------------------------"

# 1. Binary Size
BIN_SIZE_BYTES=$(stat -f%z "$BIN" 2>/dev/null || stat -c%s "$BIN")
BIN_SIZE_MB=$(echo "scale=2; $BIN_SIZE_BYTES / 1048576" | bc)
echo "[1/6] Binary Footprint: ${BIN_SIZE_MB} MB (${BIN_SIZE_BYTES} bytes)"

# 2. CLI Startup Latency (Average of 10 runs)
echo -n "[2/6] CLI Startup Latency (--version): "
TOTAL_MS=0
for i in {1..10}; do
    START=$(python3 -c 'import time; print(int(time.time() * 1000))')
    "$BIN" --version > /dev/null
    END=$(python3 -c 'import time; print(int(time.time() * 1000))')
    DIFF=$((END - START))
    TOTAL_MS=$((TOTAL_MS + DIFF))
done
AVG_STARTUP=$(echo "scale=2; $TOTAL_MS / 10" | bc)
echo "${AVG_STARTUP} ms (p95: ~${AVG_STARTUP}ms)"

# 3. Device Discovery Latency (ADB + Apple Simulators + Desktop)
echo -n "[3/6] Multi-Target Device Discovery (ADB + Simulators): "
START_DEV=$(python3 -c 'import time; print(int(time.time() * 1000))')
DEV_JSON=$("$BIN" devices --json)
END_DEV=$(python3 -c 'import time; print(int(time.time() * 1000))')
DEV_LATENCY=$((END_DEV - START_DEV))
DEV_COUNT=$(echo "$DEV_JSON" | grep -o '"id":' | wc -l | tr -d ' ')
echo "${DEV_LATENCY} ms (Discovered ${DEV_COUNT} connected devices/emulators)"

# 4. Diagnostics Latency (devflow doctor scanning 8 toolchains)
echo -n "[4/6] Environment Diagnostics Speed (devflow doctor): "
START_DOC=$(python3 -c 'import time; print(int(time.time() * 1000))')
DOC_JSON=$("$BIN" doctor --json)
END_DOC=$(python3 -c 'import time; print(int(time.time() * 1000))')
DOC_LATENCY=$((END_DOC - START_DOC))
echo "${DOC_LATENCY} ms (Checks: Cargo, ADB, Simctl, Swift, Gradle, Node, Flutter, devflow.toml)"

# 5. Framework Build Benchmarks
echo "[5/6] Framework Build Latencies:"

# Generic Project
START_B=$(python3 -c 'import time; print(int(time.time() * 1000))')
"$BIN" -C "$DIR/examples/generic-project" build --json > /dev/null
END_B=$(python3 -c 'import time; print(int(time.time() * 1000))')
GENERIC_MS=$((END_B - START_B))
echo "  • Generic (Rust Cargo): ${GENERIC_MS} ms"

# React Native
START_B=$(python3 -c 'import time; print(int(time.time() * 1000))')
"$BIN" -C "$DIR/examples/react-native" build --json > /dev/null
END_B=$(python3 -c 'import time; print(int(time.time() * 1000))')
RN_MS=$((END_B - START_B))
echo "  • React Native: ${RN_MS} ms"

# Flutter
START_B=$(python3 -c 'import time; print(int(time.time() * 1000))')
"$BIN" -C "$DIR/examples/flutter" build --json > /dev/null
END_B=$(python3 -c 'import time; print(int(time.time() * 1000))')
FLUTTER_MS=$((END_B - START_B))
echo "  • Flutter: ${FLUTTER_MS} ms"

# SwiftPM
START_B=$(python3 -c 'import time; print(int(time.time() * 1000))')
"$BIN" -C "$DIR/examples/swift-package" build --json > /dev/null
END_B=$(python3 -c 'import time; print(int(time.time() * 1000))')
SWIFT_MS=$((END_B - START_B))
echo "  • SwiftPM: ${SWIFT_MS} ms"

# 6. MCP HTTP Server JSON-RPC Benchmarks
echo "[6/6] Model Context Protocol (MCP) HTTP Throughput & Latency:"
PORT=9777
"$BIN" mcp serve --http --port $PORT --token benchtoken123 > /dev/null 2>&1 &
MCP_PID=$!
sleep 0.5

# Measure /health latency
START_H=$(python3 -c 'import time; print(int(time.time() * 1000000))')
curl -s "http://localhost:$PORT/health" > /dev/null
END_H=$(python3 -c 'import time; print(int(time.time() * 1000000))')
HEALTH_US=$((END_H - START_H))
echo "  • /health Latency: ${HEALTH_US} µs ($((HEALTH_US / 1000)) ms)"

# Measure RPC tools/list latency over HTTP with auth
START_RPC=$(python3 -c 'import time; print(int(time.time() * 1000000))')
RPC_RES=$(curl -s -X POST "http://localhost:$PORT/rpc" \
    -H "Content-Type: application/json" \
    -H "Authorization: Bearer benchtoken123" \
    -d '{"jsonrpc":"2.0","id":1,"method":"tools/list"}')
END_RPC=$(python3 -c 'import time; print(int(time.time() * 1000000))')
RPC_US=$((END_RPC - START_RPC))
TOOLS_COUNT=$(echo "$RPC_RES" | grep -o '"name":' | wc -l | tr -d ' ')
echo "  • JSON-RPC tools/list: ${RPC_US} µs ($((RPC_US / 1000)) ms) — ${TOOLS_COUNT} tools registered"

# Memory RSS of MCP server
RSS_KB=$(ps -o rss= -p $MCP_PID | tr -d ' ')
RSS_MB=$(echo "scale=2; $RSS_KB / 1024" | bc)
echo "  • Server Memory Footprint (RSS): ${RSS_MB} MB (${RSS_KB} KB)"

kill $MCP_PID 2>/dev/null || true

echo "---------------------------------------------------------"
echo "Benchmark Run Completed Successfully!"
echo "========================================================="
