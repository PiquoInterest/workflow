use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ModuleFormat {
    #[default]
    Esm,
    Cjs,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeModuleBoundaryOptions {
    pub entry_file: PathBuf,
    pub format: ModuleFormat,
    pub main_fields: Vec<String>,
}

impl NodeModuleBoundaryOptions {
    pub fn new(entry_file: PathBuf) -> Self {
        Self {
            entry_file,
            format: ModuleFormat::Esm,
            main_fields: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViolationLocation {
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub line_text: String,
    pub length: usize,
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleViolation {
    pub text: String,
    pub location: Option<ViolationLocation>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NodeModuleBoundaryObservation {
    pub errors: Vec<ModuleViolation>,
}

pub fn inspect_node_module_boundary(
    options: &NodeModuleBoundaryOptions,
) -> Result<NodeModuleBoundaryObservation, String> {
    let _ = options;
    panic!(
        "TDD RED: packages/builders/src/node-module-esbuild-plugin.test.ts implementation pending"
    )
}

pub fn get_package_name(path: &str) -> Option<String> {
    let _ = path;
    panic!(
        "TDD RED: packages/builders/src/node-module-esbuild-plugin.test.ts implementation pending"
    )
}

pub fn escape_reg_exp(value: &str) -> String {
    let _ = value;
    panic!(
        "TDD RED: packages/builders/src/node-module-esbuild-plugin.test.ts implementation pending"
    )
}

pub fn get_imported_identifier(import_clause: &str) -> Option<String> {
    let _ = import_clause;
    panic!(
        "TDD RED: packages/builders/src/node-module-esbuild-plugin.test.ts implementation pending"
    )
}

pub fn get_violation_location(
    cwd: &Path,
    relative_file: &Path,
    package_name: &str,
) -> Option<ViolationLocation> {
    let _ = (cwd, relative_file, package_name);
    panic!(
        "TDD RED: packages/builders/src/node-module-esbuild-plugin.test.ts implementation pending"
    )
}
