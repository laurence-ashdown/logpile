# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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

[0.1.0]: https://github.com/lashdown/logpile/releases/tag/v0.1.0

