use std::time::{SystemTime, UNIX_EPOCH};

use crate::slot_identity::is_slot_body;

/// Maximum accepted client-generated ULID age: 24 hours.
pub const DEFAULT_TIMESTAMP_THRESHOLD_PAST_MS: i64 = 24 * 60 * 60 * 1_000;
/// Maximum accepted future clock skew: 5 minutes.
pub const DEFAULT_TIMESTAMP_THRESHOLD_FUTURE_MS: i64 = 5 * 60 * 1_000;
/// Backward-compatible alias for the past threshold.
pub const DEFAULT_TIMESTAMP_THRESHOLD_MS: i64 = DEFAULT_TIMESTAMP_THRESHOLD_PAST_MS;
/// Prefix of client-generated workflow run ids.
pub const WORKFLOW_RUN_ID_PREFIX: &str = "wrun_";

/// Whether a string is a canonical 26-character ULID body.
pub fn is_valid_ulid(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 26 || !matches!(bytes[0], b'0'..=b'7') {
        return false;
    }
    bytes.iter().all(|byte| crockford_value(*byte).is_some())
}

/// Whether a string has the exact workflow-run-id shape.
pub fn is_valid_workflow_run_id(value: &str) -> bool {
    value
        .strip_prefix(WORKFLOW_RUN_ID_PREFIX)
        .is_some_and(is_valid_ulid)
}

/// Decodes the embedded millisecond timestamp, rejecting slot-numbered bodies.
pub fn ulid_timestamp_ms(maybe_ulid: &str) -> Option<u64> {
    if is_slot_body(maybe_ulid) || !is_valid_ulid(maybe_ulid) {
        return None;
    }

    let mut timestamp = 0_u64;
    for byte in maybe_ulid.bytes().take(10) {
        timestamp = timestamp
            .checked_mul(32)?
            .checked_add(u64::from(crockford_value(byte)?))?;
    }
    Some(timestamp)
}

/// Validates a prefixed ULID timestamp against an injected wall clock.
pub fn validate_ulid_timestamp_at(
    prefixed_ulid: &str,
    prefix: &str,
    now_ms: i64,
    past_threshold_ms: i64,
    future_threshold_ms: i64,
) -> Option<String> {
    let raw = prefixed_ulid.strip_prefix(prefix).unwrap_or(prefixed_ulid);
    let Some(timestamp) = ulid_timestamp_ms(raw) else {
        return Some(format!(
            "Invalid runId: \"{prefixed_ulid}\" is not a valid ULID"
        ));
    };
    let Ok(timestamp) = i64::try_from(timestamp) else {
        return Some(format!(
            "Invalid runId: \"{prefixed_ulid}\" is not a valid ULID"
        ));
    };

    let diff_ms = now_ms.saturating_sub(timestamp);
    if diff_ms > 0 && diff_ms <= past_threshold_ms {
        return None;
    }
    if diff_ms <= 0 && diff_ms.saturating_neg() <= future_threshold_ms {
        return None;
    }

    let drift_seconds = ((diff_ms.unsigned_abs() as f64) / 1_000.0).round() as u64;
    let (direction, threshold_ms) = if diff_ms > 0 {
        ("past", past_threshold_ms)
    } else {
        ("future", future_threshold_ms)
    };
    let threshold_seconds = ((threshold_ms.max(0) as f64) / 1_000.0).round() as u64;
    Some(format!(
        "Invalid runId timestamp: embedded timestamp is {drift_seconds}s in the {direction} (threshold: {threshold_seconds}s)"
    ))
}

/// Validates a prefixed ULID against the current system clock.
pub fn validate_ulid_timestamp(
    prefixed_ulid: &str,
    prefix: &str,
    past_threshold_ms: i64,
    future_threshold_ms: i64,
) -> Option<String> {
    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .and_then(|duration| i64::try_from(duration.as_millis()).ok())
        .unwrap_or(i64::MAX);
    validate_ulid_timestamp_at(
        prefixed_ulid,
        prefix,
        now_ms,
        past_threshold_ms,
        future_threshold_ms,
    )
}

fn crockford_value(byte: u8) -> Option<u8> {
    match byte.to_ascii_uppercase() {
        b'0'..=b'9' => Some(byte.to_ascii_uppercase() - b'0'),
        b'A' => Some(10),
        b'B' => Some(11),
        b'C' => Some(12),
        b'D' => Some(13),
        b'E' => Some(14),
        b'F' => Some(15),
        b'G' => Some(16),
        b'H' => Some(17),
        b'J' => Some(18),
        b'K' => Some(19),
        b'M' => Some(20),
        b'N' => Some(21),
        b'P' => Some(22),
        b'Q' => Some(23),
        b'R' => Some(24),
        b'S' => Some(25),
        b'T' => Some(26),
        b'V' => Some(27),
        b'W' => Some(28),
        b'X' => Some(29),
        b'Y' => Some(30),
        b'Z' => Some(31),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::slot_identity::slot_to_event_id;

    #[test]
    fn validates_exact_run_id_shape() {
        assert!(is_valid_workflow_run_id(
            "wrun_01ARZ3NDEKTSV4RRFFQ69G5FAV"
        ));
        assert!(!is_valid_workflow_run_id(
            "step_01ARZ3NDEKTSV4RRFFQ69G5FAV"
        ));
        assert!(!is_valid_workflow_run_id("wrun_not-a-ulid"));
    }

    #[test]
    fn rejects_slot_ids_instead_of_dating_them_to_the_epoch() {
        let slot = slot_to_event_id(1).unwrap();
        let body = slot.strip_prefix("evnt_").unwrap();
        assert_eq!(ulid_timestamp_ms(body), None);
    }

    #[test]
    fn enforces_asymmetric_timestamp_windows() {
        // This ULID has an embedded timestamp of 1_469_918_176_385ms.
        let body = "01ARYZ6S41TSV4RRFFQ69G5FAV";
        let timestamp = ulid_timestamp_ms(body).unwrap() as i64;
        assert_eq!(timestamp, 1_469_918_176_385);

        assert_eq!(
            validate_ulid_timestamp_at(
                &format!("wrun_{body}"),
                "wrun_",
                timestamp + 1_000,
                DEFAULT_TIMESTAMP_THRESHOLD_PAST_MS,
                DEFAULT_TIMESTAMP_THRESHOLD_FUTURE_MS,
            ),
            None
        );
        assert!(validate_ulid_timestamp_at(
            &format!("wrun_{body}"),
            "wrun_",
            timestamp + DEFAULT_TIMESTAMP_THRESHOLD_PAST_MS + 1,
            DEFAULT_TIMESTAMP_THRESHOLD_PAST_MS,
            DEFAULT_TIMESTAMP_THRESHOLD_FUTURE_MS,
        )
        .is_some());
        assert!(validate_ulid_timestamp_at(
            &format!("wrun_{body}"),
            "wrun_",
            timestamp - DEFAULT_TIMESTAMP_THRESHOLD_FUTURE_MS - 1,
            DEFAULT_TIMESTAMP_THRESHOLD_PAST_MS,
            DEFAULT_TIMESTAMP_THRESHOLD_FUTURE_MS,
        )
        .is_some());
    }
}
