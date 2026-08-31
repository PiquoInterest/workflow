use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Mutex, OnceLock};

/// Options for a numeric environment override.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EnvNumberOptions {
    /// Inclusive lower bound. Defaults to zero.
    pub min: f64,
    /// Inclusive upper bound.
    pub max: Option<f64>,
    /// Whether fractional values are invalid.
    pub integer: bool,
}

impl Default for EnvNumberOptions {
    fn default() -> Self {
        Self {
            min: 0.0,
            max: None,
            integer: false,
        }
    }
}

static WARNED_ENV_VALUES: OnceLock<Mutex<BTreeSet<String>>> = OnceLock::new();

fn warning_cache() -> &'static Mutex<BTreeSet<String>> {
    WARNED_ENV_VALUES.get_or_init(|| Mutex::new(BTreeSet::new()))
}

fn warn_once(key: String, message: String) {
    let mut cache = warning_cache()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if cache.insert(key) {
        eprintln!("[workflow] {message}");
    }
}

/// Clears process-global warning state. Intended for tests.
pub fn reset_env_warning_cache_for_tests() {
    warning_cache()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clear();
}

/// Resolves a numeric override from a supplied environment map.
pub fn env_number_from(
    name: &str,
    fallback: f64,
    options: EnvNumberOptions,
    environment: &BTreeMap<String, String>,
) -> f64 {
    let Some(raw) = environment.get(name) else {
        return fallback;
    };
    if raw.is_empty() {
        return fallback;
    }

    let parsed = parse_javascript_number(raw);
    let Some(parsed) = parsed.filter(|value| value.is_finite()) else {
        let expected = if options.integer {
            "finite integer"
        } else {
            "finite number"
        };
        warn_once(
            format!("{name}={raw}"),
            format!("Ignoring {name}: not a {expected}; using default {fallback}"),
        );
        return fallback;
    };

    if options.integer && parsed.fract() != 0.0 {
        warn_once(
            format!("{name}={raw}"),
            format!("Ignoring {name}: not a finite integer; using default {fallback}"),
        );
        return fallback;
    }

    if parsed < options.min {
        warn_once(
            format!("{name}={raw}"),
            format!("{name} below minimum {}; clamped", options.min),
        );
        return options.min;
    }
    if let Some(max) = options.max {
        if parsed > max {
            warn_once(
                format!("{name}={raw}"),
                format!("{name} above maximum {max}; clamped"),
            );
            return max;
        }
    }
    parsed
}

/// Resolves a numeric override from the process environment.
pub fn env_number(name: &str, fallback: f64, options: EnvNumberOptions) -> f64 {
    let environment = std::env::var(name)
        .ok()
        .map(|value| BTreeMap::from([(name.to_owned(), value)]))
        .unwrap_or_default();
    env_number_from(name, fallback, options, &environment)
}

/// Resolves a boolean flag from a supplied environment map.
pub fn env_flag_from(
    name: &str,
    fallback: bool,
    environment: &BTreeMap<String, String>,
) -> bool {
    let Some(raw) = environment.get(name) else {
        return fallback;
    };
    if raw.is_empty() {
        return fallback;
    }

    match raw.to_ascii_lowercase().as_str() {
        "0" | "false" => false,
        "1" | "true" => true,
        _ => {
            warn_once(
                format!("{name}={raw}"),
                format!(
                    "Ignoring {name}: expected 0/1/true/false; using default {fallback}"
                ),
            );
            fallback
        }
    }
}

/// Resolves a boolean flag from the process environment.
pub fn env_flag(name: &str, fallback: bool) -> bool {
    let environment = std::env::var(name)
        .ok()
        .map(|value| BTreeMap::from([(name.to_owned(), value)]))
        .unwrap_or_default();
    env_flag_from(name, fallback, &environment)
}

/// Default per-run event ceiling.
pub const DEFAULT_MAX_EVENTS_PER_RUN: u64 = 25_000;

/// Resolves the per-run event ceiling from a supplied environment map.
pub fn max_events_per_run_from(environment: &BTreeMap<String, String>) -> u64 {
    let value = env_number_from(
        "WORKFLOW_MAX_EVENTS",
        DEFAULT_MAX_EVENTS_PER_RUN as f64,
        EnvNumberOptions {
            integer: true,
            ..EnvNumberOptions::default()
        },
        environment,
    );

    if value > 0.0 && value <= u64::MAX as f64 {
        value as u64
    } else {
        DEFAULT_MAX_EVENTS_PER_RUN
    }
}

fn parse_javascript_number(raw: &str) -> Option<f64> {
    let value = raw.trim();
    if value.is_empty() {
        return Some(0.0);
    }

    match value {
        "Infinity" | "+Infinity" => return Some(f64::INFINITY),
        "-Infinity" => return Some(f64::NEG_INFINITY),
        "NaN" | "+NaN" | "-NaN" => return Some(f64::NAN),
        _ => {}
    }

    if value.starts_with('+') || value.starts_with('-') {
        return value.parse::<f64>().ok();
    }

    if let Some(digits) = value
        .strip_prefix("0x")
        .or_else(|| value.strip_prefix("0X"))
    {
        return parse_radix_number(digits, 16);
    }
    if let Some(digits) = value
        .strip_prefix("0b")
        .or_else(|| value.strip_prefix("0B"))
    {
        return parse_radix_number(digits, 2);
    }
    if let Some(digits) = value
        .strip_prefix("0o")
        .or_else(|| value.strip_prefix("0O"))
    {
        return parse_radix_number(digits, 8);
    }

    value.parse::<f64>().ok()
}

fn parse_radix_number(digits: &str, radix: u32) -> Option<f64> {
    if digits.is_empty() {
        return None;
    }

    let mut output: f64 = 0.0;
    for character in digits.chars() {
        let digit = character.to_digit(radix)?;
        output = output.mul_add(radix as f64, digit as f64);
    }
    Some(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    const NAME: &str = "WORKFLOW_TEST_ENV_CONFIG_FIXTURE";

    fn environment(value: &str) -> BTreeMap<String, String> {
        BTreeMap::from([(NAME.to_owned(), value.to_owned())])
    }

    #[test]
    fn matches_numeric_fallback_and_clamping_rules() {
        assert_eq!(
            env_number_from(NAME, 100.0, EnvNumberOptions::default(), &BTreeMap::new()),
            100.0
        );
        assert_eq!(
            env_number_from(NAME, 100.0, EnvNumberOptions::default(), &environment("42")),
            42.0
        );
        assert_eq!(
            env_number_from(NAME, 100.0, EnvNumberOptions::default(), &environment("-5")),
            0.0
        );
        assert_eq!(
            env_number_from(
                NAME,
                100.0,
                EnvNumberOptions {
                    max: Some(500.0),
                    ..EnvNumberOptions::default()
                },
                &environment("9999")
            ),
            500.0
        );
    }

    #[test]
    fn matches_javascript_number_radix_forms() {
        assert_eq!(parse_javascript_number("0x10"), Some(16.0));
        assert_eq!(parse_javascript_number("0b10"), Some(2.0));
        assert_eq!(parse_javascript_number("0o10"), Some(8.0));
        assert_eq!(parse_javascript_number("   "), Some(0.0));
    }

    #[test]
    fn integer_mode_rejects_fractions() {
        assert_eq!(
            env_number_from(
                NAME,
                100.0,
                EnvNumberOptions {
                    integer: true,
                    ..EnvNumberOptions::default()
                },
                &environment("2.5")
            ),
            100.0
        );
    }

    #[test]
    fn flag_parser_is_case_insensitive_and_conservative() {
        assert!(!env_flag_from(NAME, true, &environment("FALSE")));
        assert!(env_flag_from(NAME, false, &environment("TRUE")));
        assert!(env_flag_from(NAME, true, &environment("yes")));
        assert!(!env_flag_from(NAME, false, &environment("yes")));
    }

    #[test]
    fn non_positive_event_limits_return_the_compiled_default() {
        for raw in ["0", "-1"] {
            let environment = BTreeMap::from([(
                "WORKFLOW_MAX_EVENTS".to_owned(),
                raw.to_owned(),
            )]);
            assert_eq!(
                max_events_per_run_from(&environment),
                DEFAULT_MAX_EVENTS_PER_RUN
            );
        }
    }
}
