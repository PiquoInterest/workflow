use workflow_cli_tdd::bulk_cancel::{
    BulkCancelObservation, BulkCancelParams, BulkCancelRequest, BulkCancelResult,
    BulkCancelRunResult, BulkCancelSummary, CANCELLABLE_STATUSES, CLI_CANCEL_REASON,
    CancelManyBehavior, CreatedCancelEvent, FakeRun, FakeWorld, HAS_MORE_GUIDANCE, RunStatus,
    bulk_cancel_failure_lines, format_bulk_cancel_summary, perform_bulk_cancel,
    validate_bulk_cancel_limit,
};

fn perform(
    world: FakeWorld,
    status: Option<RunStatus>,
    limit: usize,
    confirm: bool,
    prompt_confirmation: Option<bool>,
) -> BulkCancelObservation {
    perform_bulk_cancel(BulkCancelParams {
        world,
        status,
        workflow_name: None,
        limit,
        confirm,
        prompt_confirmation,
    })
}

#[test]
fn accepts_integer_limits_within_one_and_five_hundred() {
    for limit in [1.0, 50.0, 500.0] {
        assert_eq!(validate_bulk_cancel_limit(limit), None);
    }
}

#[test]
fn rejects_out_of_range_and_non_integer_limits() {
    for limit in [0.0, 501.0, -5.0, 1.5] {
        let message = validate_bulk_cancel_limit(limit).unwrap();
        assert!(message.contains("between 1 and 500"));
    }
}

#[test]
fn renders_every_summary_outcome_category() {
    let result = BulkCancelResult {
        summary: BulkCancelSummary {
            requested: 7,
            cancelled: 2,
            already_cancelled: 1,
            not_cancellable: 1,
            not_found: 3,
            failed: 0,
        },
        results: Vec::new(),
    };
    assert_eq!(
        format_bulk_cancel_summary(&result),
        "Done:\n  2 cancelled\n  1 already cancelled\n  1 not cancellable\n  3 not found\n  0 failed"
    );
}

#[test]
fn surfaces_only_not_found_not_cancellable_and_failed_runs() {
    let result = BulkCancelResult {
        summary: BulkCancelSummary {
            requested: 5,
            cancelled: 1,
            already_cancelled: 1,
            not_cancellable: 1,
            not_found: 1,
            failed: 1,
        },
        results: vec![
            BulkCancelRunResult::cancelled("a"),
            BulkCancelRunResult::AlreadyCancelled {
                run_id: "b".to_owned(),
            },
            BulkCancelRunResult::NotCancellable {
                run_id: "c".to_owned(),
                status: "completed".to_owned(),
            },
            BulkCancelRunResult::NotFound {
                run_id: "d".to_owned(),
            },
            BulkCancelRunResult::Failed {
                run_id: "e".to_owned(),
                code: "internal_error".to_owned(),
                retryable: true,
            },
        ],
    };
    assert_eq!(
        bulk_cancel_failure_lines(&result),
        vec![
            "  ✗ c: not cancellable (completed)",
            "  ✗ d: not found",
            "  ✗ e: failed (internal_error, retryable)",
        ]
    );
}

#[test]
fn warns_and_exits_zero_when_no_runs_match() {
    let observation = perform(FakeWorld::new(Vec::new()), None, 50, true, None);
    assert_eq!(observation.exit_code, 0);
    assert_eq!(observation.result, None);
    assert!(
        observation
            .warns
            .iter()
            .any(|line| line == "No matching runs found.")
    );
}

#[test]
fn restricts_unpinned_queries_to_cancellable_statuses_and_excludes_terminal_runs() {
    let mut world = FakeWorld::new(vec![
        FakeRun::new("p1", RunStatus::Pending),
        FakeRun::new("r1", RunStatus::Running),
        FakeRun::new("done", RunStatus::Completed),
    ]);
    world.cancel_many = CancelManyBehavior::CancelAll;

    let observation = perform(world, None, 50, true, None);
    assert_eq!(observation.exit_code, 0);
    assert_eq!(
        observation.requested_statuses,
        CANCELLABLE_STATUSES.to_vec()
    );
    assert!(
        !observation
            .requested_statuses
            .contains(&RunStatus::Completed)
    );
    assert_eq!(observation.cancel_many_requests.len(), 1);
    let mut run_ids = observation.cancel_many_requests[0].run_ids.clone();
    run_ids.sort();
    assert_eq!(run_ids, vec!["p1".to_owned(), "r1".to_owned()]);
}

#[test]
fn round_robins_status_pages_before_applying_the_batch_limit() {
    let mut runs = Vec::new();
    for index in 1..=5 {
        let mut run = FakeRun::new(&format!("r{index}"), RunStatus::Running);
        run.started_at = Some(format!("2026-01-{index:02}T00:00:00.000Z"));
        runs.push(run);
    }
    for index in 1..=3 {
        runs.push(FakeRun::new(&format!("p{index}"), RunStatus::Pending));
    }
    let mut world = FakeWorld::new(runs);
    world.cancel_many = CancelManyBehavior::CancelAll;

    let observation = perform(world, None, 5, true, None);
    assert_eq!(
        observation.cancel_many_requests,
        vec![BulkCancelRequest {
            run_ids: ["p1", "r1", "p2", "r2", "p3"]
                .into_iter()
                .map(str::to_owned)
                .collect(),
            cancel_reason: CLI_CANCEL_REASON.to_owned(),
        }]
    );
}

#[test]
fn uses_the_single_call_cancel_many_fast_path_and_skips_per_run_events() {
    let mut world = FakeWorld::new(vec![
        FakeRun::new("r1", RunStatus::Running),
        FakeRun::new("r2", RunStatus::Running),
    ]);
    world.cancel_many = CancelManyBehavior::Fixed(BulkCancelResult::cancelled(&["r1", "r2"]));

    let observation = perform(world, Some(RunStatus::Running), 50, true, None);
    assert_eq!(observation.exit_code, 0);
    assert_eq!(observation.cancel_many_requests.len(), 1);
    assert_eq!(
        observation.cancel_many_requests[0].run_ids,
        vec!["r1".to_owned(), "r2".to_owned()]
    );
    assert_eq!(
        observation.cancel_many_requests[0].cancel_reason,
        CLI_CANCEL_REASON
    );
    assert!(observation.created_events.is_empty());
}

#[test]
fn falls_back_to_per_run_cancellation_when_cancel_many_is_absent() {
    let world = FakeWorld::new(vec![
        FakeRun::new("r1", RunStatus::Running),
        FakeRun::new("r2", RunStatus::Running),
    ]);

    let observation = perform(world, Some(RunStatus::Running), 50, true, None);
    assert_eq!(observation.exit_code, 0);
    assert_eq!(
        observation.created_events,
        vec![
            CreatedCancelEvent {
                run_id: "r1".to_owned(),
                cancel_reason: CLI_CANCEL_REASON.to_owned(),
            },
            CreatedCancelEvent {
                run_id: "r2".to_owned(),
                cancel_reason: CLI_CANCEL_REASON.to_owned(),
            },
        ]
    );
    assert_eq!(observation.result.as_ref().unwrap().summary.cancelled, 2);
    assert!(observation.logs.join("\n").contains("2 cancelled"));
}

#[test]
fn prints_rerun_guidance_when_more_runs_match_than_were_fetched() {
    let mut world = FakeWorld::new(vec![FakeRun::new("r1", RunStatus::Running)]);
    world.has_more = true;
    world.cancel_many = CancelManyBehavior::CancelAll;

    let observation = perform(world, Some(RunStatus::Running), 1, true, None);
    assert!(
        observation
            .warns
            .iter()
            .any(|line| line == HAS_MORE_GUIDANCE)
    );
}

#[test]
fn exits_one_and_surfaces_per_run_failures() {
    let mut world = FakeWorld::new(vec![
        FakeRun::new("r1", RunStatus::Running),
        FakeRun::new("r2", RunStatus::Running),
    ]);
    world.cancel_many = CancelManyBehavior::Fixed(BulkCancelResult {
        summary: BulkCancelSummary {
            requested: 2,
            cancelled: 1,
            failed: 1,
            ..BulkCancelSummary::default()
        },
        results: vec![
            BulkCancelRunResult::cancelled("r1"),
            BulkCancelRunResult::Failed {
                run_id: "r2".to_owned(),
                code: "internal_error".to_owned(),
                retryable: true,
            },
        ],
    });

    let observation = perform(world, Some(RunStatus::Running), 50, true, None);
    assert_eq!(observation.exit_code, 1);
    assert!(
        observation
            .warns
            .iter()
            .any(|line| line == "  ✗ r2: failed (internal_error, retryable)")
    );
}

#[test]
fn aborts_without_cancelling_when_confirmation_is_declined() {
    let mut world = FakeWorld::new(vec![FakeRun::new("r1", RunStatus::Running)]);
    world.cancel_many = CancelManyBehavior::CancelAll;

    let observation = perform(world, Some(RunStatus::Running), 50, false, Some(false));
    assert_eq!(observation.exit_code, 0);
    assert!(observation.logs.iter().any(|line| line == "Aborted."));
    assert!(observation.cancel_many_requests.is_empty());
    assert!(observation.created_events.is_empty());
}
