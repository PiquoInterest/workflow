#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleScopeFinding {
    pub path: String,
    pub line: usize,
    pub description: String,
}

/// Scans the Rust core package for prohibited mutable module-scope state.
pub fn scan_core_module_scope_state() -> Vec<ModuleScopeFinding> {
    panic!("TDD RED: packages/core/src/module-scope-state.test.ts implementation pending")
}
