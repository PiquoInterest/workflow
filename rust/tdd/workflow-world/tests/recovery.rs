use workflow_world_tdd::{ActiveRun, Environment, RecoveryLogLevel, reenqueue_active_runs};

fn active_runs() -> Vec<ActiveRun> {
    vec![ActiveRun {
        run_id: "wrun_AAA".to_owned(),
        workflow_name: "myWorkflow".to_owned(),
    }]
}

#[test]
fn uses_the_environment_queue_namespace_for_recovered_runs() {
    let environment =
        Environment::from([("WORKFLOW_QUEUE_NAMESPACE".to_owned(), "custom".to_owned())]);
    let mut calls = Vec::new();

    let report = reenqueue_active_runs(
        &active_runs(),
        &environment,
        None,
        "test",
        None,
        |queue_name, run_id| {
            calls.push((queue_name.to_owned(), run_id.to_owned()));
            Ok(())
        },
    );

    assert_eq!(
        calls,
        vec![(
            "__custom_wkf_workflow_myWorkflow".to_owned(),
            "wrun_AAA".to_owned()
        )]
    );
    assert_eq!(report.enqueued, 1);
}

#[test]
fn explicit_namespace_takes_precedence_over_the_environment() {
    let environment = Environment::from([(
        "WORKFLOW_QUEUE_NAMESPACE".to_owned(),
        "environment".to_owned(),
    )]);
    let mut calls = Vec::new();

    reenqueue_active_runs(
        &active_runs(),
        &environment,
        Some("explicit"),
        "test",
        None,
        |queue_name, run_id| {
            calls.push((queue_name.to_owned(), run_id.to_owned()));
            Ok(())
        },
    );

    assert_eq!(
        calls,
        vec![(
            "__explicit_wkf_workflow_myWorkflow".to_owned(),
            "wrun_AAA".to_owned()
        )]
    );
}

#[test]
fn successful_recovery_is_silent_without_debug() {
    let report = reenqueue_active_runs(
        &active_runs(),
        &Environment::new(),
        None,
        "test",
        None,
        |_queue_name, _run_id| Ok(()),
    );

    assert!(report.logs.is_empty());
}

#[test]
fn successful_recovery_is_reported_under_workflow_debug() {
    let report = reenqueue_active_runs(
        &active_runs(),
        &Environment::new(),
        None,
        "test",
        Some("workflow:*"),
        |_queue_name, _run_id| Ok(()),
    );

    assert_eq!(report.logs.len(), 1);
    assert_eq!(report.logs[0].level, RecoveryLogLevel::Debug);
    assert_eq!(
        report.logs[0].message,
        "[test] Re-enqueued 1 active run(s) on startup"
    );
}

#[test]
fn enqueue_failures_are_always_reported() {
    let report = reenqueue_active_runs(
        &active_runs(),
        &Environment::new(),
        None,
        "test",
        None,
        |_queue_name, _run_id| Err("nope".to_owned()),
    );

    assert_eq!(report.enqueued, 0);
    assert!(report.logs.iter().any(|entry| {
        entry.level == RecoveryLogLevel::Warn
            && entry.message.contains("Failed to re-enqueue run wrun_AAA")
    }));
}
