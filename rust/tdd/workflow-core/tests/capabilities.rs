use workflow_core_tdd::capabilities::{SerializationFormat, get_run_capabilities};

fn assert_capabilities(version: Option<&str>, encrypted: bool, framed: bool) {
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
            assert_capabilities($version, $encrypted, $framed);
        }
    };
}

capability_case!(undefined_version_uses_baseline_formats, None, false, false);
capability_case!(dev_version_uses_baseline_formats, Some("dev"), false, false);
capability_case!(nonsense_version_uses_baseline_formats, Some("not-a-version"), false, false);
capability_case!(empty_version_uses_baseline_formats, Some(""), false, false);
capability_case!(two_component_version_uses_baseline_formats, Some("4.2"), false, false);
capability_case!(one_component_version_uses_baseline_formats, Some("4"), false, false);
capability_case!(v_prefixed_cutoff_supports_encryption, Some("v4.2.0-beta.64"), true, false);
capability_case!(pre_cutoff_beta_63_does_not_encrypt, Some("4.1.0-beta.63"), false, false);
capability_case!(older_beta_does_not_encrypt, Some("4.0.1-beta.27"), false, false);
capability_case!(major_three_does_not_encrypt, Some("3.0.0"), false, false);
capability_case!(encryption_cutoff_is_inclusive, Some("4.2.0-beta.64"), true, false);
capability_case!(later_encryption_beta_is_supported, Some("4.2.0-beta.74"), true, false);
capability_case!(stable_four_two_is_encrypted, Some("4.2.0"), true, false);
capability_case!(stable_five_is_encrypted, Some("5.0.0"), true, true);
capability_case!(framing_invalid_nonsense_is_false, Some("not-a-version"), false, false);
capability_case!(framing_invalid_empty_is_false, Some(""), false, false);
capability_case!(framing_invalid_dev_is_false, Some("dev"), false, false);
capability_case!(encryption_cutoff_predates_framing, Some("4.2.0-beta.64"), true, false);
capability_case!(stable_four_two_predates_framing, Some("4.2.0"), true, false);
capability_case!(all_four_x_predate_framing, Some("4.99.99"), true, false);
capability_case!(five_beta_two_predates_framing, Some("5.0.0-beta.2"), true, false);
capability_case!(five_beta_fourteen_predates_framing, Some("5.0.0-beta.14"), true, false);
capability_case!(framing_cutoff_is_inclusive, Some("5.0.0-beta.15"), true, true);
capability_case!(later_framing_beta_is_supported, Some("5.0.0-beta.16"), true, true);
capability_case!(stable_five_supports_framing, Some("5.0.0"), true, true);
capability_case!(five_one_supports_framing, Some("5.1.0"), true, true);
capability_case!(major_six_supports_framing, Some("6.0.0"), true, true);
