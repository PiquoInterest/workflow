use workflow_world_tdd::{
    CreateEventInput, DateInput, EventDataInput, StoredEventInput, UtcTimestamp,
    parse_create_event, parse_stored_event,
};

fn create_event(event_type: &str) -> CreateEventInput {
    CreateEventInput {
        event_type: event_type.to_owned(),
        correlation_id: None,
        spec_version: None,
        event_data: None,
    }
}

fn stored_event(event_type: &str) -> StoredEventInput {
    StoredEventInput {
        event_type: event_type.to_owned(),
        run_id: "wrun_00000000000000000000000000".to_owned(),
        event_id: "evnt_00000000000000000000000000".to_owned(),
        correlation_id: None,
        created_at: DateInput::Text("2026-08-01T00:00:00.000Z".to_owned()),
        spec_version: None,
        event_data: None,
    }
}

#[test]
fn hook_created_coerces_token_retention_until_to_a_timestamp() {
    let parsed = parse_create_event(CreateEventInput {
        event_type: "hook_created".to_owned(),
        correlation_id: Some("hook_1".to_owned()),
        spec_version: Some(5),
        event_data: Some(EventDataInput {
            token: Some("order:123".to_owned()),
            token_retention_until: Some(DateInput::Text("2026-08-01T00:00:00.000Z".to_owned())),
            ..EventDataInput::default()
        }),
    })
    .unwrap();

    assert_eq!(parsed.event_type, "hook_created");
    assert_eq!(
        parsed
            .event_data
            .and_then(|data| data.token_retention_until),
        Some(UtcTimestamp {
            unix_millis: 1_785_542_400_000,
            iso8601: "2026-08-01T00:00:00.000Z".to_owned(),
        })
    );
}

#[test]
fn accepts_a_bare_legacy_step_started_event() {
    let parsed = parse_create_event(CreateEventInput {
        event_type: "step_started".to_owned(),
        correlation_id: Some("step_00000000000000000000000000".to_owned()),
        spec_version: Some(4),
        event_data: None,
    })
    .unwrap();

    assert_eq!(parsed.event_type, "step_started");
}

#[test]
fn accepts_owner_message_id_on_step_started_create_requests() {
    let parsed = parse_create_event(CreateEventInput {
        event_type: "step_started".to_owned(),
        correlation_id: Some("step_00000000000000000000000000".to_owned()),
        spec_version: Some(4),
        event_data: Some(EventDataInput {
            step_name: Some("step//file//fn".to_owned()),
            owner_message_id: Some("msg_abc123".to_owned()),
            ..EventDataInput::default()
        }),
    })
    .unwrap();

    assert_eq!(
        parsed
            .event_data
            .and_then(|data| data.owner_message_id)
            .as_deref(),
        Some("msg_abc123")
    );
}

#[test]
fn retains_owner_message_id_on_stored_step_started_events() {
    let parsed = parse_stored_event(StoredEventInput {
        event_type: "step_started".to_owned(),
        correlation_id: Some("step_00000000000000000000000000".to_owned()),
        spec_version: Some(4),
        event_data: Some(EventDataInput {
            step_name: Some("step//file//fn".to_owned()),
            owner_message_id: Some("msg_abc123".to_owned()),
            ..EventDataInput::default()
        }),
        ..stored_event("step_started")
    })
    .unwrap();

    assert_eq!(
        parsed
            .event_data
            .and_then(|data| data.owner_message_id)
            .as_deref(),
        Some("msg_abc123")
    );
}

#[test]
fn accepts_run_cancelled_without_event_data() {
    let parsed = parse_create_event(CreateEventInput {
        spec_version: Some(4),
        ..create_event("run_cancelled")
    })
    .unwrap();

    assert_eq!(parsed.event_type, "run_cancelled");
}

#[test]
fn accepts_optional_cancel_reason_on_run_cancelled() {
    let parsed = parse_create_event(CreateEventInput {
        event_type: "run_cancelled".to_owned(),
        spec_version: Some(4),
        event_data: Some(EventDataInput {
            cancel_reason: Some("superseded by newer run".to_owned()),
            ..EventDataInput::default()
        }),
        correlation_id: None,
    })
    .unwrap();

    assert_eq!(
        parsed
            .event_data
            .and_then(|data| data.cancel_reason)
            .as_deref(),
        Some("superseded by newer run")
    );
}

#[test]
fn rejects_cancel_reason_longer_than_512_characters() {
    let result = parse_create_event(CreateEventInput {
        event_type: "run_cancelled".to_owned(),
        spec_version: Some(4),
        event_data: Some(EventDataInput {
            cancel_reason: Some("x".repeat(513)),
            ..EventDataInput::default()
        }),
        correlation_id: None,
    });

    assert!(result.is_err());
}

#[test]
fn retains_cancel_reason_on_stored_run_cancelled_events() {
    let parsed = parse_stored_event(StoredEventInput {
        event_type: "run_cancelled".to_owned(),
        spec_version: Some(4),
        event_data: Some(EventDataInput {
            cancel_reason: Some("operator cancelled".to_owned()),
            ..EventDataInput::default()
        }),
        ..stored_event("run_cancelled")
    })
    .unwrap();

    assert_eq!(
        parsed
            .event_data
            .and_then(|data| data.cancel_reason)
            .as_deref(),
        Some("operator cancelled")
    );
}

#[test]
fn parses_a_backend_sealed_noop_event() {
    let parsed = parse_stored_event(StoredEventInput {
        event_type: "noop".to_owned(),
        event_id: "evnt_00000000000000000000000003".to_owned(),
        spec_version: Some(7),
        event_data: Some(EventDataInput {
            sealed: Some(true),
            ..EventDataInput::default()
        }),
        ..stored_event("noop")
    })
    .unwrap();

    assert_eq!(parsed.event_type, "noop");
}

#[test]
fn parses_a_noop_without_event_data() {
    let parsed = parse_stored_event(StoredEventInput {
        event_type: "noop".to_owned(),
        event_id: "evnt_00000000000000000000000003".to_owned(),
        ..stored_event("noop")
    })
    .unwrap();

    assert_eq!(parsed.event_type, "noop");
}

#[test]
fn rejects_user_created_noop_events() {
    assert!(parse_create_event(create_event("noop")).is_err());
}
