//! Latency histogram for P50/P95/P99/P999 percentile tracking (FT-024).
//!
//! Uses a fixed-size bucket array with atomic counters for lock-free
//! concurrent recording. Bucket resolution is configurable (default 10µs
//! per bucket, 0–100ms range = 10,000 buckets ≈ 80KB).
//!
//! All operations are wait-free (O(1) record, O(buckets) percentile query).

use std::sync::atomic::{AtomicU64, Ordering};

/// Default bucket width in microseconds (10µs).
const DEFAULT_BUCKET_WIDTH_US: u64 = 10;

/// Default maximum tracked latency in microseconds (100ms).
const DEFAULT_MAX_LATENCY_US: u64 = 100_000;

/// Configuration for a latency histogram.
#[derive(Debug, Clone)]
pub struct HistogramConfig {
    /// Width of each bucket in microseconds.
    pub bucket_width_us: u64,
    /// Maximum tracked latency in microseconds. Values above this are
    /// counted in an overflow bucket.
    pub max_latency_us: u64,
}

impl Default for HistogramConfig {
    fn default() -> Self {
        Self {
            bucket_width_us: DEFAULT_BUCKET_WIDTH_US,
            max_latency_us: DEFAULT_MAX_LATENCY_US,
        }
    }
}

/// Snapshot of histogram percentiles (read-only, no locks held).
#[derive(Debug, Clone, PartialEq)]
pub struct PercentileSnapshot {
    /// Total number of recorded samples.
    pub count: u64,
    /// Arithmetic mean latency in microseconds.
    pub mean_us: f64,
    /// Minimum recorded latency in microseconds (0 if no samples).
    pub min_us: u64,
    /// Maximum recorded latency in microseconds (0 if no samples).
    pub max_us: u64,
    /// 50th percentile latency in microseconds.
    pub p50_us: u64,
    /// 95th percentile latency in microseconds.
    pub p95_us: u64,
    /// 99th percentile latency in microseconds.
    pub p99_us: u64,
    /// 99.9th percentile latency in microseconds.
    pub p999_us: u64,
}

/// Lock-free latency histogram using atomic counters.
///
/// Tracks latency samples in fixed-width buckets. Each bucket is an
/// `AtomicU64` counter, so concurrent threads can record without locks.
/// Percentile queries iterate the bucket array (O(n) where n = number of
/// buckets, typically 10,000).
pub struct LatencyHistogram {
    /// Counters per bucket. Bucket `i` covers latency range
    /// `[i * width, (i+1) * width)` microseconds.
    buckets: Vec<AtomicU64>,
    /// Width of each bucket in microseconds.
    bucket_width_us: u64,
    /// Number of regular buckets (excluding overflow).
    num_buckets: usize,
    /// Total number of recorded samples.
    total_count: AtomicU64,
    /// Sum of all recorded latencies in microseconds (for mean).
    total_sum_us: AtomicU64,
    /// Minimum recorded latency in microseconds.
    min_us: AtomicU64,
    /// Maximum recorded latency in microseconds.
    max_us: AtomicU64,
}

impl LatencyHistogram {
    /// Create a new histogram with default config (10µs buckets, 0–100ms).
    pub fn new() -> Self {
        Self::with_config(HistogramConfig::default())
    }

    /// Create a new histogram with the given config.
    pub fn with_config(config: HistogramConfig) -> Self {
        let num_buckets = (config.max_latency_us / config.bucket_width_us) as usize;
        let mut buckets = Vec::with_capacity(num_buckets);
        for _ in 0..num_buckets {
            buckets.push(AtomicU64::new(0));
        }
        Self {
            buckets,
            bucket_width_us: config.bucket_width_us,
            num_buckets,
            total_count: AtomicU64::new(0),
            total_sum_us: AtomicU64::new(0),
            min_us: AtomicU64::new(u64::MAX),
            max_us: AtomicU64::new(0),
        }
    }

    /// Record a latency sample in microseconds.
    ///
    /// This is wait-free (O(1)) and safe to call from any thread.
    pub fn record_us(&self, latency_us: u64) {
        let bucket = (latency_us / self.bucket_width_us) as usize;
        let idx = if bucket >= self.num_buckets {
            self.num_buckets - 1
        } else {
            bucket
        };
        self.buckets[idx].fetch_add(1, Ordering::Relaxed);
        self.total_count.fetch_add(1, Ordering::Relaxed);
        self.total_sum_us.fetch_add(latency_us, Ordering::Relaxed);

        // Update min (lock-free CAS loop).
        self.min_us.fetch_min(latency_us, Ordering::Relaxed);
        // Update max (lock-free CAS loop).
        self.max_us.fetch_max(latency_us, Ordering::Relaxed);
    }

    /// Record a latency sample from a `Duration`.
    pub fn record_duration(&self, duration: std::time::Duration) {
        self.record_us(duration.as_micros() as u64);
    }

    /// Compute percentiles from the current bucket state.
    ///
    /// This iterates all buckets (O(num_buckets)) but does not hold any lock.
    /// The result is a consistent snapshot of percentiles at the time of the call.
    pub fn percentiles(&self) -> PercentileSnapshot {
        let count = self.total_count.load(Ordering::Relaxed);
        let sum = self.total_sum_us.load(Ordering::Relaxed);
        let min = self.min_us.load(Ordering::Relaxed);
        let max = self.max_us.load(Ordering::Relaxed);

        if count == 0 {
            return PercentileSnapshot {
                count: 0,
                mean_us: 0.0,
                min_us: 0,
                max_us: 0,
                p50_us: 0,
                p95_us: 0,
                p99_us: 0,
                p999_us: 0,
            };
        }

        let mean_us = sum as f64 / count as f64;

        // Target sample indices for each percentile.
        let targets = [
            (count * 500) / 1000,    // p50
            (count * 950) / 1000,    // p95
            (count * 990) / 1000,    // p99
            (count * 999) / 1000,    // p999
        ];
        let mut results = [0u64; 4];
        let mut found = [false; 4];

        let mut cumulative = 0u64;
        for (i, bucket) in self.buckets.iter().enumerate() {
            let c = bucket.load(Ordering::Relaxed);
            if c == 0 {
                continue;
            }
            let prev = cumulative;
            cumulative += c;

            for (j, &target) in targets.iter().enumerate() {
                if !found[j] && cumulative >= target {
                    // Interpolate within the bucket.
                    let bucket_start_us = (i as u64) * self.bucket_width_us;
                    let offset_in_bucket = if c > 0 {
                        ((target - prev) as f64 / c as f64) * self.bucket_width_us as f64
                    } else {
                        0.0
                    };
                    results[j] = bucket_start_us + offset_in_bucket as u64;
                    found[j] = true;
                }
            }

            if found.iter().all(|&f| f) {
                break;
            }
        }

        // For any percentile not found (shouldn't happen if count > 0), use max.
        for j in 0..4 {
            if !found[j] {
                results[j] = max;
            }
        }

        let min_us = if min == u64::MAX { 0 } else { min };

        PercentileSnapshot {
            count,
            mean_us,
            min_us,
            max_us: max,
            p50_us: results[0],
            p95_us: results[1],
            p99_us: results[2],
            p999_us: results[3],
        }
    }

    /// Reset all counters to zero.
    pub fn reset(&self) {
        for bucket in &self.buckets {
            bucket.store(0, Ordering::Relaxed);
        }
        self.total_count.store(0, Ordering::Relaxed);
        self.total_sum_us.store(0, Ordering::Relaxed);
        self.min_us.store(u64::MAX, Ordering::Relaxed);
        self.max_us.store(0, Ordering::Relaxed);
    }

    /// Get the total number of recorded samples.
    pub fn count(&self) -> u64 {
        self.total_count.load(Ordering::Relaxed)
    }

    /// Get the number of buckets.
    pub fn bucket_count(&self) -> usize {
        self.num_buckets
    }
}

impl Default for LatencyHistogram {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn empty_histogram_zero_percentiles() {
        let h = LatencyHistogram::new();
        let snap = h.percentiles();
        assert_eq!(snap.count, 0);
        assert_eq!(snap.p50_us, 0);
        assert_eq!(snap.p99_us, 0);
    }

    #[test]
    fn single_sample() {
        let h = LatencyHistogram::new();
        h.record_us(5000); // 5ms
        let snap = h.percentiles();
        assert_eq!(snap.count, 1);
        assert_eq!(snap.min_us, 5000);
        assert_eq!(snap.max_us, 5000);
        assert!((snap.mean_us - 5000.0).abs() < 1.0);
        assert_eq!(snap.p50_us, 5000);
        assert_eq!(snap.p99_us, 5000);
    }

    #[test]
    fn percentile_ordering() {
        let h = LatencyHistogram::new();
        // Record 1000 samples within 0–100ms range:
        // 0µs, 100µs, 200µs, ..., 99900µs
        for i in 0..1000 {
            h.record_us(i * 100);
        }
        let snap = h.percentiles();
        assert_eq!(snap.count, 1000);
        assert!(snap.p50_us <= snap.p95_us);
        assert!(snap.p95_us <= snap.p99_us);
        assert!(snap.p99_us <= snap.p999_us);
        // p50 should be around 50000µs (50ms)
        assert!(snap.p50_us >= 40_000 && snap.p50_us <= 60_000);
    }

    #[test]
    fn sub_millisecond_resolution() {
        let h = LatencyHistogram::new();
        // Record 100 samples at 100µs intervals: 100, 200, ..., 10000µs
        for i in 1..=100 {
            h.record_us(i * 100);
        }
        let snap = h.percentiles();
        // p50 should be around 5000µs (5ms)
        assert!(snap.p50_us >= 4000 && snap.p50_us <= 6000);
        // p99 should be around 9900µs (9.9ms)
        assert!(snap.p99_us >= 9000 && snap.p99_us <= 10100);
    }

    #[test]
    fn overflow_counted() {
        let config = HistogramConfig {
            bucket_width_us: 100,
            max_latency_us: 1000, // only 10 buckets, up to 1ms
        };
        let h = LatencyHistogram::with_config(config);
        h.record_us(500);   // in range
        h.record_us(50_000); // overflow (50ms)
        assert_eq!(h.count(), 2);
        let snap = h.percentiles();
        assert_eq!(snap.count, 2);
        assert_eq!(snap.max_us, 50_000);
    }

    #[test]
    fn reset_clears_all() {
        let h = LatencyHistogram::new();
        h.record_us(1000);
        h.record_us(2000);
        assert_eq!(h.count(), 2);
        h.reset();
        assert_eq!(h.count(), 0);
        let snap = h.percentiles();
        assert_eq!(snap.count, 0);
    }

    #[test]
    fn duration_recording() {
        let h = LatencyHistogram::new();
        h.record_duration(std::time::Duration::from_millis(5));
        let snap = h.percentiles();
        assert_eq!(snap.count, 1);
        assert!((snap.mean_us - 5000.0).abs() < 10.0);
    }

    #[test]
    fn concurrent_recording() {
        use std::sync::Arc;
        use std::thread;

        let h = Arc::new(LatencyHistogram::new());
        let mut handles = Vec::new();

        for t in 0..8 {
            let h = Arc::clone(&h);
            handles.push(thread::spawn(move || {
                for i in 0..1000 {
                    h.record_us((t * 1000 + i) as u64);
                }
            }));
        }

        for handle in handles {
            handle.join().expect("thread panicked");
        }

        assert_eq!(h.count(), 8000);
        let snap = h.percentiles();
        assert!(snap.p50_us > 0);
        assert!(snap.p99_us > snap.p50_us);
    }

    #[test]
    fn real_world_latency_distribution() {
        let h = LatencyHistogram::new();
        // Simulate: 80% at 1-2ms, 15% at 3-5ms, 5% at 8-10ms
        // Use deterministic values: 800 samples at 1500µs, 150 at 4000µs, 50 at 9000µs
        for _ in 0..800 {
            h.record_us(1500);
        }
        for _ in 0..150 {
            h.record_us(4000);
        }
        for _ in 0..50 {
            h.record_us(9000);
        }
        let snap = h.percentiles();
        assert_eq!(snap.count, 1000);
        // p50 should be in the 1-2ms range (80% of samples are at 1500µs)
        assert!(snap.p50_us >= 1000 && snap.p50_us <= 2000);
        // p95 should be in the 3-5ms range
        assert!(snap.p95_us >= 3000 && snap.p95_us <= 5000);
        // p99 should be in the 8-10ms range
        assert!(snap.p99_us >= 8000 && snap.p99_us <= 10000);
    }
}
