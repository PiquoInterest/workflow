/// Expected-RED contracts translated from `packages/core/e2e/benchmark.test.ts`.
///
/// These types describe the observable benchmark plan without manufacturing
/// measurements. Final GREEN must drive a real Rust workflow deployment and
/// collect timestamps emitted by that deployment.
pub const BENCH_METHODOLOGY_VERSION: u32 = 2;
pub const REPLAY_CADENCE_EVE: &str = "eve-gpt-5.6-sol-2000t";
pub const REPLAY_CADENCE_GATEWAY: &str = "gateway-gpt-5.4-nano-2000t";
pub const RTT_INDEX_BUCKETS: [&str; 3] = ["seq 0", "seq 1-20", "seq 21+"];
pub const TDD_RED_MARKER: &str =
    "TDD RED: packages/core/e2e/benchmark.test.ts implementation pending";

fn pending<T>() -> T {
    panic!("{TDD_RED_MARKER}")
}

#[derive(Debug, Clone, PartialEq)]
pub enum BenchArgument {
    Integer(u64),
    Number(f64),
    Text(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BenchmarkScenario {
    StepTurbo,
    StreamingStepTurbo,
    HookAndStepNonTurbo,
    PacedControl,
    SizeSweep,
    ReplayGatewayReality,
    ReplayEveReality,
    ReplayEveStress,
    FanOut,
    Sequential,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricKind {
    Ttfs,
    Stream,
    CrttDetail,
    WriteSlip,
    FanOutTtfs,
    FanOutTtls,
    StsoInline,
    StsoQueueHop,
    WorkflowOverhead,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MetricTargets {
    pub p75_ms: u64,
    pub p90_ms: u64,
    pub p99_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricExpectation {
    pub kind: MetricKind,
    pub scenario: String,
    pub detail: bool,
    pub group: Option<String>,
    pub bucket: Option<String>,
    pub targets: Option<MetricTargets>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TriggerRequest {
    pub workflow_fn: String,
    pub arguments: Vec<BenchArgument>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReturnIntegrity {
    StepTimings { exact_count: usize },
    ChunkRtt { exact_received: usize },
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScenarioPlan {
    pub scenario: BenchmarkScenario,
    pub trigger: TriggerRequest,
    pub iterations: usize,
    pub warmup_iterations: usize,
    pub extra_attempts: usize,
    pub run_sequentially: bool,
    pub zero_success_abort_attempts: usize,
    pub timeout_ms: u64,
    pub return_integrity: ReturnIntegrity,
    pub metrics: Vec<MetricExpectation>,
    pub cadence_semantic_hash_required: bool,
}

impl ScenarioPlan {
    pub const fn max_attempts(&self) -> usize {
        self.iterations + self.extra_attempts
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BenchmarkObservation {
    pub methodology_version: u32,
    pub preflight: TriggerRequest,
    pub deployment_clock_anchor: bool,
    pub trigger_response_validated: bool,
    pub negative_clock_skew_clamped: bool,
    pub preflight_timeout_ms: u64,
    pub plan: ScenarioPlan,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BenchmarkConfig {
    pub stream_iterations: usize,
    pub crtt_iterations: usize,
    pub sequential_iterations: usize,
    pub sequential_step_count: usize,
    pub fanout_iterations: usize,
    pub fanout_step_count: usize,
    pub warmup_iterations: usize,
    pub crtt_chunk_rate_per_sec: usize,
    pub crtt_duration_seconds: usize,
    pub replay_speed: usize,
    pub replay_eve_iterations: usize,
    pub replay_reality_iterations: usize,
    pub replay_gateway_iterations: usize,
    pub replay_eve_events: usize,
    pub replay_gateway_events: usize,
    pub replay_eve_span_ms: u64,
    pub replay_gateway_span_ms: u64,
    pub run_timeout_ms: u64,
    pub per_step_timeout_allowance_ms: u64,
    pub max_failure_ratio: f64,
    pub zero_success_abort_attempts: usize,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            stream_iterations: 30,
            crtt_iterations: 10,
            sequential_iterations: 1,
            sequential_step_count: 1_020,
            fanout_iterations: 10,
            fanout_step_count: 100,
            warmup_iterations: 2,
            crtt_chunk_rate_per_sec: 100,
            crtt_duration_seconds: 3,
            replay_speed: 2,
            replay_eve_iterations: 3,
            replay_reality_iterations: 2,
            replay_gateway_iterations: 3,
            replay_eve_events: 2_593,
            replay_gateway_events: 1_765,
            replay_eve_span_ms: 52_377,
            replay_gateway_span_ms: 19_943,
            run_timeout_ms: 120_000,
            per_step_timeout_allowance_ms: 2_000,
            max_failure_ratio: 0.2,
            zero_success_abort_attempts: 3,
        }
    }
}

impl BenchmarkConfig {
    pub const fn crtt_chunk_count(&self) -> usize {
        self.crtt_chunk_rate_per_sec * self.crtt_duration_seconds
    }

    pub fn crtt_interval_ms(&self) -> f64 {
        1_000.0 / self.crtt_chunk_rate_per_sec as f64
    }

    pub fn default_extra_attempts(&self, iterations: usize) -> usize {
        (iterations as f64 * self.max_failure_ratio).ceil() as usize
    }

    pub const fn count_scaled_timeout_ms(&self, count: usize) -> u64 {
        self.run_timeout_ms + count as u64 * self.per_step_timeout_allowance_ms
    }

    pub const fn replay_timeout_ms(&self, span_ms: u64, speed: usize) -> u64 {
        let speed = speed as u64;
        self.run_timeout_ms + span_ms.div_ceil(speed)
    }
}

/// Executes one benchmark scenario through the future Rust workflow runtime.
///
/// Final GREEN requirements:
/// - trigger the deployment route and validate `runId` plus `clientStart`;
/// - anchor every latency metric to deployment-side timestamps;
/// - run recorded iterations sequentially with bounded retries;
/// - reject malformed timing arrays and incorrect step/chunk counts;
/// - preserve cadence semantic hashes in the emitted result artifact.
pub fn run_benchmark_scenario(
    scenario: BenchmarkScenario,
    config: &BenchmarkConfig,
) -> BenchmarkObservation {
    let _ = (scenario, config);
    pending()
}
