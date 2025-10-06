# logpile

A fast CLI tool to search logs by regex, bucket matches by time, and visualize patterns with charts.

## Features

- 🔍 **Regex search** across multiple log files with full regex support
- 📊 **Time-based bucketing** with configurable intervals (including sub-second precision)
- 📈 **Multiple output formats**: tables, CSV, JSON, ASCII plots, PNG charts
- 🗜️ **Automatic gzip support** for `.gz` files with transparent decompression
- ⏱️ **Timestamp auto-detection** for 10+ common log formats
- 🔄 **Follow mode** for live log monitoring (like `tail -f`) with real-time updates
- ⚡ **Enhanced CLI** with short flags (`-c`, `-j`, `-p`, `-o`, `-f`, `-v`, `-q`, `-n`)
- 🧪 **Log generator** for realistic test data with multiple timestamp formats
- 🎯 **Sub-second bucketing** for high-precision analysis (0.1s, 0.5s, etc.)
- 📊 **Terminal-responsive plotting** with automatic size detection
- 🎛️ **CSV header control** with `--no-headers` option
- 🔍 **Verbose mode** for debugging and detailed output
- ⚡ **Fail-fast mode** for CI/CD environments
- 📈 **Y-axis zero option** for consistent plot scaling
- 🧪 **Comprehensive testing** with 89 tests (65 unit + 24 integration)

## Installation

```bash
cargo install --path .
```

Or build from source:

```bash
cargo build --release
./target/release/logpile --help
```

## Usage

### Basic Usage

Search for a pattern in log files:

```bash
# Search for ERROR in multiple files
logpile "ERROR" app.log server.log.gz

# Search from stdin
cat logs.txt | logpile "WARN"

# Search with time bucketing (60 second intervals)
logpile "ERROR" app.log --bucket 60

# Sub-second bucketing for high-precision analysis
logpile "ERROR" app.log --bucket 0.5
```

### Time Bucketing

```bash
# 5 minute buckets (300 seconds)
logpile "timeout" requests.log --bucket 300

# 1 hour buckets (3600 seconds)
logpile "ERROR" app.log --bucket 3600

# Auto-detect optimal bucket size
logpile "ERROR" app.log --bucket auto
```

### Output Formats

```bash
# Default table output
logpile "ERROR" app.log --bucket 60

# CSV output (with short flag)
logpile "ERROR" app.log --bucket 60 -c

# CSV without headers
logpile "ERROR" app.log --bucket 60 -c --no-headers

# JSON output (with short flag)
logpile "ERROR" app.log --bucket 60 -j

# ASCII plot (with short flag)
logpile "ERROR" app.log --bucket 60 -p

# PNG chart (with short flag)
logpile "ERROR" app.log --bucket 60 -o error_plot.png
```

### Timestamp Parsing

```bash
# Auto-detect timestamp format (default)
logpile "ERROR" app.log

# Specify custom time format (chrono-compatible)
logpile "ERROR" app.log --time-format "%Y/%m/%d %H:%M:%S"
```

### Advanced Features

```bash
# Multiple patterns: search for ERROR OR WARN
logpile "ERROR" logs.txt --grep "WARN"

# Count all lines (no pattern filtering)
logpile --no-default-pattern app.log --bucket 300

# Follow mode (live updates) with short flags
logpile "ERROR" /var/log/app.log -f -p

# Verbose mode for debugging
logpile "ERROR" app.log --verbose

# Fail-fast mode for CI/CD
logpile "ERROR" app.log --fail-quick

# Y-axis zero for consistent plots
logpile "ERROR" app.log --plot --y-zero
```

### Supported Timestamp Formats

The tool auto-detects these common formats:

- **ISO 8601**: `2025-10-03T14:30:45.123Z` (with/without timezone)
- **Standard**: `2025-10-03 14:30:45.123456` (with microsecond precision)
- **Syslog**: `Oct 03 14:30:45` (RFC 3164)
- **Apache/Nginx**: `03/Oct/2025:14:30:45 +0000` (with microsecond support)
- **European**: `03/10/2025 14:30:45` (DD/MM/YYYY)
- **US Format**: `10/03/2025 14:30:45` (MM/DD/YYYY)
- **Unix Timestamp**: `1727962496` (epoch seconds)
- **RFC 2822**: `Fri, 03 Oct 2025 14:30:45 GMT`
- **Java Logs**: `2025-10-03 14:30:45.123 INFO [thread] class - message`
- **Yearless ISO**: `09-24T23:45:29.362Z` (with automatic year injection)
- **Time-only**: `05:40:12` (with automatic date injection)

## Examples

### Example 1: Basic Error Analysis

```bash
logpile "ERROR" application.log --bucket 300 --plot
```

This searches for "ERROR" in `application.log`, groups matches into 5-minute buckets, and displays an ASCII plot.

### Example 2: Multi-file Analysis with JSON Output

```bash
logpile "timeout" app1.log app2.log.gz --bucket 3600 --json
```

Searches for "timeout" across multiple files (including gzipped), buckets by hour, and outputs JSON.

### Example 3: Monitor Logs in Real-time

```bash
logpile "CRITICAL" /var/log/app.log --follow --plot
```

Continuously monitors the log file for "CRITICAL" entries and updates the ASCII plot in real-time.

### Example 5: Sub-second Analysis

```bash
logpile "ERROR" app.log --bucket 0.5 --csv --no-headers
```

Analyzes errors with 500ms precision and outputs CSV without headers for further processing.

### Example 6: Generate Test Data

```bash
# Generate realistic test logs
cargo run --example log_generator 60 1000 30 > test.log

# Analyze the generated logs
logpile "ERROR" test.log --bucket 10 --plot
```

### Example 4: Custom Time Format

```bash
logpile "failed" custom.log --time-format "%d/%b/%Y:%H:%M:%S" --csv
```

Parses timestamps in Apache-style format and outputs results as CSV.

## Options

```
Usage: logpile [OPTIONS] [REGEX] [FILES]...

Arguments:
  [REGEX]     Regex pattern to search for (required unless --no-default-pattern)
  [FILES]...  Log files to search (supports .gz files). If empty, reads from stdin

Options:
  -c, --csv                  Output as CSV
      --no-headers           Exclude column headers from CSV output
  -j, --json                 Output as JSON
  -p, --plot                 Output as ASCII chart
      --y-zero               Start Y-axis at zero in ASCII plots
  -o, --png <FILE>           Output as PNG chart to the specified file
  -t, --time-format <FMT>     Custom timestamp format (e.g., "%Y-%m-%d %H:%M:%S")
  -b, --bucket <SECONDS>      Time bucket size in seconds, or "auto" for automatic
  -g, --grep <REGEX>          Additional regex patterns to match
  -n, --no-default-pattern   Process all lines without requiring a search pattern
  -f, --follow                Follow log file and update display in real-time
  -v, --verbose               Enable verbose output with warnings
  -q, --fail-quick            Exit immediately if any file has no matching lines
  -h, --help                  Print help
```

## Dependencies

### Core Libraries
- `clap` - CLI argument parsing with derive macros
- `chrono` - Timestamp parsing and date/time handling
- `regex` - Pattern matching with compiled regexes
- `anyhow` - Error handling
- `thiserror` - Custom error types

### I/O & Serialization
- `flate2` - Gzip decompression
- `serde/serde_json` - JSON serialization
- `csv` - CSV formatting

### Visualization
- `textplots` - ASCII plotting with Braille characters
- `plotters` - PNG chart generation
- `terminal_size` - Terminal size detection for responsive charts
- `console` - Enhanced console output
- `rgb` - Color handling

### Development
- `rand` - Random number generation (for log generator)
- `tempfile` - Temporary file handling (for tests)

## License

MIT

## Contributing

Contributions welcome! Please open an issue or submit a pull request.


