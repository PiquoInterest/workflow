#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RuntimeImportObservation {
    pub runtime_defined: bool,
    pub loaded_platform_modules: Vec<String>,
}

/// Evaluates the runtime entrypoint without platform adapter side effects.
pub fn import_runtime() -> RuntimeImportObservation {
    panic!("TDD RED: packages/core/src/runtime-import.test.ts implementation pending")
}
