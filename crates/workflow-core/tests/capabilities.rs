use std::collections::BTreeSet;

use workflow_core::capabilities::{
    JAVASCRIPT_MAX_SAFE_INTEGER, NODE_SEMVER_MAX_LENGTH, RunCapabilities, SerializationFormat,
    get_run_capabilities,
};

fn formats(values: &[SerializationFormat]) -> BTreeSet<SerializationFormat> {
    values.iter().copied().collect()
}

fn assert_legacy_capabilities(version: Option<&str>, encrypted: bool, framed: bool) {
    let capabilities = get_run_capabilities(version);
    assert!(capabilities.supports(SerializationFormat::DevalueV1));
    assert_eq!(
        capabilities.supports(SerializationFormat::Encrypted),
        encrypted
    );
    assert_eq!(capabilities.framed_byte_streams, framed);
}

macro_rules! capability_case {
    ($name:ident, $version:expr, $encrypted:expr, $framed:expr) => {
        #[test]
        fn $name() {
            assert_legacy_capabilities($version, $encrypted, $framed);
        }
    };
}

capability_case!(undefined_version_uses_baseline_formats, None, false, false);
capability_case!(dev_version_uses_baseline_formats, Some("dev"), false, false);
capability_case!(
    nonsense_version_uses_baseline_formats,
    Some("not-a-version"),
    false,
    false
);
capability_case!(empty_version_uses_baseline_formats, Some(""), false, false);
capability_case!(
    two_component_version_uses_baseline_formats,
    Some("4.2"),
    false,
    false
);
capability_case!(
    one_component_version_uses_baseline_formats,
    Some("4"),
    false,
    false
);
capability_case!(
    v_prefixed_cutoff_supports_encryption,
    Some("v4.2.0-beta.64"),
    true,
    false
);
capability_case!(
    pre_cutoff_beta_63_does_not_encrypt,
    Some("4.1.0-beta.63"),
    false,
    false
);
capability_case!(
    older_beta_does_not_encrypt,
    Some("4.0.1-beta.27"),
    false,
    false
);
capability_case!(major_three_does_not_encrypt, Some("3.0.0"), false, false);
capability_case!(
    encryption_cutoff_is_inclusive,
    Some("4.2.0-beta.64"),
    true,
    false
);
capability_case!(
    later_encryption_beta_is_supported,
    Some("4.2.0-beta.74"),
    true,
    false
);
capability_case!(stable_four_two_is_encrypted, Some("4.2.0"), true, false);
capability_case!(stable_five_is_encrypted, Some("5.0.0"), true, true);
capability_case!(
    framing_invalid_nonsense_is_false,
    Some("not-a-version"),
    false,
    false
);
capability_case!(framing_invalid_empty_is_false, Some(""), false, false);
capability_case!(framing_invalid_dev_is_false, Some("dev"), false, false);
capability_case!(
    encryption_cutoff_predates_framing,
    Some("4.2.0-beta.64"),
    true,
    false
);
capability_case!(stable_four_two_predates_framing, Some("4.2.0"), true, false);
capability_case!(all_four_x_predate_framing, Some("4.99.99"), true, false);
capability_case!(
    five_beta_two_predates_framing,
    Some("5.0.0-beta.2"),
    true,
    false
);
capability_case!(
    five_beta_fourteen_predates_framing,
    Some("5.0.0-beta.14"),
    true,
    false
);
capability_case!(
    framing_cutoff_is_inclusive,
    Some("5.0.0-beta.15"),
    true,
    true
);
capability_case!(
    later_framing_beta_is_supported,
    Some("5.0.0-beta.16"),
    true,
    true
);
capability_case!(stable_five_supports_framing, Some("5.0.0"), true, true);
capability_case!(five_one_supports_framing, Some("5.1.0"), true, true);
capability_case!(major_six_supports_framing, Some("6.0.0"), true, true);

#[test]
fn default_is_the_fail_closed_baseline() {
    assert_eq!(
        RunCapabilities::default().supported_formats,
        formats(&[SerializationFormat::DevalueV1])
    );
    assert!(!RunCapabilities::default().framed_byte_streams);
}

#[test]
fn serialization_format_prefixes_match_the_typescript_wire_contract() {
    assert_eq!(SerializationFormat::DevalueV1.as_prefix(), "devl");
    assert_eq!(SerializationFormat::Encrypted.as_prefix(), "encr");
    assert_eq!(SerializationFormat::Gzip.as_prefix(), "gzip");
    assert_eq!(SerializationFormat::Zstd.as_prefix(), "zstd");
    assert_eq!(SerializationFormat::Sealed.as_prefix(), "encp");
}

#[test]
fn compression_cutoff_adds_gzip_and_zstd_together() {
    let before = get_run_capabilities(Some("5.0.0-beta.17"));
    assert!(!before.supports(SerializationFormat::Gzip));
    assert!(!before.supports(SerializationFormat::Zstd));

    let at_cutoff = get_run_capabilities(Some("5.0.0-beta.18"));
    assert!(at_cutoff.supports(SerializationFormat::Gzip));
    assert!(at_cutoff.supports(SerializationFormat::Zstd));
}

#[test]
fn sealed_cutoff_is_independent_and_inclusive() {
    let before = get_run_capabilities(Some("5.0.0-beta.36"));
    assert!(!before.supports(SerializationFormat::Sealed));

    let at_cutoff = get_run_capabilities(Some("5.0.0-beta.37"));
    assert!(at_cutoff.supports(SerializationFormat::Sealed));
}

#[test]
fn stable_five_supports_every_current_format() {
    let capabilities = get_run_capabilities(Some("5.0.0"));
    assert_eq!(
        capabilities.supported_formats,
        formats(&[
            SerializationFormat::DevalueV1,
            SerializationFormat::Encrypted,
            SerializationFormat::Gzip,
            SerializationFormat::Zstd,
            SerializationFormat::Sealed,
        ])
    );
    assert!(capabilities.framed_byte_streams);
}

#[test]
fn whitespace_and_build_metadata_follow_node_semver_normalization() {
    let capabilities = get_run_capabilities(Some("  v5.0.0-beta.37+build.9  "));
    assert!(capabilities.supports(SerializationFormat::Sealed));
    assert!(capabilities.framed_byte_streams);
}

#[test]
fn malformed_numeric_components_fail_closed() {
    for version in [
        "01.0.0",
        "1.00.0",
        "1.0.00",
        "5.0.0-beta.01",
        "V5.0.0",
    ] {
        assert_eq!(
            get_run_capabilities(Some(version)),
            RunCapabilities::default(),
            "{version} must not negotiate optional capabilities"
        );
    }
}

#[test]
fn components_above_javascript_safe_integer_fail_closed() {
    let version = format!("{}.0.0", JAVASCRIPT_MAX_SAFE_INTEGER + 1);
    assert_eq!(
        get_run_capabilities(Some(&version)),
        RunCapabilities::default()
    );
}

#[test]
fn raw_versions_longer_than_node_semver_limit_fail_closed_before_trimming() {
    let version = format!("{}5.0.0", " ".repeat(NODE_SEMVER_MAX_LENGTH));
    assert!(version.encode_utf16().count() > NODE_SEMVER_MAX_LENGTH);
    assert_eq!(
        get_run_capabilities(Some(&version)),
        RunCapabilities::default()
    );
}
