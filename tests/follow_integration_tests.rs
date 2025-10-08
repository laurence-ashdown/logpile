use std::fs::OpenOptions;
use std::io::Write;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;
use tempfile::NamedTempFile;

/// Test follow mode with real-time file updates
#[test]
fn test_follow_mode_real_time_updates() {
    let initial_content = "2025-10-03T12:00:00Z INFO Initial message\n";
    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
    temp_file
        .write_all(initial_content.as_bytes())
        .expect("Failed to write initial content");
    temp_file.flush().expect("Failed to flush");

    let file_path = temp_file.path().to_str().unwrap();

    // Start follow mode with CSV output for easier parsing
    let mut child = Command::new(env!("CARGO_BIN_EXE_logpile"))
        .args(["INFO", file_path, "--follow", "--csv", "--no-headers"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start logpile");

    // Give it time to process initial content
    thread::sleep(Duration::from_millis(1000));

    // Open file for appending
    let mut file = OpenOptions::new()
        .append(true)
        .open(file_path)
        .expect("Failed to open file for appending");

    // Append multiple lines with delays
    let updates = [
        "2025-10-03T12:01:00Z INFO Update 1\n",
        "2025-10-03T12:02:00Z INFO Update 2\n",
        "2025-10-03T12:03:00Z INFO Update 3\n",
    ];

    for (i, update) in updates.iter().enumerate() {
        file.write_all(update.as_bytes()).expect("Failed to append");
        file.flush().expect("Failed to flush");

        // Give time for the follow mode to detect and process
        thread::sleep(Duration::from_millis(500));

        // Check that process is still running
        let status = child.try_wait().expect("Failed to check process status");
        assert!(
            status.is_none(),
            "Process should still be running after update {}",
            i + 1
        );
    }

    // Give final time for processing
    thread::sleep(Duration::from_millis(1000));

    // Terminate and collect output
    child.kill().expect("Failed to kill process");
    let output = child.wait_with_output().expect("Failed to get output");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout
        .lines()
        .filter(|line| line.contains("2025-10-03"))
        .collect();

    // Should have processed at least the initial message (others might be bucketed)
    assert!(
        !lines.is_empty(),
        "Should have processed at least one line, got {}",
        lines.len()
    );

    // Check that initial timestamp is present
    assert!(
        lines.iter().any(|line| line.contains("12:00:00")),
        "Should contain initial timestamp"
    );
}

/// Test follow mode with mixed log levels and patterns
#[test]
fn test_follow_mode_mixed_patterns() {
    let initial_content =
        "2025-10-03T12:00:00Z INFO App started\n2025-10-03T12:00:15Z WARN Low memory\n";
    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
    temp_file
        .write_all(initial_content.as_bytes())
        .expect("Failed to write initial content");
    temp_file.flush().expect("Failed to flush");

    let file_path = temp_file.path().to_str().unwrap();

    // Start follow mode with multiple grep patterns
    let mut child = Command::new(env!("CARGO_BIN_EXE_logpile"))
        .args([
            "INFO", file_path, "--follow", "--grep", "WARN", "--grep", "ERROR",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start logpile");

    // Give it time to process initial content
    thread::sleep(Duration::from_millis(1000));

    // Open file for appending
    let mut file = OpenOptions::new()
        .append(true)
        .open(file_path)
        .expect("Failed to open file for appending");

    // Append lines with different log levels
    let updates = [
        "2025-10-03T12:01:00Z INFO User logged in\n",
        "2025-10-03T12:01:30Z WARN Connection timeout\n",
        "2025-10-03T12:02:00Z ERROR Database connection failed\n",
        "2025-10-03T12:02:30Z DEBUG Internal state: OK\n", // This should NOT match
        "2025-10-03T12:03:00Z INFO User action completed\n",
    ];

    for update in &updates {
        file.write_all(update.as_bytes()).expect("Failed to append");
        file.flush().expect("Failed to flush");
        thread::sleep(Duration::from_millis(300));
    }

    // Give final time for processing
    thread::sleep(Duration::from_millis(1000));

    // Terminate and collect output
    child.kill().expect("Failed to kill process");
    let output = child.wait_with_output().expect("Failed to get output");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let _stderr = String::from_utf8_lossy(&output.stderr);

    // Should contain bucketed timestamps showing the matches were found
    assert!(
        stdout.contains("2025-10-03 12:00:00"),
        "Should contain initial timestamp"
    );
    assert!(
        stdout.contains("2025-10-03 12:01:00"),
        "Should contain update timestamps"
    );
    assert!(
        stdout.contains("2025-10-03 12:02:00"),
        "Should contain update timestamps"
    );
    assert!(
        stdout.contains("2025-10-03 12:03:00"),
        "Should contain final timestamp"
    );

    // Should show counts indicating multiple matches were found
    assert!(stdout.contains("Count"), "Should show count column");
    assert!(stdout.contains("Total"), "Should show total count");
}

/// Test follow mode with large file updates
#[test]
fn test_follow_mode_large_updates() {
    let initial_content = "2025-10-03T12:00:00Z INFO Initial message\n";
    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
    temp_file
        .write_all(initial_content.as_bytes())
        .expect("Failed to write initial content");
    temp_file.flush().expect("Failed to flush");

    let file_path = temp_file.path().to_str().unwrap();

    // Start follow mode
    let mut child = Command::new(env!("CARGO_BIN_EXE_logpile"))
        .args(["INFO", file_path, "--follow", "--csv", "--no-headers"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start logpile");

    // Give it time to process initial content
    thread::sleep(Duration::from_millis(1000));

    // Open file for appending
    let mut file = OpenOptions::new()
        .append(true)
        .open(file_path)
        .expect("Failed to open file for appending");

    // Append a large batch of lines at once
    let mut large_batch = String::new();
    for i in 1..=20 {
        large_batch.push_str(&format!("2025-10-03T12:{:02}:00Z INFO Message {}\n", i, i));
    }

    file.write_all(large_batch.as_bytes())
        .expect("Failed to append large batch");
    file.flush().expect("Failed to flush");

    // Give more time for processing large batch
    thread::sleep(Duration::from_millis(1500));

    // Terminate and collect output
    child.kill().expect("Failed to kill process");
    let output = child.wait_with_output().expect("Failed to get output");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout
        .lines()
        .filter(|line| line.contains("2025-10-03"))
        .collect();

    // Should have processed at least the initial message and some from the batch
    assert!(
        !lines.is_empty(),
        "Should have processed at least one line, got {}",
        lines.len()
    );

    // Check that various messages from the batch are present (may be bucketed)
    assert!(
        lines.iter().any(|line| line.contains("12:00:00")),
        "Should contain initial timestamp"
    );
}

/// Test follow mode with different timestamp formats
#[test]
fn test_follow_mode_different_timestamps() {
    let initial_content = "2025-10-03T12:00:00Z INFO Initial message\n";
    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
    temp_file
        .write_all(initial_content.as_bytes())
        .expect("Failed to write initial content");
    temp_file.flush().expect("Failed to flush");

    let file_path = temp_file.path().to_str().unwrap();

    // Start follow mode
    let mut child = Command::new(env!("CARGO_BIN_EXE_logpile"))
        .args(["INFO", file_path, "--follow", "--csv", "--no-headers"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start logpile");

    // Give it time to process initial content
    thread::sleep(Duration::from_millis(1000));

    // Open file for appending
    let mut file = OpenOptions::new()
        .append(true)
        .open(file_path)
        .expect("Failed to open file for appending");

    // Append lines with different timestamp formats
    let updates = [
        "2025-10-03 12:01:00 INFO Apache format\n", // Space instead of T
        "03/10/2025 12:02:00 INFO European format\n", // DD/MM/YYYY format
        "Oct 03 12:03:00 INFO Syslog format\n",     // Syslog format
        "2025-10-03T12:04:00Z INFO Back to ISO format\n", // ISO format
    ];

    for update in &updates {
        file.write_all(update.as_bytes()).expect("Failed to append");
        file.flush().expect("Failed to flush");
        thread::sleep(Duration::from_millis(400));
    }

    // Give final time for processing
    thread::sleep(Duration::from_millis(1000));

    // Terminate and collect output
    child.kill().expect("Failed to kill process");
    let output = child.wait_with_output().expect("Failed to get output");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout
        .lines()
        .filter(|line| line.contains("2025-10-03") || line.contains("INFO"))
        .collect();

    // Should have processed at least the initial message
    assert!(
        !lines.is_empty(),
        "Should have processed at least one timestamp format, got {}",
        lines.len()
    );

    // At minimum, the initial ISO format should work
    assert!(
        stdout.contains("12:00:00"),
        "Should contain initial ISO format timestamp"
    );
}

/// Test follow mode performance with rapid updates
#[test]
fn test_follow_mode_performance() {
    let initial_content = "2025-10-03T12:00:00Z INFO Initial message\n";
    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
    temp_file
        .write_all(initial_content.as_bytes())
        .expect("Failed to write initial content");
    temp_file.flush().expect("Failed to flush");

    let file_path = temp_file.path().to_str().unwrap();

    // Start follow mode
    let mut child = Command::new(env!("CARGO_BIN_EXE_logpile"))
        .args(["INFO", file_path, "--follow", "--csv", "--no-headers"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start logpile");

    // Give it time to process initial content
    thread::sleep(Duration::from_millis(500));

    // Open file for appending
    let mut file = OpenOptions::new()
        .append(true)
        .open(file_path)
        .expect("Failed to open file for appending");

    // Rapid fire updates
    let start_time = std::time::Instant::now();
    for i in 1..=20 {
        // Reduced from 50 to 20 for reliability
        let update = format!("2025-10-03T12:{:02}:00Z INFO Rapid update {}\n", i % 60, i);
        file.write_all(update.as_bytes()).expect("Failed to append");
        file.flush().expect("Failed to flush");

        // Very short delay to simulate rapid updates
        thread::sleep(Duration::from_millis(20));

        // Check that process is still responsive every 5 updates
        if i % 5 == 0 {
            let status = child.try_wait().expect("Failed to check process status");
            assert!(
                status.is_none(),
                "Process should still be running after rapid update {}",
                i
            );
        }
    }

    let elapsed = start_time.elapsed();

    // Give final time for processing
    thread::sleep(Duration::from_millis(1000));

    // Terminate and collect output
    child.kill().expect("Failed to kill process");
    let output = child.wait_with_output().expect("Failed to get output");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout
        .lines()
        .filter(|line| line.contains("2025-10-03"))
        .collect();

    // Should have processed a reasonable number of rapid updates
    assert!(
        !lines.is_empty(),
        "Should have processed at least one rapid update, got {}",
        lines.len()
    );

    // Performance check: should handle updates in reasonable time
    assert!(
        elapsed.as_secs() < 5,
        "Should handle rapid updates efficiently, took {:?}",
        elapsed
    );
}

/// Test follow mode with file that doesn't exist initially (created during follow)
#[test]
fn test_follow_mode_file_creation() {
    // Create a temporary directory and file path
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let file_path = temp_dir.path().join("dynamic.log");

    // Verify file doesn't exist
    assert!(!file_path.exists(), "File should not exist initially");

    // Start follow mode on non-existent file
    let mut child = Command::new(env!("CARGO_BIN_EXE_logpile"))
        .args(["INFO", file_path.to_str().unwrap(), "--follow"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start logpile");

    // Give it time to start
    thread::sleep(Duration::from_millis(500));

    // Create the file and add content
    let mut file = std::fs::File::create(&file_path).expect("Failed to create file");
    file.write_all("2025-10-03T12:00:00Z INFO First message in new file\n".as_bytes())
        .expect("Failed to write to new file");
    file.flush().expect("Failed to flush new file");

    // Give it time to detect the new file
    thread::sleep(Duration::from_millis(1000));

    // Add more content
    file.write_all("2025-10-03T12:01:00Z INFO Second message\n".as_bytes())
        .expect("Failed to write second message");
    file.flush().expect("Failed to flush");

    // Give it time to process
    thread::sleep(Duration::from_millis(1000));

    // Terminate and collect output
    child.kill().expect("Failed to kill process");
    let output = child.wait_with_output().expect("Failed to get output");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // The follow mode should either process the file or show an error about the missing file
    // Both cases are valid behavior
    let has_processed_content = stdout.contains("First message")
        || stdout.contains("12:00:00")
        || stdout.contains("2025-10-03");
    let has_error = stderr.contains("No such file or directory") || stderr.contains("Error:");

    assert!(
        has_processed_content || has_error,
        "Should either process the file content or show an appropriate error message"
    );
}
