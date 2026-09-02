use std::collections::BTreeMap;

pub type Environment = BTreeMap<String, String>;

#[must_use]
pub fn resolve_workflow_target_world(environment: &Environment) -> String {
    if let Some(configured_world) = environment
        .get("WORKFLOW_TARGET_WORLD")
        .filter(|value| !value.is_empty())
    {
        return configured_world.clone();
    }

    if environment
        .get("VERCEL_DEPLOYMENT_ID")
        .is_some_and(|value| !value.is_empty())
    {
        "vercel".to_owned()
    } else {
        "local".to_owned()
    }
}

#[must_use]
pub fn is_vercel_world_target(target_world: &str) -> bool {
    matches!(target_world, "vercel" | "@workflow/world-vercel")
}

#[must_use]
pub fn uses_vercel_world(environment: &Environment) -> bool {
    is_vercel_world_target(&resolve_workflow_target_world(environment))
}
