use workflow_builders_tdd::{build_progress_log, compile_summary};

#[test]
fn hides_progress_logs_by_default() {
    assert_eq!(
        build_progress_log(None, &["Created step registrations", "10ms"]),
        None
    );
}

#[test]
fn shows_progress_logs_when_debug_matches_workflow_build() {
    let call = build_progress_log(
        Some("workflow:build"),
        &["Created step registrations", "10ms"],
    )
    .unwrap();

    assert_eq!(
        call.message,
        "[workflow:build] Created step registrations 10ms"
    );
    assert_eq!(call.trailing, "");
}

#[test]
fn shows_progress_logs_for_matching_wildcards() {
    for selector in ["workflow:*", "*"] {
        let call =
            build_progress_log(Some(selector), &["Created step registrations", "10ms"]).unwrap();
        assert_eq!(
            call.message,
            "[workflow:build] Created step registrations 10ms"
        );
    }
}

#[test]
fn hides_progress_logs_when_debug_negates_workflow_build() {
    assert_eq!(
        build_progress_log(
            Some("workflow:*,-workflow:build"),
            &["Created step registrations", "10ms"],
        ),
        None
    );
}

#[test]
fn emits_next_like_compile_summaries_every_time_a_manifest_is_created() {
    let first = compile_summary(2, 1, 10);
    let second = compile_summary(2, 1, 10);

    assert_eq!(first, "✓ Compiled workflows in 10ms (2 steps, 1 workflow)");
    assert_eq!(second, first);
}
