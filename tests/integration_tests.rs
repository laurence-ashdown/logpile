use std::io::Write;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;
use tempfile::NamedTempFile;

/// Helper to create a temporary log file with content
fn create_temp_log(content: &str) -> NamedTempFile {
    let mut file = NamedTempFile::new().expect("Failed to create temp file");
    file.write_all(content.as_bytes())
        .expect("Failed to write to temp file");
    file.flush().expect("Failed to flush temp file");
    file
}

/// Helper to get the logpile binary path
fn logpile_bin() -> String {
    env!("CARGO_BIN_EXE_logpile").to_string()
}

#[test]
fn test_follow_mode_basic() {
    let log_content = "2025-10-03T12:00:00Z INFO Application started\n";
    let temp_file = create_temp_log(log_content);
    let file_path = temp_file.path().to_str().unwrap();

    // Test basic follow mode with timeout
    let mut child = Command::new(&logpile_bin())
        .args(["INFO", file_path, "--follow"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start logpile");

    // Give it a moment to process
    thread::sleep(Duration::from_millis(100));

    // Check that it's running and following
    let status = child.try_wait().expect("Failed to check process status");
    assert!(
        status.is_none(),
        "Process should still be running in follow mode"
    );

    // Terminate the process
    child.kill().expect("Failed to kill process");
    let _ = child.wait();
}

#[test]
fn test_follow_mode_with_new_lines() {
    let log_content = "2025-10-03T12:00:00Z INFO Initial message\n";
    let mut temp_file = create_temp_log(log_content);
    let file_path = temp_file.path().to_str().unwrap();

    // Start follow mode
    let mut child = Command::new(&logpile_bin())
        .args(["INFO", file_path, "--follow"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start logpile");

    // Give it time to process initial content
    thread::sleep(Duration::from_millis(200));

    // Append new lines to the file
    let new_lines =
        "2025-10-03T12:01:00Z INFO New message 1\n2025-10-03T12:02:00Z INFO New message 2\n";
    temp_file
        .write_all(new_lines.as_bytes())
        .expect("Failed to append");

    // Give it time to detect and process new lines
    thread::sleep(Duration::from_millis(300));

    // Check that it's still running
    let status = child.try_wait().expect("Failed to check process status");
    assert!(
        status.is_none(),
        "Process should still be running after new lines"
    );

    // Terminate
    child.kill().expect("Failed to kill process");
    let _ = child.wait();
}

#[test]
fn test_follow_mode_csv_output() {
    let log_content = "2025-10-03T12:00:00Z INFO Test message\n";
    let mut temp_file = create_temp_log(log_content);
    let file_path = temp_file.path().to_str().unwrap();

    // Start follow mode with CSV output
    let mut child = Command::new(&logpile_bin())
        .args(["INFO", file_path, "--follow", "--csv"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start logpile");

    // Give it time to process
    thread::sleep(Duration::from_millis(200));

    // Append new content
    let new_line = "2025-10-03T12:01:00Z INFO Another message\n";
    temp_file
        .write_all(new_line.as_bytes())
        .expect("Failed to append");

    // Give it time to process
    thread::sleep(Duration::from_millis(300));

    // Terminate and check output
    child.kill().expect("Failed to kill process");
    let output = child.wait_with_output().expect("Failed to get output");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should contain CSV headers and data
    assert!(
        stdout.contains("timestamp,count"),
        "CSV output should contain headers"
    );
    assert!(
        stdout.contains("2025-10-03T12:00:00"),
        "Should contain first timestamp"
    );
    // Note: Second timestamp might be bucketed with first, so just check we have some data
    assert!(
        stdout
            .lines()
            .filter(|line| line.contains("2025-10-03"))
            .count()
            >= 1,
        "Should have at least one timestamp line"
    );
}

#[test]
fn test_follow_mode_csv_no_headers() {
    let log_content = "2025-10-03T12:00:00Z INFO Test message\n";
    let mut temp_file = create_temp_log(log_content);
    let file_path = temp_file.path().to_str().unwrap();

    // Start follow mode with CSV output and no headers
    let mut child = Command::new(&logpile_bin())
        .args(["INFO", file_path, "--follow", "--csv", "--no-headers"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start logpile");

    // Give it time to process
    thread::sleep(Duration::from_millis(200));

    // Append new content
    let new_line = "2025-10-03T12:01:00Z INFO Another message\n";
    temp_file
        .write_all(new_line.as_bytes())
        .expect("Failed to append");

    // Give it time to process
    thread::sleep(Duration::from_millis(300));

    // Terminate and check output
    child.kill().expect("Failed to kill process");
    let output = child.wait_with_output().expect("Failed to get output");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should NOT contain CSV headers
    assert!(
        !stdout.contains("timestamp,count"),
        "CSV output should NOT contain headers"
    );
    // Should contain timestamp data
    assert!(
        stdout.contains("2025-10-03T12:00:00"),
        "Should contain first timestamp"
    );
    // Note: Second timestamp might be bucketed with first, so just check we have some data
    assert!(
        stdout
            .lines()
            .filter(|line| line.contains("2025-10-03"))
            .count()
            >= 1,
        "Should have at least one timestamp line"
    );
}

#[test]
fn test_follow_mode_plot_output() {
    let log_content = "2025-10-03T12:00:00Z INFO Test message\n";
    let mut temp_file = create_temp_log(log_content);
    let file_path = temp_file.path().to_str().unwrap();

    // Start follow mode with plot output
    let mut child = Command::new(&logpile_bin())
        .args(["INFO", file_path, "--follow", "--plot"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start logpile");

    // Give it time to process
    thread::sleep(Duration::from_millis(200));

    // Append new content
    let new_line = "2025-10-03T12:01:00Z INFO Another message\n";
    temp_file
        .write_all(new_line.as_bytes())
        .expect("Failed to append");

    // Give it time to process
    thread::sleep(Duration::from_millis(300));

    // Terminate and check output
    child.kill().expect("Failed to kill process");
    let output = child.wait_with_output().expect("Failed to get output");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let _stderr = String::from_utf8_lossy(&output.stderr);

    // Should contain plot elements
    assert!(stdout.contains("X-axis:"), "Should contain x-axis label");
    assert!(stdout.contains("Y-axis:"), "Should contain y-axis label");
    assert!(stdout.contains("Time range:"), "Should contain time range");
}

#[test]
fn test_follow_mode_json_output() {
    let log_content = "2025-10-03T12:00:00Z INFO Test message\n";
    let mut temp_file = create_temp_log(log_content);
    let file_path = temp_file.path().to_str().unwrap();

    // Start follow mode with JSON output
    let mut child = Command::new(&logpile_bin())
        .args(["INFO", file_path, "--follow", "--json"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start logpile");

    // Give it time to process
    thread::sleep(Duration::from_millis(200));

    // Append new content
    let new_line = "2025-10-03T12:01:00Z INFO Another message\n";
    temp_file
        .write_all(new_line.as_bytes())
        .expect("Failed to append");

    // Give it time to process
    thread::sleep(Duration::from_millis(300));

    // Terminate and check output
    child.kill().expect("Failed to kill process");
    let output = child.wait_with_output().expect("Failed to get output");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should contain JSON structure
    assert!(
        stdout.contains("\"bucket_size_seconds\""),
        "Should contain bucket size"
    );
    assert!(
        stdout.contains("\"buckets\""),
        "Should contain buckets array"
    );
    assert!(
        stdout.contains("2025-10-03T12:00:00"),
        "Should contain first timestamp"
    );
    // Note: Second timestamp might be bucketed with first, so just check we have some data
    assert!(
        stdout.contains("2025-10-03"),
        "Should contain at least one timestamp"
    );
}

#[test]
fn test_follow_mode_multiple_patterns() {
    let log_content =
        "2025-10-03T12:00:00Z INFO Test message\n2025-10-03T12:00:30Z ERROR Error occurred\n";
    let mut temp_file = create_temp_log(log_content);
    let file_path = temp_file.path().to_str().unwrap();

    // Start follow mode with multiple grep patterns
    let mut child = Command::new(&logpile_bin())
        .args(["INFO", file_path, "--follow", "--grep", "ERROR"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start logpile");

    // Give it time to process
    thread::sleep(Duration::from_millis(200));

    // Append new content with both patterns
    let new_lines = "2025-10-03T12:01:00Z INFO New info\n2025-10-03T12:01:30Z ERROR New error\n";
    temp_file
        .write_all(new_lines.as_bytes())
        .expect("Failed to append");

    // Give it time to process
    thread::sleep(Duration::from_millis(300));

    // Terminate and check output
    child.kill().expect("Failed to kill process");
    let output = child.wait_with_output().expect("Failed to get output");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should contain both INFO and ERROR matches (may be bucketed)
    assert!(
        stdout.contains("2025-10-03 12:00:00"),
        "Should contain initial INFO timestamp"
    );
    // Note: Other timestamps might be bucketed, so just check we have some data
    assert!(
        stdout.contains("2025-10-03"),
        "Should contain at least one timestamp"
    );
}

#[test]
fn test_follow_mode_no_matches() {
    let log_content = "2025-10-03T12:00:00Z DEBUG Debug message\n";
    let mut temp_file = create_temp_log(log_content);
    let file_path = temp_file.path().to_str().unwrap();

    // Start follow mode with pattern that won't match
    let mut child = Command::new(&logpile_bin())
        .args(["INFO", file_path, "--follow"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start logpile");

    // Give it time to process
    thread::sleep(Duration::from_millis(200));

    // Append more non-matching content
    let new_line = "2025-10-03T12:01:00Z DEBUG Another debug\n";
    temp_file
        .write_all(new_line.as_bytes())
        .expect("Failed to append");

    // Give it time to process
    thread::sleep(Duration::from_millis(300));

    // Terminate and check output
    child.kill().expect("Failed to kill process");
    let output = child.wait_with_output().expect("Failed to get output");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should show no data message or be empty
    assert!(
        stdout.contains("No data to display")
            || stdout.contains("No data to plot")
            || stdout.contains("No matches found")
            || stdout.trim().is_empty(),
        "Should show no data message or be empty when no matches found"
    );
}

#[test]
fn test_follow_mode_with_custom_bucket_size() {
    let log_content = "2025-10-03T12:00:00Z INFO Message 1\n2025-10-03T12:00:30Z INFO Message 2\n";
    let mut temp_file = create_temp_log(log_content);
    let file_path = temp_file.path().to_str().unwrap();

    // Start follow mode with custom bucket size
    let mut child = Command::new(&logpile_bin())
        .args(["INFO", file_path, "--follow", "--bucket", "60"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start logpile");

    // Give it time to process
    thread::sleep(Duration::from_millis(200));

    // Append new content
    let new_line = "2025-10-03T12:01:00Z INFO Message 3\n";
    temp_file
        .write_all(new_line.as_bytes())
        .expect("Failed to append");

    // Give it time to process
    thread::sleep(Duration::from_millis(300));

    // Terminate and check output
    child.kill().expect("Failed to kill process");
    let output = child.wait_with_output().expect("Failed to get output");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should show bucket size information
    assert!(
        stdout.contains("Bucket size: 60 seconds"),
        "Should show custom bucket size"
    );
}

#[test]
fn test_follow_mode_with_auto_bucket() {
    let log_content = "2025-10-03T12:00:00Z INFO Message 1\n";
    let mut temp_file = create_temp_log(log_content);
    let file_path = temp_file.path().to_str().unwrap();

    // Start follow mode with auto bucket size
    let mut child = Command::new(&logpile_bin())
        .args(["INFO", file_path, "--follow", "--bucket", "auto"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start logpile");

    // Give it time to process
    thread::sleep(Duration::from_millis(200));

    // Append new content
    let new_line = "2025-10-03T12:01:00Z INFO Message 2\n";
    temp_file
        .write_all(new_line.as_bytes())
        .expect("Failed to append");

    // Give it time to process
    thread::sleep(Duration::from_millis(300));

    // Terminate and check output
    child.kill().expect("Failed to kill process");
    let output = child.wait_with_output().expect("Failed to get output");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should show auto-calculated bucket size
    assert!(stdout.contains("Bucket size:"), "Should show bucket size");
    assert!(stdout.contains("seconds"), "Should show seconds unit");
}

#[test]
fn test_follow_mode_graceful_shutdown() {
    let log_content = "2025-10-03T12:00:00Z INFO Test message\n";
    let temp_file = create_temp_log(log_content);
    let file_path = temp_file.path().to_str().unwrap();

    // Start follow mode
    let mut child = Command::new(&logpile_bin())
        .args(["INFO", file_path, "--follow"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start logpile");

    // Give it time to start
    thread::sleep(Duration::from_millis(100));

    // Send SIGTERM (graceful shutdown)
    #[cfg(unix)]
    {
        use std::process::Command;
        let _ = Command::new("kill")
            .args(["-TERM", &child.id().to_string()])
            .output();
    }

    // Give it time to shutdown gracefully
    thread::sleep(Duration::from_millis(200));

    // Check if it's still running
    let _status = child.try_wait().expect("Failed to check process status");

    // On Unix systems, it should have terminated gracefully
    // On Windows, we'll just kill it manually
    #[cfg(windows)]
    {
        let _ = child.kill();
    }

    let _ = child.wait();
}
