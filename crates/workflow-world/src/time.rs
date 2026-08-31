use crate::{ValidationError, ValidationResult};

/// ECMAScript `Date` TimeClip limit in milliseconds.
pub const MAX_DATE_MS: f64 = 8_640_000_000_000_000.0;

/// Language-neutral retry duration input.
#[derive(Debug, Clone, PartialEq)]
pub enum DurationInput {
    /// Duration string accepted by the JavaScript `ms` package.
    String(String),
    /// Relative duration in milliseconds.
    Milliseconds(f64),
    /// Absolute Unix timestamp in milliseconds.
    DateMilliseconds(f64),
}

/// Converts a retry duration into an absolute Unix timestamp in milliseconds.
///
/// The result follows JavaScript `Date` TimeClip behavior by truncating
/// fractional milliseconds toward zero, while rejecting invalid or overflowing
/// dates instead of allowing an unusable timestamp into persistent state.
pub fn parse_duration_to_unix_ms(input: DurationInput, now_ms: f64) -> ValidationResult<i64> {
    if !now_ms.is_finite() || now_ms.abs() > MAX_DATE_MS {
        return Err(ValidationError::new(
            "invalid_clock",
            "Current time must be a finite JavaScript Date timestamp",
        ));
    }

    let absolute_ms = match input {
        DurationInput::String(value) => {
            let duration_ms = parse_duration_string(&value).ok_or_else(|| {
                ValidationError::new(
                    "invalid_duration",
                    format!(
                        "Invalid duration: \"{value}\". Expected a valid duration string like \"1s\", \"1m\", \"1h\", etc."
                    ),
                )
            })?;
            if duration_ms < 0.0 || !duration_ms.is_finite() {
                return Err(ValidationError::new(
                    "invalid_duration",
                    format!(
                        "Invalid duration: \"{value}\". Expected a valid duration string like \"1s\", \"1m\", \"1h\", etc."
                    ),
                ));
            }
            now_ms + duration_ms
        }
        DurationInput::Milliseconds(value) => {
            if value < 0.0 || !value.is_finite() {
                return Err(ValidationError::new(
                    "invalid_duration",
                    format!(
                        "Invalid duration: {value}. Expected a non-negative finite number of milliseconds."
                    ),
                ));
            }
            now_ms + value
        }
        DurationInput::DateMilliseconds(value) => {
            if !is_valid_date_ms(value) {
                return Err(ValidationError::new(
                    "invalid_duration_date",
                    "Invalid duration Date. Expected a valid Date with a finite timestamp.",
                ));
            }
            value
        }
    };

    if !is_valid_date_ms(absolute_ms) {
        return Err(ValidationError::new(
            "duration_out_of_range",
            "Invalid duration. Resulting date is outside the supported range.",
        ));
    }

    Ok(absolute_ms.trunc() as i64)
}

fn is_valid_date_ms(value: f64) -> bool {
    value.is_finite() && value.abs() <= MAX_DATE_MS
}

fn parse_duration_string(value: &str) -> Option<f64> {
    // Mirrors the JavaScript `ms` package's defensive input limit.
    if value.is_empty() || value.len() > 100 {
        return None;
    }

    let mut numeric_end = 0;
    let mut seen_digit = false;
    let mut seen_dot = false;

    for (index, character) in value.char_indices() {
        let accepted = match character {
            '-' if index == 0 => true,
            '0'..='9' => {
                seen_digit = true;
                true
            }
            '.' if !seen_dot => {
                seen_dot = true;
                true
            }
            _ => false,
        };
        if !accepted {
            break;
        }
        numeric_end = index + character.len_utf8();
    }

    if !seen_digit || numeric_end == 0 {
        return None;
    }

    let number = value[..numeric_end].parse::<f64>().ok()?;
    let suffix = &value[numeric_end..];
    let suffix = suffix
        .strip_prefix(' ')
        .map_or(suffix, |rest| rest.trim_start_matches(' '));
    if suffix.ends_with(' ') || suffix.contains(char::is_whitespace) {
        return None;
    }

    let multiplier = match suffix.to_ascii_lowercase().as_str() {
        "" | "ms" | "msec" | "msecs" | "millisecond" | "milliseconds" => 1.0,
        "s" | "sec" | "secs" | "second" | "seconds" => 1_000.0,
        "m" | "min" | "mins" | "minute" | "minutes" => 60_000.0,
        "h" | "hr" | "hrs" | "hour" | "hours" => 3_600_000.0,
        "d" | "day" | "days" => 86_400_000.0,
        "w" | "week" | "weeks" => 604_800_000.0,
        "y" | "yr" | "yrs" | "year" | "years" => 31_557_600_000.0,
        _ => return None,
    };

    Some(number * multiplier)
}

#[cfg(test)]
mod tests {
    use super::*;

    const NOW: f64 = 1_700_000_000_000.0;

    #[test]
    fn parses_documented_ms_units() {
        let fixtures = [
            ("5", 5),
            ("5ms", 5),
            ("2s", 2_000),
            ("1.5m", 90_000),
            ("1h", 3_600_000),
            ("1 day", 86_400_000),
            ("1w", 604_800_000),
            ("1y", 31_557_600_000),
        ];

        for (input, expected_delta) in fixtures {
            assert_eq!(
                parse_duration_to_unix_ms(DurationInput::String(input.to_owned()), NOW).unwrap(),
                NOW as i64 + expected_delta
            );
        }
    }

    #[test]
    fn rejects_negative_non_finite_and_overflowing_values() {
        for input in [
            DurationInput::Milliseconds(-1.0),
            DurationInput::Milliseconds(f64::INFINITY),
            DurationInput::Milliseconds(f64::MAX),
            DurationInput::DateMilliseconds(f64::NAN),
            DurationInput::DateMilliseconds(MAX_DATE_MS + 1.0),
        ] {
            assert!(parse_duration_to_unix_ms(input, NOW).is_err());
        }
    }

    #[test]
    fn truncates_fractional_milliseconds_like_date_timeclip() {
        assert_eq!(
            parse_duration_to_unix_ms(DurationInput::Milliseconds(1.9), NOW).unwrap(),
            NOW as i64 + 1
        );
    }
}
