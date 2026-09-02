use workflow_builders_tdd::sourcemap::{
    SourcemapConfig, SourcemapMode, default_sourcemap_mode, is_development_build,
    resolve_sourcemap, sourcemaps_enabled,
};

fn config(explicit: Option<SourcemapMode>, watch: bool) -> SourcemapConfig {
    SourcemapConfig {
        explicit,
        watch,
        ..SourcemapConfig::default()
    }
}

#[test]
fn resolve_sourcemap_returns_the_default_without_config_or_environment() {
    let config = config(None, false);
    assert_eq!(
        resolve_sourcemap(&config, SourcemapMode::Inline),
        SourcemapMode::Inline
    );
    assert_eq!(
        resolve_sourcemap(&config, SourcemapMode::Disabled),
        SourcemapMode::Disabled
    );
    assert_eq!(
        resolve_sourcemap(&config, SourcemapMode::Enabled),
        SourcemapMode::Enabled
    );
}

#[test]
fn resolve_sourcemap_prefers_explicit_config_over_the_default() {
    for (explicit, default_mode, expected) in [
        (
            SourcemapMode::Disabled,
            SourcemapMode::Inline,
            SourcemapMode::Disabled,
        ),
        (
            SourcemapMode::External,
            SourcemapMode::Inline,
            SourcemapMode::External,
        ),
        (
            SourcemapMode::Linked,
            SourcemapMode::Disabled,
            SourcemapMode::Linked,
        ),
        (
            SourcemapMode::Enabled,
            SourcemapMode::Inline,
            SourcemapMode::Enabled,
        ),
    ] {
        assert_eq!(
            resolve_sourcemap(&config(Some(explicit), false), default_mode),
            expected
        );
    }
}

#[test]
fn resolve_sourcemap_prefers_explicit_config_over_environment() {
    let mut disabled = config(Some(SourcemapMode::Disabled), false);
    disabled.environment = Some("inline".to_owned());
    assert_eq!(
        resolve_sourcemap(&disabled, SourcemapMode::Inline),
        SourcemapMode::Disabled
    );

    let mut external = config(Some(SourcemapMode::External), false);
    external.environment = Some("inline".to_owned());
    assert_eq!(
        resolve_sourcemap(&external, SourcemapMode::Inline),
        SourcemapMode::External
    );
}

#[test]
fn resolve_sourcemap_uses_environment_when_config_is_absent() {
    for (raw, default_mode, expected) in [
        ("false", SourcemapMode::Inline, SourcemapMode::Disabled),
        ("true", SourcemapMode::Disabled, SourcemapMode::Enabled),
        ("inline", SourcemapMode::Inline, SourcemapMode::Inline),
        ("linked", SourcemapMode::Inline, SourcemapMode::Linked),
        ("external", SourcemapMode::Inline, SourcemapMode::External),
        ("both", SourcemapMode::Inline, SourcemapMode::Both),
    ] {
        let mut config = config(None, false);
        config.environment = Some(raw.to_owned());
        assert_eq!(resolve_sourcemap(&config, default_mode), expected);
    }
}

#[test]
fn resolve_sourcemap_accepts_numeric_boolean_aliases() {
    let mut disabled = config(None, false);
    disabled.environment = Some("0".to_owned());
    assert_eq!(
        resolve_sourcemap(&disabled, SourcemapMode::Inline),
        SourcemapMode::Disabled
    );

    let mut enabled = config(None, false);
    enabled.environment = Some("1".to_owned());
    assert_eq!(
        resolve_sourcemap(&enabled, SourcemapMode::Disabled),
        SourcemapMode::Enabled
    );
}

#[test]
fn resolve_sourcemap_falls_back_for_empty_or_unrecognized_environment() {
    for raw in ["", "nonsense"] {
        let mut config = config(None, false);
        config.environment = Some(raw.to_owned());
        assert_eq!(
            resolve_sourcemap(&config, SourcemapMode::Inline),
            SourcemapMode::Inline
        );
    }
}

#[test]
fn production_defaults_to_disabled_sourcemaps() {
    let mut config = config(None, false);
    config.node_environment = Some("production".to_owned());
    assert!(!is_development_build(&config));
    assert_eq!(default_sourcemap_mode(&config), SourcemapMode::Disabled);
}

#[test]
fn watch_mode_defaults_to_inline_sourcemaps() {
    let mut config = config(None, true);
    config.node_environment = Some("production".to_owned());
    assert!(is_development_build(&config));
    assert_eq!(default_sourcemap_mode(&config), SourcemapMode::Inline);
}

#[test]
fn development_environment_defaults_to_inline_sourcemaps() {
    let mut config = config(None, false);
    config.node_environment = Some("development".to_owned());
    assert!(is_development_build(&config));
    assert_eq!(default_sourcemap_mode(&config), SourcemapMode::Inline);
}

#[test]
fn sourcemaps_are_disabled_by_default_in_production() {
    let mut config = config(None, false);
    config.node_environment = Some("production".to_owned());
    assert!(!sourcemaps_enabled(&config));
}

#[test]
fn sourcemaps_are_enabled_by_default_for_watch_builds() {
    let mut config = config(None, true);
    config.node_environment = Some("production".to_owned());
    assert!(sourcemaps_enabled(&config));
}

#[test]
fn sourcemaps_are_enabled_by_default_in_development() {
    let mut config = config(None, false);
    config.node_environment = Some("development".to_owned());
    assert!(sourcemaps_enabled(&config));
}

#[test]
fn explicit_false_disables_sourcemaps() {
    let mut config = config(Some(SourcemapMode::Disabled), false);
    config.node_environment = Some("production".to_owned());
    assert!(!sourcemaps_enabled(&config));
}

#[test]
fn every_non_false_explicit_mode_enables_sourcemaps() {
    for mode in [
        SourcemapMode::Enabled,
        SourcemapMode::Inline,
        SourcemapMode::Linked,
        SourcemapMode::External,
        SourcemapMode::Both,
    ] {
        let mut config = config(Some(mode), false);
        config.node_environment = Some("production".to_owned());
        assert!(sourcemaps_enabled(&config));
    }
}

#[test]
fn environment_can_enable_sourcemaps_in_production() {
    let mut config = config(None, false);
    config.node_environment = Some("production".to_owned());
    config.environment = Some("inline".to_owned());
    assert!(sourcemaps_enabled(&config));
}

#[test]
fn false_environment_disables_sourcemaps() {
    let mut config = config(None, false);
    config.node_environment = Some("production".to_owned());
    config.environment = Some("false".to_owned());
    assert!(!sourcemaps_enabled(&config));
}
