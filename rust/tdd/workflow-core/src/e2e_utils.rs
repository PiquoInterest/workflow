use std::collections::BTreeMap;

pub const PICKUP_INITIAL_INTERVAL_MS: u64 = 500;
pub const PICKUP_MAX_INTERVAL_MS: u64 = 5_000;
pub const DEFAULT_PICKUP_BUDGET_MS: u64 = 20_000;
pub const DEFAULT_WARMUP_BUDGET_MS: u64 = 60_000;
pub const WARMUP_CANCEL_REASON: &str = "e2e: warmup probe stuck pending, abandoned";
pub const TDD_RED_MARKER: &str = "TDD RED: packages/core/e2e/utils.test.ts implementation pending";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceMapEnvironment {
    pub app_name: String,
    pub deployment_url: Option<String>,
    pub dev_test_config: bool,
}

fn authority_from_url(value: &str) -> &str {
    let after_scheme = value
        .split_once("://")
        .map_or(value, |(_, authority)| authority);
    after_scheme
        .split(['/', '?', '#'])
        .next()
        .unwrap_or_default()
}

fn host_from_authority(authority: &str) -> &str {
    let host_port = authority
        .rsplit_once('@')
        .map_or(authority, |(_, host)| host);
    if let Some(rest) = host_port.strip_prefix('[') {
        return rest.split_once(']').map_or(rest, |(host, _)| host);
    }
    host_port
        .split_once(':')
        .map_or(host_port, |(host, _)| host)
}

pub fn is_local_deployment(deployment_url: Option<&str>) -> bool {
    let Some(deployment_url) = deployment_url else {
        return false;
    };
    let host = host_from_authority(authority_from_url(deployment_url));
    host.eq_ignore_ascii_case("localhost") || host == "127.0.0.1"
}

pub fn has_step_source_maps(environment: &SourceMapEnvironment) -> bool {
    if environment.app_name == "nextjs-turbopack" {
        return false;
    }
    if !is_local_deployment(environment.deployment_url.as_deref()) {
        return false;
    }
    if environment.app_name == "nest" {
        return false;
    }
    environment.dev_test_config
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StatusRead {
    Status(RunStatus),
    Error(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PickupScript {
    pub run_id: String,
    pub reads: Vec<StatusRead>,
}

impl PickupScript {
    pub fn new(run_id: &str, reads: Vec<StatusRead>) -> Self {
        Self {
            run_id: run_id.to_owned(),
            reads,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PickupObservation {
    pub picked_up: bool,
    pub status_reads: usize,
    pub sleep_intervals_ms: Vec<u64>,
    pub elapsed_ms: u64,
}

/// Polls the future Rust run handle until it leaves pending or the checked
/// budget expires. Transient read failures consume the same bounded schedule.
pub fn wait_for_run_pickup(script: &PickupScript, budget_ms: u64) -> PickupObservation {
    let _ = (script, budget_ms);
    panic!("{TDD_RED_MARKER}")
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WarmProbeScript {
    pub run_id: String,
    pub start_delay_ms: u64,
    pub pickup: PickupScript,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WarmupOptions {
    pub pickup_budget_ms: u64,
    pub total_budget_ms: u64,
}

impl Default for WarmupOptions {
    fn default() -> Self {
        Self {
            pickup_budget_ms: DEFAULT_PICKUP_BUDGET_MS,
            total_budget_ms: DEFAULT_WARMUP_BUDGET_MS,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WarmupInfraEvent {
    pub run_id: String,
    pub stalled_probe_run_ids: Vec<String>,
    pub picked_up_run_id: Option<String>,
    pub waited_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WarmDeploymentObservation {
    pub started_probe_run_ids: Vec<String>,
    pub cancelled_probe_run_ids: Vec<String>,
    pub cancel_reason: &'static str,
    pub picked_up_run_id: Option<String>,
    pub infra_event: Option<WarmupInfraEvent>,
    pub elapsed_ms: u64,
    pub proceeded_after_budget: bool,
}

/// Warms a deployment through the future Rust runtime while applying one total
/// checked deadline to probe creation, pickup polling, and cancellation.
pub fn warm_deployment(
    probes: &[WarmProbeScript],
    options: WarmupOptions,
) -> WarmDeploymentObservation {
    let _ = (probes, options);
    panic!("{TDD_RED_MARKER}")
}

pub const fn remaining_budget_ms(total_budget_ms: u64, elapsed_ms: u64) -> Option<u64> {
    let remaining = total_budget_ms.saturating_sub(elapsed_ms);
    if remaining == 0 {
        None
    } else {
        Some(remaining)
    }
}

pub const fn next_pickup_budget_ms(
    pickup_budget_ms: u64,
    total_budget_ms: u64,
    elapsed_ms: u64,
) -> Option<u64> {
    match remaining_budget_ms(total_budget_ms, elapsed_ms) {
        Some(remaining) => Some(if pickup_budget_ms < remaining {
            pickup_budget_ms
        } else {
            remaining
        }),
        None => None,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScheduledTrackedRun {
    pub state_name: String,
    pub run_id: String,
    pub delay_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestStateIsolationObservation {
    pub tracked_runs_by_state: BTreeMap<String, Vec<String>>,
    pub completion_order: Vec<String>,
}

/// Exercises the future Rust task-local test state with deliberately
/// interleaved asynchronous completions.
pub fn run_interleaved_test_states(runs: &[ScheduledTrackedRun]) -> TestStateIsolationObservation {
    let _ = runs;
    panic!("{TDD_RED_MARKER}")
}
