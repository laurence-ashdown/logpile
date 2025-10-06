#!/bin/bash

# Log Generator Script for logpile testing
# This script generates various types of log files for testing

set -e

echo "🔧 Building log generator..."
cargo build --example log_generator

echo "📝 Generating test log files..."

# Get the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# Create output directory
mkdir -p "$PROJECT_ROOT/examples/generated_logs"

# Generate a basic log file
echo "Creating $PROJECT_ROOT/examples/generated_logs/basic.log (30 seconds of logs, 500ms intervals)..."
./target/debug/examples/log_generator 30 500 --simulate > "$PROJECT_ROOT/examples/generated_logs/basic.log"

# Generate a high-frequency log file
echo "Creating $PROJECT_ROOT/examples/generated_logs/high_freq.log (20 seconds of logs, 100ms intervals)..."
./target/debug/examples/log_generator 20 100 --simulate > "$PROJECT_ROOT/examples/generated_logs/high_freq.log"

# Generate a log file with different timestamp formats
echo "Creating $PROJECT_ROOT/examples/generated_logs/mixed_formats.log (25 seconds of logs, 300ms intervals)..."
./target/debug/examples/log_generator 25 300 --simulate > "$PROJECT_ROOT/examples/generated_logs/mixed_formats.log"

echo ""
echo "✅ Generated test log files:"
echo "  - $PROJECT_ROOT/examples/generated_logs/basic.log"
echo "  - $PROJECT_ROOT/examples/generated_logs/high_freq.log" 
echo "  - $PROJECT_ROOT/examples/generated_logs/mixed_formats.log"
echo ""
echo "🧪 Test examples:"
echo "  # Basic usage"
echo "  cargo run --release --bin logpile \"ERROR\" $PROJECT_ROOT/examples/generated_logs/basic.log"
echo ""
echo "  # Follow mode with CSV"
echo "  cargo run --release --bin logpile \"INFO\" $PROJECT_ROOT/examples/generated_logs/basic.log --csv --follow"
echo ""
echo "  # Plot visualization"
echo "  cargo run --release --bin logpile \"WARN\" $PROJECT_ROOT/examples/generated_logs/high_freq.log --plot"
echo ""
echo "  # JSON output"
echo "  cargo run --release --bin logpile \"ERROR\" $PROJECT_ROOT/examples/generated_logs/mixed_formats.log --json"
echo ""
echo "  # Multiple patterns"
echo "  cargo run --release --bin logpile \"ERROR\" $PROJECT_ROOT/examples/generated_logs/basic.log --grep \"WARN\" --grep \"INFO\""
echo ""
echo "  # Custom bucket size"
echo "  cargo run --release --bin logpile \"INFO\" $PROJECT_ROOT/examples/generated_logs/high_freq.log --bucket 10"
echo ""
echo "  # PNG output"
echo "  cargo run --release --bin logpile \"ERROR\" $PROJECT_ROOT/examples/generated_logs/basic.log --png error_plot.png"
echo ""
echo "🎯 Happy testing!"
