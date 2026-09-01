use workflow_core_tdd::logger::{LoggerScenario, observe_logger_scenario};

fn first_error(scenario: LoggerScenario) -> String {
    let observation = observe_logger_scenario(scenario);
    assert_eq!(observation.error_calls.len(), 1);
    assert_eq!(observation.error_calls[0].arguments.len(), 1);
    observation.error_calls[0].arguments[0].clone()
}

#[test]
fn errors_use_console_error_once_with_prefix_and_unknown_fields() {
    let output = first_error(LoggerScenario::ErrorWithUnknownFields);
    assert!(output.contains("[workflow-sdk] boom"));
    assert!(output.contains("foo"));
    assert!(output.contains("bar"));
}

#[test]
fn warnings_use_console_warn_with_the_runtime_prefix() {
    let observation = observe_logger_scenario(LoggerScenario::Warning);
    assert_eq!(observation.warning_calls.len(), 1);
    let output = &observation.warning_calls[0].arguments[0];
    assert!(output.contains("[workflow-sdk] watch out"));
    assert!(output.contains("foo"));
}

#[test]
fn info_and_debug_are_quiet_by_default() {
    let observation = observe_logger_scenario(LoggerScenario::QuietInfoAndDebug);
    assert!(observation.error_calls.is_empty());
    assert!(observation.warning_calls.is_empty());
    assert!(observation.debug_calls.is_empty());
}

#[test]
fn build_debug_uses_the_workflow_build_namespace_and_honors_exclusion() {
    let observation = observe_logger_scenario(LoggerScenario::BuildDebugNamespace);
    assert_eq!(observation.debug_calls.len(), 1);
    assert_eq!(
        observation.debug_calls[0].arguments,
        vec!["[workflow:build] verbose".to_owned(), String::new()]
    );
}

#[test]
fn child_logger_merges_parent_and_call_site_metadata() {
    let output = first_error(LoggerScenario::ChildMetadataMerge);
    assert!(output.contains("run-1"));
    assert!(output.contains("step-1"));
}

#[test]
fn call_site_metadata_overrides_parent_metadata() {
    let output = first_error(LoggerScenario::CallSiteOverride);
    assert!(output.contains("override"));
    assert!(!output.contains("parent-id"));
}

#[test]
fn child_loggers_can_be_chained() {
    let output = first_error(LoggerScenario::ChainedChild);
    assert!(output.contains("run-1"));
    assert!(output.contains("step-1"));
}

#[test]
fn for_run_attaches_run_id_and_parsed_workflow_name() {
    let output = first_error(LoggerScenario::ForRunWithWorkflowName);
    assert!(output.contains("run-1"));
    assert!(output.contains("myWorkflow (./src/jobs)"));
}

#[test]
fn for_run_without_workflow_name_still_attaches_the_run_id() {
    let output = first_error(LoggerScenario::ForRunWithoutWorkflowName);
    assert!(output.contains("run-1"));
}

#[test]
fn for_run_accepts_extra_metadata() {
    let output = first_error(LoggerScenario::ForRunWithExtraMetadata);
    assert!(output.contains("run-1"));
    assert!(output.contains("step-1"));
}

#[test]
fn no_metadata_emits_only_the_prefix_line() {
    assert_eq!(first_error(LoggerScenario::NoMetadata), "[workflow-sdk] boom");
}

#[test]
fn scoped_step_failure_snapshot_is_exact() {
    assert_eq!(
        first_error(LoggerScenario::StepFailureSnapshot),
        "[workflow-sdk] Step \"step//my-step\" threw a FatalError\n  user error · FatalError\n  run    wrun_123\n  step   step_456\n  hint: Move the call to a step function."
    );
}

#[test]
fn max_retries_snapshot_is_exact() {
    assert_eq!(
        first_error(LoggerScenario::MaxRetriesSnapshot),
        "[workflow-sdk] Step \"step//doWork\" hit max retries — bubbling error thrown by your step to the parent workflow\n  user error · Error\n  run    wrun_abc\n  step   step_xyz\n  retry  4 attempts · 3 max retries"
    );
}
