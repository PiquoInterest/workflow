use std::collections::BTreeMap;
use std::process::Command;

use workflow_world::env::{
    DEFAULT_MAX_EVENTS_PER_RUN, EnvNumberOptions, env_flag, env_flag_from, env_number_from,
    max_events_per_run_from, reset_env_warning_cache_for_tests,
};

const NAME: &str = "WORKFLOW_TEST_ENV_CONFIG_FIXTURE";
const CHILD_MARKER: &str = "WORKFLOW_ENV_PROBE_CHILD";

fn environment(value: &str) -> BTreeMap<String, String> {
    BTreeMap::from([(NAME.to_owned(), value.to_owned())])
}

fn number(value: Option<&str>, fallback: f64, options: EnvNumberOptions) -> f64 {
    let environment = value.map(environment).unwrap_or_default();
    env_number_from(NAME, fallback, options, &environment)
}

#[test]
fn env_number_returns_the_fallback_when_unset() {
    assert_eq!(number(None, 100.0, EnvNumberOptions::default()), 100.0);
}

#[test]
fn env_number_returns_the_fallback_for_an_empty_string() {
    assert_eq!(number(Some(""), 100.0, EnvNumberOptions::default()), 100.0);
}

#[test]
fn env_number_parses_a_number() {
    assert_eq!(number(Some("42"), 100.0, EnvNumberOptions::default()), 42.0);
}

#[test]
fn env_number_parses_a_decimal_and_respects_a_maximum() {
    assert_eq!(
        number(
            Some("0.05"),
            100.0,
            EnvNumberOptions {
                max: Some(1.0),
                ..EnvNumberOptions::default()
            },
        ),
        0.05,
    );
}

#[test]
fn env_number_falls_back_when_integer_mode_receives_a_fraction() {
    assert_eq!(
        number(
            Some("2.5"),
            100.0,
            EnvNumberOptions {
                integer: true,
                ..EnvNumberOptions::default()
            },
        ),
        100.0,
    );
}

#[test]
fn env_number_falls_back_for_non_numeric_and_non_finite_input() {
    for raw in ["abc", "Infinity", "NaN"] {
        assert_eq!(number(Some(raw), 100.0, EnvNumberOptions::default()), 100.0);
    }
}

#[test]
fn env_number_clamps_to_the_default_or_explicit_minimum() {
    assert_eq!(number(Some("-5"), 100.0, EnvNumberOptions::default()), 0.0);
    assert_eq!(
        number(
            Some("5"),
            100.0,
            EnvNumberOptions {
                min: 10.0,
                ..EnvNumberOptions::default()
            },
        ),
        10.0,
    );
}

#[test]
fn env_number_clamps_to_the_maximum() {
    assert_eq!(
        number(
            Some("9999"),
            100.0,
            EnvNumberOptions {
                max: Some(500.0),
                ..EnvNumberOptions::default()
            },
        ),
        500.0,
    );
}

#[test]
fn repeated_invalid_env_numbers_return_the_same_fallback() {
    reset_env_warning_cache_for_tests();
    assert_eq!(number(Some("nope"), 100.0, EnvNumberOptions::default()), 100.0);
    assert_eq!(number(Some("nope"), 100.0, EnvNumberOptions::default()), 100.0);
}

#[test]
fn env_number_matches_javascript_number_radix_and_whitespace_forms() {
    assert_eq!(number(Some("0x10"), 100.0, EnvNumberOptions::default()), 16.0);
    assert_eq!(number(Some("0b10"), 100.0, EnvNumberOptions::default()), 2.0);
    assert_eq!(number(Some("0o10"), 100.0, EnvNumberOptions::default()), 8.0);
    assert_eq!(number(Some("   "), 100.0, EnvNumberOptions::default()), 0.0);
}

#[test]
fn max_events_uses_the_default_for_zero_and_negative_values() {
    for raw in ["0", "-1"] {
        let environment = BTreeMap::from([("WORKFLOW_MAX_EVENTS".to_owned(), raw.to_owned())]);
        assert_eq!(max_events_per_run_from(&environment), DEFAULT_MAX_EVENTS_PER_RUN);
    }
}

#[test]
fn env_flag_returns_the_fallback_when_unset_or_empty() {
    assert!(env_flag_from(NAME, true, &BTreeMap::new()));
    assert!(!env_flag_from(NAME, false, &BTreeMap::new()));
    assert!(env_flag_from(NAME, true, &environment("")));
    assert!(!env_flag_from(NAME, false, &environment("")));
}

#[test]
fn env_flag_accepts_zero_false_one_and_true_case_insensitively() {
    for raw in ["0", "false", "FALSE", "False"] {
        assert!(!env_flag_from(NAME, true, &environment(raw)));
    }
    for raw in ["1", "true", "TRUE", "True"] {
        assert!(env_flag_from(NAME, false, &environment(raw)));
    }
}

#[test]
fn env_flag_falls_back_for_unrecognized_values() {
    for raw in ["yes", "on", "2", "off"] {
        assert!(env_flag_from(NAME, true, &environment(raw)));
        assert!(!env_flag_from(NAME, false, &environment(raw)));
    }
}

#[test]
fn process_environment_probe_child() {
    if std::env::var(CHILD_MARKER).ok().as_deref() != Some("1") {
        return;
    }
    assert!(!env_flag(NAME, true));
}

#[test]
fn env_flag_reads_the_process_environment_when_no_map_is_supplied() {
    let output = Command::new(std::env::current_exe().unwrap())
        .arg("--exact")
        .arg("process_environment_probe_child")
        .arg("--nocapture")
        .env(CHILD_MARKER, "1")
        .env(NAME, "0")
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "child test failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}
