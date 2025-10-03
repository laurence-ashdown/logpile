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

### 1. Basic Search
Search for "ERROR" in a log file:
```bash
logpile "ERROR" examples/sample.log
```

### 2. Time Bucketing
Group matches into 5-minute (300 second) buckets:
```bash
logpile "ERROR" examples/sample.log --bucket 300
```

### 3. Different Output Formats

**CSV:**
```bash
logpile "ERROR" examples/sample.log --bucket 300 --csv > errors.csv
```

**JSON:**
```bash
logpile "WARN" examples/sample.log --bucket 300 --json
```

**ASCII Plot:**
```bash
logpile "ERROR|WARN" examples/sample.log --bucket 300 --plot
```

**Bitmap Chart:**
```bash
logpile "ERROR" examples/sample.log --bucket 300 --png chart.ppm
```

### 4. Multiple Patterns
Search for multiple patterns:
```bash
logpile "ERROR" examples/sample.log --grep "WARN" --grep "CRITICAL" --bucket 600
```

### 5. Gzipped Files
Transparently read gzipped logs:
```bash
logpile "ERROR" examples/sample.log.gz --bucket 300
```

### 6. Stdin Input
Pipe logs from other commands:
```bash
cat examples/sample.log | logpile "INFO" --bucket 600
zcat large_log.gz | logpile "timeout" --bucket 3600
```

### 7. Auto Bucket Size
Let logpile choose the best bucket size:
```bash
logpile "ERROR" examples/sample.log --bucket auto
```

### 8. Count All Lines
Count all log entries without pattern filtering:
```bash
logpile --no-default-pattern examples/sample.log --bucket 600
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
   tail -f /var/log/app.log | logpile "ERROR" --bucket 60
   ```

## Testing

Run the test suite:
```bash
./test_examples.sh
```

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

## Project Stats

- **Total Lines of Code**: ~800 lines of Rust
- **Modules**: 8 specialized modules
- **Dependencies**: 16 crates
- **Features**: 
  - ✅ Regex search
  - ✅ Time bucketing (fixed + auto)
  - ✅ Multiple output formats (4)
  - ✅ Gzip support
  - ✅ Stdin support
  - ✅ Follow mode (basic)
  - ✅ Multiple patterns
  - ✅ ASCII + bitmap plotting

## Next Steps

1. Read [README.md](README.md) for full documentation
2. See [ARCHITECTURE.md](ARCHITECTURE.md) for technical details
3. Check out [examples/](examples/) for sample log files
4. Run `logpile --help` for all options


