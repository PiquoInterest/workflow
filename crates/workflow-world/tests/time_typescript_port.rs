use workflow_world::time::{DurationInput, parse_duration_to_unix_ms};

const NOW: f64 = 1_700_000_000_000.0;

#[test]
fn parses_duration_strings_to_a_future_timestamp() {
    let result = parse_duration_to_unix_ms(DurationInput::String("5s".to_owned()), NOW).unwrap();
    assert!(result > NOW as i64);
}

#[test]
fn parses_numbers_as_milliseconds() {
    let result = parse_duration_to_unix_ms(DurationInput::Milliseconds(1_000.0), NOW).unwrap();
    assert!(result > NOW as i64);
    assert_eq!(result, NOW as i64 + 1_000);
}

#[test]
fn preserves_absolute_date_timestamps() {
    let future = NOW + 5_000.0;
    let result = parse_duration_to_unix_ms(DurationInput::DateMilliseconds(future), NOW).unwrap();
    assert_eq!(result, future as i64);
}

#[test]
fn preserves_valid_date_like_timestamps() {
    let timestamp = NOW + 5_000.0;
    let result =
        parse_duration_to_unix_ms(DurationInput::DateMilliseconds(timestamp), NOW).unwrap();
    assert_eq!(result, timestamp as i64);
}

#[test]
fn rejects_invalid_duration_strings() {
    assert!(
        parse_duration_to_unix_ms(DurationInput::String("invalid".to_owned()), NOW).is_err()
    );
}

#[test]
fn rejects_negative_millisecond_durations() {
    assert!(parse_duration_to_unix_ms(DurationInput::Milliseconds(-1_000.0), NOW).is_err());
}

#[test]
fn rejects_invalid_date_values_instead_of_returning_a_poisoned_timestamp() {
    let error =
        parse_duration_to_unix_ms(DurationInput::DateMilliseconds(f64::NAN), NOW).unwrap_err();
    assert!(
        error
            .message()
            .contains("Expected a valid Date with a finite timestamp")
    );
}

#[test]
fn rejects_date_like_values_with_non_finite_timestamps() {
    let error = parse_duration_to_unix_ms(
        DurationInput::DateMilliseconds(f64::INFINITY),
        NOW,
    )
    .unwrap_err();
    assert!(
        error
            .message()
            .contains("Expected a valid Date with a finite timestamp")
    );
}

#[test]
fn rejects_finite_durations_whose_result_overflows_timeclip() {
    let error =
        parse_duration_to_unix_ms(DurationInput::Milliseconds(f64::MAX), NOW).unwrap_err();
    assert!(
        error
            .message()
            .contains("Resulting date is outside the supported range")
    );
}
