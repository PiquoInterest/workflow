pub const TDD_RED_MARKER: &str =
    "TDD RED: packages/core/src/bench-chunk-rtt-stats.test.ts implementation pending";

pub const RTT_HIST_EDGES_MS: [f64; 12] = [
    1.0, 2.0, 5.0, 10.0, 20.0, 50.0, 100.0, 200.0, 500.0, 1_000.0, 2_000.0,
    5_000.0,
];
pub const RTT_PROGRESS_BINS: usize = 10;
pub const RTT_SIZE_BIN_EDGES_BYTES: [usize; 6] = [256, 512, 1_024, 2_048, 4_096, 8_192];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RttIndexBucket {
    SeqZero,
    SeqOneThroughTwenty,
    SeqTwentyOnePlus,
}

pub const RTT_INDEX_BUCKETS: [RttIndexBucket; 3] = [
    RttIndexBucket::SeqZero,
    RttIndexBucket::SeqOneThroughTwenty,
    RttIndexBucket::SeqTwentyOnePlus,
];

#[derive(Debug, Clone, PartialEq)]
pub struct BenchRttMeanProfile {
    pub counts: Vec<usize>,
    pub total_ms: Vec<f64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BenchDelayTail {
    pub count: usize,
    pub avg_ms: f64,
    pub p99_ms: f64,
    pub max_ms: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RatePoint {
    pub at_ms: f64,
    pub bytes: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BenchSteadyRate {
    pub chunks_per_sec: f64,
    pub kib_per_sec: f64,
    pub window_chunks: usize,
    pub window_ms: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CdvArrival {
    pub seq: usize,
    pub written_at: f64,
    pub read_at: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BenchCdvComputation {
    pub cdv_ms: Vec<f64>,
    pub positive_by_seq: Vec<Option<f64>>,
    pub duplicate_seqs: usize,
    pub reordered_arrivals: usize,
    pub skipped_pairs: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SizeRttSample {
    pub bytes: usize,
    pub rtt_ms: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BenchRttSummary {
    pub count: usize,
    pub best: f64,
    pub avg: f64,
    pub hist: Vec<usize>,
    pub p50: f64,
    pub p75: f64,
    pub p90: f64,
    pub p99: f64,
}

fn pending<T>() -> T {
    panic!("{TDD_RED_MARKER}")
}

pub fn rtt_index_bucket(seq: i64) -> RttIndexBucket {
    let _ = seq;
    pending()
}

pub fn progress_profile(rtt_by_seq: &[Option<f64>]) -> BenchRttMeanProfile {
    let _ = rtt_by_seq;
    pending()
}

pub fn merge_mean_profiles(
    profiles: &[Option<BenchRttMeanProfile>],
) -> Option<BenchRttMeanProfile> {
    let _ = profiles;
    pending()
}

pub fn rtt_size_bin(serialized_bytes: usize) -> usize {
    let _ = serialized_bytes;
    pending()
}

pub fn size_profile(samples: &[SizeRttSample]) -> BenchRttMeanProfile {
    let _ = samples;
    pending()
}

pub fn histogram_rtt_samples(samples: &[f64]) -> Vec<usize> {
    let _ = samples;
    pending()
}

pub fn summarize_rtt_samples(samples: &[f64]) -> Option<BenchRttSummary> {
    let _ = samples;
    pending()
}

pub fn summarize_delay_tail(samples: &[f64]) -> Option<BenchDelayTail> {
    let _ = samples;
    pending()
}

pub fn steady_rate(points: &[RatePoint]) -> Option<BenchSteadyRate> {
    steady_rate_with_trim(points, 0.1)
}

pub fn steady_rate_with_trim(
    points: &[RatePoint],
    trim_fraction: f64,
) -> Option<BenchSteadyRate> {
    let _ = (points, trim_fraction);
    pending()
}

pub fn compute_cdv(arrivals: &[CdvArrival]) -> BenchCdvComputation {
    let _ = arrivals;
    pending()
}

pub fn merge_rtt_summaries(
    summaries: &[Option<BenchRttSummary>],
) -> Option<BenchRttSummary> {
    let _ = summaries;
    pending()
}
