use workflow_cli_tdd::output::{
    ApiError, EXPIRED_DATA_MESSAGE, format_table_value,
    get_observability_upgrade_required_message, has_expired_data,
    is_observability_upgrade_required_error,
};

const DAY_MS: i64 = 24 * 60 * 60 * 1_000;
const NOW_MS: i64 = 1_800_000_000_000;

#[test]
fn expired_data_is_false_when_expired_at_is_absent() {
    assert!(!has_expired_data(None, NOW_MS));
}

#[test]
fn expired_data_is_false_when_expired_at_is_in_the_future() {
    assert!(!has_expired_data(Some(NOW_MS + DAY_MS), NOW_MS));
}

#[test]
fn expired_data_is_true_when_expired_at_is_in_the_past() {
    assert!(has_expired_data(Some(NOW_MS - DAY_MS), NOW_MS));
}

#[test]
fn table_value_keeps_input_when_expiration_is_in_the_future() {
    let result = format_table_value("input", "hello", Some(NOW_MS + DAY_MS), NOW_MS);
    assert!(!result.contains("expired"));
    assert_eq!(result, "hello");
}

#[test]
fn table_value_hides_output_when_expiration_is_in_the_past() {
    let result = format_table_value("output", "hello", Some(NOW_MS - DAY_MS), NOW_MS);
    assert!(result.contains("data expired"));
    assert_eq!(result, EXPIRED_DATA_MESSAGE);
}

#[test]
fn table_value_keeps_input_when_expiration_is_absent() {
    let result = format_table_value("input", "hello", None, NOW_MS);
    assert!(!result.contains("expired"));
    assert_eq!(result, "hello");
}

#[test]
fn detects_observability_upgrade_by_top_level_code_on_402() {
    assert!(is_observability_upgrade_required_error(&ApiError {
        status: Some(402),
        code: Some("observability-upgrade-required".to_owned()),
        body_error: None,
    }));
}

#[test]
fn detects_observability_upgrade_by_response_body_on_402() {
    assert!(is_observability_upgrade_required_error(&ApiError {
        status: Some(402),
        code: None,
        body_error: Some("observability-upgrade-required".to_owned()),
    }));
}

#[test]
fn does_not_treat_404_as_an_upgrade_prompt() {
    assert!(!is_observability_upgrade_required_error(&ApiError {
        status: Some(404),
        code: Some("observability-upgrade-required".to_owned()),
        body_error: None,
    }));
}

#[test]
fn upgrade_message_names_observability_plus() {
    assert!(
        get_observability_upgrade_required_message().contains("Upgrade Observability Plus")
    );
}
