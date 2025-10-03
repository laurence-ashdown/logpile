# Contributing to Logpile

Thank you for your interest in contributing to logpile! This document provides guidelines and instructions for contributing.

## Getting Started

1. Fork the repository on GitHub
2. Clone your fork locally:
   ```bash
   git clone https://github.com/YOUR_USERNAME/logpile.git
   cd logpile
   ```

3. Install system dependencies:
   - **Ubuntu/Debian**: `sudo apt-get install pkg-config libfontconfig1-dev`
   - **macOS**: `brew install pkg-config fontconfig`
   - **Windows**: Install via vcpkg or use WSL

4. Build and test:
   ```bash
   cargo build
   cargo test
   ```

## Development Workflow

### Making Changes

1. Create a new branch:
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. Make your changes following the coding standards below

3. Run tests:
   ```bash
   cargo test
   cargo clippy --all-targets --all-features
   cargo fmt --all -- --check
   ```

4. Commit your changes:
   ```bash
   git commit -m "Add feature: description"
   ```

5. Push to your fork:
   ```bash
   git push origin feature/your-feature-name
   ```

6. Open a Pull Request on GitHub

### Coding Standards

- **Formatting**: Run `cargo fmt` before committing
- **Linting**: Ensure `cargo clippy` passes with no warnings
- **Testing**: Add unit tests for new functionality
- **Documentation**: Update relevant docs (README, QUICK_START, etc.)
- **Commit Messages**: Use clear, descriptive commit messages

### Adding New Features

When adding new features, please:

1. **Write tests first** (TDD approach recommended)
2. **Update documentation**:
   - README.md for user-facing features
   - ARCHITECTURE.md for internal changes
   - Add examples to `examples/` directory
3. **Update CHANGELOG.md** in the Unreleased section
4. **Ensure backward compatibility** when possible

### Testing Guidelines

- **Unit tests**: Add to the relevant module's `#[cfg(test)]` section
- **Integration tests**: Add to `tests/` directory (create if needed)
- **Test coverage**: Aim for high coverage of new code
- **Test naming**: Use descriptive names like `test_parse_iso8601_with_timezone`

Example test:
```rust
#[test]
fn test_my_feature() {
    let result = my_function();
    assert_eq!(result, expected_value);
}
```

## Types of Contributions

### Bug Reports

When reporting bugs, please include:
- Rust version (`rustc --version`)
- Operating system
- Steps to reproduce
- Expected vs actual behavior
- Relevant log files or examples

Use the issue template:
```markdown
**Describe the bug**
A clear description of the bug.

**To Reproduce**
1. Run command: `logpile ...`
2. See error

**Expected behavior**
What you expected to happen.

**Environment**
- OS: [e.g., Ubuntu 22.04]
- Rust version: [e.g., 1.70.0]
```

### Feature Requests

For feature requests, describe:
- The problem you're trying to solve
- Proposed solution
- Alternative solutions considered
- Impact on existing functionality

### Pull Requests

Good pull requests:
- Solve one problem at a time
- Include tests
- Update documentation
- Pass all CI checks
- Have clear commit messages

## Project Structure

```
logpile/
├── src/
│   ├── main.rs          # Entry point
│   ├── lib.rs           # Library exports
│   ├── cli.rs           # Argument parsing
│   ├── timestamp.rs     # Timestamp parsing
│   ├── bucket.rs        # Time bucketing
│   ├── reader.rs        # File reading
│   ├── processor.rs     # Main processing logic
│   ├── output.rs        # Output formatting
│   └── plot.rs          # Chart generation
├── examples/            # Example log files
├── tests/              # Integration tests
└── docs/               # Additional documentation
```

## Adding New Timestamp Formats

To add support for a new timestamp format:

1. Add the format string to `COMMON_FORMATS` in `src/timestamp.rs`
2. Add a regex pattern to extract it in `TimestampParser::new()`
3. Update the extraction logic in `extract_timestamp_candidates()`
4. Add unit tests in the `timestamp::tests` module
5. Add an example log file to `examples/`
6. Update `examples/TIMESTAMP_FORMATS.md`

## Code Review Process

1. All PRs require at least one review
2. CI must pass (tests, clippy, formatting)
3. Maintainers may request changes
4. Once approved, maintainers will merge

## Community Guidelines

- Be respectful and inclusive
- Help others learn and grow
- Focus on constructive feedback
- Follow the [Rust Code of Conduct](https://www.rust-lang.org/policies/code-of-conduct)

## Questions?

- Open an issue for discussion
- Tag maintainers with @mentions
- Check existing issues/PRs first

## License

By contributing, you agree that your contributions will be licensed under the MIT License.

