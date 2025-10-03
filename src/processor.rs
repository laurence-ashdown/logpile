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

        for (source, mut reader) in readers {
            if let Some(ref src) = source {
                eprintln!("Processing: {}", src);
            }

            for line_result in reader.lines() {
                let line = line_result?;

                if self.matches_patterns(&line) {
                    // Try to extract timestamp
                    if let Some(timestamp) = self.timestamp_parser.parse_line(&line) {
                        self.bucket.add(timestamp);
                    } else {
                        // If no timestamp found, we might want to count it anyway
                        // For now, we skip lines without timestamps when bucketing
                        eprintln!(
                            "Warning: Could not parse timestamp from: {}",
                            &line.chars().take(80).collect::<String>()
                        );
                    }
                }
            }
        }

        self.output_results()
    }

    fn run_follow_mode(&mut self) -> Result<()> {
        if self.args.files.is_empty() {
            anyhow::bail!("Follow mode requires at least one file argument");
        }

        if self.args.files.len() > 1 {
            anyhow::bail!("Follow mode only supports a single file");
        }

        let file_path = &self.args.files[0];

        eprintln!("Following: {} (press Ctrl+C to stop)", file_path);

        // Initial read
        let mut reader = LogReader::new(Some(file_path))?;
        for line_result in reader.lines() {
            let line = line_result?;
            if self.matches_patterns(&line) {
                if let Some(timestamp) = self.timestamp_parser.parse_line(&line) {
                    self.bucket.add(timestamp);
                }
            }
        }

        // For a real tail -f implementation, we'd need to use inotify or similar
        // For simplicity, we'll poll the file
        loop {
            thread::sleep(StdDuration::from_secs(1));

            let mut reader = LogReader::new(Some(file_path))?;
            for line_result in reader.lines() {
                let line = line_result?;
                if self.matches_patterns(&line) {
                    if let Some(timestamp) = self.timestamp_parser.parse_line(&line) {
                        self.bucket.add(timestamp);
                    }
                }
            }

            // Clear screen and show updated results
            if self.args.output_format() == OutputFormat::AsciiPlot {
                print!("\x1B[2J\x1B[1;1H"); // Clear screen
                let buckets = self.bucket.get_buckets();
                let _ = plot_ascii(&buckets);
            }
        }
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
            OutputFormat::Csv => output_csv(&buckets),
            OutputFormat::Json => output_json(&buckets, bucket_size, time_range),
            OutputFormat::AsciiPlot => plot_ascii(&buckets),
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
