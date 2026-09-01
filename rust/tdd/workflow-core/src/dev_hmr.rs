use std::fmt::{self, Display, Formatter};

pub const PREWARM_FETCH_TIMEOUT_MS: u64 = 5_000;
pub const HMR_QUIESCENCE_QUIET_MS: u64 = 2_000;
pub const TDD_RED_MARKER: &str =
    "TDD RED: packages/core/e2e/dev.test.ts implementation pending";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    Unix,
    Windows,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppKind {
    NextTurbopack,
    NextWebpack,
    Vite,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DevTestConfig {
    pub generated_workflow_path: String,
    pub canary: bool,
    pub platform: Platform,
    pub app: AppKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigError;

impl Display for ConfigError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str("No dev test config provided via parameter or DEV_TEST_CONFIG env var")
    }
}

pub fn resolve_config(
    parameter: Option<DevTestConfig>,
    environment: Option<DevTestConfig>,
) -> Result<DevTestConfig, ConfigError> {
    parameter.or(environment).ok_or(ConfigError)
}

impl DevTestConfig {
    pub fn uses_next_flow_route(&self) -> bool {
        let normalized = self.generated_workflow_path.replace('\\', "/");
        normalized.contains("app/.well-known/workflow/v1/flow/route.js")
    }

    pub fn should_run_next_flow_route_hmr_tests(&self) -> bool {
        self.uses_next_flow_route() && self.platform != Platform::Windows
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DevTimeouts {
    pub step_registration_convergence_ms: u64,
    pub cleanup_hook_ms: u64,
    pub hmr_rediscovery_ms: u64,
    pub hmr_test_ms: u64,
    pub multi_phase_hmr_test_ms: u64,
    pub flow_route_hmr_rediscovery_ms: u64,
    pub flow_route_hmr_fuzz_ms: u64,
}

pub fn dev_timeouts(config: &DevTestConfig) -> DevTimeouts {
    let step_registration_convergence_ms = match config.platform {
        Platform::Windows => 60_000,
        Platform::Unix => 20_000,
    };
    let hmr_rediscovery_ms = if config.canary {
        180_000
    } else if config.platform == Platform::Windows {
        120_000
    } else {
        50_000
    };
    let hmr_test_ms = if config.canary {
        210_000
    } else if config.platform == Platform::Windows {
        140_000
    } else {
        70_000
    };
    let flow_route_hmr_rediscovery_ms = if config.canary {
        if config.app == AppKind::NextWebpack {
            300_000
        } else {
            240_000
        }
    } else {
        hmr_rediscovery_ms
    };

    DevTimeouts {
        step_registration_convergence_ms,
        cleanup_hook_ms: PREWARM_FETCH_TIMEOUT_MS * 4 + step_registration_convergence_ms,
        hmr_rediscovery_ms,
        hmr_test_ms,
        multi_phase_hmr_test_ms: hmr_test_ms + hmr_rediscovery_ms,
        flow_route_hmr_rediscovery_ms,
        flow_route_hmr_fuzz_ms: if config.canary { 480_000 } else { 240_000 },
    }
}

fn decode_utf16le(content: &[u8]) -> String {
    let units = content
        .chunks_exact(2)
        .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
        .collect::<Vec<_>>();
    String::from_utf16_lossy(&units)
}

pub fn decode_dev_server_log(content: &[u8]) -> String {
    if content.starts_with(&[0xff, 0xfe]) {
        return decode_utf16le(content);
    }

    let sample_len = content.len().min(200);
    let null_bytes = content[..sample_len]
        .iter()
        .filter(|byte| **byte == 0)
        .count();
    if sample_len > 0 && null_bytes > sample_len / 4 {
        decode_utf16le(content)
    } else {
        String::from_utf8_lossy(content).into_owned()
    }
}

pub fn count_log_message(log: &str, message: &str) -> usize {
    log.match_indices(message).count()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogCountExpectation {
    Exact(usize),
    Range {
        min: usize,
        max: Option<usize>,
    },
}

impl LogCountExpectation {
    pub fn matches(self, actual: usize, canary: bool) -> bool {
        match self {
            Self::Exact(expected) if canary => actual >= expected,
            Self::Exact(expected) => actual == expected,
            Self::Range { min, max } => actual >= min && max.is_none_or(|max| actual <= max),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct HmrLogCounts {
    pub skip: usize,
    pub hot: usize,
    pub full: usize,
    pub complete: usize,
}

pub fn hmr_pipeline_is_quiescent(counts: HmrLogCounts, quiet_for_ms: u64) -> bool {
    counts.complete >= counts.hot + counts.full && quiet_for_ms >= HMR_QUIESCENCE_QUIET_MS
}

pub fn join_generated_workflow_outputs(outputs: &[Option<String>]) -> Result<String, String> {
    let present = outputs.iter().flatten().cloned().collect::<Vec<_>>();
    if present.is_empty() {
        return Err("Generated workflow outputs were not found".to_owned());
    }
    Ok(present.join("\n"))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeletedFile {
    pub relative_posix_path: String,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CleanupRecovery {
    pub restored: Vec<DeletedFile>,
    pub error: String,
}

pub fn recover_stranded_step_registrations(
    generated_registration_path: &str,
    registration_source: &str,
    deleted: &[DeletedFile],
    timeout_ms: u64,
) -> Option<CleanupRecovery> {
    let restored = deleted
        .iter()
        .filter(|file| registration_source.contains(&file.relative_posix_path))
        .cloned()
        .collect::<Vec<_>>();
    if restored.is_empty() {
        return None;
    }
    let paths = restored
        .iter()
        .map(|file| file.relative_posix_path.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    Some(CleanupRecovery {
        restored,
        error: format!(
            "Deleted workflow files are still imported by {generated_registration_path} after {timeout_ms}ms: {paths}. The dev server missed the deletion, so the flow route would 500 for every later request. The files have been restored to keep the dev server usable."
        ),
    })
}

pub fn poll_timeout_error(description: &str, timeout_ms: u64, last_error: Option<&str>) -> String {
    match last_error {
        Some(error) => format!(
            "Timed out after {timeout_ms}ms waiting for {description}. Last error: {error}"
        ),
        None => format!("Timed out after {timeout_ms}ms waiting for {description}."),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DevHmrCase {
    NextPageBodyOnly,
    NextPageDirectiveAdded,
    RegistryImportChanged,
    WorkflowChanged,
    StepChanged,
    ViteStepLogicUpdated,
    WorkflowFileAdded,
    SourceMapWarningSuppressed,
    FuzzStepBody,
    FuzzStepHelper,
    FuzzWorkflowBody,
    FuzzWorkflowHelper,
    FuzzSharedHelper,
    FuzzSerde,
    FuzzWorkflowImportGraph,
    FuzzStepDefinitionAdded,
    FuzzWorkflowDefinitionAdded,
    FuzzWorkflowFileAdded,
    FuzzWorkflowFileRemoved,
    FuzzUnrelatedFileAdded,
    FuzzUnrelatedFileRemoved,
}

pub fn case_is_enabled(case: DevHmrCase, config: &DevTestConfig) -> bool {
    match case {
        DevHmrCase::NextPageBodyOnly
        | DevHmrCase::NextPageDirectiveAdded
        | DevHmrCase::FuzzStepBody
        | DevHmrCase::FuzzStepHelper
        | DevHmrCase::FuzzWorkflowBody
        | DevHmrCase::FuzzSharedHelper
        | DevHmrCase::FuzzSerde
        | DevHmrCase::FuzzWorkflowImportGraph
        | DevHmrCase::FuzzStepDefinitionAdded
        | DevHmrCase::FuzzWorkflowDefinitionAdded
        | DevHmrCase::FuzzWorkflowFileAdded
        | DevHmrCase::FuzzWorkflowFileRemoved
        | DevHmrCase::FuzzUnrelatedFileAdded
        | DevHmrCase::FuzzUnrelatedFileRemoved => config.should_run_next_flow_route_hmr_tests(),
        DevHmrCase::FuzzWorkflowHelper => {
            config.should_run_next_flow_route_hmr_tests() && !config.canary
        }
        DevHmrCase::RegistryImportChanged => {
            config.should_run_next_flow_route_hmr_tests() && config.app == AppKind::NextTurbopack
        }
        DevHmrCase::StepChanged => !config.uses_next_flow_route(),
        DevHmrCase::ViteStepLogicUpdated => config.app == AppKind::Vite,
        DevHmrCase::SourceMapWarningSuppressed => config.app == AppKind::NextTurbopack,
        DevHmrCase::WorkflowChanged | DevHmrCase::WorkflowFileAdded => true,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArtifactExpectation {
    Unspecified,
    Unchanged,
    WorkflowHotOnly,
    StepNonDecreasing,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ManifestExpectation {
    pub workflows_present: Vec<String>,
    pub workflows_absent: Vec<String>,
    pub steps_present: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ExecutionExpectation {
    pub exact_before: Option<String>,
    pub exact_after: Option<String>,
    pub step_contains: Option<String>,
    pub workflow_contains: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DevHmrExpectation {
    pub skip: Option<LogCountExpectation>,
    pub hot: Option<LogCountExpectation>,
    pub full: Option<LogCountExpectation>,
    pub artifacts: ArtifactExpectation,
    pub manifest: ManifestExpectation,
    pub execution: ExecutionExpectation,
    pub source_map_warning_absent: bool,
}

fn exact(value: usize) -> Option<LogCountExpectation> {
    Some(LogCountExpectation::Exact(value))
}

pub fn expected_case(case: DevHmrCase) -> DevHmrExpectation {
    let mut expectation = DevHmrExpectation {
        skip: None,
        hot: None,
        full: None,
        artifacts: ArtifactExpectation::Unspecified,
        manifest: ManifestExpectation::default(),
        execution: ExecutionExpectation::default(),
        source_map_warning_absent: false,
    };
    match case {
        DevHmrCase::NextPageBodyOnly => {
            expectation.skip = exact(1);
            expectation.artifacts = ArtifactExpectation::Unchanged;
        }
        DevHmrCase::NextPageDirectiveAdded => {
            expectation.full = exact(1);
            expectation.skip = Some(LogCountExpectation::Range {
                min: 0,
                max: Some(1),
            });
            expectation
                .manifest
                .workflows_present
                .push("hmrPageWorkflow".to_owned());
        }
        DevHmrCase::RegistryImportChanged => expectation
            .manifest
            .workflows_present
            .push("simple".to_owned()),
        DevHmrCase::WorkflowChanged => expectation
            .manifest
            .workflows_present
            .push("myNewWorkflow".to_owned()),
        DevHmrCase::StepChanged => expectation
            .manifest
            .steps_present
            .push("myNewStep".to_owned()),
        DevHmrCase::ViteStepLogicUpdated => {
            expectation.execution.exact_before = Some("before HMR".to_owned());
            expectation.execution.exact_after = Some("after HMR".to_owned());
        }
        DevHmrCase::WorkflowFileAdded => expectation
            .manifest
            .workflows_present
            .push("newWorkflowFile".to_owned()),
        DevHmrCase::SourceMapWarningSuppressed => {
            expectation.source_map_warning_absent = true;
            expectation
                .manifest
                .workflows_present
                .push("sourceMapWarningFixtureWorkflow".to_owned());
        }
        DevHmrCase::FuzzStepBody => {
            expectation.skip = exact(1);
            expectation.execution.step_contains = Some("step-only-1".to_owned());
        }
        DevHmrCase::FuzzStepHelper => {
            expectation.skip = exact(1);
            expectation.execution.step_contains = Some("step-helper-only-2".to_owned());
        }
        DevHmrCase::FuzzWorkflowBody => {
            expectation.hot = exact(1);
            expectation.artifacts = ArtifactExpectation::WorkflowHotOnly;
            expectation.execution.workflow_contains = Some("workflow-body-3".to_owned());
        }
        DevHmrCase::FuzzWorkflowHelper => {
            expectation.hot = exact(1);
            expectation.artifacts = ArtifactExpectation::WorkflowHotOnly;
            expectation.execution.workflow_contains =
                Some("workflow-helper-body-4".to_owned());
        }
        DevHmrCase::FuzzSharedHelper => {
            expectation.hot = exact(1);
            expectation.artifacts = ArtifactExpectation::WorkflowHotOnly;
            expectation.execution.step_contains = Some("shared-body-5".to_owned());
            expectation.execution.workflow_contains = Some("shared-body-5".to_owned());
        }
        DevHmrCase::FuzzSerde => {
            expectation.hot = exact(1);
            expectation.artifacts = ArtifactExpectation::StepNonDecreasing;
        }
        DevHmrCase::FuzzWorkflowImportGraph => {
            expectation.full = exact(1);
            expectation.execution.workflow_contains = Some("imported-stable".to_owned());
        }
        DevHmrCase::FuzzStepDefinitionAdded => {
            expectation.full = exact(1);
            expectation
                .manifest
                .steps_present
                .push("hmrFuzzAddedStep".to_owned());
        }
        DevHmrCase::FuzzWorkflowDefinitionAdded => {
            expectation.full = exact(1);
            expectation
                .manifest
                .workflows_present
                .push("hmrFuzzAddedWorkflow".to_owned());
        }
        DevHmrCase::FuzzWorkflowFileAdded => {
            expectation.full = exact(1);
            expectation
                .manifest
                .workflows_present
                .push("hmrFuzzAddedFileWorkflow".to_owned());
        }
        DevHmrCase::FuzzWorkflowFileRemoved => {
            expectation.full = exact(1);
            expectation.skip = exact(1);
            expectation
                .manifest
                .workflows_absent
                .push("hmrFuzzAddedFileWorkflow".to_owned());
        }
        DevHmrCase::FuzzUnrelatedFileAdded | DevHmrCase::FuzzUnrelatedFileRemoved => {
            expectation.skip = exact(1);
            expectation.artifacts = ArtifactExpectation::Unchanged;
        }
    }
    expectation
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DevHmrObservation {
    pub prewarm_paths: Vec<String>,
    pub fetch_timeout_ms: u64,
    pub log_cursor_opened_after_quiescence: bool,
    pub cleanup_converged: bool,
    pub expectation: DevHmrExpectation,
}

/// Runs one mutation through the future Rust builder and dev-server watcher.
///
/// Final GREEN must mutate real files, observe generated artifacts and manifest
/// state, execute affected workflows where the TypeScript suite does, and run
/// cleanup even when the assertion path fails.
pub fn run_dev_hmr_case(case: DevHmrCase, config: &DevTestConfig) -> DevHmrObservation {
    let _ = (case, config);
    panic!("{TDD_RED_MARKER}")
}
