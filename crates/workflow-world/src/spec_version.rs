use std::collections::BTreeMap;

use crate::env::env_flag_from;

/// Legacy, direct-mutation protocol.
pub const SPEC_VERSION_LEGACY: u32 = 1;
/// First event-sourced protocol.
pub const SPEC_VERSION_SUPPORTS_EVENT_SOURCING: u32 = 2;
/// First CBOR queue-transport protocol.
pub const SPEC_VERSION_SUPPORTS_CBOR_QUEUE_TRANSPORT: u32 = 3;
/// First protocol with native attributes.
pub const SPEC_VERSION_SUPPORTS_ATTRIBUTES: u32 = 4;
/// First protocol that may carry compressed payloads.
pub const SPEC_VERSION_SUPPORTS_COMPRESSION: u32 = 5;
/// First protocol with dense slot-numbered event ids.
pub const SPEC_VERSION_SUPPORTS_SLOT_IDENTITY: u32 = 6;
/// First protocol whose event log may contain backend-sealed noops.
pub const SPEC_VERSION_SUPPORTS_SEALED_LOG: u32 = 7;
/// Default version stamped by this build.
pub const SPEC_VERSION_CURRENT: u32 = SPEC_VERSION_SUPPORTS_SEALED_LOG;
/// Highest version this build can read.
pub const SPEC_VERSION_MAX_SUPPORTED: u32 = SPEC_VERSION_SUPPORTS_SEALED_LOG;
/// Kill switch for creating sealed logs.
pub const SEALED_LOG_ENV_VAR: &str = "WORKFLOW_SEALED_LOG";

/// Chooses the protocol version stamped on newly created runs.
pub fn minted_spec_version(environment: &BTreeMap<String, String>) -> u32 {
    if env_flag_from(SEALED_LOG_ENV_VAR, true, environment) {
        SPEC_VERSION_CURRENT
    } else {
        SPEC_VERSION_SUPPORTS_SLOT_IDENTITY
    }
}

/// Whether a run uses the legacy, pre-event-sourcing protocol.
pub const fn is_legacy_spec_version(version: Option<u32>) -> bool {
    match version {
        None => true,
        Some(version) => version <= SPEC_VERSION_LEGACY,
    }
}

/// Whether reading a run requires a newer World implementation.
pub const fn requires_newer_world(version: Option<u32>) -> bool {
    match version {
        None => false,
        Some(version) => version > SPEC_VERSION_MAX_SUPPORTED,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_and_readable_versions_are_consistent() {
        assert_eq!(SPEC_VERSION_SUPPORTS_SLOT_IDENTITY, 6);
        assert_eq!(SPEC_VERSION_SUPPORTS_SEALED_LOG, 7);
        assert_eq!(SPEC_VERSION_CURRENT, SPEC_VERSION_SUPPORTS_SEALED_LOG);
        assert_eq!(SPEC_VERSION_MAX_SUPPORTED, SPEC_VERSION_CURRENT);
    }

    #[test]
    fn sealed_log_is_default_on_with_a_safe_kill_switch() {
        assert_eq!(minted_spec_version(&BTreeMap::new()), SPEC_VERSION_CURRENT);
        for off in ["0", "false"] {
            let environment = BTreeMap::from([(SEALED_LOG_ENV_VAR.to_owned(), off.to_owned())]);
            assert_eq!(
                minted_spec_version(&environment),
                SPEC_VERSION_SUPPORTS_SLOT_IDENTITY
            );
        }
        for malformed in ["", "yes-please"] {
            let environment =
                BTreeMap::from([(SEALED_LOG_ENV_VAR.to_owned(), malformed.to_owned())]);
            assert_eq!(minted_spec_version(&environment), SPEC_VERSION_CURRENT);
        }
    }

    #[test]
    fn legacy_and_newer_world_checks_match_the_typescript_contract() {
        assert!(is_legacy_spec_version(None));
        assert!(is_legacy_spec_version(Some(1)));
        assert!(!is_legacy_spec_version(Some(2)));
        assert!(!requires_newer_world(None));
        assert!(!requires_newer_world(Some(SPEC_VERSION_MAX_SUPPORTED)));
        assert!(requires_newer_world(Some(SPEC_VERSION_MAX_SUPPORTED + 1)));
    }
}
