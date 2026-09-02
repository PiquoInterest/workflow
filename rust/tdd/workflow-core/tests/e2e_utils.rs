use workflow_core_tdd::e2e_utils::{
    PICKUP_INITIAL_INTERVAL_MS, PickupScript, RunStatus, ScheduledTrackedRun, SourceMapEnvironment,
    StatusRead, WARMUP_CANCEL_REASON, WarmProbeScript, WarmupOptions, has_step_source_maps,
    is_local_deployment, next_pickup_budget_ms, remaining_budget_ms, run_interleaved_test_states,
    wait_for_run_pickup, warm_deployment,
};

fn source_maps(app_name: &str, deployment_url: &str, dev_test_config: bool) -> bool {
    has_step_source_maps(&SourceMapEnvironment {
        app_name: app_name.to_owned(),
        deployment_url: Some(deployment_url.to_owned()),
        dev_test_config,
    })
}

#[test]
fn enables_step_source_maps_for_local_vite_dev_mode() {
    assert!(source_maps("vite", "http://localhost:3000", true));
}

#[test]
fn disables_step_source_maps_for_vercel_even_with_dev_config() {
    assert!(!source_maps("vite", "https://example.vercel.app", true));
}

#[test]
fn disables_step_source_maps_for_local_production_builds() {
    assert!(!source_maps("vite", "http://localhost:3000", false));
}

#[test]
fn disables_step_source_maps_for_local_nest_dev_mode() {
    assert!(!source_maps("nest", "http://localhost:3000", true));
}

#[test]
fn disables_step_source_maps_for_local_turbopack_dev_mode() {
    assert!(!source_maps(
        "nextjs-turbopack",
        "http://localhost:3000",
        true
    ));
}

#[test]
fn enables_step_source_maps_for_local_next_webpack_dev_mode() {
    assert!(source_maps("nextjs-webpack", "http://localhost:3000", true));
}

#[test]
fn local_deployment_requires_an_exact_local_host_not_a_substring() {
    assert!(is_local_deployment(Some("http://localhost:3000")));
    assert!(is_local_deployment(Some("http://127.0.0.1:3000/path")));
    assert!(!is_local_deployment(Some(
        "https://localhost.attacker.example"
    )));
    assert!(!is_local_deployment(Some(
        "https://example.test/path?redirect=localhost"
    )));
    assert!(!is_local_deployment(None));
}

#[test]
fn returns_true_when_run_leaves_pending_before_the_budget() {
    let observation = wait_for_run_pickup(
        &PickupScript::new(
            "run-1",
            vec![
                StatusRead::Status(RunStatus::Pending),
                StatusRead::Status(RunStatus::Running),
            ],
        ),
        1_000,
    );
    assert!(observation.picked_up);
    assert_eq!(observation.status_reads, 2);
    assert_eq!(observation.sleep_intervals_ms, vec![500]);
    assert_eq!(observation.elapsed_ms, 500);
}

#[test]
fn any_non_pending_status_counts_as_pickup() {
    let observation = wait_for_run_pickup(
        &PickupScript::new("run-1", vec![StatusRead::Status(RunStatus::Completed)]),
        1_000,
    );
    assert!(observation.picked_up);
    assert_eq!(observation.status_reads, 1);
    assert!(observation.sleep_intervals_ms.is_empty());
    assert_eq!(observation.elapsed_ms, 0);
}

#[test]
fn returns_false_when_the_run_stays_pending_for_the_whole_budget() {
    let observation = wait_for_run_pickup(
        &PickupScript::new(
            "run-1",
            vec![
                StatusRead::Status(RunStatus::Pending),
                StatusRead::Status(RunStatus::Pending),
                StatusRead::Status(RunStatus::Pending),
            ],
        ),
        1_200,
    );
    assert!(!observation.picked_up);
    assert_eq!(observation.sleep_intervals_ms, vec![500, 700]);
    assert_eq!(observation.elapsed_ms, 1_200);
}

#[test]
fn transient_status_read_failures_are_retried_inside_the_same_budget() {
    let observation = wait_for_run_pickup(
        &PickupScript::new(
            "run-1",
            vec![
                StatusRead::Error("transient".to_owned()),
                StatusRead::Status(RunStatus::Running),
            ],
        ),
        1_000,
    );
    assert!(observation.picked_up);
    assert_eq!(observation.status_reads, 2);
    assert_eq!(observation.sleep_intervals_ms, vec![500]);
}

#[test]
fn short_budget_caps_the_first_sleep() {
    let observation = wait_for_run_pickup(
        &PickupScript::new("run-1", vec![StatusRead::Status(RunStatus::Pending)]),
        100,
    );
    assert!(!observation.picked_up);
    assert_eq!(observation.status_reads, 1);
    assert_eq!(observation.sleep_intervals_ms, vec![100]);
    assert_eq!(observation.elapsed_ms, 100);
}

#[test]
fn pickup_schedule_starts_at_five_hundred_milliseconds() {
    assert_eq!(PICKUP_INITIAL_INTERVAL_MS, 500);
}

#[test]
fn abandons_a_stuck_probe_and_retries_until_one_is_picked_up() {
    let probes = [
        WarmProbeScript {
            run_id: "probe-1".to_owned(),
            start_delay_ms: 0,
            pickup: PickupScript::new("probe-1", vec![StatusRead::Status(RunStatus::Pending)]),
        },
        WarmProbeScript {
            run_id: "probe-2".to_owned(),
            start_delay_ms: 0,
            pickup: PickupScript::new("probe-2", vec![StatusRead::Status(RunStatus::Running)]),
        },
    ];
    let observation = warm_deployment(
        &probes,
        WarmupOptions {
            pickup_budget_ms: 500,
            total_budget_ms: 5_000,
        },
    );
    assert_eq!(
        observation.started_probe_run_ids,
        vec!["probe-1".to_owned(), "probe-2".to_owned()]
    );
    assert_eq!(
        observation.cancelled_probe_run_ids,
        vec!["probe-1".to_owned()]
    );
    assert_eq!(observation.cancel_reason, WARMUP_CANCEL_REASON);
    assert_eq!(observation.picked_up_run_id.as_deref(), Some("probe-2"));
    let event = observation.infra_event.expect("warmup infra event");
    assert_eq!(event.run_id, "probe-1");
    assert_eq!(event.stalled_probe_run_ids, vec!["probe-1".to_owned()]);
    assert_eq!(event.picked_up_run_id.as_deref(), Some("probe-2"));
    assert!(!observation.proceeded_after_budget);
}

#[test]
fn returns_after_total_budget_when_all_probes_stay_pending() {
    let probes = [
        WarmProbeScript {
            run_id: "probe-1".to_owned(),
            start_delay_ms: 0,
            pickup: PickupScript::new("probe-1", vec![StatusRead::Status(RunStatus::Pending)]),
        },
        WarmProbeScript {
            run_id: "probe-2".to_owned(),
            start_delay_ms: 0,
            pickup: PickupScript::new("probe-2", vec![StatusRead::Status(RunStatus::Pending)]),
        },
    ];
    let observation = warm_deployment(
        &probes,
        WarmupOptions {
            pickup_budget_ms: 500,
            total_budget_ms: 900,
        },
    );
    assert_eq!(
        observation.started_probe_run_ids,
        vec!["probe-1".to_owned(), "probe-2".to_owned()]
    );
    assert_eq!(
        observation.cancelled_probe_run_ids,
        vec!["probe-1".to_owned(), "probe-2".to_owned()]
    );
    assert_eq!(observation.picked_up_run_id, None);
    assert_eq!(observation.elapsed_ms, 900);
    assert!(observation.proceeded_after_budget);
    let event = observation.infra_event.expect("warmup infra event");
    assert_eq!(
        event.stalled_probe_run_ids,
        vec!["probe-1".to_owned(), "probe-2".to_owned()]
    );
    assert_eq!(event.picked_up_run_id, None);
}

#[test]
fn fast_probe_returns_without_cancellation_or_infra_event() {
    let probes = [WarmProbeScript {
        run_id: "probe-1".to_owned(),
        start_delay_ms: 0,
        pickup: PickupScript::new("probe-1", vec![StatusRead::Status(RunStatus::Running)]),
    }];
    let observation = warm_deployment(
        &probes,
        WarmupOptions {
            pickup_budget_ms: 500,
            total_budget_ms: 5_000,
        },
    );
    assert_eq!(
        observation.started_probe_run_ids,
        vec!["probe-1".to_owned()]
    );
    assert!(observation.cancelled_probe_run_ids.is_empty());
    assert_eq!(observation.picked_up_run_id.as_deref(), Some("probe-1"));
    assert_eq!(observation.infra_event, None);
    assert!(!observation.proceeded_after_budget);
}

#[test]
fn total_budget_includes_probe_creation_and_never_yields_a_negative_poll_budget() {
    assert_eq!(remaining_budget_ms(900, 0), Some(900));
    assert_eq!(remaining_budget_ms(900, 899), Some(1));
    assert_eq!(remaining_budget_ms(900, 900), None);
    assert_eq!(remaining_budget_ms(900, 1_200), None);
    assert_eq!(next_pickup_budget_ms(500, 900, 700), Some(200));
    assert_eq!(next_pickup_budget_ms(500, 900, 900), None);
}

#[test]
fn probe_start_that_consumes_the_remaining_budget_must_not_receive_a_status_read() {
    let probes = [WarmProbeScript {
        run_id: "late-probe".to_owned(),
        start_delay_ms: 1_000,
        pickup: PickupScript::new("late-probe", vec![StatusRead::Status(RunStatus::Running)]),
    }];
    let observation = warm_deployment(
        &probes,
        WarmupOptions {
            pickup_budget_ms: 500,
            total_budget_ms: 900,
        },
    );
    assert_eq!(observation.picked_up_run_id, None);
    assert_eq!(observation.elapsed_ms, 900);
    assert!(observation.proceeded_after_budget);
}

#[test]
fn interleaved_test_contexts_keep_run_tracking_isolated() {
    let observation = run_interleaved_test_states(&[
        ScheduledTrackedRun {
            state_name: "test A".to_owned(),
            run_id: "run-a".to_owned(),
            delay_ms: 10,
        },
        ScheduledTrackedRun {
            state_name: "test B".to_owned(),
            run_id: "run-b".to_owned(),
            delay_ms: 0,
        },
    ]);
    assert_eq!(
        observation
            .tracked_runs_by_state
            .get("test A")
            .map(Vec::as_slice),
        Some(["run-a".to_owned()].as_slice())
    );
    assert_eq!(
        observation
            .tracked_runs_by_state
            .get("test B")
            .map(Vec::as_slice),
        Some(["run-b".to_owned()].as_slice())
    );
    assert_eq!(
        observation.completion_order,
        vec!["run-b".to_owned(), "run-a".to_owned()]
    );
}
