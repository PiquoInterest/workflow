#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalPackageFixture {
    pub package_name: String,
    pub module_entry_source: String,
    pub main_entry_source: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildFixture {
    pub file_name: String,
    pub workflow_source: String,
    pub external_packages: Vec<ExternalPackageFixture>,
}

impl BuildFixture {
    pub fn new(file_name: &str, workflow_source: &str) -> Self {
        Self {
            file_name: file_name.to_owned(),
            workflow_source: workflow_source.to_owned(),
            external_packages: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BuildObservation {
    pub succeeded: bool,
    pub output: String,
    pub reported_modules: Vec<String>,
}

/// Runs the future Rust workflow build-time sandbox validation boundary.
pub fn analyze_workflow_build(fixture: &BuildFixture) -> BuildObservation {
    let _ = fixture;
    panic!("TDD RED: packages/core/e2e/build-errors.test.ts implementation pending")
}
