use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportPath {
    pub import_path: String,
    pub is_package: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleSpecifierResolution {
    pub module_specifier: Option<String>,
}

pub fn get_import_path(file_path: &Path, project_root: &Path) -> Result<ImportPath, String> {
    let _ = (file_path, project_root);
    panic!("TDD RED: packages/builders/src/module-specifier.test.ts implementation pending")
}

pub fn resolve_module_specifier(
    file_path: &Path,
    project_root: &Path,
) -> Result<ModuleSpecifierResolution, String> {
    let _ = (file_path, project_root);
    panic!("TDD RED: packages/builders/src/module-specifier.test.ts implementation pending")
}
