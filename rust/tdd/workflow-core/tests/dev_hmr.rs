use workflow_core_tdd::dev_hmr::{
    AppKind, ArtifactExpectation, DeletedFile, DevHmrCase, DevTestConfig, HMR_QUIESCENCE_QUIET_MS,
    HmrLogCounts, LogCountExpectation, PREWARM_FETCH_TIMEOUT_MS, Platform, case_is_enabled,
    count_log_message, decode_dev_server_log, dev_timeouts, expected_case,
    hmr_pipeline_is_quiescent, join_generated_workflow_outputs, poll_timeout_error,
    recover_stranded_step_registrations, resolve_config, run_dev_hmr_case,
};

fn config(platform: Platform, app: AppKind, canary: bool, flow_route: bool) -> DevTestConfig {
    DevTestConfig {
        generated_workflow_path: if flow_route {
            "app/.well-known/workflow/v1/flow/route.js".to_owned()
        } else {
            ".workflow/workflows.js".to_owned()
        },
        canary,
        platform,
        app,
    }
}

fn next_stable() -> DevTestConfig {
    config(Platform::Unix, AppKind::NextTurbopack, false, true)
}

fn assert_scenario(case: DevHmrCase, config: DevTestConfig) {
    assert!(case_is_enabled(case, &config));
    let observation = run_dev_hmr_case(case, &config);
    assert_eq!(
        &observation.prewarm_paths,
        &vec!["/".to_owned(), "/api/chat".to_owned()]
    );
    assert_eq!(observation.fetch_timeout_ms, PREWARM_FETCH_TIMEOUT_MS);
    assert!(observation.log_cursor_opened_after_quiescence);
    assert!(observation.cleanup_converged);
    assert_eq!(observation.expectation, expected_case(case));
}

#[test]
fn requires_parameter_or_environment_configuration() {
    let error = resolve_config(None, None).unwrap_err();
    assert_eq!(
        error.to_string(),
        "No dev test config provided via parameter or DEV_TEST_CONFIG env var"
    );
    let explicit = next_stable();
    assert_eq!(
        resolve_config(Some(explicit.clone()), None),
        Ok(explicit.clone())
    );
    assert_eq!(resolve_config(None, Some(explicit.clone())), Ok(explicit));
}

#[test]
fn derives_stable_unix_timeouts() {
    let timeouts = dev_timeouts(&next_stable());
    assert_eq!(timeouts.step_registration_convergence_ms, 20_000);
    assert_eq!(timeouts.cleanup_hook_ms, 40_000);
    assert_eq!(timeouts.hmr_rediscovery_ms, 50_000);
    assert_eq!(timeouts.hmr_test_ms, 70_000);
    assert_eq!(timeouts.multi_phase_hmr_test_ms, 120_000);
    assert_eq!(timeouts.flow_route_hmr_rediscovery_ms, 50_000);
    assert_eq!(timeouts.flow_route_hmr_fuzz_ms, 240_000);
}

#[test]
fn derives_stable_windows_timeouts() {
    let timeouts = dev_timeouts(&config(
        Platform::Windows,
        AppKind::NextTurbopack,
        false,
        true,
    ));
    assert_eq!(timeouts.step_registration_convergence_ms, 60_000);
    assert_eq!(timeouts.cleanup_hook_ms, 80_000);
    assert_eq!(timeouts.hmr_rediscovery_ms, 120_000);
    assert_eq!(timeouts.hmr_test_ms, 140_000);
    assert_eq!(timeouts.multi_phase_hmr_test_ms, 260_000);
}

#[test]
fn derives_canary_flow_route_timeout_variants() {
    let webpack = dev_timeouts(&config(Platform::Unix, AppKind::NextWebpack, true, true));
    assert_eq!(webpack.hmr_rediscovery_ms, 180_000);
    assert_eq!(webpack.hmr_test_ms, 210_000);
    assert_eq!(webpack.multi_phase_hmr_test_ms, 390_000);
    assert_eq!(webpack.flow_route_hmr_rediscovery_ms, 300_000);
    assert_eq!(webpack.flow_route_hmr_fuzz_ms, 480_000);

    let turbopack = dev_timeouts(&config(Platform::Unix, AppKind::NextTurbopack, true, true));
    assert_eq!(turbopack.flow_route_hmr_rediscovery_ms, 240_000);
}

#[test]
fn detects_flow_routes_portably_and_disables_them_on_windows() {
    let unix = next_stable();
    assert!(unix.uses_next_flow_route());
    assert!(unix.should_run_next_flow_route_hmr_tests());

    let windows_path = DevTestConfig {
        generated_workflow_path: r"app\.well-known\workflow\v1\flow\route.js".to_owned(),
        canary: false,
        platform: Platform::Windows,
        app: AppKind::NextTurbopack,
    };
    assert!(windows_path.uses_next_flow_route());
    assert!(!windows_path.should_run_next_flow_route_hmr_tests());
}

#[test]
fn decodes_utf16_bom_null_heuristic_and_utf8_logs() {
    let utf16 = "workflow dev hmr: ready"
        .encode_utf16()
        .flat_map(u16::to_le_bytes)
        .collect::<Vec<_>>();
    let mut with_bom = vec![0xff, 0xfe];
    with_bom.extend_from_slice(&utf16);
    assert!(decode_dev_server_log(&with_bom).contains("workflow dev hmr: ready"));
    assert_eq!(decode_dev_server_log(&utf16), "workflow dev hmr: ready");
    assert_eq!(decode_dev_server_log(b"plain utf8 log"), "plain utf8 log");
}

#[test]
fn applies_exact_stable_and_lower_bound_canary_log_counts() {
    let exact = LogCountExpectation::Exact(1);
    assert!(exact.matches(1, false));
    assert!(!exact.matches(2, false));
    assert!(exact.matches(2, true));
    let range = LogCountExpectation::Range {
        min: 1,
        max: Some(2),
    };
    assert!(range.matches(1, false));
    assert!(range.matches(2, true));
    assert!(!range.matches(3, true));
    assert_eq!(count_log_message("skip; skip; hot", "skip"), 2);
}

#[test]
fn opens_exact_log_windows_only_after_quiescence() {
    let counts = HmrLogCounts {
        hot: 2,
        full: 1,
        complete: 3,
        skip: 4,
    };
    assert!(!hmr_pipeline_is_quiescent(
        counts,
        HMR_QUIESCENCE_QUIET_MS - 1
    ));
    assert!(hmr_pipeline_is_quiescent(counts, HMR_QUIESCENCE_QUIET_MS));
    assert!(!hmr_pipeline_is_quiescent(
        HmrLogCounts {
            complete: 2,
            ..counts
        },
        HMR_QUIESCENCE_QUIET_MS
    ));
}

#[test]
fn rejects_missing_generated_outputs_and_joins_present_outputs() {
    assert_eq!(
        join_generated_workflow_outputs(&[None, None]).unwrap_err(),
        "Generated workflow outputs were not found"
    );
    assert_eq!(
        join_generated_workflow_outputs(&[
            Some("first".to_owned()),
            None,
            Some("second".to_owned()),
        ]),
        Ok("first\nsecond".to_owned())
    );
}

#[test]
fn restores_only_stranded_deleted_steps_and_reports_the_poisoned_route() {
    let deleted = [
        DeletedFile {
            relative_posix_path: "workflows/removed-step.ts".to_owned(),
            content: "export async function removed() {}\n".to_owned(),
        },
        DeletedFile {
            relative_posix_path: "workflows/forgotten-workflow.ts".to_owned(),
            content: "export async function forgotten() {}\n".to_owned(),
        },
    ];
    let recovery = recover_stranded_step_registrations(
        ".workflow/steps.js",
        "import './workflows/removed-step.ts';",
        &deleted,
        20_000,
    )
    .expect("one deleted step remains imported");
    assert_eq!(&recovery.restored, &vec![deleted[0].clone()]);
    assert!(recovery.error.contains(".workflow/steps.js"));
    assert!(recovery.error.contains("workflows/removed-step.ts"));
    assert!(recovery.error.contains("would 500 for every later request"));
    assert!(
        recover_stranded_step_registrations(".workflow/steps.js", "export {};", &deleted, 20_000,)
            .is_none()
    );
}

#[test]
fn bounded_poll_errors_preserve_the_last_diagnostic() {
    assert_eq!(
        poll_timeout_error("manifest convergence", 25_000, Some("still stale")),
        "Timed out after 25000ms waiting for manifest convergence. Last error: still stale"
    );
    assert_eq!(
        poll_timeout_error("manifest convergence", 25_000, None),
        "Timed out after 25000ms waiting for manifest convergence."
    );
}

#[test]
fn preserves_source_conditional_case_availability() {
    let next = next_stable();
    assert!(case_is_enabled(DevHmrCase::NextPageBodyOnly, &next));
    assert!(case_is_enabled(DevHmrCase::RegistryImportChanged, &next));
    assert!(!case_is_enabled(DevHmrCase::StepChanged, &next));
    assert!(!case_is_enabled(DevHmrCase::ViteStepLogicUpdated, &next));

    let vite = config(Platform::Unix, AppKind::Vite, false, false);
    assert!(case_is_enabled(DevHmrCase::StepChanged, &vite));
    assert!(case_is_enabled(DevHmrCase::ViteStepLogicUpdated, &vite));

    let canary = config(Platform::Unix, AppKind::NextTurbopack, true, true);
    assert!(!case_is_enabled(DevHmrCase::FuzzWorkflowHelper, &canary));
    assert!(case_is_enabled(DevHmrCase::FuzzSharedHelper, &canary));
}

#[test]
fn next_page_body_only_is_skipped_without_artifact_rebuilds() {
    assert_scenario(DevHmrCase::NextPageBodyOnly, next_stable());
}

#[test]
fn next_page_directive_change_forces_full_rediscovery() {
    assert_scenario(DevHmrCase::NextPageDirectiveAdded, next_stable());
}

#[test]
fn registry_import_change_keeps_manifest_discovery_readable() {
    assert_scenario(DevHmrCase::RegistryImportChanged, next_stable());
}

#[test]
fn workflow_change_reaches_generated_output_or_manifest() {
    assert_scenario(DevHmrCase::WorkflowChanged, next_stable());
}

#[test]
fn non_flow_route_step_change_reaches_step_registration() {
    assert_scenario(
        DevHmrCase::StepChanged,
        config(Platform::Unix, AppKind::Other, false, false),
    );
}

#[test]
fn vite_executes_updated_step_logic_after_hmr() {
    assert_scenario(
        DevHmrCase::ViteStepLogicUpdated,
        config(Platform::Unix, AppKind::Vite, false, false),
    );
}

#[test]
fn adding_a_workflow_file_updates_discovery() {
    assert_scenario(DevHmrCase::WorkflowFileAdded, next_stable());
}

#[test]
fn next_turbopack_suppresses_dependency_source_map_warnings() {
    assert_scenario(DevHmrCase::SourceMapWarningSuppressed, next_stable());
}

macro_rules! next_fuzz_case {
    ($name:ident, $case:expr) => {
        #[test]
        fn $name() {
            assert_scenario($case, next_stable());
        }
    };
}

next_fuzz_case!(
    step_body_change_is_classified_as_skip,
    DevHmrCase::FuzzStepBody
);
next_fuzz_case!(
    step_helper_change_is_classified_as_skip,
    DevHmrCase::FuzzStepHelper
);
next_fuzz_case!(
    workflow_body_change_is_hot_only,
    DevHmrCase::FuzzWorkflowBody
);
next_fuzz_case!(
    workflow_helper_change_is_hot_only,
    DevHmrCase::FuzzWorkflowHelper
);
next_fuzz_case!(
    shared_helper_change_is_hot_and_updates_both_paths,
    DevHmrCase::FuzzSharedHelper
);
next_fuzz_case!(
    serde_change_is_hot_and_can_refresh_step_output,
    DevHmrCase::FuzzSerde
);
next_fuzz_case!(
    workflow_import_graph_change_forces_full_rediscovery,
    DevHmrCase::FuzzWorkflowImportGraph
);
next_fuzz_case!(
    step_definition_addition_forces_full_rediscovery,
    DevHmrCase::FuzzStepDefinitionAdded
);
next_fuzz_case!(
    workflow_definition_addition_forces_full_rediscovery,
    DevHmrCase::FuzzWorkflowDefinitionAdded
);
next_fuzz_case!(
    api_imported_workflow_file_addition_forces_full_rediscovery,
    DevHmrCase::FuzzWorkflowFileAdded
);
next_fuzz_case!(
    api_imported_workflow_file_removal_forces_full_then_skip,
    DevHmrCase::FuzzWorkflowFileRemoved
);
next_fuzz_case!(
    unrelated_file_addition_is_skipped_without_artifact_changes,
    DevHmrCase::FuzzUnrelatedFileAdded
);
next_fuzz_case!(
    unrelated_file_removal_is_skipped_without_artifact_changes,
    DevHmrCase::FuzzUnrelatedFileRemoved
);

#[test]
fn representative_contracts_keep_the_exact_manifest_and_artifact_expectations() {
    let page = expected_case(DevHmrCase::NextPageBodyOnly);
    assert_eq!(page.artifacts, ArtifactExpectation::Unchanged);
    let directive = expected_case(DevHmrCase::NextPageDirectiveAdded);
    assert_eq!(
        directive.manifest.workflows_present,
        vec!["hmrPageWorkflow".to_owned()]
    );
    let removed = expected_case(DevHmrCase::FuzzWorkflowFileRemoved);
    assert_eq!(
        removed.manifest.workflows_absent,
        vec!["hmrFuzzAddedFileWorkflow".to_owned()]
    );
    let source_map = expected_case(DevHmrCase::SourceMapWarningSuppressed);
    assert!(source_map.source_map_warning_absent);
}
