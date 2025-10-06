use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(name = "logpile")]
#[command(about = "Search logs by regex, bucket matches by time, and output summaries")]
#[command(
    long_about = "A fast CLI tool to search logs by regex, bucket matches by time, and visualize patterns with charts.

Default output format is a human-readable table. Use output options (-c, -j, -p, -o) to change format."
)]
pub struct Args {
    /// Regex pattern to search for (required unless --no-default-pattern is set)
    #[arg(
        value_name = "REGEX",
        help = "Regular expression to match in log lines"
    )]
    pub pattern: Option<String>,

    /// Log files to search (supports .gz files). If no files provided, reads from stdin.
    #[arg(
        value_name = "FILES",
        help = "Log files to process (supports glob patterns and .gz files). If omitted, reads from stdin."
    )]
    pub files: Vec<String>,

    // === OUTPUT OPTIONS ===
    /// Output as CSV
    #[arg(long, short = 'c', conflicts_with_all = &["json", "plot", "png"], help = "Output results in CSV format")]
    pub csv: bool,

    /// Exclude headers from CSV output
    #[arg(long, requires = "csv", conflicts_with_all = &["json", "plot", "png"], help = "Exclude column headers from CSV output")]
    pub no_headers: bool,

    /// Output as JSON
    #[arg(long, short = 'j', conflicts_with_all = &["csv", "plot", "png"], help = "Output results in JSON format")]
    pub json: bool,

    /// Output as ASCII chart
    #[arg(long, short = 'p', conflicts_with_all = &["csv", "json", "png"], help = "Display results as ASCII chart")]
    pub plot: bool,

    /// Start Y-axis at zero (only applies to ASCII plots)
    #[arg(long, requires = "plot", help = "Start Y-axis at zero in ASCII plots")]
    pub y_zero: bool,

    /// Output as PNG chart to the specified file
    #[arg(long, short = 'o', value_name = "FILE", conflicts_with_all = &["csv", "json", "plot"], help = "Save chart as PNG file")]
    pub png: Option<String>,

    // === PROCESSING OPTIONS ===
    /// Time format string (chrono-compatible). If not provided, auto-detects.
    #[arg(
        long,
        short = 't',
        value_name = "FMT",
        help = "Custom timestamp format (e.g., \"%Y-%m-%d %H:%M:%S\")"
    )]
    pub time_format: Option<String>,

    /// Bucket size in seconds, or "auto" for automatic selection
    #[arg(
        long,
        short = 'b',
        value_name = "SECONDS",
        help = "Time bucket size in seconds, or \"auto\" for automatic"
    )]
    pub bucket: Option<String>,

    /// Additional regex patterns to filter (can be used multiple times)
    #[arg(
        long,
        short = 'g',
        value_name = "REGEX",
        help = "Additional regex patterns to match"
    )]
    pub grep: Vec<String>,

    /// Run without a required positional regex (count all lines)
    #[arg(
        long,
        short = 'n',
        help = "Process all lines without requiring a search pattern"
    )]
    pub no_default_pattern: bool,

    // === BEHAVIOR OPTIONS ===
    /// Streaming mode (like tail -f) with live updates
    #[arg(
        long,
        short = 'f',
        help = "Follow log file and update display in real-time"
    )]
    pub follow: bool,

    /// Enable verbose output (show warnings and debug info)
    #[arg(long, short = 'v', help = "Enable verbose output with warnings")]
    pub verbose: bool,

    /// Fail fast if any file has no matches
    #[arg(
        long,
        short = 'q',
        help = "Exit immediately if any file has no matching lines"
    )]
    pub fail_quick: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OutputFormat {
    Table,
    Csv,
    Json,
    AsciiPlot,
    Png,
}

impl Args {
    pub fn output_format(&self) -> OutputFormat {
        if self.csv {
            OutputFormat::Csv
        } else if self.json {
            OutputFormat::Json
        } else if self.plot {
            OutputFormat::AsciiPlot
        } else if self.png.is_some() {
            OutputFormat::Png
        } else {
            OutputFormat::Table
        }
    }

    pub fn validate(&self) -> anyhow::Result<()> {
        if self.pattern.is_none() && !self.no_default_pattern {
            anyhow::bail!("REGEX pattern is required unless --no-default-pattern is set");
        }
        Ok(())
    }

    /// Get the actual pattern to use (None if --no-default-pattern)
    pub fn get_pattern(&self) -> Option<&str> {
        if self.no_default_pattern {
            None
        } else {
            self.pattern.as_deref()
        }
    }

    /// Get the list of files, including pattern as first file if --no-default-pattern was used
    pub fn get_files(&self) -> Vec<String> {
        if self.no_default_pattern && self.pattern.is_some() {
            // When --no-default-pattern is set, treat the pattern positional arg as a file
            let mut files = vec![self.pattern.clone().unwrap()];
            files.extend(self.files.clone());
            files
        } else {
            self.files.clone()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_format_detection() {
        let args = Args {
            pattern: Some("test".to_string()),
            files: vec![],
            time_format: None,
            bucket: None,
            csv: false,
            no_headers: false,
            json: false,
            plot: false,
            y_zero: false,
            png: None,
            follow: false,
            grep: vec![],
            no_default_pattern: false,
            verbose: false,
            fail_quick: false,
        };
        assert_eq!(args.output_format(), OutputFormat::Table);

        let args_csv = Args {
            csv: true,
            ..args.clone()
        };
        assert_eq!(args_csv.output_format(), OutputFormat::Csv);

        let args_json = Args {
            json: true,
            ..args.clone()
        };
        assert_eq!(args_json.output_format(), OutputFormat::Json);

        let args_plot = Args {
            plot: true,
            y_zero: false,
            ..args.clone()
        };
        assert_eq!(args_plot.output_format(), OutputFormat::AsciiPlot);

        let args_png = Args {
            png: Some("out.png".to_string()),
            ..args.clone()
        };
        assert_eq!(args_png.output_format(), OutputFormat::Png);
    }

    #[test]
    fn test_validate_pattern_required() {
        let args = Args {
            pattern: None,
            files: vec![],
            time_format: None,
            bucket: None,
            csv: false,
            no_headers: false,
            json: false,
            plot: false,
            y_zero: false,
            png: None,
            follow: false,
            grep: vec![],
            no_default_pattern: false,
            verbose: false,
            fail_quick: false,
        };
        assert!(args.validate().is_err());

        let args_valid = Args {
            pattern: Some("test".to_string()),
            ..args.clone()
        };
        assert!(args_valid.validate().is_ok());

        let args_no_pattern = Args {
            no_default_pattern: true,
            ..args
        };
        assert!(args_no_pattern.validate().is_ok());
    }

    #[test]
    fn test_get_pattern() {
        let args = Args {
            pattern: Some("test".to_string()),
            files: vec![],
            time_format: None,
            bucket: None,
            csv: false,
            no_headers: false,
            json: false,
            plot: false,
            y_zero: false,
            png: None,
            follow: false,
            grep: vec![],
            no_default_pattern: false,
            verbose: false,
            fail_quick: false,
        };
        assert_eq!(args.get_pattern(), Some("test"));

        let args_no_pattern = Args {
            no_default_pattern: true,
            ..args
        };
        assert_eq!(args_no_pattern.get_pattern(), None);
    }

    #[test]
    fn test_get_files_normal() {
        let args = Args {
            pattern: Some("ERROR".to_string()),
            files: vec!["file1.log".to_string(), "file2.log".to_string()],
            time_format: None,
            bucket: None,
            csv: false,
            no_headers: false,
            json: false,
            plot: false,
            y_zero: false,
            png: None,
            follow: false,
            grep: vec![],
            no_default_pattern: false,
            verbose: false,
            fail_quick: false,
        };

        let files = args.get_files();
        assert_eq!(files.len(), 2);
        assert_eq!(files[0], "file1.log");
        assert_eq!(files[1], "file2.log");
    }

    #[test]
    fn test_get_files_no_default_pattern() {
        let args = Args {
            pattern: Some("myfile.log".to_string()),
            files: vec!["file2.log".to_string()],
            time_format: None,
            bucket: None,
            csv: false,
            no_headers: false,
            json: false,
            plot: false,
            y_zero: false,
            png: None,
            follow: false,
            grep: vec![],
            no_default_pattern: true,
            verbose: false,
            fail_quick: false,
        };

        let files = args.get_files();
        assert_eq!(files.len(), 2);
        assert_eq!(files[0], "myfile.log");
        assert_eq!(files[1], "file2.log");
    }

    #[test]
    fn test_get_files_empty() {
        let args = Args {
            pattern: Some("ERROR".to_string()),
            files: vec![],
            time_format: None,
            bucket: None,
            csv: false,
            no_headers: false,
            json: false,
            plot: false,
            y_zero: false,
            png: None,
            follow: false,
            grep: vec![],
            no_default_pattern: false,
            verbose: false,
            fail_quick: false,
        };

        assert_eq!(args.get_files().len(), 0);
    }
}
