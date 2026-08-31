#![forbid(unsafe_code)]

use std::collections::BTreeMap;

/// Test-first API skeleton for the TypeScript `pluralize` contract.
///
/// The behavior is intentionally absent. The translated Rust tests must be
/// observed failing before a production implementation is added.
#[must_use]
pub fn pluralize<'a>(singular: &'a str, plural: &'a str, count: f64) -> &'a str {
    let _ = (singular, plural, count);
    panic!("TDD RED: packages/utils/src/pluralize.test.ts implementation pending")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkflowRoute {
    Flow,
    Health,
    Step,
}

#[derive(Debug, Default)]
pub struct WorkflowRoutes {
    base_path: Option<String>,
}

impl WorkflowRoutes {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_workflow_base_path(&mut self, base_path: Option<&str>) {
        let _ = base_path;
        panic!("TDD RED: packages/utils/src/workflow-routes.test.ts implementation pending")
    }

    pub fn create_workflow_url(
        &self,
        origin: &str,
        route: WorkflowRoute,
    ) -> Result<String, String> {
        let _ = (&self.base_path, origin, route);
        panic!("TDD RED: packages/utils/src/workflow-routes.test.ts implementation pending")
    }

    pub fn create_workflow_health_endpoint(&self) -> String {
        let _ = &self.base_path;
        panic!("TDD RED: packages/utils/src/workflow-routes.test.ts implementation pending")
    }
}

pub type Environment = BTreeMap<String, String>;

#[must_use]
pub fn resolve_workflow_target_world(environment: &Environment) -> String {
    let _ = environment;
    panic!("TDD RED: packages/utils/src/world-target.test.ts implementation pending")
}

#[must_use]
pub fn is_vercel_world_target(target: &str) -> bool {
    let _ = target;
    panic!("TDD RED: packages/utils/src/world-target.test.ts implementation pending")
}

#[must_use]
pub fn uses_vercel_world(environment: &Environment) -> bool {
    let _ = environment;
    panic!("TDD RED: packages/utils/src/world-target.test.ts implementation pending")
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedName {
    pub short_name: String,
    pub module_specifier: String,
    pub function_name: String,
}

#[must_use]
pub fn parse_workflow_name(name: &str) -> Option<ParsedName> {
    let _ = name;
    panic!("TDD RED: packages/utils/src/parse-name.test.ts implementation pending")
}

#[must_use]
pub fn parse_step_name(name: &str) -> Option<ParsedName> {
    let _ = name;
    panic!("TDD RED: packages/utils/src/parse-name.test.ts implementation pending")
}

#[must_use]
pub fn parse_class_name(name: &str) -> Option<ParsedName> {
    let _ = name;
    panic!("TDD RED: packages/utils/src/parse-name.test.ts implementation pending")
}

#[must_use]
pub fn format_step_name(name: &str) -> String {
    let _ = name;
    panic!("TDD RED: packages/utils/src/parse-name.test.ts implementation pending")
}

#[must_use]
pub fn format_workflow_name(name: &str) -> String {
    let _ = name;
    panic!("TDD RED: packages/utils/src/parse-name.test.ts implementation pending")
}

#[must_use]
pub fn step_display_name(name: &str) -> String {
    let _ = name;
    panic!("TDD RED: packages/utils/src/parse-name.test.ts implementation pending")
}

#[must_use]
pub fn workflow_display_name(name: &str) -> String {
    let _ = name;
    panic!("TDD RED: packages/utils/src/parse-name.test.ts implementation pending")
}
