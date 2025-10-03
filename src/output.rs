use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Serialize)]
struct BucketEntry {
    timestamp: String,
    count: usize,
}

pub fn output_table(buckets: &[(DateTime<Utc>, usize)], bucket_size_seconds: i64) -> Result<()> {
    if buckets.is_empty() {
        println!("No matches found.");
        return Ok(());
    }

    let total: usize = buckets.iter().map(|(_, count)| count).sum();

    println!("\n{:^30} | {:>10}", "Timestamp", "Count");
    println!("{:-^30}-+-{:-^10}", "", "");

    for (timestamp, count) in buckets {
        println!(
            "{:30} | {:>10}",
            timestamp.format("%Y-%m-%d %H:%M:%S"),
            count
        );
    }

    println!("{:-^30}-+-{:-^10}", "", "");
    println!("{:30} | {:>10}", "Total", total);
    println!("\nBucket size: {} seconds", bucket_size_seconds);

    Ok(())
}

pub fn output_csv(buckets: &[(DateTime<Utc>, usize)]) -> Result<()> {
    let mut wtr = csv::Writer::from_writer(std::io::stdout());

    wtr.write_record(["timestamp", "count"])?;

    for (timestamp, count) in buckets {
        wtr.write_record(&[timestamp.to_rfc3339(), count.to_string()])?;
    }

    wtr.flush()?;
    Ok(())
}

pub fn output_json(
    buckets: &[(DateTime<Utc>, usize)],
    bucket_size_seconds: i64,
    time_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
) -> Result<()> {
    let entries: Vec<BucketEntry> = buckets
        .iter()
        .map(|(ts, count)| BucketEntry {
            timestamp: ts.to_rfc3339(),
            count: *count,
        })
        .collect();

    let total: usize = buckets.iter().map(|(_, count)| count).sum();

    let output = serde_json::json!({
        "buckets": entries,
        "total_matches": total,
        "bucket_size_seconds": bucket_size_seconds,
        "time_range": time_range.map(|(start, end)| {
            serde_json::json!({
                "start": start.to_rfc3339(),
                "end": end.to_rfc3339(),
            })
        }),
    });

    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn create_test_buckets() -> Vec<(DateTime<Utc>, usize)> {
        vec![
            (Utc.with_ymd_and_hms(2025, 10, 3, 12, 0, 0).unwrap(), 10),
            (Utc.with_ymd_and_hms(2025, 10, 3, 12, 1, 0).unwrap(), 15),
            (Utc.with_ymd_and_hms(2025, 10, 3, 12, 2, 0).unwrap(), 8),
        ]
    }

    #[test]
    fn test_output_table_with_data() {
        let buckets = create_test_buckets();
        let result = output_table(&buckets, 60);
        assert!(result.is_ok());
    }

    #[test]
    fn test_output_table_empty() {
        let buckets = vec![];
        let result = output_table(&buckets, 60);
        assert!(result.is_ok());
    }

    #[test]
    fn test_output_csv_with_data() {
        let buckets = create_test_buckets();
        let result = output_csv(&buckets);
        assert!(result.is_ok());
    }

    #[test]
    fn test_output_csv_empty() {
        let buckets = vec![];
        let result = output_csv(&buckets);
        assert!(result.is_ok());
    }

    #[test]
    fn test_output_json_with_data() {
        let buckets = create_test_buckets();
        let start = Utc.with_ymd_and_hms(2025, 10, 3, 12, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2025, 10, 3, 12, 2, 0).unwrap();
        let result = output_json(&buckets, 60, Some((start, end)));
        assert!(result.is_ok());
    }

    #[test]
    fn test_output_json_without_time_range() {
        let buckets = create_test_buckets();
        let result = output_json(&buckets, 60, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_output_json_empty() {
        let buckets = vec![];
        let result = output_json(&buckets, 60, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_bucket_entry_serialization() {
        let entry = BucketEntry {
            timestamp: "2025-10-03T12:00:00Z".to_string(),
            count: 42,
        };

        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("2025-10-03T12:00:00Z"));
        assert!(json.contains("42"));
    }

    #[test]
    fn test_json_output_structure() {
        // We can verify the JSON structure by capturing it
        let buckets = [(Utc.with_ymd_and_hms(2025, 10, 3, 12, 0, 0).unwrap(), 5)];

        let entries: Vec<BucketEntry> = buckets
            .iter()
            .map(|(ts, count)| BucketEntry {
                timestamp: ts.to_rfc3339(),
                count: *count,
            })
            .collect();

        let total: usize = buckets.iter().map(|(_, count)| count).sum();

        let output = serde_json::json!({
            "buckets": entries,
            "total_matches": total,
            "bucket_size_seconds": 60,
            "time_range": None::<()>,
        });

        assert_eq!(output["total_matches"], 5);
        assert_eq!(output["bucket_size_seconds"], 60);
        assert!(output["buckets"].is_array());
        assert_eq!(output["buckets"].as_array().unwrap().len(), 1);
    }
}
