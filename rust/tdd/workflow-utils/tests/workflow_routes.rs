use workflow_utils::{WorkflowRoute, WorkflowRoutes};

#[test]
fn builds_urls_for_the_combined_flow_route() {
    let routes = WorkflowRoutes::new();
    assert_eq!(
        routes
            .create_workflow_url("https://example.com", WorkflowRoute::Flow)
            .unwrap(),
        "https://example.com/.well-known/workflow/v1/flow"
    );
    assert_eq!(
        routes
            .create_workflow_url("https://example.com", WorkflowRoute::Health)
            .unwrap(),
        "https://example.com/.well-known/workflow/v1/flow?__health"
    );
}

#[test]
fn applies_a_configured_base_path() {
    let mut routes = WorkflowRoutes::new();
    routes.set_workflow_base_path(Some("/base"));
    assert_eq!(
        routes.create_workflow_health_endpoint(),
        "/base/.well-known/workflow/v1/flow?__health"
    );
}

#[test]
fn rejects_the_retired_standalone_step_route_at_runtime() {
    let routes = WorkflowRoutes::new();
    assert_eq!(
        routes
            .create_workflow_url("https://example.com", WorkflowRoute::Step)
            .unwrap_err(),
        "Unsupported workflow route: step"
    );
}

#[test]
fn builds_manifest_and_percent_encoded_webhook_routes() {
    let routes = WorkflowRoutes::new();
    assert_eq!(
        routes
            .create_workflow_url(
                "https://example.com/base/?old=1#fragment",
                WorkflowRoute::Manifest,
            )
            .unwrap(),
        "https://example.com/base/.well-known/workflow/v1/manifest.json"
    );
    assert_eq!(
        routes
            .create_workflow_url(
                "https://example.com",
                WorkflowRoute::Webhook("a/b ?#💥".to_owned()),
            )
            .unwrap(),
        "https://example.com/.well-known/workflow/v1/webhook/a%2Fb%20%3F%23%F0%9F%92%A5"
    );
}

#[test]
fn builds_a_base_url_after_removing_query_fragment_and_trailing_slashes() {
    let mut routes = WorkflowRoutes::new();
    routes.set_workflow_base_path(Some("/base"));
    assert_eq!(
        routes
            .create_workflow_base_url("https://example.com/app///?old=1#fragment")
            .unwrap(),
        "https://example.com/app/base"
    );
}

#[test]
fn rejects_relative_or_whitespace_confused_base_urls() {
    let routes = WorkflowRoutes::new();
    for invalid in ["/relative", "https://", "https://exa mple.com"] {
        assert!(
            routes
                .create_workflow_url(invalid, WorkflowRoute::Flow)
                .is_err(),
            "expected invalid URL to be rejected: {invalid:?}"
        );
    }
}
