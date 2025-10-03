use chrono::{DateTime, Utc};
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub enum BucketSize {
    Seconds(i64),
    Auto,
}

impl BucketSize {
    pub fn from_string(s: &str) -> anyhow::Result<Self> {
        if s.to_lowercase() == "auto" {
            Ok(BucketSize::Auto)
        } else {
            let seconds: i64 = s
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
            None => BucketSize::Seconds(60), // Default: 1 minute
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
        let bucket_key = (timestamp.timestamp() / bucket_seconds) * bucket_seconds;

        *self.buckets.entry(bucket_key).or_insert(0) += 1;
    }

    fn get_bucket_size(&self) -> i64 {
        match &self.bucket_size {
            BucketSize::Seconds(s) => *s,
            BucketSize::Auto => {
                // Calculate auto bucket size based on time range
                if let (Some(first), Some(last)) = (self.first_timestamp, self.last_timestamp) {
                    let duration = last.signed_duration_since(first).num_seconds();
                    self.calculate_auto_bucket_size(duration)
                } else {
                    60 // Default to 1 minute
                }
            }
        }
    }

    fn calculate_auto_bucket_size(&self, total_seconds: i64) -> i64 {
        // Aim for around 20-50 buckets for good visualization
        const TARGET_BUCKETS: i64 = 30;

        let ideal = total_seconds / TARGET_BUCKETS;

        // Round to nice intervals
        if ideal < 60 {
            60 // 1 minute minimum
        } else if ideal < 300 {
            300 // 5 minutes
        } else if ideal < 900 {
            900 // 15 minutes
        } else if ideal < 3600 {
            3600 // 1 hour
        } else if ideal < 21600 {
            21600 // 6 hours
        } else if ideal < 86400 {
            86400 // 1 day
        } else {
            (ideal / 86400) * 86400 // Multiple of days
        }
    }

    pub fn get_buckets(&self) -> Vec<(DateTime<Utc>, usize)> {
        self.buckets
            .iter()
            .map(|(key, count)| {
                let dt = DateTime::from_timestamp(*key, 0).unwrap_or_else(|| Utc::now());
                (dt, *count)
            })
            .collect()
    }

    pub fn total_matches(&self) -> usize {
        self.buckets.values().sum()
    }

    pub fn bucket_size_seconds(&self) -> i64 {
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
            BucketSize::Seconds(s) => assert_eq!(s, 60),
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
        assert_eq!(bucket.bucket_size_seconds(), 60);

        let bucket = TimeBucket::new(Some("300".to_string())).unwrap();
        assert_eq!(bucket.bucket_size_seconds(), 300);

        let bucket = TimeBucket::new(Some("auto".to_string())).unwrap();
        assert_eq!(bucket.bucket_size_seconds(), 60); // Default before data
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
        assert!(size >= 60); // Should pick reasonable size
    }

    #[test]
    fn test_calculate_auto_bucket_size() {
        let bucket = TimeBucket::new(Some("auto".to_string())).unwrap();

        // Test various durations
        // The algorithm divides by 30 then rounds to nice intervals:
        // ideal < 60 -> 60
        // ideal < 300 -> 300
        // ideal < 900 -> 900
        // ideal < 3600 -> 3600
        // ideal < 21600 -> 21600
        // ideal < 86400 -> 86400

        assert_eq!(bucket.calculate_auto_bucket_size(1200), 60); // 1200/30=40 < 60 -> 60
        assert_eq!(bucket.calculate_auto_bucket_size(6000), 300); // 6000/30=200 >= 60 but < 300 -> 300
        assert_eq!(bucket.calculate_auto_bucket_size(15000), 900); // 15000/30=500 >= 300 but < 900 -> 900
        assert_eq!(bucket.calculate_auto_bucket_size(60000), 3600); // 60000/30=2000 >= 900 but < 3600 -> 3600
        assert_eq!(bucket.calculate_auto_bucket_size(180000), 21600); // 180000/30=6000 >= 3600 but < 21600 -> 21600
        assert_eq!(bucket.calculate_auto_bucket_size(1000000), 86400); // 1000000/30=33333 >= 21600 but < 86400 -> 86400
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
}
