use workflow_utils::{
    Environment, is_vercel_world_target, resolve_workflow_target_world, uses_vercel_world,
};

fn environment(entries: &[(&str, &str)]) -> Environment {
    entries
        .iter()
        .map(|(key, value)| ((*key).to_owned(), (*value).to_owned()))
        .collect()
}

fn vercel_environment_without_deployment() -> Environment {
    environment(&[
        ("VERCEL", "1"),
        ("VERCEL_ENV", "production"),
        ("VERCEL_TARGET_ENV", "production"),
        ("VERCEL_URL", "example.vercel.app"),
        ("NODE_ENV", "production"),
    ])
}

#[test]
fn resolves_local_when_vercel_environment_has_no_deployment_id() {
    assert_eq!(
        resolve_workflow_target_world(&vercel_environment_without_deployment()),
        "local"
    );
}

#[test]
fn workflow_target_world_can_opt_a_local_vercel_like_process_in() {
    let mut values = vercel_environment_without_deployment();
    values.insert("WORKFLOW_TARGET_WORLD".to_owned(), "vercel".to_owned());
    assert_eq!(resolve_workflow_target_world(&values), "vercel");
}

#[test]
fn workflow_target_world_can_opt_a_deployment_out_to_local() {
    let values = environment(&[
        ("VERCEL_DEPLOYMENT_ID", "dpl_123"),
        ("WORKFLOW_TARGET_WORLD", "local"),
    ]);
    assert_eq!(resolve_workflow_target_world(&values), "local");
}

#[test]
fn returns_an_explicitly_configured_world() {
    let values = environment(&[
        ("WORKFLOW_TARGET_WORLD", "@workflow/world-postgres"),
        ("VERCEL_DEPLOYMENT_ID", "deployment-id"),
    ]);
    assert_eq!(
        resolve_workflow_target_world(&values),
        "@workflow/world-postgres"
    );
}

#[test]
fn defaults_to_vercel_when_a_deployment_id_exists() {
    let values = environment(&[("VERCEL_DEPLOYMENT_ID", "deployment-id")]);
    assert_eq!(resolve_workflow_target_world(&values), "vercel");
}

#[test]
fn defaults_to_local_when_no_world_environment_is_set() {
    assert_eq!(resolve_workflow_target_world(&Environment::new()), "local");
}

#[test]
fn recognizes_vercel_world_targets() {
    assert!(is_vercel_world_target("vercel"));
    assert!(is_vercel_world_target("@workflow/world-vercel"));
}

#[test]
fn rejects_non_vercel_world_targets() {
    assert!(!is_vercel_world_target("local"));
    assert!(!is_vercel_world_target("@workflow/world-postgres"));
}

#[test]
fn uses_vercel_world_for_a_resolved_vercel_deployment() {
    let values = environment(&[("VERCEL_DEPLOYMENT_ID", "deployment-id")]);
    assert!(uses_vercel_world(&values));
}

#[test]
fn does_not_use_vercel_world_for_a_resolved_local_process() {
    assert!(!uses_vercel_world(&Environment::new()));
}
