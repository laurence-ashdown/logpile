# logpile Architecture

## Project Structure

```
logpile/
├── Cargo.toml                # Project dependencies and metadata
├── src/
│   ├── main.rs              # Entry point
│   ├── lib.rs               # Library exports
│   ├── cli.rs               # Command-line argument parsing (clap)
│   ├── timestamp.rs         # Timestamp parsing and auto-detection
│   ├── bucket.rs            # Time-based bucketing logic
│   ├── reader.rs            # File/stdin reading with gzip support
│   ├── output.rs            # Output formatters (table, CSV, JSON)
│   ├── plot.rs              # Plotting (ASCII and bitmap)
│   └── processor.rs         # Main processing orchestration
├── examples/
│   ├── sample.log           # Example log file for testing
│   ├── sample.log.gz        # Gzipped example
│   ├── log_generator.rs     # Log generation tool
│   └── scripts/             # Demo and test scripts
├── tests/                   # Integration tests
├── README.md                # User documentation
├── ARCHITECTURE.md          # This file
└── LICENSE                  # MIT License
```

## Module Overview

### `main.rs`
- Entry point for the binary
- Parses CLI arguments using clap
- Creates and runs the LogProcessor

### `cli.rs`
- Defines the `Args` struct with all CLI options
- Uses clap's derive macro for argument parsing
- Provides validation and helper methods
- Handles the special case of `--no-default-pattern`

### `timestamp.rs`
- `TimestampParser` struct for parsing timestamps from log lines
- Auto-detects common timestamp formats:
  - ISO 8601
  - Common log formats (YYYY-MM-DD HH:MM:SS)
  - Syslog format
  - Apache/Nginx formats
- Supports custom time format strings via `--time-format`
- Uses regex to extract timestamp candidates from log lines

### `bucket.rs`
- `TimeBucket` struct for time-based aggregation
- Supports fixed bucket sizes (in seconds)
- **NEW**: Sub-second bucketing support (0.1s, 0.5s, etc.)
- Supports automatic bucket size selection based on time range
- Uses `BTreeMap` for ordered bucket storage
- Tracks first/last timestamps for time range calculation
- **NEW**: Microsecond precision for high-resolution analysis

### `reader.rs`
- `LogReader` enum for different input sources:
  - Plain text files
  - Gzipped files (.gz)
  - Stdin
- Transparent decompression for gzipped files using flate2
- Provides unified iterator interface for all sources

### `output.rs`
- Functions for different output formats:
  - `output_table()`: Human-readable table with borders
  - `output_csv()`: CSV format for data export
  - `output_json()`: JSON format with metadata
- Uses serde for JSON serialization
- Uses csv crate for proper CSV formatting

### `plot.rs`
- `plot_ascii()`: ASCII charts using textplots
  - Uses Braille characters for smooth lines
  - Shows time range and bucket information
  - **NEW**: Terminal size detection for responsive charts
  - **NEW**: Y-axis zero option for consistent scaling
- `plot_png()`: Bitmap charts using plotters
  - Generates PPM format (can be converted to PNG)
  - Includes line series and data points
  - Labeled axes with timestamps

### `processor.rs`
- `LogProcessor`: Main orchestration logic with enhanced error handling
- Implements two modes:
  - **Batch mode**: Process files once with graceful degradation
  - **Follow mode**: Continuously monitor file (like tail -f) with real-time updates
- **NEW**: Verbose mode for debugging and detailed output
- **NEW**: Fail-fast mode for CI/CD environments
- **NEW**: Better error messages and warnings
- Compiles regex patterns
- Iterates through log lines
- Extracts timestamps and matches patterns
- Aggregates matches into time buckets
- Calls appropriate output formatter

## Data Flow

```
1. CLI Arguments → Args struct (clap parsing with enhanced options)
2. Args → LogProcessor initialization
   - Compile regex patterns
   - Create TimestampParser with microsecond precision
   - Initialize TimeBucket with sub-second support
3. LogProcessor → Read input
   - Files or stdin with streaming
   - Decompress if .gz
   - Follow mode for real-time monitoring
4. For each line:
   - Check regex match
   - Parse timestamp with enhanced detection
   - Add to bucket with microsecond precision
5. After processing:
   - Get bucket data
   - Format output (table/CSV/JSON/plot) with responsive sizing
   - Handle errors gracefully
```

## Key Design Decisions

### 1. **Modular Architecture**
Each module has a single responsibility:
- Separation of concerns
- Easy to test
- Clear interfaces

### 2. **Regex-Based Timestamp Extraction**
- Flexible auto-detection
- Handles various formats without user configuration
- Falls back to custom format if provided

### 3. **BTreeMap for Buckets**
- Maintains sorted order by timestamp
- Efficient range queries
- Natural ordering for output

### 4. **Iterator-Based File Reading**
- Memory efficient for large files
- Uniform interface for all input types
- Lazy evaluation

### 5. **Enum for Output Formats**
- Type-safe format selection
- Compile-time validation
- Easy to extend

### 6. **Auto Bucket Size**
- Aims for ~30 buckets for good visualization
- Rounds to "nice" intervals (1m, 5m, 15m, 1h, 6h, 1d, etc.)
- Adapts to time range automatically

## Dependencies

### Core
- `clap`: CLI argument parsing with derive macros
- `chrono`: Timestamp parsing and manipulation
- `regex`: Pattern matching
- `anyhow`/`thiserror`: Error handling

### I/O
- `flate2`: Gzip decompression
- `csv`: CSV output formatting
- `serde`/`serde_json`: JSON serialization

### Visualization
- `textplots`: ASCII chart rendering
- `plotters`: Bitmap chart generation (minimal features to avoid system dependencies)

## Performance Considerations

1. **Streaming Processing**: Lines are processed as they're read, not loaded into memory
2. **Compiled Regexes**: Patterns are compiled once at startup
3. **Efficient Bucketing**: O(log n) insertion into BTreeMap
4. **Lazy Evaluation**: Iterator-based pipeline

## Future Enhancements

### Planned Features
- [ ] Severity grouping (INFO/WARN/ERROR breakdown)
- [ ] Prometheus metrics export
- [ ] Interactive TUI with zoom/pan
- [ ] Multi-threaded file processing
- [ ] Real tail -f implementation (inotify on Linux)
- [ ] Histogram distribution analysis
- [ ] Custom aggregation functions (min/max/avg)
- [ ] Support for structured logs (JSON logs)

### Possible Optimizations
- [ ] Parallel processing of multiple files
- [ ] Memoization of timestamp parsing
- [ ] Skip non-matching lines early
- [ ] Compressed output for large result sets


