use crate::{ValidationError, ValidationResult};

/// Characters in an event id body, ULID or slot alike.
pub const EVENT_ID_BODY_LENGTH: usize = 26;
const SLOT_LEADING_ZEROS: usize = 10;
/// First slot in a run's dense event log.
pub const FIRST_EVENT_SLOT: u64 = 1;
/// JavaScript's largest exactly representable integer.
pub const MAX_EVENT_SLOT: u64 = 9_007_199_254_740_991;
/// Canonical event-id prefix.
pub const EVENT_ID_PREFIX: &str = "evnt_";

/// Whether a 26-character event-id body is a slot rather than a ULID.
pub fn is_slot_body(body: &str) -> bool {
    let bytes = body.as_bytes();
    bytes.len() == EVENT_ID_BODY_LENGTH
        && bytes[..SLOT_LEADING_ZEROS].iter().all(|byte| *byte == b'0')
        && bytes[SLOT_LEADING_ZEROS..]
            .iter()
            .all(u8::is_ascii_digit)
}

fn strip_event_id_prefix(event_id: &str) -> &str {
    event_id
        .split_once('_')
        .map_or(event_id, |(_, body)| body)
}

/// Whether a possibly prefixed event id is slot-numbered.
pub fn is_slot_event_id(event_id: &str) -> bool {
    is_slot_body(strip_event_id_prefix(event_id))
}

/// Formats a slot as a fixed-width event id.
pub fn slot_to_event_id(slot: u64) -> ValidationResult<String> {
    if !(FIRST_EVENT_SLOT..=MAX_EVENT_SLOT).contains(&slot) {
        return Err(ValidationError::new(
            "invalid_event_slot",
            format!("Invalid event slot: {slot}"),
        ));
    }

    let digits = slot.to_string();
    let padding = "0".repeat(EVENT_ID_BODY_LENGTH - digits.len());
    Ok(format!("{EVENT_ID_PREFIX}{padding}{digits}"))
}

/// JavaScript-number compatible slot formatter used at dynamic boundaries.
pub fn number_to_event_id(slot: f64) -> ValidationResult<String> {
    if !slot.is_finite()
        || slot.fract() != 0.0
        || slot < FIRST_EVENT_SLOT as f64
        || slot > MAX_EVENT_SLOT as f64
    {
        return Err(ValidationError::new(
            "invalid_event_slot",
            format!("Invalid event slot: {slot}"),
        ));
    }
    slot_to_event_id(slot as u64)
}

/// Reads the slot out of a possibly prefixed event id.
pub fn event_id_to_slot(event_id: &str) -> Option<u64> {
    let body = strip_event_id_prefix(event_id);
    if !is_slot_body(body) {
        return None;
    }
    let slot = body.parse::<u64>().ok()?;
    (FIRST_EVENT_SLOT..=MAX_EVENT_SLOT)
        .contains(&slot)
        .then_some(slot)
}

/// Reads a required slot or reports a protocol mismatch.
pub fn require_event_slot(event_id: &str) -> ValidationResult<u64> {
    event_id_to_slot(event_id).ok_or_else(|| {
        ValidationError::new(
            "event_id_not_slot_numbered",
            format!(
                "Event id is not slot-numbered: {event_id}. This World allocates event positions the runtime cannot read."
            ),
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_width_order_matches_numeric_order() {
        let ids: Vec<_> = [1, 2, 9, 10, 99, 100, 1_000]
            .into_iter()
            .map(|slot| slot_to_event_id(slot).unwrap())
            .collect();
        assert!(ids
            .iter()
            .all(|id| id.len() == EVENT_ID_PREFIX.len() + EVENT_ID_BODY_LENGTH));
        let mut sorted = ids.clone();
        sorted.sort();
        assert_eq!(sorted, ids);
    }

    #[test]
    fn round_trips_every_boundary() {
        for slot in [FIRST_EVENT_SLOT, 7, 12_345, MAX_EVENT_SLOT] {
            assert_eq!(event_id_to_slot(&slot_to_event_id(slot).unwrap()), Some(slot));
        }
    }

    #[test]
    fn accepts_legacy_and_bare_prefix_forms() {
        let body = format!("{:0>width$}", 42, width = EVENT_ID_BODY_LENGTH);
        assert!(is_slot_event_id(&format!("evnt_{body}")));
        assert!(is_slot_event_id(&format!("wevt_{body}")));
        assert!(is_slot_event_id(&body));
    }

    #[test]
    fn rejects_wrong_shapes_and_inexact_numbers() {
        assert!(number_to_event_id(0.0).is_err());
        assert!(number_to_event_id(-1.0).is_err());
        assert!(number_to_event_id(1.5).is_err());
        assert!(number_to_event_id(MAX_EVENT_SLOT as f64 + 2.0).is_err());
        assert!(!is_slot_body("00000000010000000000000000"));
        assert!(!is_slot_body("0000000000000000000000000A"));
    }
}
