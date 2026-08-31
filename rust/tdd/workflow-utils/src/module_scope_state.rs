use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleScopeFinding {
    pub name: String,
    pub keyword: Option<String>,
    pub reason: Option<String>,
}

#[must_use]
pub fn discover_bundled_runtime_packages(repo_root: &Path) -> Vec<PathBuf> {
    let _ = repo_root;
    panic!("TDD RED: packages/utils/src/module-scope-state.test.ts implementation pending")
}

#[must_use]
pub fn scan_package(package_dir: &Path, repo_root: &Path) -> Vec<ModuleScopeFinding> {
    let _ = (package_dir, repo_root);
    panic!("TDD RED: packages/utils/src/module-scope-state.test.ts implementation pending")
}

#[must_use]
pub fn scan_module_scope_sources(
    files: &BTreeMap<String, String>,
) -> Vec<ModuleScopeFinding> {
    let _ = files;
    panic!("TDD RED: packages/utils/src/module-scope-state.test.ts implementation pending")
}

#[must_use]
pub fn format_module_scope_findings(findings: &[ModuleScopeFinding]) -> String {
    let _ = findings;
    panic!("TDD RED: packages/utils/src/module-scope-state.test.ts implementation pending")
}
