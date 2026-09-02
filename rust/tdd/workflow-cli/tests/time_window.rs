use workflow_cli_tdd::time_window::{
    AnalyticsPageInfo, InspectPageResponse, TimeWindow, parse_time_input,
    plan_window_start_from_response, resolve_time_window,
};

const NOW_MS: i64 = 1_783_425_600_000;

#[test]
fn parses_relative_durations_as_that_long_ago() {
    for (input, expected) in [
        ("30m", "2026-07-07T11:30:00.000Z"),
        ("12h", "2026-07-07T00:00:00.000Z"),
        ("7d", "2026-06-30T12:00:00.000Z"),
        ("2w", "2026-06-23T12:00:00.000Z"),
    ] {
        assert_eq!(parse_time_input(input, "since", NOW_MS).unwrap(), expected);
    }
}

#[test]
fn parses_absolute_timestamps() {
    assert_eq!(
        parse_time_input("2026-07-01T00:00:00Z", "since", NOW_MS).unwrap(),
        "2026-07-01T00:00:00.000Z"
    );
}

#[test]
fn invalid_values_name_the_flag_in_the_error() {
    let error = parse_time_input("yesterday-ish", "since", NOW_MS).unwrap_err();
    assert!(error.contains("--since"));
}

#[test]
fn returns_none_when_neither_window_flag_is_given() {
    assert_eq!(resolve_time_window(None, None, NOW_MS).unwrap(), None);
}

#[test]
fn defaults_the_end_to_now_when_only_since_is_given() {
    assert_eq!(
        resolve_time_window(Some("24h"), None, NOW_MS).unwrap(),
        Some(TimeWindow {
            start_time: "2026-07-06T12:00:00.000Z".to_owned(),
            end_time: "2026-07-07T12:00:00.000Z".to_owned(),
        })
    );
}

#[test]
fn rejects_until_without_since() {
    let error = resolve_time_window(None, Some("1h"), NOW_MS).unwrap_err();
    assert!(error.contains("--until requires --since"));
}

#[test]
fn rejects_inverted_windows() {
    let error = resolve_time_window(Some("1h"), Some("2h"), NOW_MS).unwrap_err();
    assert!(error.contains("--since must be earlier than --until"));
}

#[test]
fn extracts_the_plan_window_start_from_page_metadata() {
    let response = InspectPageResponse {
        page_info: Some(AnalyticsPageInfo {
            current_lookback_days: 30,
            max_lookback_days: 30,
            current_window_start: "2026-06-07T12:00:00.000Z".to_owned(),
            max_window_start: "2026-06-07T12:00:00.000Z".to_owned(),
            upgrade_available: false,
        }),
    };

    assert_eq!(
        plan_window_start_from_response(Some(&response)),
        Some("2026-06-07T12:00:00.000Z".to_owned())
    );
}

#[test]
fn returns_none_without_page_metadata_or_a_response() {
    assert_eq!(
        plan_window_start_from_response(Some(&InspectPageResponse::default())),
        None
    );
    assert_eq!(plan_window_start_from_response(None), None);
}
