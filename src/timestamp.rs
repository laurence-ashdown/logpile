use chrono::{DateTime, Datelike, NaiveDateTime, Utc};
use regex::Regex;

/// Common timestamp formats to auto-detect
const COMMON_FORMATS: &[&str] = &[
    // ISO 8601
    "%Y-%m-%dT%H:%M:%S%.fZ",
    "%Y-%m-%dT%H:%M:%S%.f%:z",
    "%Y-%m-%dT%H:%M:%S%:z",
    "%Y-%m-%dT%H:%M:%SZ",
    "%Y-%m-%dT%H:%M:%S%.f", // ISO 8601 without timezone
    "%Y-%m-%dT%H:%M:%S",    // ISO 8601 basic without timezone
    // Common log formats
    "%Y-%m-%d %H:%M:%S%.f",
    "%Y-%m-%d %H:%M:%S",
    "%Y/%m/%d %H:%M:%S",
    "%d/%m/%Y %H:%M:%S", // European date format
    "%m/%d/%Y %H:%M:%S", // US date format
    // Syslog format
    "%b %d %H:%M:%S",
    // Apache/Nginx
    "%d/%b/%Y:%H:%M:%S %z",
    // RFC 2822
    "%a, %d %b %Y %H:%M:%S",
];

/// Timestamp parser with auto-detection capabilities
pub struct TimestampParser {
    custom_format: Option<String>,
    // Compiled regex patterns for extracting timestamps
    iso_regex: Regex,
    datetime_regex: Regex,
    syslog_regex: Regex,
    apache_regex: Regex,
    rfc2822_regex: Regex,
    unix_timestamp_regex: Regex,
}

impl TimestampParser {
    pub fn new(custom_format: Option<String>) -> Self {
        Self {
            custom_format,
            iso_regex: Regex::new(
                r"\d{4}-\d{2}-\d{2}[T ]\d{2}:\d{2}:\d{2}(?:\.\d+)?(?:Z|[+-]\d{2}:\d{2})?",
            )
            .unwrap(),
            datetime_regex: Regex::new(
                r"\d{1,2}[-/]\d{1,2}[-/]\d{2,4}\s+\d{2}:\d{2}:\d{2}(?:\.\d+)?",
            )
            .unwrap(),
            syslog_regex: Regex::new(r"[A-Z][a-z]{2}\s+\d{1,2}\s+\d{2}:\d{2}:\d{2}").unwrap(),
            apache_regex: Regex::new(r"\d{2}/[A-Z][a-z]{2}/\d{4}:\d{2}:\d{2}:\d{2}\s+[+-]\d{4}")
                .unwrap(),
            rfc2822_regex: Regex::new(
                r"[A-Z][a-z]{2},\s+\d{2}\s+[A-Z][a-z]{2}\s+\d{4}\s+\d{2}:\d{2}:\d{2}",
            )
            .unwrap(),
            unix_timestamp_regex: Regex::new(r"^\d{10}(?:\.\d+)?").unwrap(),
        }
    }

    /// Extract and parse timestamp from a log line
    pub fn parse_line(&self, line: &str) -> Option<DateTime<Utc>> {
        // Try custom format first if provided
        if let Some(ref fmt) = self.custom_format {
            if let Some(ts) = self.parse_with_format(line, fmt) {
                return Some(ts);
            }
        }

        // Try to extract timestamp-like strings using regex
        let candidates = self.extract_timestamp_candidates(line);

        for candidate in candidates {
            // Try each common format
            for format in COMMON_FORMATS {
                if let Some(ts) = self.parse_with_format(&candidate, format) {
                    return Some(ts);
                }
            }
        }

        None
    }

    fn extract_timestamp_candidates(&self, line: &str) -> Vec<String> {
        let mut candidates = Vec::new();

        // Try Unix timestamp first (at start of line only)
        if let Some(mat) = self.unix_timestamp_regex.find(line) {
            if mat.start() < 5 {
                // Must be near start of line
                candidates.push(mat.as_str().to_string());
            }
        }

        // Try ISO format
        if let Some(mat) = self.iso_regex.find(line) {
            candidates.push(mat.as_str().to_string());
        }

        // Try Apache/Nginx format
        if let Some(mat) = self.apache_regex.find(line) {
            candidates.push(mat.as_str().to_string());
        }

        // Try RFC 2822 format
        if let Some(mat) = self.rfc2822_regex.find(line) {
            candidates.push(mat.as_str().to_string());
        }

        // Try datetime format (EU/US dates)
        if let Some(mat) = self.datetime_regex.find(line) {
            candidates.push(mat.as_str().to_string());
        }

        // Try syslog format
        if let Some(mat) = self.syslog_regex.find(line) {
            candidates.push(mat.as_str().to_string());
        }

        // Also try the first 50 chars as a fallback
        if candidates.is_empty() && line.len() >= 10 {
            candidates.push(line[..line.len().min(50)].to_string());
        }

        candidates
    }

    fn parse_with_format(&self, text: &str, format: &str) -> Option<DateTime<Utc>> {
        let trimmed = text.trim();

        // Try parsing Unix timestamp
        if let Ok(unix_ts) = trimmed.parse::<i64>() {
            if unix_ts > 1000000000 && unix_ts < 9999999999 {
                // Reasonable timestamp range
                return DateTime::from_timestamp(unix_ts, 0);
            }
        }

        // Try parsing as DateTime with timezone
        if let Ok(dt) = DateTime::parse_from_str(trimmed, format) {
            return Some(dt.with_timezone(&Utc));
        }

        // Try parsing as NaiveDateTime (no timezone)
        if let Ok(ndt) = NaiveDateTime::parse_from_str(trimmed, format) {
            return Some(DateTime::from_naive_utc_and_offset(ndt, Utc));
        }

        // For syslog format, we need to add the year
        if format.contains("%b") && !format.contains("%Y") {
            let current_year = Utc::now().year();
            let with_year = format!("{} {}", current_year, trimmed);
            let format_with_year = format!("%Y {}", format);
            if let Ok(ndt) = NaiveDateTime::parse_from_str(&with_year, &format_with_year) {
                return Some(DateTime::from_naive_utc_and_offset(ndt, Utc));
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Timelike;

    #[test]
    fn test_parse_iso8601_with_timezone() {
        let parser = TimestampParser::new(None);
        let line = "2025-10-03T14:30:45.123Z INFO: Application started";
        assert!(parser.parse_line(line).is_some());

        let line2 = "2025-10-03T14:30:45.123+00:00 INFO: Application started";
        assert!(parser.parse_line(line2).is_some());
    }

    #[test]
    fn test_parse_iso8601_without_timezone() {
        let parser = TimestampParser::new(None);
        let line = "2025-10-03T14:30:45 INFO: Application started";
        assert!(parser.parse_line(line).is_some());

        let line2 = "2025-10-03T14:30:45.123 DEBUG: Processing";
        assert!(parser.parse_line(line2).is_some());
    }

    #[test]
    fn test_parse_common_format() {
        let parser = TimestampParser::new(None);
        let line = "2025-10-03 14:30:45 ERROR: Connection failed";
        assert!(parser.parse_line(line).is_some());

        let line2 = "2025-10-03 14:30:45.123456 WARN: Slow query";
        assert!(parser.parse_line(line2).is_some());
    }

    #[test]
    fn test_parse_european_date() {
        let parser = TimestampParser::new(None);
        let line = "03/10/2025 14:30:45 INFO: User logged in";
        assert!(parser.parse_line(line).is_some());
    }

    #[test]
    fn test_parse_us_date() {
        let parser = TimestampParser::new(None);
        let line = "10/03/2025 14:30:45 INFO: Request processed";
        assert!(parser.parse_line(line).is_some());
    }

    #[test]
    fn test_parse_syslog_format() {
        let parser = TimestampParser::new(None);
        let line = "Oct 03 14:30:45 myserver app[1234]: ERROR: Connection lost";
        let result = parser.parse_line(line);
        assert!(result.is_some());
        // Should add current year
        let ts = result.unwrap();
        assert_eq!(ts.month(), 10);
        assert_eq!(ts.day(), 3);
    }

    #[test]
    fn test_parse_apache_format() {
        let parser = TimestampParser::new(None);
        let line = r#"192.168.1.1 - - [03/Oct/2025:14:30:45 +0000] "GET /api HTTP/1.1" 200 1234"#;
        assert!(parser.parse_line(line).is_some());
    }

    #[test]
    fn test_parse_rfc2822_format() {
        let parser = TimestampParser::new(None);
        let line = "Fri, 03 Oct 2025 14:30:45 GMT ERROR: Service unavailable";
        assert!(parser.parse_line(line).is_some());
    }

    #[test]
    fn test_parse_unix_timestamp() {
        let parser = TimestampParser::new(None);
        let line = "1727962496 INFO: Background job completed";
        let result = parser.parse_line(line);
        assert!(result.is_some());
    }

    #[test]
    fn test_parse_java_format() {
        let parser = TimestampParser::new(None);
        let line = "2025-10-03 14:30:45.123 ERROR [http-nio-8080-exec-1] com.example.Service - Request failed";
        assert!(parser.parse_line(line).is_some());
    }

    #[test]
    fn test_custom_format() {
        let parser = TimestampParser::new(Some("%Y/%m/%d %H:%M:%S".to_string()));
        let line = "2025/10/03 14:30:45 - Custom log entry";
        assert!(parser.parse_line(line).is_some());
    }

    #[test]
    fn test_no_timestamp() {
        let parser = TimestampParser::new(None);
        let line = "This line has no timestamp at all";
        assert!(parser.parse_line(line).is_none());
    }

    #[test]
    fn test_invalid_timestamp() {
        let parser = TimestampParser::new(None);
        let line = "99/99/9999 99:99:99 Invalid timestamp";
        assert!(parser.parse_line(line).is_none());
    }

    #[test]
    fn test_multiple_timestamps_uses_first() {
        let parser = TimestampParser::new(None);
        let line = "2025-10-03 14:30:45 Processing item created at 2025-10-03 12:00:00";
        let result = parser.parse_line(line);
        assert!(result.is_some());
        let ts = result.unwrap();
        assert_eq!(ts.hour(), 14);
        assert_eq!(ts.minute(), 30);
    }

    #[test]
    fn test_timestamp_extraction() {
        let parser = TimestampParser::new(None);

        // Test ISO regex
        assert!(parser.iso_regex.is_match("2025-10-03T14:30:45.123Z"));
        assert!(parser.iso_regex.is_match("2025-10-03 14:30:45"));

        // Test datetime regex
        assert!(parser.datetime_regex.is_match("03/10/2025 14:30:45"));
        assert!(parser.datetime_regex.is_match("10/03/2025 14:30:45"));

        // Test syslog regex
        assert!(parser.syslog_regex.is_match("Oct 03 14:30:45"));

        // Test apache regex
        assert!(parser.apache_regex.is_match("03/Oct/2025:14:30:45 +0000"));

        // Test RFC 2822 regex
        assert!(parser.rfc2822_regex.is_match("Fri, 03 Oct 2025 14:30:45"));

        // Test unix timestamp regex
        assert!(parser.unix_timestamp_regex.is_match("1727962496"));
    }

    #[test]
    fn test_parse_with_format_unix() {
        let parser = TimestampParser::new(None);
        let result = parser.parse_with_format("1727962496", "");
        assert!(result.is_some());
    }

    #[test]
    fn test_parse_with_format_invalid_unix() {
        let parser = TimestampParser::new(None);
        // Too small to be a valid Unix timestamp
        let result = parser.parse_with_format("123456", "");
        assert!(result.is_none());

        // Too large
        let result = parser.parse_with_format("99999999999", "");
        assert!(result.is_none());
    }

    #[test]
    fn test_extract_timestamp_candidates() {
        let parser = TimestampParser::new(None);

        let line = "2025-10-03T14:30:45.123Z INFO: Application started";
        let candidates = parser.extract_timestamp_candidates(line);
        assert!(!candidates.is_empty());

        let line_no_ts = "Just a line with no timestamp";
        let candidates = parser.extract_timestamp_candidates(line_no_ts);
        // Should still return fallback candidate
        assert!(!candidates.is_empty());
    }

    #[test]
    fn test_syslog_year_injection() {
        let parser = TimestampParser::new(None);
        let line = "Oct 03 14:30:45 myserver app: INFO message";
        let result = parser.parse_line(line);
        assert!(result.is_some());

        // Verify it uses current year
        let current_year = Utc::now().year();
        assert_eq!(result.unwrap().year(), current_year);
    }
}
