use workflow_world::slot_identity::{
    EVENT_ID_BODY_LENGTH, EVENT_ID_PREFIX, FIRST_EVENT_SLOT, MAX_EVENT_SLOT, event_id_to_slot,
    is_slot_body, is_slot_event_id, number_to_event_id, slot_to_event_id,
};
use workflow_world::ulid::{
    DEFAULT_TIMESTAMP_THRESHOLD_FUTURE_MS, DEFAULT_TIMESTAMP_THRESHOLD_PAST_MS, ulid_timestamp_ms,
    validate_ulid_timestamp_at,
};

#[test]
fn mints_fixed_width_ids_whose_string_order_is_slot_order() {
    let ids: Vec<_> = [1, 2, 9, 10, 99, 100, 1_000]
        .into_iter()
        .map(|slot| slot_to_event_id(slot).unwrap())
        .collect();
    assert!(
        ids.iter()
            .all(|id| id.len() == EVENT_ID_PREFIX.len() + EVENT_ID_BODY_LENGTH)
    );
    let mut sorted = ids.clone();
    sorted.sort();
    assert_eq!(sorted, ids);
}

#[test]
fn slot_ids_round_trip_through_event_id_to_slot() {
    for slot in [FIRST_EVENT_SLOT, 7, 12_345, MAX_EVENT_SLOT] {
        assert_eq!(
            event_id_to_slot(&slot_to_event_id(slot).unwrap()),
            Some(slot)
        );
    }
}

#[test]
fn refuses_slots_that_cannot_be_represented_exactly() {
    assert!(number_to_event_id(0.0).is_err());
    assert!(number_to_event_id(-1.0).is_err());
    assert!(number_to_event_id(1.5).is_err());
    assert!(number_to_event_id(MAX_EVENT_SLOT as f64 + 2.0).is_err());
}

#[test]
fn reads_slot_ids_with_canonical_legacy_or_no_prefix() {
    let body = format!("{:0>width$}", 42, width = EVENT_ID_BODY_LENGTH);
    assert!(is_slot_event_id(&format!("evnt_{body}")));
    assert!(is_slot_event_id(&format!("wevt_{body}")));
    assert!(is_slot_event_id(&body));
    assert_eq!(event_id_to_slot(&format!("wevt_{body}")), Some(42));
}

#[test]
fn never_mistakes_a_valid_ulid_for_a_slot() {
    const CROCKFORD: &[u8] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";
    let prefix = "01ARYZ6S41TSV4RRFFQ69G5F";
    for index in 0..100 {
        let body = format!(
            "{prefix}{}{}",
            char::from(CROCKFORD[(index / CROCKFORD.len()) % CROCKFORD.len()]),
            char::from(CROCKFORD[index % CROCKFORD.len()]),
        );
        assert_eq!(body.len(), EVENT_ID_BODY_LENGTH);
        assert!(!is_slot_body(&body));
        assert_eq!(event_id_to_slot(&format!("evnt_{body}")), None);
    }
}

#[test]
fn rejects_slot_bodies_of_the_wrong_shape() {
    assert!(!is_slot_body(
        &"0000000001".pad_end(EVENT_ID_BODY_LENGTH, '0')
    ));
    assert!(!is_slot_body(&format!(
        "{}A",
        "0".repeat(EVENT_ID_BODY_LENGTH - 1)
    )));
    assert!(!is_slot_body(&format!(
        "{}11",
        "0".repeat(EVENT_ID_BODY_LENGTH - 1)
    )));
    assert!(!is_slot_body(""));
}

#[test]
fn slot_time_decoding_returns_none_instead_of_the_epoch() {
    let first = slot_to_event_id(1).unwrap();
    let later = slot_to_event_id(999_999).unwrap();
    assert_eq!(
        ulid_timestamp_ms(first.strip_prefix(EVENT_ID_PREFIX).unwrap()),
        None
    );
    assert_eq!(
        ulid_timestamp_ms(later.strip_prefix(EVENT_ID_PREFIX).unwrap()),
        None
    );
}

#[test]
fn a_real_ulid_still_decodes_to_a_positive_timestamp() {
    assert!(ulid_timestamp_ms("01ARYZ6S41TSV4RRFFQ69G5FAV").unwrap() > 0);
}

#[test]
fn slot_shaped_run_ids_fail_validation_as_invalid_ulids() {
    let body = slot_to_event_id(1)
        .unwrap()
        .strip_prefix(EVENT_ID_PREFIX)
        .unwrap()
        .to_owned();
    let error = validate_ulid_timestamp_at(
        &format!("wrun_{body}"),
        "wrun_",
        0,
        DEFAULT_TIMESTAMP_THRESHOLD_PAST_MS,
        DEFAULT_TIMESTAMP_THRESHOLD_FUTURE_MS,
    )
    .unwrap();
    assert!(error.contains("is not a valid ULID"));
}

trait PadEnd {
    fn pad_end(&self, width: usize, character: char) -> String;
}

impl PadEnd for str {
    fn pad_end(&self, width: usize, character: char) -> String {
        let mut output = self.to_owned();
        output.extend(std::iter::repeat_n(
            character,
            width.saturating_sub(output.len()),
        ));
        output
    }
}
