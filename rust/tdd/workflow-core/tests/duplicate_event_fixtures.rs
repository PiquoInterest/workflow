use workflow_core_tdd::duplicate_event_fixtures::{DuplicateEventSpec, ignored_duplicate_indices};

fn events(values: &[(&str, Option<&str>)]) -> Vec<DuplicateEventSpec> {
    values
        .iter()
        .map(|(event_type, entity)| DuplicateEventSpec::new(event_type, *entity))
        .collect()
}

fn assert_ignored(values: &[(&str, Option<&str>)], expected: &[usize]) {
    assert_eq!(ignored_duplicate_indices(&events(values)), expected);
}

#[test]
fn ignores_a_start_after_its_step_completed() {
    assert_ignored(
        &[
            ("run_created", None),
            ("run_started", None),
            ("step_created", Some("step_a")),
            ("step_started", Some("step_a")),
            ("step_completed", Some("step_a")),
            ("step_started", Some("step_a")),
        ],
        &[5],
    );
}

#[test]
fn consumes_every_retry_attempt_event() {
    assert_ignored(
        &[
            ("step_created", Some("step_a")),
            ("step_started", Some("step_a")),
            ("step_retrying", Some("step_a")),
            ("step_started", Some("step_a")),
            ("step_retrying", Some("step_a")),
            ("step_started", Some("step_a")),
            ("step_completed", Some("step_a")),
        ],
        &[],
    );
}

#[test]
fn consumes_a_second_creation_while_the_step_is_open() {
    assert_ignored(
        &[
            ("step_created", Some("step_a")),
            ("step_created", Some("step_a")),
            ("step_started", Some("step_a")),
        ],
        &[],
    );
}

#[test]
fn ignores_a_second_terminal_outcome_for_one_step() {
    assert_ignored(
        &[
            ("step_created", Some("step_a")),
            ("step_started", Some("step_a")),
            ("step_failed", Some("step_a")),
            ("step_completed", Some("step_a")),
        ],
        &[3],
    );
}

#[test]
fn does_not_hide_a_class_the_log_has_not_recorded() {
    assert_ignored(
        &[
            ("step_created", Some("step_a")),
            ("step_completed", Some("step_a")),
            ("step_started", Some("step_a")),
        ],
        &[],
    );
}

#[test]
fn stops_before_a_repeat_of_an_unrecorded_class() {
    assert_ignored(
        &[
            ("step_created", Some("step_a")),
            ("step_completed", Some("step_a")),
            ("step_started", Some("step_a")),
            ("step_started", Some("step_a")),
        ],
        &[],
    );
}

#[test]
fn ignores_a_sleep_recreated_after_completion() {
    assert_ignored(
        &[
            ("wait_created", Some("wait_a")),
            ("wait_completed", Some("wait_a")),
            ("wait_created", Some("wait_a")),
        ],
        &[2],
    );
}

#[test]
fn consumes_repeated_hook_deliveries() {
    assert_ignored(
        &[
            ("hook_created", Some("hook_a")),
            ("hook_received", Some("hook_a")),
            ("hook_received", Some("hook_a")),
            ("hook_disposed", Some("hook_a")),
        ],
        &[],
    );
}

#[test]
fn ignores_a_second_hook_disposal() {
    assert_ignored(
        &[
            ("hook_created", Some("hook_a")),
            ("hook_disposed", Some("hook_a")),
            ("hook_disposed", Some("hook_a")),
        ],
        &[2],
    );
}

#[test]
fn ignores_a_hook_recreated_after_disposal() {
    assert_ignored(
        &[
            ("hook_created", Some("hook_a")),
            ("hook_disposed", Some("hook_a")),
            ("hook_created", Some("hook_a")),
        ],
        &[2],
    );
}

#[test]
fn keeps_two_sibling_steps_independent() {
    assert_ignored(
        &[
            ("step_created", Some("step_a")),
            ("step_created", Some("step_b")),
            ("step_started", Some("step_a")),
            ("step_started", Some("step_b")),
            ("step_completed", Some("step_b")),
            ("step_completed", Some("step_a")),
        ],
        &[],
    );
}

#[test]
fn ignores_a_second_attribute_write_under_one_id() {
    assert_ignored(
        &[
            ("run_created", None),
            ("run_started", None),
            ("attr_set", Some("attr_a")),
            ("attr_set", Some("attr_a")),
            ("step_created", Some("step_b")),
            ("step_started", Some("step_b")),
            ("step_completed", Some("step_b")),
        ],
        &[3],
    );
}

#[test]
fn consumes_entityless_attribute_writes_from_step_bodies() {
    assert_ignored(
        &[
            ("step_created", Some("step_a")),
            ("step_started", Some("step_a")),
            ("attr_set", None),
            ("attr_set", None),
            ("attr_set", None),
            ("step_completed", Some("step_a")),
        ],
        &[],
    );
}

#[test]
fn ignores_only_the_id_bearing_attribute_repeat_in_a_mixed_log() {
    assert_ignored(
        &[
            ("run_created", None),
            ("run_started", None),
            ("attr_set", None),
            ("attr_set", Some("attr_a")),
            ("attr_set", Some("attr_a")),
            ("attr_set", None),
            ("step_created", Some("step_b")),
        ],
        &[4],
    );
}

#[test]
fn keeps_distinct_attribute_body_positions_independent() {
    assert_ignored(
        &[
            ("attr_set", Some("attr_a")),
            ("attr_set", Some("attr_b")),
            ("step_created", Some("step_c")),
        ],
        &[],
    );
}

#[test]
fn ignores_a_second_run_start() {
    assert_ignored(
        &[
            ("run_created", None),
            ("run_started", None),
            ("run_started", None),
            ("step_created", Some("step_a")),
            ("step_started", Some("step_a")),
        ],
        &[2],
    );
}
