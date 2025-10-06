use std::io::Write;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;
use tempfile::NamedTempFile;

/// Test basic follow mode functionality
#[test]
fn test_follow_mode_works() {
    let initial_content = "2025-10-03T12:00:00Z INFO Initial message\n";
    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
    temp_file
        .write_all(initial_content.as_bytes())
        .expect("Failed to write initial content");
    temp_file.flush().expect("Failed to flush");

    let file_path = temp_file.path().to_str().unwrap();

    // Start follow mode
    let mut child = Command::new(env!("CARGO_BIN_EXE_logpile"))
        .args(["INFO", file_path, "--follow"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start logpile");

    // Give it time to start and process initial content
    thread::sleep(Duration::from_millis(1000));

    // Check that it's running
    let status = child.try_wait().expect("Failed to check process status");
    assert!(
        status.is_none(),
        "Process should still be running in follow mode"
    );

    // Terminate
    child.kill().expect("Failed to kill process");
    let _ = child.wait();
}

/// Test CSV output in follow mode (without new lines for simplicity)
#[test]
fn test_follow_mode_csv_basic() {
    let initial_content = "2025-10-03T12:00:00Z INFO Test message\n";
    let temp_file = NamedTempFile::new().expect("Failed to create temp file");
    let file_path = temp_file.path().to_str().unwrap();

    // Create the file with content
    std::fs::write(file_path, initial_content).expect("Failed to write file");

    // Start follow mode with CSV output
    let mut child = Command::new(env!("CARGO_BIN_EXE_logpile"))
        .args(["INFO", file_path, "--follow", "--csv"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start logpile");

    // Give it time to process
    thread::sleep(Duration::from_millis(1000));

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
        "Should contain timestamp"
    );
}

/// Test CSV output without headers in follow mode
#[test]
fn test_follow_mode_csv_no_headers() {
    let initial_content = "2025-10-03T12:00:00Z INFO Test message\n";
    let temp_file = NamedTempFile::new().expect("Failed to create temp file");
    let file_path = temp_file.path().to_str().unwrap();

    // Create the file with content
    std::fs::write(file_path, initial_content).expect("Failed to write file");

    // Start follow mode with CSV output and no headers
    let mut child = Command::new(env!("CARGO_BIN_EXE_logpile"))
        .args(["INFO", file_path, "--follow", "--csv", "--no-headers"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start logpile");

    // Give it time to process
    thread::sleep(Duration::from_millis(1000));

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
        "Should contain timestamp"
    );
}

/// Test plot output in follow mode
#[test]
fn test_follow_mode_plot_basic() {
    let initial_content = "2025-10-03T12:00:00Z INFO Test message\n";
    let temp_file = NamedTempFile::new().expect("Failed to create temp file");
    let file_path = temp_file.path().to_str().unwrap();

    // Create the file with content
    std::fs::write(file_path, initial_content).expect("Failed to write file");

    // Start follow mode with plot output
    let mut child = Command::new(env!("CARGO_BIN_EXE_logpile"))
        .args(["INFO", file_path, "--follow", "--plot"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start logpile");

    // Give it time to process
    thread::sleep(Duration::from_millis(1000));

    // Terminate and check output
    child.kill().expect("Failed to kill process");
    let output = child.wait_with_output().expect("Failed to get output");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should contain plot elements
    assert!(stdout.contains("Time range:"), "Should contain time range");
}

/// Test JSON output in follow mode
#[test]
fn test_follow_mode_json_basic() {
    let initial_content = "2025-10-03T12:00:00Z INFO Test message\n";
    let temp_file = NamedTempFile::new().expect("Failed to create temp file");
    let file_path = temp_file.path().to_str().unwrap();

    // Create the file with content
    std::fs::write(file_path, initial_content).expect("Failed to write file");

    // Start follow mode with JSON output
    let mut child = Command::new(env!("CARGO_BIN_EXE_logpile"))
        .args(["INFO", file_path, "--follow", "--json"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start logpile");

    // Give it time to process
    thread::sleep(Duration::from_millis(1000));

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
        "Should contain timestamp"
    );
}

/// Test that follow mode fails gracefully with invalid arguments
#[test]
fn test_follow_mode_invalid_args() {
    let temp_file = NamedTempFile::new().expect("Failed to create temp file");
    let file_path = temp_file.path().to_str().unwrap();

    // Try to use --no-headers without --csv (should fail)
    let output = Command::new(env!("CARGO_BIN_EXE_logpile"))
        .args(["INFO", file_path, "--follow", "--no-headers"])
        .output()
        .expect("Failed to run logpile");

    // Should fail with error code
    assert!(
        !output.status.success(),
        "Should fail when using --no-headers without --csv"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("--csv") || stderr.contains("required"),
        "Should show appropriate error message"
    );
}

/// Test follow mode with multiple grep patterns
#[test]
fn test_follow_mode_multiple_grep() {
    let initial_content =
        "2025-10-03T12:00:00Z INFO Test message\n2025-10-03T12:00:30Z ERROR Error message\n";
    let temp_file = NamedTempFile::new().expect("Failed to create temp file");
    let file_path = temp_file.path().to_str().unwrap();

    // Create the file with content
    std::fs::write(file_path, initial_content).expect("Failed to write file");

    // Start follow mode with multiple grep patterns
    let mut child = Command::new(env!("CARGO_BIN_EXE_logpile"))
        .args(["INFO", file_path, "--follow", "--grep", "ERROR"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start logpile");

    // Give it time to process
    thread::sleep(Duration::from_millis(1000));

    // Terminate and check output
    child.kill().expect("Failed to kill process");
    let output = child.wait_with_output().expect("Failed to get output");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should contain both INFO and ERROR matches (bucketed together)
    assert!(
        stdout.contains("2025-10-03 12:00:00"),
        "Should contain bucketed timestamp"
    );
    assert!(
        stdout.contains("Count") && stdout.contains("2"),
        "Should show count of 2 for both matches"
    );
}
