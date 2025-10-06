# Log Generator for Testing

This tool generates realistic log files for testing the `logpile` application.

## Quick Start

```bash
# Generate test logs automatically
./generate_test_logs.sh

# Or generate manually
./target/debug/examples/log_generator 60 1000 > myapp.log
```

## Usage

```bash
./target/debug/examples/log_generator [duration_seconds] [base_interval_ms] [variation_percent]
```

### Parameters:
- `duration_seconds`: How long to generate logs (default: 30)
- `base_interval_ms`: Base milliseconds between log entries (default: 500)
- `variation_percent`: Random variation percentage (default: 25)

**Note:** The generator always outputs to stdout for easy piping to other tools or redirection to files.

### Features:
- **Realistic log patterns**: Simulates different application states (startup, normal, busy, error, maintenance)
- **Varied intervals**: Random timing variations to emulate real applications
- **Multiple timestamp formats**: ISO 8601, Apache, Syslog, European dates
- **State-based logging**: Different log patterns based on simulated application state
- **Piping support**: Always outputs to stdout for easy piping

### Examples:

```bash
# Generate 30 seconds of logs with 1-second intervals, ±30% variation
./target/debug/examples/log_generator 30 1000 30 > examples/generated_logs/app.log

# Generate high-frequency logs (100ms intervals, ±50% variation)
./target/debug/examples/log_generator 60 100 50 > examples/generated_logs/busy.log

# Generate logs for 5 minutes with 2-second intervals, ±20% variation
./target/debug/examples/log_generator 300 2000 20 > examples/generated_logs/long.log

# Pipe logs directly to stdout
./target/debug/examples/log_generator 60 500 25

# Pipe directly to logpile for real-time analysis
./target/debug/examples/log_generator 60 800 30 | ./target/release/logpile "ERROR"

# Generate realistic logs with high variation (like a real app)
./target/debug/examples/log_generator 120 1000 60 > examples/generated_logs/realistic.log
```

## Generated Log Features

The generator creates realistic log entries with:

- **Multiple log levels**: INFO, WARN, ERROR, DEBUG
- **Different timestamp formats**: ISO 8601, Apache, Syslog, European dates
- **Varied message types**: Application startup, user actions, system events, errors
- **Dynamic content**: User IDs, request IDs, durations, IP addresses, sessions
- **Burst patterns**: Occasional rapid-fire log entries
- **Realistic scenarios**: Database connections, API calls, file operations, etc.

## Testing with logpile

Once you have generated logs, test them with various logpile commands:

```bash
# Basic search
./target/release/logpile "ERROR" examples/generated_logs/app.log

# CSV output
./target/release/logpile "WARN" examples/generated_logs/app.log --csv

# Plot visualization
./target/release/logpile "INFO" examples/generated_logs/app.log --plot

# Follow mode (for real-time logs)
./target/release/logpile "ERROR" examples/generated_logs/app.log --follow

# Multiple patterns
./target/release/logpile "ERROR" examples/generated_logs/app.log --grep "WARN" --grep "INFO"

# PNG output
./target/release/logpile "ERROR" examples/generated_logs/app.log --png examples/generated_logs/error_plot.png

# Custom bucket size
./target/release/logpile "INFO" examples/generated_logs/app.log --bucket 10

# JSON output
./target/release/logpile "ERROR" examples/generated_logs/app.log --json
```

## Log Patterns Generated

The generator cycles through these realistic log patterns:

### Application Events
- Application startup/shutdown
- Database connections
- Server startup
- Cache initialization
- Background workers

### User Operations
- User logins
- API calls
- File uploads
- Payments
- Email sending

### System Monitoring
- Memory usage warnings
- Database pool status
- Rate limiting
- Cache performance
- Disk space monitoring

### Errors
- Database connection failures
- Authentication errors
- File not found errors
- Network timeouts
- Memory allocation failures

### Debug Information
- Function entry/exit
- Variable values
- SQL queries
- Cache hits/misses

This gives you a comprehensive set of realistic log data to test all features of logpile!
