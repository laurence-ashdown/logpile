# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0] - 2025-10-06

### Added
- **Sub-second bucketing support** - Bucket sizes can now be fractional seconds (e.g., 0.5s, 0.1s)
- **Follow mode (`--follow`)** - Real-time log monitoring with live updates
- **Enhanced CLI interface** - Short flags for common options (`-c`, `-j`, `-p`, `-o`, `-f`, `-v`, `-q`, `-n`)
- **Log generator tool** - Realistic test data generation with multiple timestamp formats
- **Comprehensive test suite** - 991 lines of new integration and follow mode tests
- **Enhanced plotting** - Terminal size detection and responsive ASCII charts
- **CSV header control** - `--no-headers` option for CSV output
- **Verbose mode** - `--verbose` flag for debugging and detailed output
- **Fail-fast option** - `--fail-quick` for CI/CD environments
- **Y-axis zero option** - `--y-zero` for consistent plot scaling
- **Demo scripts** - Automated demonstration and testing scripts
- **Enhanced documentation** - Complete guides for all features

### Changed
- **Improved error handling** - Graceful degradation instead of hard failures
- **Better timestamp detection** - Enhanced regex patterns for microsecond precision
- **Enhanced auto-bucket calculation** - More intelligent bucket size selection
- **Improved output formatting** - Better display of sub-second precision
- **Updated dependencies** - Added terminal_size, console, rgb, and rand crates

### Fixed
- **Better file processing** - Continue processing other files when one fails
- **Improved timestamp parsing** - Support for Apache logs with microsecond precision
- **Enhanced error messages** - More helpful debugging information

## [0.2.0] - 2025-10-03

### Added
- Support for yearless ISO timestamp formats (e.g., `09-24T23:45:29.362Z`)
- Support for time-only timestamp formats (e.g., `05:40:12`)
- Enhanced timestamp auto-detection with improved regex patterns
- Comprehensive unit tests for new timestamp formats

### Changed
- Improved early failure logic to distinguish between "no matching lines" vs "no timestamps"
- Better error messages for debugging timestamp parsing issues
- Enhanced timestamp parsing with automatic year/date injection for partial formats

### Fixed
- Improved code formatting and consistency

## [0.1.0] - 2025-10-03

### Added
- Initial release of logpile
- Regex pattern matching for log filtering
- Support for 10 different timestamp formats:
  - ISO 8601 (with and without timezone)
  - Apache/Nginx Common Log Format
  - Syslog RFC 3164
  - European date format (DD/MM/YYYY)
  - US date format (MM/DD/YYYY)
  - Unix timestamps
  - RFC 2822
  - Java application logs
  - Microsecond precision timestamps
  - Custom formats via `--time-format`
- Time bucketing with auto-sizing or manual configuration
- Multiple output formats:
  - Table (default, human-readable console output)
  - CSV for spreadsheet analysis
  - JSON for programmatic consumption
  - ASCII plots for quick terminal visualization
  - PNG charts with matplotlib-style formatting
- Support for multiple files and glob patterns
- Gzip compressed log file support (`.gz`)
- Stdin support for piping
- Follow mode (`--follow`) for real-time log monitoring
- Multiple grep patterns with `--grep` flag
- Comprehensive test suite with 49 unit tests
- Full documentation (README, QUICK_START, ARCHITECTURE, TESTING)

### Features
- **Fast**: Efficiently processes large log files
- **Flexible**: Auto-detects timestamp formats
- **Visual**: Beautiful PNG charts with titles, labels, and legends
- **Portable**: Single binary, no runtime dependencies (after build)
- **Well-tested**: 100% module test coverage

[0.3.0]: https://github.com/laurence-ashdown/logpile/releases/tag/v0.3.0
[0.2.0]: https://github.com/laurence-ashdown/logpile/releases/tag/v0.2.0
[0.1.0]: https://github.com/laurence-ashdown/logpile/releases/tag/v0.1.0

