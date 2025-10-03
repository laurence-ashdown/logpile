# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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

[0.2.0]: https://github.com/laurence-ashdown/logpile/releases/tag/v0.2.0
[0.1.0]: https://github.com/laurence-ashdown/logpile/releases/tag/v0.1.0

