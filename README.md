# logpile

A command-line tool to search logs by regex, bucket matches by time, count them, and output summaries as tables, CSV/JSON, or plots (ASCII/PNG).

## Features

- 🔍 **Regex search** across multiple log files
- 📊 **Time-based bucketing** with configurable intervals
- 📈 **Multiple output formats**: tables, CSV, JSON, ASCII plots, PNG charts
- 🗜️ **Automatic gzip support** for `.gz` files
- ⏱️ **Timestamp auto-detection** for common log formats
- 🔄 **Follow mode** for live log monitoring (like `tail -f`)

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

# CSV output
logpile "ERROR" app.log --bucket 60 --csv

# JSON output
logpile "ERROR" app.log --bucket 60 --json

# ASCII plot
logpile "ERROR" app.log --bucket 60 --plot

# PNG chart
logpile "ERROR" app.log --bucket 60 --png error_plot.png
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

# Follow mode (live updates)
logpile "ERROR" /var/log/app.log --follow --plot
```

### Supported Timestamp Formats

The tool auto-detects these common formats:

- ISO 8601: `2025-10-03T14:30:45.123Z`
- Common: `2025-10-03 14:30:45`
- Syslog: `Oct 03 14:30:45`
- Apache/Nginx: `03/Oct/2025:14:30:45 +0000`

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
      --time-format <FMT>    Time format string (chrono-compatible). Auto-detects if not provided
      --bucket <SECONDS>     Bucket size in seconds, or "auto" for automatic selection
      --csv                  Output as CSV
      --json                 Output as JSON
      --plot                 Output as ASCII chart
      --png <FILE>           Output as PNG chart to the specified file
      --follow               Streaming mode (like tail -f) with live updates
      --grep <REGEX>         Additional regex patterns to filter (can be used multiple times)
      --no-default-pattern   Run without a required positional regex (count all lines)
  -h, --help                 Print help
```

## Dependencies

- `clap` - CLI argument parsing
- `chrono` - Timestamp parsing
- `regex` - Pattern matching
- `flate2` - Gzip decompression
- `textplots` - ASCII plotting
- `plotters` - PNG chart generation
- `serde/serde_json` - JSON output
- `csv` - CSV output

## License

MIT

## Contributing

Contributions welcome! Please open an issue or submit a pull request.


