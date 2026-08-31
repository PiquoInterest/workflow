use std::collections::BTreeMap;

use workflow_world::spec_version::{
    SEALED_LOG_ENV_VAR, SPEC_VERSION_CURRENT, SPEC_VERSION_LEGACY, SPEC_VERSION_MAX_SUPPORTED,
    SPEC_VERSION_SUPPORTS_ATTRIBUTES, SPEC_VERSION_SUPPORTS_COMPRESSION,
    SPEC_VERSION_SUPPORTS_SEALED_LOG, SPEC_VERSION_SUPPORTS_SLOT_IDENTITY,
    is_legacy_spec_version, minted_spec_version, requires_newer_world,
};

fn environment(raw: &str) -> BTreeMap<String, String> {
    BTreeMap::from([(SEALED_LOG_ENV_VAR.to_owned(), raw.to_owned())])
}

#[test]
fn current_spec_version_is_the_sealed_log_version() {
    assert_eq!(SPEC_VERSION_SUPPORTS_SLOT_IDENTITY, 6);
    assert_eq!(SPEC_VERSION_SUPPORTS_SEALED_LOG, 7);
    assert_eq!(SPEC_VERSION_CURRENT, SPEC_VERSION_SUPPORTS_SEALED_LOG);
}

#[test]
fn minted_spec_version_stamps_sealed_log_by_default() {
    assert_eq!(minted_spec_version(&BTreeMap::new()), SPEC_VERSION_CURRENT);
    assert_eq!(
        minted_spec_version(&BTreeMap::new()),
        SPEC_VERSION_SUPPORTS_SEALED_LOG
    );
}

#[test]
fn minted_spec_version_falls_back_to_slot_identity_when_switched_off() {
    for off in ["0", "false"] {
        assert_eq!(
            minted_spec_version(&environment(off)),
            SPEC_VERSION_SUPPORTS_SLOT_IDENTITY
        );
    }
}

#[test]
fn minted_spec_version_stays_on_for_unset_or_malformed_values() {
    for raw in ["", "1", "true", "yes-please"] {
        assert_eq!(minted_spec_version(&environment(raw)), SPEC_VERSION_CURRENT);
    }
}

#[test]
fn minted_spec_version_never_exceeds_the_readable_ceiling() {
    assert!(minted_spec_version(&BTreeMap::new()) <= SPEC_VERSION_MAX_SUPPORTED);
}

#[test]
fn the_readable_ceiling_moves_with_the_default_version() {
    assert_eq!(
        SPEC_VERSION_MAX_SUPPORTED,
        SPEC_VERSION_SUPPORTS_SEALED_LOG
    );
    assert!(SPEC_VERSION_MAX_SUPPORTED >= SPEC_VERSION_CURRENT);
}

#[test]
fn requires_newer_world_accepts_runs_at_or_below_the_supported_version() {
    assert!(!requires_newer_world(Some(SPEC_VERSION_CURRENT)));
    assert!(!requires_newer_world(Some(SPEC_VERSION_SUPPORTS_ATTRIBUTES)));
    assert!(!requires_newer_world(Some(SPEC_VERSION_LEGACY)));
    assert!(!requires_newer_world(None));
}

#[test]
fn requires_newer_world_accepts_slot_identity_runs() {
    assert!(!requires_newer_world(Some(
        SPEC_VERSION_SUPPORTS_SLOT_IDENTITY
    )));
}

#[test]
fn requires_newer_world_rejects_runs_above_the_readable_ceiling() {
    assert!(requires_newer_world(Some(
        SPEC_VERSION_MAX_SUPPORTED + 1
    )));
}

#[test]
fn a_version_four_reader_would_reject_a_compression_era_run() {
    let v4_requires_newer_world = |version: u32| version > 4;
    assert!(v4_requires_newer_world(
        SPEC_VERSION_SUPPORTS_COMPRESSION
    ));
}

#[test]
fn legacy_detection_is_unaffected_by_the_current_version() {
    assert!(is_legacy_spec_version(Some(1)));
    assert!(is_legacy_spec_version(None));
    assert!(!is_legacy_spec_version(Some(2)));
    assert!(!is_legacy_spec_version(Some(4)));
    assert!(!is_legacy_spec_version(Some(5)));
}
