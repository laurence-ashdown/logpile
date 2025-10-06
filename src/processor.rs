use crate::bucket::TimeBucket;
use crate::cli::{Args, OutputFormat};
use crate::output::{output_csv, output_json, output_table};
use crate::plot::{plot_ascii, plot_png};
use crate::reader::{create_readers, LogReader};
use crate::timestamp::TimestampParser;
use anyhow::Result;
use regex::Regex;
use std::thread;
use std::time::Duration as StdDuration;

pub struct LogProcessor {
    args: Args,
    patterns: Vec<Regex>,
    timestamp_parser: TimestampParser,
    bucket: TimeBucket,
}

impl LogProcessor {
    pub fn new(args: Args) -> Result<Self> {
        args.validate()?;

        let mut patterns = Vec::new();

        // Add primary pattern if provided (respecting --no-default-pattern)
        if let Some(pattern) = args.get_pattern() {
            patterns.push(Regex::new(pattern)?);
        }

        // Add additional grep patterns
        for pattern in &args.grep {
            patterns.push(Regex::new(pattern)?);
        }

        let timestamp_parser = TimestampParser::new(args.time_format.clone());
        let bucket = TimeBucket::new(args.bucket.clone())?;

        Ok(Self {
            args,
            patterns,
            timestamp_parser,
            bucket,
        })
    }

    pub fn run(&mut self) -> Result<()> {
        if self.args.follow {
            self.run_follow_mode()
        } else {
            self.run_batch_mode()
        }
    }

    fn run_batch_mode(&mut self) -> Result<()> {
        let files = self.args.get_files();
        let readers = create_readers(&files)?;
        let mut total_files_processed = 0;
        let mut files_with_matches = 0;

        for (source, mut reader) in readers {
            if let Some(ref src) = source {
                if self.args.verbose {
                    eprintln!("Processing: {}", src);
                }
            }

            total_files_processed += 1;
            let mut lines_processed = 0;
            let mut matching_lines_processed = 0;
            let mut timestamp_found = false;
            let mut first_timestamp_failure = None;
            let mut first_matching_line = None;

            for line_result in reader.lines() {
                let line = line_result?;
                lines_processed += 1;

                if self.matches_patterns(&line) {
                    matching_lines_processed += 1;

                    // Track the first matching line for early exit
                    if first_matching_line.is_none() {
                        first_matching_line = Some(line.clone());
                    }

                    // Try to extract timestamp
                    if let Some(timestamp) = self.timestamp_parser.parse_line(&line) {
                        self.bucket.add(timestamp);
                        timestamp_found = true;
                    } else {
                        // Track the first timestamp failure for early exit
                        if first_timestamp_failure.is_none() {
                            first_timestamp_failure = Some(line.clone());
                        }

                        // If we've processed more than 10 matching lines and still no timestamp found,
                        // and we're not in a custom format mode, fail early
                        if matching_lines_processed > 10
                            && !timestamp_found
                            && self.args.time_format.is_none()
                        {
                            if self.args.fail_quick {
                                eprintln!(
                                    "Error: No valid timestamps found in first {} matching lines. First failure: {}",
                                    matching_lines_processed,
                                    &first_timestamp_failure.unwrap().chars().take(80).collect::<String>()
                                );
                                eprintln!("Use --time-format to specify a custom timestamp format, or check if your log file has timestamps.");
                                anyhow::bail!("No valid timestamps detected in log file");
                            } else {
                                if self.args.verbose {
                                    eprintln!(
                                        "Warning: No valid timestamps found in first {} matching lines in {}",
                                        matching_lines_processed,
                                        source.as_ref().unwrap_or(&"<stdin>".to_string())
                                    );
                                }
                                // Continue to next file instead of failing
                                break;
                            }
                        }

                        if self.args.verbose {
                            eprintln!(
                                "Warning: Could not parse timestamp from: {}",
                                &line.chars().take(80).collect::<String>()
                            );
                        }
                    }
                }
            }

            // Handle files with no matching lines
            if lines_processed > 0 && matching_lines_processed == 0 {
                if self.args.fail_quick {
                    eprintln!(
                        "No lines matched the search pattern in {} lines processed",
                        lines_processed
                    );
                    eprintln!("Try a different search pattern or check if your log file contains the expected content.");
                    anyhow::bail!("No matching lines found in log file");
                } else {
                    if self.args.verbose {
                        eprintln!(
                            "No lines matched the search pattern in {} ({} lines processed)",
                            source.as_ref().unwrap_or(&"<stdin>".to_string()),
                            lines_processed
                        );
                    }
                    // Continue to next file instead of failing
                    continue;
                }
            }

            // Handle files with matching lines but no timestamps
            if matching_lines_processed > 0 && !timestamp_found && self.args.time_format.is_none() {
                if self.args.fail_quick {
                    eprintln!(
                        "Error: No valid timestamps found in {} matching lines",
                        matching_lines_processed
                    );
                    eprintln!("Use --time-format to specify a custom timestamp format, or check if your log file has timestamps.");
                    anyhow::bail!("No valid timestamps detected in log file");
                } else {
                    if self.args.verbose {
                        eprintln!(
                            "Warning: No valid timestamps found in {} matching lines in {}",
                            matching_lines_processed,
                            source.as_ref().unwrap_or(&"<stdin>".to_string())
                        );
                    }
                    // Continue to next file instead of failing
                    continue;
                }
            }

            // Track files that had matches
            if matching_lines_processed > 0 {
                files_with_matches += 1;
            }
        }

        // Check if any files had matches
        if total_files_processed > 0 && files_with_matches == 0 {
            eprintln!(
                "No matches found in any of the {} files processed",
                total_files_processed
            );
            if !self.args.fail_quick {
                eprintln!("Use --fail-quick to exit immediately when no matches are found");
            }
            anyhow::bail!("No matches found in any files");
        }

        self.output_results()
    }

    fn run_follow_mode(&mut self) -> Result<()> {
        if self.args.files.is_empty() {
            // Follow mode with stdin
            eprintln!("Following: stdin (press Ctrl+C to stop)");
            self.run_follow_stdin()
        } else if self.args.files.len() > 1 {
            anyhow::bail!("Follow mode only supports a single file");
        } else {
            // Follow mode with file
            let file_path = self.args.files[0].clone();
            eprintln!("Following: {} (press Ctrl+C to stop)", file_path);
            self.run_follow_file(&file_path)
        }
    }

    fn run_follow_stdin(&mut self) -> Result<()> {
        use std::io::{self, BufRead};
        use std::time::{Duration, Instant};

        let stdin = io::stdin();
        let handle = stdin.lock();
        let mut last_display = Instant::now();
        let display_interval = Duration::from_secs(1);

        for line_result in handle.lines() {
            let line = line_result?;
            if self.matches_patterns(&line) {
                if let Some(timestamp) = self.timestamp_parser.parse_line(&line) {
                    self.bucket.add(timestamp);
                } else if self.args.verbose {
                    eprintln!(
                        "Warning: Could not parse timestamp from: {}",
                        &line.chars().take(80).collect::<String>()
                    );
                }
            }

            // Only refresh display every 1 second
            if last_display.elapsed() >= display_interval {
                self.display_follow_results()?;
                last_display = Instant::now();
            }
        }

        // Final display
        self.display_follow_results()?;
        Ok(())
    }

    fn run_follow_file(&mut self, file_path: &str) -> Result<()> {
        // Track file position to only read new lines
        let mut last_position = 0u64;

        // Initial read
        let mut reader = LogReader::new(Some(file_path))?;
        let lines = reader.lines();

        for line_result in lines {
            let line = line_result?;
            last_position += 1;
            if self.matches_patterns(&line) {
                if let Some(timestamp) = self.timestamp_parser.parse_line(&line) {
                    self.bucket.add(timestamp);
                } else if self.args.verbose {
                    eprintln!(
                        "Warning: Could not parse timestamp from: {}",
                        &line.chars().take(80).collect::<String>()
                    );
                }
            }
        }

        // Show initial results
        self.display_follow_results()?;

        // For a real tail -f implementation, we'd need to use inotify or similar
        // For simplicity, we'll poll the file
        loop {
            thread::sleep(StdDuration::from_secs(1));

            let mut reader = LogReader::new(Some(file_path))?;
            let mut lines = reader.lines();

            // Skip to the last position we read
            for _ in 0..last_position {
                if lines.next().is_none() {
                    break;
                }
            }

            let mut new_lines_found = false;
            for line_result in lines {
                let line = line_result?;
                last_position += 1;
                new_lines_found = true;

                if self.matches_patterns(&line) {
                    if let Some(timestamp) = self.timestamp_parser.parse_line(&line) {
                        self.bucket.add(timestamp);
                    } else if self.args.verbose {
                        eprintln!(
                            "Warning: Could not parse timestamp from: {}",
                            &line.chars().take(80).collect::<String>()
                        );
                    }
                }
            }

            // Only update display if new lines were found
            if new_lines_found {
                self.display_follow_results()?;
            }
        }
    }

    fn display_follow_results(&self) -> Result<()> {
        let buckets = self.bucket.get_buckets();

        match self.args.output_format() {
            OutputFormat::Table => {
                print!("\x1B[2J\x1B[1;1H"); // Clear screen
                let bucket_size = self.bucket.bucket_size_seconds();
                let _ = output_table(&buckets, bucket_size);
            }
            OutputFormat::Csv => {
                // For CSV in follow mode, we need to clear and rewrite
                print!("\x1B[2J\x1B[1;1H"); // Clear screen
                let _ = output_csv(&buckets, self.args.no_headers);
            }
            OutputFormat::Json => {
                print!("\x1B[2J\x1B[1;1H"); // Clear screen
                let bucket_size = self.bucket.bucket_size_seconds();
                let time_range = self.bucket.time_range();
                let _ = output_json(&buckets, bucket_size, time_range);
            }
            OutputFormat::AsciiPlot => {
                print!("\x1B[2J\x1B[1;1H"); // Clear screen
                let time_range = self.bucket.time_range();
                let bucket_size = self.bucket.bucket_size_seconds();
                let pattern = self.args.get_pattern().unwrap_or("(no pattern)");
                let files = &self.args.files;
                let _ = plot_ascii(
                    &buckets,
                    time_range,
                    bucket_size,
                    pattern,
                    files,
                    self.args.y_zero,
                );
            }
            OutputFormat::Png => {
                // PNG in follow mode doesn't make much sense, but handle it
                if let Some(ref png_file) = self.args.png {
                    let _ = plot_png(&buckets, png_file);
                }
            }
        }

        Ok(())
    }

    fn matches_patterns(&self, line: &str) -> bool {
        if self.patterns.is_empty() {
            // No patterns means match everything (when --no-default-pattern is used)
            true
        } else {
            // Line must match at least one pattern
            self.patterns.iter().any(|p| p.is_match(line))
        }
    }

    fn output_results(&self) -> Result<()> {
        let buckets = self.bucket.get_buckets();
        let bucket_size = self.bucket.bucket_size_seconds();
        let time_range = self.bucket.time_range();

        match self.args.output_format() {
            OutputFormat::Table => output_table(&buckets, bucket_size),
            OutputFormat::Csv => output_csv(&buckets, self.args.no_headers),
            OutputFormat::Json => output_json(&buckets, bucket_size, time_range),
            OutputFormat::AsciiPlot => {
                let time_range = self.bucket.time_range();
                let bucket_size = self.bucket.bucket_size_seconds();
                let pattern = self.args.get_pattern().unwrap_or("(no pattern)");
                let files = &self.args.files;
                plot_ascii(
                    &buckets,
                    time_range,
                    bucket_size,
                    pattern,
                    files,
                    self.args.y_zero,
                )
            }
            OutputFormat::Png => {
                if let Some(ref png_file) = self.args.png {
                    plot_png(&buckets, png_file)
                } else {
                    anyhow::bail!("PNG output requires --png <file> argument")
                }
            }
        }
    }
}
