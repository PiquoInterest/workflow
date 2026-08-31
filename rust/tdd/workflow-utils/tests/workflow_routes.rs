use workflow_utils_tdd::{WorkflowRoute, WorkflowRoutes};

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
