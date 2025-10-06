#!/bin/bash

# Live demo script - generates logs and follows them with logpile

set -e

echo "🎯 Live Logpile Demo - Real-time Log Generation and Analysis"
echo "============================================================"
echo ""

# Get the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# Create output directory
mkdir -p "$PROJECT_ROOT/examples/generated_logs"

echo "🔧 Building tools..."
cargo build --release --bin logpile
cargo build --example log_generator

echo ""
echo "📝 Starting live log generation..."
echo "   Generating logs to: $PROJECT_ROOT/examples/generated_logs/live.log"
echo "   Press Ctrl+C to stop both processes"
echo ""

# Start log generation in background
./target/debug/examples/log_generator 300 1000 40 > "$PROJECT_ROOT/examples/generated_logs/live.log" &

# Give it a moment to start generating
sleep 2

echo "🧪 Starting logpile in follow mode with plot..."
echo "   Watching for: ERROR messages"
echo "   Output: Live plot visualization"
echo ""

# Start logpile in follow mode
./target/release/logpile "ERROR" "$PROJECT_ROOT/examples/generated_logs/live.log" --follow --plot

# Clean up background process when logpile exits
echo ""
echo "🛑 Stopping log generation..."
pkill -f "log_generator.*live.log" || true

echo ""
echo "🎉 Live demo complete!"
echo "📁 Generated log file: $PROJECT_ROOT/examples/generated_logs/live.log"
