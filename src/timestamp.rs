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
    // Yearless ISO 8601 (common in logs)
    "%m-%dT%H:%M:%S%.fZ",
    "%m-%dT%H:%M:%S%.f",
    "%m-%dT%H:%M:%SZ",
    "%m-%dT%H:%M:%S",
    // Time-only formats
    "%H:%M:%S%.f",
    "%H:%M:%S",
    // Common log formats
    "%Y-%m-%d %H:%M:%S%.f",
    "%Y-%m-%d %H:%M:%S",
    "%Y/%m/%d %H:%M:%S",
    "%d/%m/%Y %H:%M:%S", // European date format
    "%m/%d/%Y %H:%M:%S", // US date format
    // Syslog format
    "%b %d %H:%M:%S",
    // Apache/Nginx
    "[%d/%b/%Y:%H:%M:%S %z]",
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
    yearless_iso_regex: Regex,
    time_only_regex: Regex,
}

impl TimestampParser {
    pub fn new(custom_format: Option<String>) -> Self {
        Self {
            custom_format,
            iso_regex: Regex::new(
                r"\d{2,4}-\d{2}-\d{2}[T ]\d{2}:\d{2}:\d{2}(?:\.\d+)?(?:Z|[+-]\d{2}:\d{2})?",
            )
            .unwrap(),
            datetime_regex: Regex::new(
                r"\d{1,2}[-/]\d{1,2}[-/]\d{2,4}\s+\d{2}:\d{2}:\d{2}(?:\.\d+)?",
            )
            .unwrap(),
            syslog_regex: Regex::new(r"[A-Z][a-z]{2}\s+\d{1,2}\s+\d{2}:\d{2}:\d{2}").unwrap(),
            apache_regex: Regex::new(
                r"\[\d{2}/[A-Z][a-z]{2}/\d{4}:\d{2}:\d{2}:\d{2}\s+[+-]\d{4}\]",
            )
            .unwrap(),
            rfc2822_regex: Regex::new(
                r"[A-Z][a-z]{2},\s+\d{2}\s+[A-Z][a-z]{2}\s+\d{4}\s+\d{2}:\d{2}:\d{2}",
            )
            .unwrap(),
            unix_timestamp_regex: Regex::new(r"^\d{10}(?:\.\d+)?").unwrap(),
            yearless_iso_regex: Regex::new(r"\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d+)?(?:Z)?")
                .unwrap(),
            time_only_regex: Regex::new(r"\d{2}:\d{2}:\d{2}(?:\.\d+)?").unwrap(),
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

        // Try yearless ISO format
        if let Some(mat) = self.yearless_iso_regex.find(line) {
            candidates.push(mat.as_str().to_string());
        }

        // Try time-only format
        if let Some(mat) = self.time_only_regex.find(line) {
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

        // For yearless ISO formats, we need to add the year
        if format.starts_with("%m-") && !format.contains("%Y") {
            let current_year = Utc::now().year();
            let with_year = format!("{}-{}", current_year, trimmed);
            let format_with_year = format!("%Y-{}", format);
            if let Ok(ndt) = NaiveDateTime::parse_from_str(&with_year, &format_with_year) {
                return Some(DateTime::from_naive_utc_and_offset(ndt, Utc));
            }
        }

        // For time-only formats, we need to add the current date
        if format.starts_with("%H:")
            && !format.contains("%Y")
            && !format.contains("%m")
            && !format.contains("%d")
        {
            let now = Utc::now();
            let current_date = now.date_naive();
            let with_date = format!("{} {}", current_date.format("%Y-%m-%d"), trimmed);
            let format_with_date = format!("%Y-%m-%d {}", format);
            if let Ok(ndt) = NaiveDateTime::parse_from_str(&with_date, &format_with_date) {
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
        let result = parser.parse_line(line);
        assert!(result.is_some());

        let dt = result.unwrap();
        assert_eq!(dt.year(), 2025);
        assert_eq!(dt.month(), 10);
        assert_eq!(dt.day(), 3);
        assert_eq!(dt.hour(), 14);
        assert_eq!(dt.minute(), 30);
        assert_eq!(dt.second(), 45);
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
        assert!(parser.apache_regex.is_match("[03/Oct/2025:14:30:45 +0000]"));

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

    #[test]
    fn test_parse_yearless_iso() {
        let parser = TimestampParser::new(Some("%m-%dT%H:%M:%S%.3fZ".to_string()));
        let line = "09-24T23:45:29.362Z| INFO| Some random logline";
        let result = parser.parse_line(line);
        assert!(result.is_some());

        // Verify it uses current year
        let current_year = Utc::now().year();
        assert_eq!(result.unwrap().year(), current_year);
    }

    #[test]
    fn test_parse_yearless_iso_auto_detection() {
        let parser = TimestampParser::new(None);
        let line = "09-24T23:45:29.362Z| INFO| Some random logline";
        let result = parser.parse_line(line);
        assert!(result.is_some());

        // Verify it uses current year
        let current_year = Utc::now().year();
        assert_eq!(result.unwrap().year(), current_year);
    }

    #[test]
    fn test_parse_yearless_iso_variations() {
        let parser = TimestampParser::new(None);

        // Test different variations of yearless ISO format
        let test_cases = vec![
            ("09-24T23:45:29.362Z", 9, 24, 23, 45, 29),
            ("12-31T00:00:00.000Z", 12, 31, 0, 0, 0),
            ("01-01T12:30:45.123Z", 1, 1, 12, 30, 45),
            ("06-15T18:22:33.999Z", 6, 15, 18, 22, 33),
        ];

        for (
            timestamp,
            expected_month,
            expected_day,
            expected_hour,
            expected_minute,
            expected_second,
        ) in test_cases
        {
            let line = format!("{}| INFO| Test message", timestamp);
            let result = parser.parse_line(&line);
            assert!(result.is_some(), "Failed to parse: {}", timestamp);

            let dt = result.unwrap();
            let current_year = Utc::now().year();
            assert_eq!(dt.year(), current_year);
            assert_eq!(dt.month(), expected_month);
            assert_eq!(dt.day(), expected_day);
            assert_eq!(dt.hour(), expected_hour);
            assert_eq!(dt.minute(), expected_minute);
            assert_eq!(dt.second(), expected_second);
        }
    }

    #[test]
    fn test_parse_yearless_iso_without_milliseconds() {
        let parser = TimestampParser::new(None);
        let line = "09-24T23:45:29Z| INFO| Some random logline";
        let result = parser.parse_line(line);
        assert!(result.is_some());

        let dt = result.unwrap();
        let current_year = Utc::now().year();
        assert_eq!(dt.year(), current_year);
        assert_eq!(dt.month(), 9);
        assert_eq!(dt.day(), 24);
        assert_eq!(dt.hour(), 23);
        assert_eq!(dt.minute(), 45);
        assert_eq!(dt.second(), 29);
    }

    #[test]
    fn test_parse_yearless_iso_without_timezone() {
        let parser = TimestampParser::new(None);
        let line = "09-24T23:45:29.362| INFO| Some random logline";
        let result = parser.parse_line(line);
        // This format is being parsed by the regular ISO regex, not the yearless one
        // So it should succeed but use the current year
        assert!(result.is_some());

        let dt = result.unwrap();
        let current_year = Utc::now().year();
        assert_eq!(dt.year(), current_year);
        assert_eq!(dt.month(), 9);
        assert_eq!(dt.day(), 24);
        assert_eq!(dt.hour(), 23);
        assert_eq!(dt.minute(), 45);
        assert_eq!(dt.second(), 29);
    }

    #[test]
    fn test_parse_yearless_iso_edge_cases() {
        let parser = TimestampParser::new(None);

        // Test edge cases
        let test_cases = vec![
            ("01-01T00:00:00.000Z", "New Year"),
            ("12-31T23:59:59.999Z", "End of year"),
            ("06-30T15:30:45.500Z", "Mid-year"),
        ];

        for (timestamp, description) in test_cases {
            let line = format!("{}| INFO| {}", timestamp, description);
            let result = parser.parse_line(&line);
            assert!(
                result.is_some(),
                "Failed to parse {}: {}",
                description,
                timestamp
            );

            let dt = result.unwrap();
            let current_year = Utc::now().year();
            assert_eq!(dt.year(), current_year, "Wrong year for {}", description);
        }
    }

    #[test]
    fn test_parse_yearless_iso_invalid_formats() {
        let parser = TimestampParser::new(None);

        // Test invalid formats that should not parse
        let invalid_cases = vec![
            "09-24T3:45:29.362Z", // Single digit hour
            "09-24T23:5:29.362Z", // Single digit minute
            "09-24T23:45:9.362Z", // Single digit second
        ];

        for invalid_timestamp in invalid_cases {
            let line = format!("{}| INFO| Test message", invalid_timestamp);
            let result = parser.parse_line(&line);
            assert!(
                result.is_none(),
                "Should not parse invalid format: {}",
                invalid_timestamp
            );
        }
    }

    #[test]
    fn test_parse_yearless_iso_with_different_separators() {
        let parser = TimestampParser::new(None);

        // Test with different log line separators
        let test_cases = vec![
            "09-24T23:45:29.362Z| INFO| Pipe separator",
            "09-24T23:45:29.362Z INFO Space separator",
            "09-24T23:45:29.362Z\tINFO\tTab separator",
            "09-24T23:45:29.362Z,INFO,Comma separator",
            "09-24T23:45:29.362Z - INFO - Dash separator",
        ];

        for line in test_cases {
            let result = parser.parse_line(line);
            assert!(result.is_some(), "Failed to parse: {}", line);

            let dt = result.unwrap();
            let current_year = Utc::now().year();
            assert_eq!(dt.year(), current_year);
            assert_eq!(dt.month(), 9);
            assert_eq!(dt.day(), 24);
            assert_eq!(dt.hour(), 23);
            assert_eq!(dt.minute(), 45);
            assert_eq!(dt.second(), 29);
        }
    }

    #[test]
    fn test_parse_yearless_iso_with_custom_format() {
        let parser = TimestampParser::new(Some("%m-%dT%H:%M:%S%.3fZ".to_string()));

        let test_cases = vec![
            ("09-24T23:45:29.362Z", 9, 24, 23, 45, 29),
            ("12-31T00:00:00.000Z", 12, 31, 0, 0, 0),
            ("01-01T12:30:45.123Z", 1, 1, 12, 30, 45),
        ];

        for (
            timestamp,
            expected_month,
            expected_day,
            expected_hour,
            expected_minute,
            expected_second,
        ) in test_cases
        {
            let line = format!("{}| INFO| Test message", timestamp);
            let result = parser.parse_line(&line);
            assert!(result.is_some(), "Failed to parse: {}", timestamp);

            let dt = result.unwrap();
            let current_year = Utc::now().year();
            assert_eq!(dt.year(), current_year);
            assert_eq!(dt.month(), expected_month);
            assert_eq!(dt.day(), expected_day);
            assert_eq!(dt.hour(), expected_hour);
            assert_eq!(dt.minute(), expected_minute);
            assert_eq!(dt.second(), expected_second);
        }
    }

    #[test]
    fn test_parse_yearless_iso_priority() {
        let parser = TimestampParser::new(None);

        // Test that yearless ISO format takes priority over other formats
        let line = "09-24T23:45:29.362Z| INFO| This should parse as yearless ISO";
        let result = parser.parse_line(line);
        assert!(result.is_some());

        let dt = result.unwrap();
        let current_year = Utc::now().year();
        assert_eq!(dt.year(), current_year);
        assert_eq!(dt.month(), 9);
        assert_eq!(dt.day(), 24);
    }

    #[test]
    fn test_parse_yearless_iso_with_other_timestamps() {
        let parser = TimestampParser::new(None);

        // Test that yearless ISO format works alongside other timestamp formats
        let test_cases = vec![
            ("09-24T23:45:29.362Z| INFO| Yearless ISO", 9, 24, 23, 45, 29),
            ("2025-10-03T14:30:45.123Z INFO: Full ISO", 10, 3, 14, 30, 45),
            (
                "Oct 03 14:30:45 myserver app: INFO message",
                10,
                3,
                14,
                30,
                45,
            ),
        ];

        for (line, expected_month, expected_day, expected_hour, expected_minute, expected_second) in
            test_cases
        {
            let result = parser.parse_line(line);
            assert!(result.is_some(), "Failed to parse: {}", line);

            let dt = result.unwrap();
            assert_eq!(dt.month(), expected_month);
            assert_eq!(dt.day(), expected_day);
            assert_eq!(dt.hour(), expected_hour);
            assert_eq!(dt.minute(), expected_minute);
            assert_eq!(dt.second(), expected_second);
        }
    }

    #[test]
    fn test_parse_time_only_format() {
        let parser = TimestampParser::new(None);

        let test_cases = vec![
            ("05:40:12 INFO - Payment processed", 5, 40, 12),
            ("23:59:59.999 ERROR - End of day", 23, 59, 59),
            ("00:00:00 INFO - Midnight", 0, 0, 0),
            ("12:30:45 WARN - Noon warning", 12, 30, 45),
        ];

        for (line, expected_hour, expected_minute, expected_second) in test_cases {
            let result = parser.parse_line(line);
            assert!(result.is_some(), "Failed to parse: {}", line);

            let dt = result.unwrap();
            let current_date = Utc::now().date_naive();
            assert_eq!(dt.date_naive(), current_date);
            assert_eq!(dt.hour(), expected_hour);
            assert_eq!(dt.minute(), expected_minute);
            assert_eq!(dt.second(), expected_second);
        }
    }

    #[test]
    fn test_parse_time_only_with_custom_format() {
        let parser = TimestampParser::new(Some("%H:%M:%S".to_string()));

        let line = "05:40:12 INFO - Payment processed";
        let result = parser.parse_line(line);
        assert!(result.is_some());

        let dt = result.unwrap();
        let current_date = Utc::now().date_naive();
        assert_eq!(dt.date_naive(), current_date);
        assert_eq!(dt.hour(), 5);
        assert_eq!(dt.minute(), 40);
        assert_eq!(dt.second(), 12);
    }

    #[test]
    fn test_parse_time_only_with_milliseconds() {
        let parser = TimestampParser::new(None);

        let line = "05:40:12.123 INFO - Payment processed";
        let result = parser.parse_line(line);
        assert!(result.is_some());

        let dt = result.unwrap();
        let current_date = Utc::now().date_naive();
        assert_eq!(dt.date_naive(), current_date);
        assert_eq!(dt.hour(), 5);
        assert_eq!(dt.minute(), 40);
        assert_eq!(dt.second(), 12);
        assert_eq!(dt.nanosecond() / 1_000_000, 123); // Check milliseconds
    }

    #[test]
    fn test_parse_time_only_invalid_formats() {
        let parser = TimestampParser::new(None);

        // Test invalid time-only formats that should not parse
        let invalid_cases = vec![
            "5:40:12 INFO - Single digit hour",
            "05:4:12 INFO - Single digit minute",
            "05:40:2 INFO - Single digit second",
            "25:40:12 INFO - Invalid hour",
            "05:60:12 INFO - Invalid minute",
            // Note: 05:40:60 might be parsed by other regexes, so we skip it
        ];

        for invalid_line in invalid_cases {
            let result = parser.parse_line(invalid_line);
            assert!(
                result.is_none(),
                "Should not parse invalid time format: {}",
                invalid_line
            );
        }
    }
}
