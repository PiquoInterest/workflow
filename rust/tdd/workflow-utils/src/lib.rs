#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::marker::PhantomData;
use std::sync::Arc;

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

impl From<workflow_utils::ParsedName> for ParsedName {
    fn from(parsed: workflow_utils::ParsedName) -> Self {
        Self {
            short_name: parsed.short_name,
            module_specifier: parsed.module_specifier,
            function_name: parsed.function_name,
        }
    }
}

#[must_use]
pub fn parse_workflow_name(name: &str) -> Option<ParsedName> {
    workflow_utils::parse_workflow_name(name).map(Into::into)
}

#[must_use]
pub fn parse_step_name(name: &str) -> Option<ParsedName> {
    workflow_utils::parse_step_name(name).map(Into::into)
}

#[must_use]
pub fn parse_class_name(name: &str) -> Option<ParsedName> {
    workflow_utils::parse_class_name(name).map(Into::into)
}

#[must_use]
pub fn format_step_name(name: &str) -> String {
    workflow_utils::format_step_name(name)
}

#[must_use]
pub fn format_workflow_name(name: &str) -> String {
    workflow_utils::format_workflow_name(name)
}

#[must_use]
pub fn step_display_name(name: &str) -> String {
    workflow_utils::step_display_name(name)
}

#[must_use]
pub fn workflow_display_name(name: &str) -> String {
    workflow_utils::workflow_display_name(name)
}

/// Planned process-wide, versioned registry corresponding to the JavaScript
/// `globalThis` singleton helper. Rust unit tests cover registry semantics;
/// JavaScript binding tests must still prove cross-module realm sharing.
#[derive(Debug, Default)]
pub struct GlobalSingletonRegistry;

impl GlobalSingletonRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    pub fn global_singleton<T, F>(&self, name: &str, shape_version: u32, create: F) -> Arc<T>
    where
        T: Send + Sync + 'static,
        F: FnOnce() -> T,
    {
        let _ = (name, shape_version, create);
        panic!("TDD RED: packages/utils/src/global-singleton.test.ts implementation pending")
    }

    pub fn get<T>(&self, name: &str, shape_version: u32) -> Option<Arc<T>>
    where
        T: Send + Sync + 'static,
    {
        let _ = (name, shape_version);
        panic!("TDD RED: packages/utils/src/global-singleton.test.ts implementation pending")
    }

    pub fn reset_for_test(&self, name: &str, shape_version: u32) {
        let _ = (name, shape_version);
        panic!("TDD RED: packages/utils/src/global-singleton.test.ts implementation pending")
    }
}

/// Planned resolver/promise core. The eventual JavaScript binding must expose
/// native Promise semantics on top of this one-shot state transition.
#[derive(Debug)]
pub struct Deferred<T> {
    marker: PhantomData<T>,
}

impl<T> Deferred<T> {
    #[must_use]
    pub fn new() -> Self {
        panic!("TDD RED: packages/utils/src/promise.test.ts implementation pending")
    }

    #[must_use]
    pub fn is_pending(&self) -> bool {
        let _ = &self.marker;
        panic!("TDD RED: packages/utils/src/promise.test.ts implementation pending")
    }

    pub fn resolve(&self, value: T) {
        let _ = value;
        panic!("TDD RED: packages/utils/src/promise.test.ts implementation pending")
    }

    pub fn reject(&self, reason: impl Into<String>) {
        let _ = reason.into();
        panic!("TDD RED: packages/utils/src/promise.test.ts implementation pending")
    }

    pub fn wait(self) -> Result<T, String> {
        panic!("TDD RED: packages/utils/src/promise.test.ts implementation pending")
    }
}

impl<T> Default for Deferred<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct OnceValue<T, F> {
    marker: PhantomData<(T, F)>,
}

#[must_use]
pub fn once<T, F>(initializer: F) -> OnceValue<T, F>
where
    F: FnOnce() -> T,
{
    let _ = initializer;
    panic!("TDD RED: packages/utils/src/promise.test.ts implementation pending")
}

impl<T, F> OnceValue<T, F>
where
    F: FnOnce() -> T,
{
    #[must_use]
    pub fn value(&self) -> &T {
        let _ = &self.marker;
        panic!("TDD RED: packages/utils/src/promise.test.ts implementation pending")
    }
}

mod check_data_dir;
#[path = "debug_log.rs"]
mod debug_log_contract;
mod get_port;
mod module_scope_state;

pub use check_data_dir::{
    DataDirContext, POSSIBLE_WORKFLOW_DATA_PATHS, WorkflowDataDirResult, find_workflow_data_dir,
};
pub use debug_log_contract::{DebugArgument, DebugSink, debug_log, is_workflow_debug_enabled};
pub use get_port::{
    WorkflowPortOptions, get_all_ports, get_port, get_workflow_port,
    parse_windows_netstat_ports_for_pid,
};
pub use module_scope_state::{
    ModuleScopeFinding, discover_bundled_runtime_packages, format_module_scope_findings,
    scan_module_scope_sources, scan_package,
};
