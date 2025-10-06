use chrono::{DateTime, Utc};
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub enum BucketSize {
    Seconds(f64),
    Auto,
}

impl BucketSize {
    pub fn from_string(s: &str) -> anyhow::Result<Self> {
        if s.to_lowercase() == "auto" {
            Ok(BucketSize::Auto)
        } else {
            let seconds: f64 = s
                .parse()
                .map_err(|_| anyhow::anyhow!("Invalid bucket size: must be a number or 'auto'"))?;
            Ok(BucketSize::Seconds(seconds))
        }
    }
}

pub struct TimeBucket {
    bucket_size: BucketSize,
    buckets: BTreeMap<i64, usize>,
    first_timestamp: Option<DateTime<Utc>>,
    last_timestamp: Option<DateTime<Utc>>,
}

impl TimeBucket {
    pub fn new(bucket_size: Option<String>) -> anyhow::Result<Self> {
        let size = match bucket_size {
            Some(s) => BucketSize::from_string(&s)?,
            None => BucketSize::Seconds(60.0), // Default: 1 minute
        };

        Ok(Self {
            bucket_size: size,
            buckets: BTreeMap::new(),
            first_timestamp: None,
            last_timestamp: None,
        })
    }

    pub fn add(&mut self, timestamp: DateTime<Utc>) {
        // Update first/last timestamps
        if self.first_timestamp.is_none() || Some(timestamp) < self.first_timestamp {
            self.first_timestamp = Some(timestamp);
        }
        if self.last_timestamp.is_none() || Some(timestamp) > self.last_timestamp {
            self.last_timestamp = Some(timestamp);
        }

        let bucket_seconds = self.get_bucket_size();
        let timestamp_micros = timestamp.timestamp_micros();
        let bucket_micros = (bucket_seconds * 1_000_000.0) as i64;
        let bucket_key = (timestamp_micros / bucket_micros) * bucket_micros;

        *self.buckets.entry(bucket_key).or_insert(0) += 1;
    }

    fn get_bucket_size(&self) -> f64 {
        match &self.bucket_size {
            BucketSize::Seconds(s) => *s,
            BucketSize::Auto => {
                // Calculate auto bucket size based on time range
                if let (Some(first), Some(last)) = (self.first_timestamp, self.last_timestamp) {
                    let duration = last.signed_duration_since(first).num_seconds() as f64;
                    self.calculate_auto_bucket_size(duration)
                } else {
                    60.0 // Default to 1 minute
                }
            }
        }
    }

    fn calculate_auto_bucket_size(&self, total_seconds: f64) -> f64 {
        // Aim for around 15-20 buckets for reasonable output size
        const TARGET_BUCKETS: f64 = 15.0;

        let ideal = total_seconds / TARGET_BUCKETS;

        // Round to nice intervals
        if ideal < 0.1 {
            0.1 // 100ms minimum
        } else if ideal < 1.0 {
            1.0 // 1 second
        } else if ideal < 60.0 {
            60.0 // 1 minute
        } else if ideal < 300.0 {
            300.0 // 5 minutes
        } else if ideal < 900.0 {
            900.0 // 15 minutes
        } else if ideal < 3600.0 {
            3600.0 // 1 hour
        } else if ideal < 21600.0 {
            21600.0 // 6 hours
        } else if ideal < 86400.0 {
            86400.0 // 1 day
        } else {
            (ideal / 86400.0).floor() * 86400.0 // Multiple of days
        }
    }

    pub fn get_buckets(&self) -> Vec<(DateTime<Utc>, usize)> {
        self.buckets
            .iter()
            .map(|(key, count)| {
                let dt = DateTime::from_timestamp_micros(*key).unwrap_or_else(Utc::now);
                (dt, *count)
            })
            .collect()
    }

    pub fn total_matches(&self) -> usize {
        self.buckets.values().sum()
    }

    pub fn bucket_size_seconds(&self) -> f64 {
        self.get_bucket_size()
    }

    pub fn time_range(&self) -> Option<(DateTime<Utc>, DateTime<Utc>)> {
        match (self.first_timestamp, self.last_timestamp) {
            (Some(first), Some(last)) => Some((first, last)),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_bucket_size_from_string() {
        let size = BucketSize::from_string("60").unwrap();
        match size {
            BucketSize::Seconds(s) => assert_eq!(s, 60.0),
            _ => panic!("Expected Seconds variant"),
        }

        let auto = BucketSize::from_string("auto").unwrap();
        matches!(auto, BucketSize::Auto);

        assert!(BucketSize::from_string("invalid").is_err());
    }

    #[test]
    fn test_time_bucket_creation() {
        let bucket = TimeBucket::new(None).unwrap();
        assert_eq!(bucket.total_matches(), 0);
        assert_eq!(bucket.bucket_size_seconds(), 60.0);

        let bucket = TimeBucket::new(Some("300".to_string())).unwrap();
        assert_eq!(bucket.bucket_size_seconds(), 300.0);

        let bucket = TimeBucket::new(Some("auto".to_string())).unwrap();
        assert_eq!(bucket.bucket_size_seconds(), 60.0); // Default before data
    }

    #[test]
    fn test_add_timestamps() {
        let mut bucket = TimeBucket::new(Some("60".to_string())).unwrap();

        let ts1 = Utc.with_ymd_and_hms(2025, 10, 3, 12, 30, 45).unwrap();
        let ts2 = Utc.with_ymd_and_hms(2025, 10, 3, 12, 31, 15).unwrap();
        let ts3 = Utc.with_ymd_and_hms(2025, 10, 3, 12, 32, 30).unwrap();

        bucket.add(ts1);
        bucket.add(ts2);
        bucket.add(ts3);

        assert_eq!(bucket.total_matches(), 3);
        let buckets = bucket.get_buckets();
        assert!(buckets.len() <= 3); // Should be bucketed
    }

    #[test]
    fn test_bucket_grouping() {
        let mut bucket = TimeBucket::new(Some("60".to_string())).unwrap();

        // Add multiple timestamps in the same minute
        let ts1 = Utc.with_ymd_and_hms(2025, 10, 3, 12, 30, 10).unwrap();
        let ts2 = Utc.with_ymd_and_hms(2025, 10, 3, 12, 30, 30).unwrap();
        let ts3 = Utc.with_ymd_and_hms(2025, 10, 3, 12, 30, 50).unwrap();

        bucket.add(ts1);
        bucket.add(ts2);
        bucket.add(ts3);

        let buckets = bucket.get_buckets();
        assert_eq!(buckets.len(), 1); // All in same bucket
        assert_eq!(buckets[0].1, 3); // Count is 3
    }

    #[test]
    fn test_time_range() {
        let mut bucket = TimeBucket::new(None).unwrap();
        assert!(bucket.time_range().is_none());

        let ts1 = Utc.with_ymd_and_hms(2025, 10, 3, 12, 0, 0).unwrap();
        let ts2 = Utc.with_ymd_and_hms(2025, 10, 3, 14, 0, 0).unwrap();

        bucket.add(ts1);
        bucket.add(ts2);

        let range = bucket.time_range().unwrap();
        assert_eq!(range.0, ts1);
        assert_eq!(range.1, ts2);
    }

    #[test]
    fn test_auto_bucket_size() {
        let mut bucket = TimeBucket::new(Some("auto".to_string())).unwrap();

        // Add timestamps spanning 1 hour
        let start = Utc.with_ymd_and_hms(2025, 10, 3, 12, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2025, 10, 3, 13, 0, 0).unwrap();

        bucket.add(start);
        bucket.add(end);

        let size = bucket.bucket_size_seconds();
        assert!(size >= 60.0); // Should pick reasonable size
    }

    #[test]
    fn test_calculate_auto_bucket_size() {
        let bucket = TimeBucket::new(Some("auto".to_string())).unwrap();

        // Test various durations
        // The algorithm divides by 15 then rounds to nice intervals:
        // ideal < 60 -> 60
        // ideal < 300 -> 300
        // ideal < 900 -> 900
        // ideal < 3600 -> 3600
        // ideal < 21600 -> 21600
        // ideal < 86400 -> 86400

        assert_eq!(bucket.calculate_auto_bucket_size(600.0), 60.0); // 600/15=40 < 60 -> 60
        assert_eq!(bucket.calculate_auto_bucket_size(3000.0), 300.0); // 3000/15=200 >= 60 but < 300 -> 300
        assert_eq!(bucket.calculate_auto_bucket_size(7500.0), 900.0); // 7500/15=500 >= 300 but < 900 -> 900
        assert_eq!(bucket.calculate_auto_bucket_size(30000.0), 3600.0); // 30000/15=2000 >= 900 but < 3600 -> 3600
        assert_eq!(bucket.calculate_auto_bucket_size(90000.0), 21600.0); // 90000/15=6000 >= 3600 but < 21600 -> 21600
        assert_eq!(bucket.calculate_auto_bucket_size(500000.0), 86400.0); // 500000/15=33333 >= 21600 but < 86400 -> 86400
    }

    #[test]
    fn test_buckets_sorted() {
        let mut bucket = TimeBucket::new(Some("60".to_string())).unwrap();

        // Add timestamps out of order
        let ts2 = Utc.with_ymd_and_hms(2025, 10, 3, 12, 32, 0).unwrap();
        let ts1 = Utc.with_ymd_and_hms(2025, 10, 3, 12, 30, 0).unwrap();
        let ts3 = Utc.with_ymd_and_hms(2025, 10, 3, 12, 34, 0).unwrap();

        bucket.add(ts2);
        bucket.add(ts1);
        bucket.add(ts3);

        let buckets = bucket.get_buckets();
        // BTreeMap should keep them sorted
        for i in 1..buckets.len() {
            assert!(buckets[i].0 > buckets[i - 1].0);
        }
    }

    #[test]
    fn test_empty_bucket() {
        let bucket = TimeBucket::new(None).unwrap();
        assert_eq!(bucket.total_matches(), 0);
        assert_eq!(bucket.get_buckets().len(), 0);
        assert!(bucket.time_range().is_none());
    }

    #[test]
    fn test_sub_second_bucketing() {
        let mut bucket = TimeBucket::new(Some("0.5".to_string())).unwrap();
        assert_eq!(bucket.bucket_size_seconds(), 0.5);

        // Add timestamps within 0.5 second intervals
        let base_time = Utc.with_ymd_and_hms(2025, 10, 3, 12, 30, 0).unwrap();
        let ts1 = base_time + chrono::Duration::milliseconds(100); // 0.1s
        let ts2 = base_time + chrono::Duration::milliseconds(300); // 0.3s
        let ts3 = base_time + chrono::Duration::milliseconds(600); // 0.6s (next bucket)

        bucket.add(ts1);
        bucket.add(ts2);
        bucket.add(ts3);

        let buckets = bucket.get_buckets();
        assert_eq!(buckets.len(), 2); // Two separate 0.5s buckets
        assert_eq!(bucket.total_matches(), 3);
    }
}
