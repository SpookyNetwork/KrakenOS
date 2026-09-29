//! Compression — Entropy reduction from raw events to canonical facts.

/// Compression statistics from a single pass.
#[derive(Debug, Clone)]
pub struct CompressionStats {
    pub raw_events_processed: usize,
    pub canonical_facts_produced: usize,
    pub compression_ratio: f64,
}

/// Compute compression ratio.
pub fn compute_ratio(raw: usize, canonical: usize) -> f64 {
    if raw == 0 { return 0.0; }
    1.0 - (canonical as f64 / raw as f64)
}
