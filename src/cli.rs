use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(name = "logpile")]
#[command(about = "Search logs by regex, bucket matches by time, and output summaries", long_about = None)]
pub struct Args {
    /// Regex pattern to search for (required unless --no-default-pattern is set)
    #[arg(value_name = "REGEX")]
    pub pattern: Option<String>,

    /// Log files to search (supports .gz files). If empty, reads from stdin.
    #[arg(value_name = "FILES")]
    pub files: Vec<String>,

    /// Time format string (chrono-compatible). If not provided, auto-detects.
    #[arg(long, value_name = "FMT")]
    pub time_format: Option<String>,

    /// Bucket size in seconds, or "auto" for automatic selection
    #[arg(long, value_name = "SECONDS")]
    pub bucket: Option<String>,

    /// Output as CSV
    #[arg(long, conflicts_with_all = &["json", "plot", "png"])]
    pub csv: bool,

    /// Output as JSON
    #[arg(long, conflicts_with_all = &["csv", "plot", "png"])]
    pub json: bool,

    /// Output as ASCII chart
    #[arg(long, conflicts_with_all = &["csv", "json", "png"])]
    pub plot: bool,

    /// Output as PNG chart to the specified file
    #[arg(long, value_name = "FILE", conflicts_with_all = &["csv", "json", "plot"])]
    pub png: Option<String>,

    /// Streaming mode (like tail -f) with live updates
    #[arg(long)]
    pub follow: bool,

    /// Additional regex patterns to filter (can be used multiple times)
    #[arg(long, value_name = "REGEX")]
    pub grep: Vec<String>,

    /// Run without a required positional regex (count all lines)
    #[arg(long)]
    pub no_default_pattern: bool,
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
            json: false,
            plot: false,
            png: None,
            follow: false,
            grep: vec![],
            no_default_pattern: false,
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
            json: false,
            plot: false,
            png: None,
            follow: false,
            grep: vec![],
            no_default_pattern: false,
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
            json: false,
            plot: false,
            png: None,
            follow: false,
            grep: vec![],
            no_default_pattern: false,
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
            json: false,
            plot: false,
            png: None,
            follow: false,
            grep: vec![],
            no_default_pattern: false,
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
            json: false,
            plot: false,
            png: None,
            follow: false,
            grep: vec![],
            no_default_pattern: true,
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
            json: false,
            plot: false,
            png: None,
            follow: false,
            grep: vec![],
            no_default_pattern: false,
        };

        assert_eq!(args.get_files().len(), 0);
    }
}
