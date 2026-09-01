use workflow_core_tdd::runtime::count_step_started_events::{
    MAX_STEP_ATTEMPT, STEP_ATTEMPT_ADVANCE_ERROR, StepEvent, StepStartScope,
    count_step_started_events, next_step_attempt,
};

const STEP_ID: &str = "step_TARGET";

fn start(owner: Option<&str>) -> StepEvent {
    StepEvent::started(STEP_ID, owner)
}

fn owned(message_id: &str) -> StepStartScope {
    StepStartScope::OwnedBy(message_id.to_owned())
}

macro_rules! advance_case {
    ($name:ident, $prior:expr, $expected:expr) => {
        #[test]
        fn $name() {
            assert_eq!(next_step_attempt($prior), Ok($expected));
        }
    };
}

advance_case!(advances_zero_to_one, 0.0, 1);
advance_case!(advances_one_to_two, 1.0, 2);
advance_case!(
    advances_the_last_safe_prior_without_precision_loss,
    (MAX_STEP_ATTEMPT - 1) as f64,
    MAX_STEP_ATTEMPT
);

macro_rules! reject_case {
    ($name:ident, $prior:expr) => {
        #[test]
        fn $name() {
            let error = next_step_attempt($prior).unwrap_err();
            assert_eq!(error.message, STEP_ATTEMPT_ADVANCE_ERROR);
        }
    };
}

reject_case!(rejects_negative_prior_counts, -1.0);
reject_case!(rejects_fractional_prior_counts, 1.5);
reject_case!(
    rejects_max_safe_integer_as_a_prior_count,
    MAX_STEP_ATTEMPT as f64
);
reject_case!(
    rejects_values_above_max_safe_integer,
    (MAX_STEP_ATTEMPT + 1) as f64
);
reject_case!(rejects_nan_prior_counts, f64::NAN);
reject_case!(rejects_positive_infinity_prior_counts, f64::INFINITY);

#[test]
fn returns_zero_for_absent_and_empty_logs() {
    assert_eq!(
        count_step_started_events(None, STEP_ID, StepStartScope::Unscoped),
        0
    );
    assert_eq!(
        count_step_started_events(Some(&[]), STEP_ID, StepStartScope::Unscoped),
        0
    );
}

#[test]
fn unscoped_count_includes_every_matching_start_only() {
    let events = vec![
        start(Some("msg_A")),
        start(None),
        StepEvent::started("step_OTHER", Some("msg_B")),
        StepEvent::completed(STEP_ID),
    ];
    assert_eq!(
        count_step_started_events(Some(&events), STEP_ID, StepStartScope::Unscoped),
        2
    );
}

#[test]
fn owned_scope_counts_only_the_selected_queue_message() {
    let events = vec![
        start(Some("msg_OWNER")),
        start(Some("msg_RACER_1")),
        start(Some("msg_RACER_2")),
        start(None),
        start(Some("msg_OWNER")),
    ];
    assert_eq!(
        count_step_started_events(Some(&events), STEP_ID, owned("msg_OWNER")),
        2
    );
}

#[test]
fn total_attempts_adds_bare_starts_to_the_largest_single_owner() {
    let events = vec![
        start(Some("msg_OWNER")),
        start(Some("msg_OWNER")),
        start(Some("msg_RACER_1")),
        start(Some("msg_RACER_2")),
        start(None),
    ];
    assert_eq!(
        count_step_started_events(Some(&events), STEP_ID, StepStartScope::TotalAttempts),
        3
    );
}

#[test]
fn racing_invocations_do_not_exhaust_the_owned_recovery_retry_ceiling() {
    let events = vec![
        start(Some("msg_OWNER")),
        start(Some("msg_RACER_1")),
        start(Some("msg_RACER_2")),
        start(None),
    ];
    let max_retries = 3;

    let unscoped = count_step_started_events(Some(&events), STEP_ID, StepStartScope::Unscoped);
    let unscoped_attempt = next_step_attempt(unscoped as f64).unwrap();
    assert!(unscoped_attempt > max_retries + 1);

    let owner_count = count_step_started_events(Some(&events), STEP_ID, owned("msg_OWNER"));
    let owner_attempt = next_step_attempt(owner_count as f64).unwrap();
    assert_eq!(owner_attempt, 2);
    assert!(owner_attempt <= max_retries + 1);

    let total_count =
        count_step_started_events(Some(&events), STEP_ID, StepStartScope::TotalAttempts);
    let total_attempt = next_step_attempt(total_count as f64).unwrap();
    assert_eq!(total_attempt, 3);
    assert!(total_attempt <= max_retries + 1);
}

#[test]
fn real_timeout_retries_by_one_owner_still_trip_the_ceiling() {
    let events = vec![
        start(Some("msg_OWNER")),
        start(Some("msg_OWNER")),
        start(Some("msg_OWNER")),
        start(Some("msg_OWNER")),
    ];
    let count = count_step_started_events(Some(&events), STEP_ID, owned("msg_OWNER"));
    let attempt = next_step_attempt(count as f64).unwrap();
    assert!(attempt > 4);
}

#[test]
fn mixed_owned_then_bare_retries_trip_the_combined_background_ceiling() {
    let events = vec![
        start(Some("msg_OWNER")),
        start(Some("msg_OWNER")),
        start(Some("msg_OWNER")),
        start(None),
        start(None),
    ];
    let count = count_step_started_events(Some(&events), STEP_ID, StepStartScope::TotalAttempts);
    let attempt = next_step_attempt(count as f64).unwrap();
    assert_eq!(attempt, 6);
    assert!(attempt > 4);
}
