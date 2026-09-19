#!/bin/bash
set -e

DIR="$(cd "$(dirname "$0")/.." && pwd)"
BIN="$DIR/target/release/devflow"

echo "=== DevFlow Resource Profiler ==="
echo "Target Project: /Users/as/Dev/Projects/opensources/mac-mtp/android"

# Start devflow logs --follow in background
$BIN logs -C /Users/as/Dev/Projects/opensources/mac-mtp/android --follow > /tmp/devflow_stream.log 2>&1 &
DEVFLOW_PID=$!

echo "DevFlow streaming PID: $DEVFLOW_PID"
sleep 1

echo -e "\nSampling DevFlow system resources over 6 seconds..."
echo "Timestamp | RSS (KB) | RSS (MB) | VSZ (MB) | %CPU"
echo "---------------------------------------------------"

for i in {1..6}; do
    SAMPLE=$(ps -p $DEVFLOW_PID -o rss,vsz,%cpu | tail -n 1)
    RSS_KB=$(echo $SAMPLE | awk '{print $1}')
    VSZ_KB=$(echo $SAMPLE | awk '{print $2}')
    CPU=$(echo $SAMPLE | awk '{print $3}')
    
    if [ -n "$RSS_KB" ] && [ "$RSS_KB" -gt 0 ] 2>/dev/null; then
        RSS_MB=$(echo "scale=2; $RSS_KB / 1024" | bc)
        VSZ_MB=$(echo "scale=2; $VSZ_KB / 1024" | bc)
        echo "$(date +%H:%M:%S)  | ${RSS_KB} KB   | ${RSS_MB} MB   | ${VSZ_MB} MB  | ${CPU}%"
    fi
    sleep 1
done

LINES=$(wc -l < /tmp/devflow_stream.log | tr -d ' ')
echo -e "\nTotal log lines streamed into DevFlow: $LINES lines"

kill $DEVFLOW_PID 2>/dev/null || true
wait $DEVFLOW_PID 2>/dev/null || true

echo "=== Resource Profiling Completed ==="
