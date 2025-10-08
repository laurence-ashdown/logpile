use chrono::{Datelike, Timelike, Utc};
use rand::Rng;
use std::io::{self, Write};
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq)]
enum AppState {
    Startup,     // High frequency logs
    Normal,      // Regular frequency
    Busy,        // Medium-high frequency
    Error,       // Burst of error logs
    Maintenance, // Low frequency
}

impl AppState {
    fn get_interval_multiplier(&self) -> f64 {
        match self {
            AppState::Startup => 0.3,     // 3x faster
            AppState::Normal => 1.0,      // Normal speed
            AppState::Busy => 0.6,        // 1.7x faster
            AppState::Error => 0.1,       // 10x faster (burst)
            AppState::Maintenance => 0.8, // Slightly faster than normal
        }
    }

    fn get_next_state(&self) -> AppState {
        let mut rng = rand::rng();
        match rng.random_range(0..100) {
            0..=40 => AppState::Normal,       // 40% chance - most common
            41..=65 => AppState::Busy,        // 25% chance - high activity
            66..=80 => AppState::Error,       // 15% chance - error bursts
            81..=90 => AppState::Maintenance, // 10% chance - maintenance
            _ => AppState::Startup,           // 10% chance - restart
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    if args.is_empty() {
        println!(
            "Usage: {} [duration_seconds] [base_interval_ms] [variation_percent] [--simulate]",
            args[0]
        );
        println!("Example: {} 60 1000 30  # Generate logs for 60 seconds, 1s base interval, ±30% variation", args[0]);
        println!(
            "Example: {} 60 500 50   # Generate logs for 60 seconds, 500ms base, ±50% variation",
            args[0]
        );
        println!("Example: {} 60 1000 30 --simulate  # Generate 60 seconds of logs instantly with fake timestamps", args[0]);
        println!(
            "Example: {} | logpile ERROR  # Pipe directly to logpile",
            args[0]
        );
        return Ok(());
    }

    let duration_secs = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(30);
    let base_interval_ms = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(500);
    let variation_percent = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(25);
    let simulate_mode = args.contains(&"--simulate".to_string());

    // Always write to stdout for piping
    let mut writer: Box<dyn Write> = Box::new(io::stdout());

    if simulate_mode {
        eprintln!("Generating logs to stdout for {} seconds (base interval: {}ms, variation: ±{}%) - SIMULATED", 
                 duration_secs, base_interval_ms, variation_percent);
    } else {
        eprintln!(
            "Generating logs to stdout for {} seconds (base interval: {}ms, variation: ±{}%)",
            duration_secs, base_interval_ms, variation_percent
        );
        eprintln!("Press Ctrl+C to stop early");
    }

    let start_time = Utc::now();
    let mut counter = 0;

    // Simulate different application states with varying log frequencies
    let mut current_state = AppState::Startup; // Start with Startup state
    let mut _state_counter = 0;

    // For simulation mode, calculate how many entries we need to generate
    let total_entries = if simulate_mode {
        // Calculate approximate number of entries based on duration and intervals
        let avg_interval_ms = base_interval_ms as f64 * 0.8; // Account for state multipliers
        let total_ms = duration_secs * 1000;
        (total_ms as f64 / avg_interval_ms) as usize
    } else {
        0 // Will be calculated in real-time loop
    };

    // Sample log messages for different scenarios
    let normal_patterns = [
        // Application startup/shutdown
        ("INFO", "Application starting up"),
        ("INFO", "Database connection established"),
        ("INFO", "Server listening on port 8080"),
        ("INFO", "Cache initialized with 1000 entries"),
        ("INFO", "Background workers started"),
        // Normal operations
        ("INFO", "User login successful"),
        ("INFO", "Processing request from client"),
        ("INFO", "API endpoint called: /api/users"),
        ("INFO", "Database query executed successfully"),
        ("INFO", "Response sent to client"),
        ("INFO", "File uploaded successfully"),
        ("INFO", "Email sent to user"),
        ("INFO", "Payment processed"),
        ("INFO", "Report generated"),
        // Warnings
        ("WARN", "High memory usage detected"),
        ("WARN", "Database connection pool 80% full"),
        ("WARN", "Rate limit approaching for user"),
        ("WARN", "Cache miss rate above threshold"),
        ("WARN", "Slow query detected"),
        ("WARN", "Disk space running low"),
        ("WARN", "Backup job taking longer than expected"),
        // Errors
        ("ERROR", "Failed to connect to database"),
        ("ERROR", "Invalid user credentials"),
        ("ERROR", "File not found"),
        ("ERROR", "Network timeout"),
        ("ERROR", "Memory allocation failed"),
        ("ERROR", "Disk I/O error"),
        ("ERROR", "Configuration file corrupted"),
        ("ERROR", "Service unavailable"),
        // Debug info
        ("DEBUG", "Entering function process_request"),
        ("DEBUG", "Variable value: user_id=12345"),
        ("DEBUG", "SQL query: SELECT * FROM users WHERE id = ?"),
        ("DEBUG", "Cache hit for key: user_12345"),
        ("DEBUG", "Exiting function with result: success"),
        // System events
        ("INFO", "System health check passed"),
        ("INFO", "Metrics collected successfully"),
        ("INFO", "Garbage collection completed"),
        ("INFO", "Heartbeat signal received"),
    ];

    let startup_patterns = [
        ("INFO", "Application starting up"),
        ("INFO", "Loading configuration"),
        ("INFO", "Database connection established"),
        ("INFO", "Cache initialized"),
        ("INFO", "Server listening on port 8080"),
        ("INFO", "Background workers started"),
    ];
    let busy_patterns = [
        ("INFO", "Processing request from client"),
        ("INFO", "API endpoint called: /api/users"),
        ("INFO", "Database query executed successfully"),
        ("INFO", "Response sent to client"),
        ("WARN", "High memory usage detected"),
        ("WARN", "Database connection pool 80% full"),
        ("INFO", "Cache hit for key: user_12345"),
    ];
    let error_patterns = [
        ("ERROR", "Failed to connect to database"),
        ("ERROR", "Invalid user credentials"),
        ("ERROR", "File not found"),
        ("ERROR", "Network timeout"),
        ("ERROR", "Memory allocation failed"),
        ("ERROR", "Disk I/O error"),
        ("ERROR", "Configuration file corrupted"),
        ("ERROR", "Service unavailable"),
        ("WARN", "Recovery attempt 1"),
        ("WARN", "Retrying operation"),
    ];

    let maintenance_patterns = [
        ("INFO", "Starting maintenance window"),
        ("INFO", "Backing up database"),
        ("INFO", "Cleaning temporary files"),
        ("INFO", "Updating cache"),
        ("INFO", "Maintenance completed"),
    ];
    loop {
        let now = Utc::now();
        // Check if we should stop
        if !simulate_mode
            && now.timestamp_millis() - start_time.timestamp_millis() >= duration_secs * 1000
        {
            break;
        }
        if simulate_mode && counter >= total_entries {
            break;
        }

        counter += 1;

        // Generate timestamp
        let timestamp = if simulate_mode {
            // Generate fake timestamps spread across the duration
            let elapsed_ratio = counter as f64 / total_entries as f64;
            let elapsed_seconds = (duration_secs as f64 * elapsed_ratio) as i64;
            let elapsed_micros = (counter * 1000) as u32 % 1000000;
            start_time
                + chrono::Duration::seconds(elapsed_seconds)
                + chrono::Duration::microseconds(elapsed_micros as i64)
        } else {
            Utc::now()
        };

        // Change application state randomly
        let rng = rand::rng().random_range(0..100);
        let should_change = match current_state {
            AppState::Error => rng < 15,  // 15% chance to change from error
            AppState::Startup => rng < 5, // 5% chance to change from startup
            _ => rng < 8,                 // 8% chance to change from other states
        };

        if should_change {
            current_state = current_state.get_next_state();
            _state_counter = 0;
        }
        _state_counter += 1;

        // Calculate interval with variation and state-based multiplier
        let base_interval = base_interval_ms as f64 * current_state.get_interval_multiplier();
        let variation =
            rand::rng().random_range(-variation_percent..=variation_percent) as f64 / 100.0;
        let actual_interval = (base_interval * (1.0 + variation)) as u64;

        // Generate timestamp format based on counter with microsecond precision
        let timestamp_str = if counter % 50 == 0 {
            // ISO 8601 format with microseconds
            timestamp.format("%Y-%m-%dT%H:%M:%S%.6fZ").to_string()
        } else if counter % 30 == 0 {
            // Apache format with microseconds
            format!(
                "[{:02}/{:02}/{}:{:02}:{:02}:{:02}.{:06} +0000]",
                timestamp.day(),
                timestamp.month(),
                timestamp.year(),
                timestamp.hour(),
                timestamp.minute(),
                timestamp.second(),
                timestamp.timestamp_subsec_micros()
            )
        } else if counter % 20 == 0 {
            // Syslog format with microseconds
            format!(
                "{:02} {:02} {:02}:{:02}:{:02}.{:06}",
                timestamp.month(),
                timestamp.day(),
                timestamp.hour(),
                timestamp.minute(),
                timestamp.second(),
                timestamp.timestamp_subsec_micros()
            )
        } else if counter % 15 == 0 {
            // European format with microseconds
            format!(
                "{:02}/{:02}/{} {:02}:{:02}:{:02}.{:06}",
                timestamp.day(),
                timestamp.month(),
                timestamp.year(),
                timestamp.hour(),
                timestamp.minute(),
                timestamp.second(),
                timestamp.timestamp_subsec_micros()
            )
        } else {
            // Default ISO format with microseconds
            timestamp.format("%Y-%m-%dT%H:%M:%S%.6fZ").to_string()
        };

        // Select log pattern based on current state using random
        let (level, message) = match current_state {
            AppState::Startup => {
                let idx = rand::rng().random_range(0..startup_patterns.len());
                startup_patterns[idx]
            }
            AppState::Normal => {
                let idx = rand::rng().random_range(0..normal_patterns.len());
                normal_patterns[idx]
            }
            AppState::Busy => {
                let idx = rand::rng().random_range(0..busy_patterns.len());
                busy_patterns[idx]
            }
            AppState::Error => {
                let idx = rand::rng().random_range(0..error_patterns.len());
                error_patterns[idx]
            }
            AppState::Maintenance => {
                let idx = rand::rng().random_range(0..maintenance_patterns.len());
                maintenance_patterns[idx]
            }
        };

        // Add variety to messages based on state
        let message = match current_state {
            AppState::Error => match counter % 50 {
                0..=10 => format!("{} - Error code: {}", message, 1000 + (counter % 9000)),
                11..=20 => format!("{} - Stack trace: stack_{:06}", message, counter),
                21..=30 => format!(
                    "{} - Component: {}",
                    message,
                    ["database", "api", "cache", "auth"][counter % 4]
                ),
                _ => message.to_string(),
            },
            AppState::Busy => match counter % 40 {
                0..=10 => format!("{} - Request ID: req_{:06}", message, counter),
                11..=20 => format!("{} - Duration: {}ms", message, 50 + (counter % 500)),
                21..=30 => format!("{} - Queue depth: {}", message, 1 + (counter % 100)),
                _ => message.to_string(),
            },
            _ => match counter % 100 {
                0..=10 => format!("{} - User ID: {}", message, 1000 + (counter % 9000)),
                11..=20 => format!("{} - Request ID: req_{:06}", message, counter),
                21..=30 => format!("{} - Duration: {}ms", message, 50 + (counter % 500)),
                31..=40 => format!("{} - IP: 192.168.1.{}", message, 1 + (counter % 254)),
                41..=50 => format!("{} - Session: sess_{:08x}", message, counter),
                _ => message.to_string(),
            },
        };

        // Write the log line
        writeln!(writer, "{} {} {}", timestamp_str, level, message)?;

        // Flush frequently for stdout
        if counter % 3 == 0 {
            writer.flush()?;
        }

        // Add burst patterns in error state
        if current_state == AppState::Error && counter % 15 == 0 {
            // Burst of 3-7 error messages quickly
            let burst_count = rand::rng().random_range(3..=7);
            for i in 1..=burst_count {
                let burst_timestamp =
                    timestamp + chrono::Duration::seconds(i) + chrono::Duration::milliseconds(i);
                let burst_msg = format!("Cascading error {} of {}", i, burst_count);
                writeln!(
                    writer,
                    "{} ERROR {}",
                    burst_timestamp.format("%Y-%m-%dT%H:%M:%S%.6fZ"),
                    burst_msg
                )?;
            }
            writer.flush()?;
            if !simulate_mode {
                thread::sleep(Duration::from_millis(actual_interval / 4));
            }
        }

        if !simulate_mode {
            thread::sleep(Duration::from_millis(actual_interval));
        }
    }

    writer.flush()?;

    if simulate_mode {
        eprintln!(
            "\nLog generation complete! Generated {} log entries (simulated).",
            counter
        );
    } else {
        eprintln!(
            "\nLog generation complete! Generated {} log entries.",
            counter
        );
    }

    Ok(())
}
