use workflow_cli_tdd::vercel_api::get_vercel_dashboard_url;

#[test]
fn builds_a_run_deep_link_with_the_default_environment() {
    assert_eq!(
        get_vercel_dashboard_url("my-team", "my-project", "run", Some("wrun_123"), None),
        "https://vercel.com/my-team/my-project/workflows/runs/wrun_123?environment=production"
    );
}

#[test]
fn respects_the_preview_environment() {
    assert_eq!(
        get_vercel_dashboard_url(
            "my-team",
            "my-project",
            "run",
            Some("wrun_123"),
            Some("preview"),
        ),
        "https://vercel.com/my-team/my-project/workflows/runs/wrun_123?environment=preview"
    );
}

#[test]
fn never_emits_the_legacy_observability_segment() {
    let url = get_vercel_dashboard_url(
        "my-team",
        "my-project",
        "run",
        Some("wrun_123"),
        Some("preview"),
    );
    assert!(!url.contains("/observability"));
}

#[test]
fn builds_an_overview_link_when_no_id_is_provided() {
    assert_eq!(
        get_vercel_dashboard_url("my-team", "my-project", "run", None, None),
        "https://vercel.com/my-team/my-project/workflows?environment=production"
    );
}

#[test]
fn builds_a_resource_query_link_for_non_run_resources() {
    assert_eq!(
        get_vercel_dashboard_url(
            "my-team",
            "my-project",
            "step",
            Some("step_456"),
            Some("preview"),
        ),
        "https://vercel.com/my-team/my-project/workflows?stepId=step_456&environment=preview"
    );
}
