//! Latency tracking using a T-Digest for adaptive hedge delay calculation.

use std::sync::RwLock;
use tdigest::TDigest;

/// A thread-safe latency histogram backed by a T-Digest.
///
/// Records observed latencies and provides percentile estimates, which are
/// used by [`HedgedClient`](crate::HedgedClient) to compute the hedge delay.
pub struct LatencyDigest {
    digest: RwLock<TDigest>,
    #[allow(dead_code)]
    compression: usize,
}

impl LatencyDigest {
    /// Creates a new `LatencyDigest` with the given compression factor.
    ///
    /// Higher compression yields more accurate quantile estimates at the cost
    /// of increased memory usage.
    pub fn new(compression: usize) -> Self {
        Self {
            digest: RwLock::new(TDigest::new_with_size(compression)),
            compression,
        }
    }

    /// Records a single latency observation (in microseconds).
    pub fn record(&self, latency_us: f64) {
        let mut d = match self.digest.write() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };

        *d = d.merge_unsorted(vec![latency_us]);
    }

    /// Returns the estimated value at the given percentile (0–100).
    ///
    /// Returns 0.0 if no observations have been recorded yet.
    pub fn percentile(&self, q: f64) -> f64 {
        let d = match self.digest.read() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        d.estimate_quantile(q / 100.0)
    }
}
