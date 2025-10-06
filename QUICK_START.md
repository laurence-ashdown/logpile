# logpile Quick Start Guide

## Installation

```bash
git clone https://github.com/laurence-ashdown/logpile.git
cd logpile
cargo build --release
```

The binary will be at `./target/release/logpile`

Optionally install globally:
```bash
cargo install --path .
```

## 5-Minute Tutorial

This guide will get you up and running with logpile v0.3.0 in just 5 minutes, covering all the major features including the new sub-second bucketing, follow mode, and enhanced CLI.

### 1. Basic Search
Search for "ERROR" in a log file:
```bash
logpile "ERROR" examples/sample-iso.log
```

### 2. Time Bucketing
Group matches into 5-minute (300 second) buckets:
```bash
logpile "ERROR" examples/sample-iso.log --bucket 300
```

### 3. Different Output Formats

**CSV:**
```bash
logpile "ERROR" examples/sample-iso.log --bucket 300 -c > errors.csv
```

**CSV without headers:**
```bash
logpile "ERROR" examples/sample-iso.log --bucket 300 -c --no-headers
```

**JSON:**
```bash
logpile "WARN" examples/sample-iso.log --bucket 300 -j
```

**ASCII Plot:**
```bash
logpile "ERROR|WARN" examples/sample-iso.log --bucket 300 -p
```

**Bitmap Chart:**
```bash
logpile "ERROR" examples/sample-iso.log --bucket 300 -o chart.png
```

### 4. Multiple Patterns
Search for multiple patterns:
```bash
logpile "ERROR" examples/sample-iso.log -g "WARN" -g "CRITICAL" --bucket 600
```

### 5. Gzipped Files
Transparently read gzipped logs:
```bash
logpile "ERROR" examples/sample-iso.log.gz --bucket 300
```

### 6. Stdin Input
Pipe logs from other commands:
```bash
cat examples/sample-iso.log | logpile "INFO" --bucket 600
zcat large_log.gz | logpile "timeout" --bucket 3600
```

### 7. Auto Bucket Size
Let logpile choose the best bucket size:
```bash
logpile "ERROR" examples/sample-iso.log --bucket auto
```

### 8. Sub-second Bucketing
High-precision analysis with fractional seconds:
```bash
# 500ms buckets for high-resolution analysis
logpile "ERROR" examples/sample-iso.log --bucket 0.5

# 100ms buckets for microsecond-level precision
logpile "ERROR" examples/sample-iso.log --bucket 0.1
```

### 9. Follow Mode
Real-time log monitoring with live updates:
```bash
# Follow mode with ASCII plot
logpile "ERROR" /var/log/app.log -f -p

# Follow mode with CSV output
logpile "ERROR" /var/log/app.log -f -c

# Follow mode with verbose output
logpile "ERROR" /var/log/app.log -f -v
```

### 10. Enhanced CLI Options
New v0.3.0 features:
```bash
# Verbose mode for debugging
logpile "ERROR" app.log --verbose

# Fail-fast mode for CI/CD
logpile "ERROR" app.log --fail-quick

# Y-axis zero for consistent plots
logpile "ERROR" app.log --plot --y-zero

# CSV without headers
logpile "ERROR" app.log -c --no-headers
```

### 11. Count All Lines
Count all log entries without pattern filtering:
```bash
logpile -n examples/sample-iso.log --bucket 600
```

### 12. Generate Test Data
Create realistic test logs with the built-in generator:
```bash
# Generate 60 seconds of logs with 1-second intervals
cargo run --example log_generator 60 1000 30 > test.log

# Generate high-frequency logs (100ms intervals)
cargo run --example log_generator 30 100 50 > high_freq.log

# Generate logs instantly (simulation mode)
cargo run --example log_generator 60 1000 30 --simulate > instant.log

# Analyze the generated logs
logpile "ERROR" test.log --bucket 10 --plot
```

## Common Use Cases

### Monitor Production Errors
```bash
# Hourly error counts
logpile "ERROR|CRITICAL" /var/log/app.log --bucket 3600 --json

# Visual error trend
logpile "ERROR" /var/log/app.log --bucket 3600 --plot
```

### Analyze Request Timeouts
```bash
logpile "timeout|timed out" access.log --bucket 300 --csv > timeout_analysis.csv
```

### Track Multiple Log Files
```bash
logpile "WARN" app1.log app2.log app3.log.gz --bucket 600
```

### Custom Timestamp Format
```bash
logpile "failed" custom.log --time-format "%d/%b/%Y:%H:%M:%S" --bucket 300
```

## Tips & Tricks

1. **Combine with grep for pre-filtering:**
   ```bash
   grep "user_id=123" app.log | logpile "ERROR" --bucket 300
   ```

2. **Export for further analysis:**
   ```bash
   logpile "ERROR" app.log --bucket 3600 --json | jq '.buckets'
   ```

3. **Process multiple days:**
   ```bash
   logpile "ERROR" app.2025-10-*.log --bucket 86400  # Daily buckets
   ```

4. **Real-time monitoring:**
   ```bash
   # Using follow mode (recommended)
   logpile "ERROR" /var/log/app.log --follow --bucket 60
   
   # Using tail + logpile
   tail -f /var/log/app.log | logpile "ERROR" --bucket 60
   ```

5. **High-precision analysis:**
   ```bash
   # Sub-second analysis for performance monitoring
   logpile "slow_query" app.log --bucket 0.5 --plot
   ```

6. **CI/CD integration:**
   ```bash
   # Fail-fast mode for automated testing
   logpile "ERROR" test.log --fail-quick --bucket 300
   ```

## Testing

Run the comprehensive test suite:
```bash
# Run all tests (89 total)
cargo test

# Run unit tests only
cargo test --lib

# Run integration tests only
cargo test --test

# Run with verbose output
cargo test -- --nocapture
```

### Test Categories
- **Unit Tests (65)**: Timestamp parsing, bucketing, CLI, output formatting
- **Integration Tests (24)**: Follow mode, real-time updates, performance testing
- **Coverage**: 100% of modules covered

## Troubleshooting

**Issue: "Could not parse timestamp"**
- Solution: Specify custom format with `--time-format`
- Example: `--time-format "%Y/%m/%d %H:%M:%S"`

**Issue: Too many or too few buckets**
- Solution: Adjust bucket size or use `--bucket auto`
- Smaller number = larger buckets, fewer rows
- Larger number = smaller buckets, more granular

**Issue: No matches found**
- Check regex pattern is correct
- Try case-insensitive: `(?i)ERROR` for case-insensitive ERROR
- Use `--no-default-pattern` to see all log volumes
- Use `--verbose` to see detailed processing information
- Use `--fail-quick` to exit immediately on no matches (CI/CD)

**Issue: Follow mode not updating**
- Ensure file exists and is readable
- Check file permissions
- Use `--verbose` to see follow mode status
- Try with smaller bucket sizes for faster updates

## Project Stats

- **Total Lines of Code**: 1,200+ lines of Rust
- **Modules**: 8 specialized modules
- **Dependencies**: 20 crates
- **Tests**: 89 tests (65 unit + 24 integration)
- **Binary Size**: 5.7MB (release)
- **Features**: 
  - ✅ Regex search with compiled patterns
  - ✅ Time bucketing (fixed + auto + sub-second)
  - ✅ Multiple output formats (5: table, CSV, JSON, ASCII, PNG)
  - ✅ Gzip support with transparent decompression
  - ✅ Stdin support with streaming
  - ✅ Follow mode with real-time updates
  - ✅ Multiple patterns with enhanced CLI
  - ✅ ASCII + PNG plotting with responsive sizing
  - ✅ Sub-second bucketing (0.1s, 0.5s precision)
  - ✅ Log generator for realistic test data
  - ✅ Terminal size detection for responsive charts
  - ✅ CSV header control
  - ✅ Verbose and fail-fast modes

## Next Steps

1. Read [README.md](README.md) for full documentation
2. See [ARCHITECTURE.md](ARCHITECTURE.md) for technical details
3. Check out [examples/](examples/) for sample log files
4. Run `logpile --help` for all options


