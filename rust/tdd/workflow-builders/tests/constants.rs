use workflow_builders_tdd::constants::{
    WorkflowEntrypointOptions, create_workflow_entrypoint_options_code,
    create_workflow_queue_trigger, get_workflow_queue_trigger,
};

#[test]
fn get_trigger_omits_max_concurrency_by_default() {
    let trigger = get_workflow_queue_trigger(None, None, None);

    assert_eq!(trigger.topic, "__wkf_workflow_*");
    assert_eq!(trigger.max_concurrency, None);
}

#[test]
fn get_trigger_sets_max_concurrency_for_exact_sequential_flag() {
    let trigger = get_workflow_queue_trigger(None, None, Some("1"));

    assert_eq!(trigger.topic, "__wkf_workflow_*");
    assert_eq!(trigger.max_concurrency, Some(1));
}

#[test]
fn get_trigger_ignores_non_one_sequential_values() {
    let trigger = get_workflow_queue_trigger(None, None, Some("true"));

    assert_eq!(trigger.max_concurrency, None);
}

#[test]
fn get_trigger_composes_sequential_replay_with_explicit_namespace() {
    let trigger = get_workflow_queue_trigger(Some("custom"), None, Some("1"));

    assert_eq!(trigger.topic, "__custom_wkf_workflow_*");
    assert_eq!(trigger.max_concurrency, Some(1));
}

#[test]
fn get_trigger_resolves_environment_namespace_at_call_time() {
    let trigger = get_workflow_queue_trigger(None, Some("callns"), None);

    assert_eq!(trigger.topic, "__callns_wkf_workflow_*");
}

#[test]
fn create_trigger_uses_default_topic_without_namespace() {
    let trigger = create_workflow_queue_trigger(None, None);

    assert_eq!(trigger.topic, "__wkf_workflow_*");
}

#[test]
fn create_trigger_uses_explicit_namespace() {
    let trigger = create_workflow_queue_trigger(Some("custom"), None);

    assert_eq!(trigger.topic, "__custom_wkf_workflow_*");
}

#[test]
fn create_trigger_uses_environment_namespace_when_explicit_is_absent() {
    let trigger = create_workflow_queue_trigger(None, Some("custom"));

    assert_eq!(trigger.topic, "__custom_wkf_workflow_*");
}

#[test]
fn entrypoint_options_omit_runtime_options_without_namespace() {
    let code = create_workflow_entrypoint_options_code(
        &WorkflowEntrypointOptions::default(),
        None,
    );

    assert_eq!(code, "");
}

#[test]
fn entrypoint_options_inline_explicit_namespace() {
    let code = create_workflow_entrypoint_options_code(
        &WorkflowEntrypointOptions {
            namespace: Some("custom".to_owned()),
            ..WorkflowEntrypointOptions::default()
        },
        None,
    );

    assert_eq!(code, ", { namespace: \"custom\" }");
}

#[test]
fn entrypoint_options_inline_environment_namespace_at_build_time() {
    let code = create_workflow_entrypoint_options_code(
        &WorkflowEntrypointOptions::default(),
        Some("custom"),
    );

    assert_eq!(code, ", { namespace: \"custom\" }");
}

#[test]
fn entrypoint_options_inline_route_timing_with_namespace_and_base_path() {
    let code = create_workflow_entrypoint_options_code(
        &WorkflowEntrypointOptions {
            namespace: Some("custom".to_owned()),
            base_path: Some("/v2".to_owned()),
            route_module_body_started_at: Some(
                "workflowRouteModuleBodyStartedAt".to_owned(),
            ),
        },
        None,
    );

    assert_eq!(
        code,
        ", { namespace: \"custom\", basePath: \"/v2\", routeModuleBodyStartedAt: workflowRouteModuleBodyStartedAt }"
    );
}
