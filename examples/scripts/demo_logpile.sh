#!/bin/bash

# Comprehensive logpile Demo - Showcasing All Features
# This script demonstrates every capability of the logpile tool

set -e

echo "🎯 logpile Comprehensive Demo - All Features Showcase"
echo "====================================================="
echo ""

echo "🔧 Building logpile and log generator..."
cargo build --release --bin logpile
cargo build --example log_generator

echo ""
echo "📝 Generating comprehensive test logs..."
echo "   Creating multiple log files with different characteristics"
echo ""

# Get the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# Create output directory relative to project root and clean up any existing files
mkdir -p "$PROJECT_ROOT/examples/generated_logs"
rm -f "$PROJECT_ROOT/examples/generated_logs"/*.log "$PROJECT_ROOT/examples/generated_logs"/*.log.gz "$PROJECT_ROOT/examples/generated_logs"/*.png

# Generate different types of logs (simulated for speed)
echo "  📄 Generating basic application logs (2 hours, ~15,000 entries)..."
./target/debug/examples/log_generator 7200 100 25 --simulate > "$PROJECT_ROOT/examples/generated_logs/basic.log"

echo "  📄 Generating high-frequency logs (1 hour, ~18,000 entries)..."
./target/debug/examples/log_generator 3600 50 20 --simulate > "$PROJECT_ROOT/examples/generated_logs/high_freq.log"

echo "  📄 Generating compressed logs (1.5 hours, ~13,500 entries)..."
./target/debug/examples/log_generator 5400 150 25 --simulate > "$PROJECT_ROOT/examples/generated_logs/compressed.log"
gzip -f "$PROJECT_ROOT/examples/generated_logs/compressed.log"

echo ""
echo "🧪 ===== BASIC USAGE DEMOS ====="
echo ""

echo "🧪 Demo 1: Basic pattern matching"
echo "=================================="
./target/debug/logpile "ERROR" "$PROJECT_ROOT/examples/generated_logs/basic.log" --bucket 600

echo ""
echo "🧪 Demo 2: Multiple patterns with grep"
echo "======================================"
./target/debug/logpile "ERROR" $PROJECT_ROOT/examples/generated_logs/basic.log --grep "WARN" --grep "CRITICAL" --bucket 600

echo ""
echo "🧪 Demo 3: Custom bucket size (10 minutes)"
echo "==========================================="
./target/debug/logpile "INFO" $PROJECT_ROOT/examples/generated_logs/high_freq.log --bucket 600

echo ""
echo "🧪 Demo 4: 10-minute bucket size"
echo "================================"
./target/debug/logpile "WARN" $PROJECT_ROOT/examples/generated_logs/basic.log --bucket 600

echo ""
echo "🧪 ===== OUTPUT FORMAT DEMOS ====="
echo ""

echo "🧪 Demo 5: CSV output with headers"
echo "=================================="
./target/debug/logpile "ERROR" $PROJECT_ROOT/examples/generated_logs/basic.log --csv --bucket 600

echo ""
echo "🧪 Demo 6: CSV output without headers"
echo "====================================="
./target/debug/logpile "WARN" $PROJECT_ROOT/examples/generated_logs/basic.log --csv --no-headers --bucket 600

echo ""
echo "🧪 Demo 7: JSON output"
echo "======================"
./target/debug/logpile "INFO" $PROJECT_ROOT/examples/generated_logs/basic.log --json --bucket 600

echo ""
echo "🧪 Demo 8: ASCII plot visualization"
echo "==================================="
./target/debug/logpile "ERROR" $PROJECT_ROOT/examples/generated_logs/high_freq.log --bucket 600 --plot

echo ""
echo "🧪 Demo 9: PNG plot output"
echo "=========================="
./target/debug/logpile "WARN" $PROJECT_ROOT/examples/generated_logs/basic.log --bucket 600 --png $PROJECT_ROOT/examples/generated_logs/demo_plot.png
echo "Generated: $PROJECT_ROOT/examples/generated_logs/demo_plot.png"

echo ""
echo "🧪 ===== ADVANCED FEATURE DEMOS ====="
echo ""

echo "🧪 Demo 10: Compressed file support"
echo "==================================="
./target/debug/logpile "ERROR" $PROJECT_ROOT/examples/generated_logs/compressed.log.gz --csv --bucket 600

echo ""
echo "🧪 Demo 11: Multiple file processing"
echo "===================================="
./target/debug/logpile "INFO" $PROJECT_ROOT/examples/generated_logs/basic.log $PROJECT_ROOT/examples/generated_logs/high_freq.log --bucket 600

echo ""
echo "🧪 Demo 12: Custom timestamp format"
echo "==================================="
./target/debug/logpile "ERROR" $PROJECT_ROOT/examples/generated_logs/basic.log --time-format "%Y-%m-%d %H:%M:%S" --csv --bucket 600

echo ""
echo "🧪 Demo 13: No default pattern (count all lines)"
echo "================================================"
./target/debug/logpile --no-default-pattern $PROJECT_ROOT/examples/generated_logs/basic.log --bucket 600

echo ""
echo "🧪 Demo 14: Follow mode (live monitoring)"
echo "=========================================="
echo "Starting follow mode for 5 seconds..."
timeout 5s ./target/debug/logpile "ERROR" $PROJECT_ROOT/examples/generated_logs/basic.log --follow --plot || true

echo ""
echo "🧪 Demo 15: Piping from stdin with live plot"
echo "============================================="
echo "Piping live log generation directly to logpile with live plot (15 seconds)..."
./target/debug/examples/log_generator 15 50 50 2>/dev/null | ./target/debug/logpile --no-default-pattern --bucket 1 --follow --plot

echo ""
echo "🧪 Demo 16: Verbose mode (shows unmatched lines)"
echo "================================================"
./target/debug/logpile "NONEXISTENT" $PROJECT_ROOT/examples/generated_logs/basic.log --verbose --bucket 600

echo ""
echo "🧪 Demo 17: Fail quick mode"
echo "==========================="
./target/debug/logpile "NONEXISTENT" $PROJECT_ROOT/examples/generated_logs/basic.log --fail-quick --bucket 600 || echo "Expected failure for demo"

echo ""
echo "🧪 ===== PERFORMANCE & EDGE CASES ====="
echo ""

echo "🧪 Demo 18: Large bucket size"
echo "============================="
./target/debug/logpile "INFO" $PROJECT_ROOT/examples/generated_logs/basic.log --bucket 1800

echo ""
echo "🧪 Demo 19: Very small bucket size"
echo "=================================="
./target/debug/logpile "ERROR" $PROJECT_ROOT/examples/generated_logs/high_freq.log --bucket 60

echo ""
echo "🧪 Demo 20: Complex pattern matching"
echo "===================================="
./target/debug/logpile "ERROR|WARN|CRITICAL" $PROJECT_ROOT/examples/generated_logs/basic.log --bucket 600 --json

echo ""
echo "🎉 ===== DEMO COMPLETE ====="
echo ""
echo "📁 Generated files:"
echo "  - $PROJECT_ROOT/examples/generated_logs/basic.log (basic application logs)"
echo "  - $PROJECT_ROOT/examples/generated_logs/high_freq.log (high-frequency logs)"
echo "  - $PROJECT_ROOT/examples/generated_logs/compressed.log.gz (compressed logs)"
echo "  - $PROJECT_ROOT/examples/generated_logs/demo_plot.png (PNG visualization)"
echo ""
echo "🚀 Advanced Usage Examples:"
echo ""
echo "  # Real-time monitoring with follow mode"
echo "  ./target/debug/logpile \"ERROR\" /var/log/app.log --follow --plot"
echo ""
echo "  # Live log generation and analysis"
echo "  ./target/debug/examples/log_generator 60 800 30 | ./target/debug/logpile \"ERROR\" --bucket 5"
echo ""
echo "  # Multiple patterns with different output formats"
echo "  ./target/debug/logpile \"ERROR\" app.log --grep \"timeout\" --grep \"failed\" --csv > errors.csv"
echo ""
echo "  # Sub-second analysis for high-frequency logs"
echo "  ./target/debug/logpile \"INFO\" high_freq.log --bucket 0.1 --png analysis.png"
echo ""
echo "  # Custom timestamp format for legacy logs"
echo "  ./target/debug/logpile \"ERROR\" legacy.log --time-format \"%d/%b/%Y:%H:%M:%S\" --json"
echo ""
echo "  # Process multiple compressed files"
echo "  ./target/debug/logpile \"WARN\" *.log.gz --bucket auto --plot"
echo ""
echo "✨ logpile demo complete! All features demonstrated."
echo "   Check the generated files and try the advanced examples above."


