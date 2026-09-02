use workflow_world_tdd::{
    AnalyticsEventInput, AnalyticsRunInput, DateInput, NullableDateInput, NullableTimestamp,
    UtcTimestamp, parse_analytics_event, parse_analytics_run,
};

fn base_run(created_at: DateInput) -> AnalyticsRunInput {
    AnalyticsRunInput {
        run_id: "wrun_01KX2M5N3RBNC12RYWYYH4WWQJ".to_owned(),
        status: "completed".to_owned(),
        deployment_id: "dpl_1".to_owned(),
        workflow_name: "workflow//./src/w//myWorkflow".to_owned(),
        created_at,
        updated_at: DateInput::Text("2026-07-13 17:09:11.593".to_owned()),
        started_at: NullableDateInput::Missing,
        completed_at: NullableDateInput::Missing,
    }
}

fn expected_timestamp(unix_millis: i64, iso8601: &str) -> UtcTimestamp {
    UtcTimestamp {
        unix_millis,
        iso8601: iso8601.to_owned(),
    }
}

#[test]
fn preserves_distinct_event_provenance_fields() {
    let event = parse_analytics_event(AnalyticsEventInput {
        run_id: "wrun_01KX2M5N3RBNC12RYWYYH4WWQJ".to_owned(),
        event_id: "evnt_01KX2M5N3RBNC12RYWYYH4WWQJ".to_owned(),
        event_type: "run_started".to_owned(),
        workflow_name: "test-workflow".to_owned(),
        deployment_id: "dpl_1".to_owned(),
        run_created_at: DateInput::Text("2026-07-13 17:09:11.000".to_owned()),
        created_at: DateInput::Text("2026-07-13 17:09:11.593".to_owned()),
        vercel_id: Some("request-grain-id".to_owned()),
        request_id: Some("sibling-request-column".to_owned()),
        compute_instance_id: Some("compute-instance-id".to_owned()),
    })
    .unwrap();

    assert_eq!(event.vercel_id.as_deref(), Some("request-grain-id"));
    assert_eq!(event.request_id.as_deref(), Some("sibling-request-column"));
    assert_eq!(
        event.compute_instance_id.as_deref(),
        Some("compute-instance-id")
    );
}

#[test]
fn parses_timezone_naive_datetime_strings_as_utc() {
    let run = parse_analytics_run(base_run(DateInput::Text(
        "2026-07-13 17:09:11.593".to_owned(),
    )))
    .unwrap();

    assert_eq!(
        run.created_at,
        expected_timestamp(1_783_962_551_593, "2026-07-13T17:09:11.593Z")
    );
}

#[test]
fn accepts_naive_strings_without_fractional_seconds() {
    let run =
        parse_analytics_run(base_run(DateInput::Text("2026-07-13 17:09:11".to_owned()))).unwrap();

    assert_eq!(
        run.created_at,
        expected_timestamp(1_783_962_551_000, "2026-07-13T17:09:11.000Z")
    );
}

#[test]
fn preserves_timezone_aware_instants() {
    let offset = parse_analytics_run(base_run(DateInput::Text(
        "2026-07-13T10:09:11.593-07:00".to_owned(),
    )))
    .unwrap();
    let zulu = parse_analytics_run(base_run(DateInput::Text(
        "2026-07-13T17:09:11.593Z".to_owned(),
    )))
    .unwrap();

    let expected = expected_timestamp(1_783_962_551_593, "2026-07-13T17:09:11.593Z");
    assert_eq!(offset.created_at, expected);
    assert_eq!(zulu.created_at, expected);
}

#[test]
fn passes_through_timestamp_objects_and_nullable_fields() {
    let run = parse_analytics_run(AnalyticsRunInput {
        created_at: DateInput::UnixMillis(1_783_962_551_593),
        started_at: NullableDateInput::Value(DateInput::Text("2026-07-13 17:09:12.000".to_owned())),
        completed_at: NullableDateInput::Null,
        ..base_run(DateInput::UnixMillis(1_783_962_551_593))
    })
    .unwrap();

    assert_eq!(
        run.created_at,
        expected_timestamp(1_783_962_551_593, "2026-07-13T17:09:11.593Z")
    );
    assert_eq!(
        run.started_at,
        NullableTimestamp::Value(expected_timestamp(
            1_783_962_552_000,
            "2026-07-13T17:09:12.000Z"
        ))
    );
    assert_eq!(run.completed_at, NullableTimestamp::Null);
}
