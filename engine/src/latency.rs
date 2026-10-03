//! Latency percentiles without keeping every sample: a histogram of
//! logarithmic buckets 1 % wide, from a microsecond to five minutes, filled
//! from many tasks at once (atomics, no lock). A percentile is read to within
//! half a percent of the true value, in constant memory, however long a load
//! runs — the honest answer to "how slow is it for one request in a hundred".

use std::sync::atomic::{AtomicU64, Ordering};

/// Each bucket is this much wider than the one before.
const GROWTH: f64 = 1.01;
/// Buckets up to 1.01^2000 µs ≈ 7 minutes; anything longer lands in the last.
const BUCKETS: usize = 2000;

pub struct LatencyHistogram {
    buckets: Vec<AtomicU64>,
    count: AtomicU64,
    sum_us: AtomicU64,
    min_us: AtomicU64,
    max_us: AtomicU64,
}

impl Default for LatencyHistogram {
    fn default() -> Self {
        LatencyHistogram {
            buckets: (0..BUCKETS).map(|_| AtomicU64::new(0)).collect(),
            count: AtomicU64::new(0),
            sum_us: AtomicU64::new(0),
            min_us: AtomicU64::new(u64::MAX),
            max_us: AtomicU64::new(0),
        }
    }
}

fn bucket_of(micros: u64) -> usize {
    if micros <= 1 {
        return 0;
    }
    ((micros as f64).ln() / GROWTH.ln()).floor().min((BUCKETS - 1) as f64) as usize
}

/// The middle of a bucket, geometrically: within half a bucket of anything in it.
fn middle_of(bucket: usize) -> f64 {
    GROWTH.powf(bucket as f64 + 0.5)
}

impl LatencyHistogram {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record(&self, micros: u64) {
        self.buckets[bucket_of(micros)].fetch_add(1, Ordering::Relaxed);
        self.count.fetch_add(1, Ordering::Relaxed);
        self.sum_us.fetch_add(micros, Ordering::Relaxed);
        self.min_us.fetch_min(micros, Ordering::Relaxed);
        self.max_us.fetch_max(micros, Ordering::Relaxed);
    }

    pub fn count(&self) -> u64 {
        self.count.load(Ordering::Relaxed)
    }

    /// Milliseconds; 0 while nothing was recorded.
    pub fn min_ms(&self) -> f64 {
        match self.min_us.load(Ordering::Relaxed) {
            u64::MAX => 0.0,
            min => min as f64 / 1000.0,
        }
    }

    pub fn max_ms(&self) -> f64 {
        self.max_us.load(Ordering::Relaxed) as f64 / 1000.0
    }

    pub fn mean_ms(&self) -> f64 {
        match self.count() {
            0 => 0.0,
            count => self.sum_us.load(Ordering::Relaxed) as f64 / count as f64 / 1000.0,
        }
    }

    /// The latency `fraction` (0..1) of the requests were at or under, in
    /// milliseconds — never outside the smallest and largest seen.
    pub fn percentile_ms(&self, fraction: f64) -> f64 {
        self.read(&[fraction])[0]
    }

    /// p50, p90, p95 and p99, in milliseconds, read in one pass.
    pub fn percentiles(&self) -> Percentiles {
        let [p50_ms, p90_ms, p95_ms, p99_ms] = self.read(&[0.50, 0.90, 0.95, 0.99]);
        Percentiles { p50_ms, p90_ms, p95_ms, p99_ms }
    }

    /// How many took up to each of `bounds_ms` (ascending) and longer than the
    /// bound before it — each bucket by its middle, so a bound is kept to
    /// within half a percent; the last count is everything slower.
    pub fn coarse(&self, bounds_ms: &[f64]) -> Vec<u64> {
        let mut counts = vec![0; bounds_ms.len() + 1];
        for (bucket, counter) in self.buckets.iter().enumerate() {
            let count = counter.load(Ordering::Relaxed);
            if count > 0 {
                counts[bounds_ms.partition_point(|bound| bound * 1000.0 < middle_of(bucket))] += count;
            }
        }
        counts
    }

    /// Each of `fractions` (ascending) read in one walk over the buckets.
    fn read<const N: usize>(&self, fractions: &[f64; N]) -> [f64; N] {
        let mut read = [0.0; N];
        let count = self.count();
        if count == 0 {
            return read;
        }
        let (min, max) = (self.min_us.load(Ordering::Relaxed) as f64, self.max_us.load(Ordering::Relaxed) as f64);
        let rank = |fraction: f64| ((fraction.clamp(0.0, 1.0) * count as f64).ceil() as u64).max(1);
        let (mut next, mut seen) = (0, 0);
        for (bucket, counter) in self.buckets.iter().enumerate() {
            seen += counter.load(Ordering::Relaxed);
            while next < N && seen >= rank(fractions[next]) {
                // Past the last bucket's start nothing is told apart any more: the slowest seen.
                // (max/min, not clamp: a sample being recorded may show in a bucket before in min.)
                read[next] = if bucket == BUCKETS - 1 { max } else { middle_of(bucket).max(min).min(max) } / 1000.0;
                next += 1;
            }
            if next == N {
                return read;
            }
        }
        // Recorded while being read: what is not reached yet reads as the slowest.
        for value in &mut read[next..] {
            *value = max / 1000.0;
        }
        read
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, serde::Serialize)]
pub struct Percentiles {
    pub p50_ms: f64,
    pub p90_ms: f64,
    pub p95_ms: f64,
    pub p99_ms: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percentiles_are_within_half_a_percent_of_the_true_ones() {
        let histogram = LatencyHistogram::new();
        // 1..=10000 ms in steps of 1 ms: the true p-th percentile is p * 10000 ms.
        for ms in 1..=10_000u64 {
            histogram.record(ms * 1000);
        }
        for (fraction, truth) in [(0.5, 5000.0), (0.9, 9000.0), (0.95, 9500.0), (0.99, 9900.0)] {
            let read = histogram.percentile_ms(fraction);
            assert!((read - truth).abs() / truth < 0.006, "p{} read {read}, is {truth}", fraction * 100.0);
        }
        let p = histogram.percentiles();
        assert_eq!([p.p50_ms, p.p90_ms, p.p95_ms, p.p99_ms], [0.5, 0.9, 0.95, 0.99].map(|fraction| histogram.percentile_ms(fraction)), "one pass reads as each alone");
        assert_eq!((histogram.count(), histogram.min_ms(), histogram.max_ms()), (10_000, 1.0, 10_000.0));
        assert!((histogram.mean_ms() - 5000.5).abs() < 1e-9);
    }

    #[test]
    fn a_few_samples_read_as_themselves_and_nothing_reads_as_zero() {
        let empty = LatencyHistogram::new();
        assert_eq!((empty.percentiles(), empty.min_ms(), empty.mean_ms()), (Percentiles::default(), 0.0, 0.0));
        let one = LatencyHistogram::new();
        one.record(42_000);
        let p = one.percentiles();
        assert!(p.p50_ms == p.p99_ms && (p.p50_ms - 42.0).abs() < 0.25, "{p:?}");
        // A slow tail shows in p99 and not in p50.
        let tail = LatencyHistogram::new();
        for _ in 0..99 {
            tail.record(10_000);
        }
        tail.record(2_000_000);
        assert!(tail.percentile_ms(0.5) < 10.1 && tail.percentile_ms(0.99) < 10.1 && tail.percentile_ms(1.0) > 1990.0);
        // Out of range: the first and the last bucket, clamped to what was seen.
        let edges = LatencyHistogram::new();
        edges.record(0);
        edges.record(3_600_000_000);
        assert!(edges.percentile_ms(0.0) < 0.01);
        assert_eq!(edges.percentile_ms(1.0), 3_600_000.0, "an hour reads as an hour, not as the last bucket");
    }

    #[test]
    fn a_coarse_histogram_counts_between_its_bounds() {
        let histogram = LatencyHistogram::new();
        for micros in [300, 900, 1500, 4_000, 12_000, 12_000, 2_000_000] {
            histogram.record(micros);
        }
        assert_eq!(histogram.coarse(&[1.0, 5.0, 10.0, 100.0]), [2, 2, 0, 2, 1], "≤1, ≤5, ≤10, ≤100 ms and slower");
        assert_eq!(LatencyHistogram::new().coarse(&[1.0]), [0, 0]);
    }
}
