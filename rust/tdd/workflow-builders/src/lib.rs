#![forbid(unsafe_code)]

use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EsbuildTsconfigOptions {
    pub tsconfig: Option<PathBuf>,
    pub tsconfig_raw: Option<String>,
    pub alias_base: Option<PathBuf>,
}

pub fn resolve_project_root(app_root: &Path) -> PathBuf {
    let _ = app_root;
    panic!("TDD RED: packages/builders/src/config-helpers.test.ts implementation pending")
}

pub fn get_esbuild_tsconfig_options(tsconfig_path: &Path) -> EsbuildTsconfigOptions {
    let _ = tsconfig_path;
    panic!("TDD RED: packages/builders/src/esbuild-tsconfig.test.ts implementation pending")
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogCall {
    pub message: String,
    pub trailing: String,
}

pub fn build_progress_log(debug: Option<&str>, parts: &[&str]) -> Option<LogCall> {
    let _ = (debug, parts);
    panic!("TDD RED: packages/builders/src/base-builder-logging.test.ts implementation pending")
}

pub fn compile_summary(step_count: usize, workflow_count: usize, elapsed_ms: u64) -> String {
    let _ = (step_count, workflow_count, elapsed_ms);
    panic!("TDD RED: packages/builders/src/base-builder-logging.test.ts implementation pending")
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct VercelEsmBuildObservation {
    pub step_result: String,
    pub combined_file_url_import_count: usize,
    pub combined_dirname_definition_count: usize,
    pub webhook_file_url_import_count: usize,
    pub webhook_dirname_definition_count: usize,
}

pub fn build_vercel_esm_fixture() -> VercelEsmBuildObservation {
    panic!(
        "TDD RED: packages/builders/src/vercel-build-output-api.test.ts implementation pending"
    )
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StepRegistrationFixture {
    SerdeOnlyFile,
    IdenticalPnpmPeerCopies,
    DivergentPnpmPeerCopies,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct StepRegistrationOutput {
    pub generated_source: String,
    pub class_files: Vec<String>,
    pub manifest_step_ids: Vec<String>,
}

pub fn create_step_registrations(
    fixture: StepRegistrationFixture,
) -> Result<StepRegistrationOutput, String> {
    let _ = fixture;
    panic!(
        "TDD RED: packages/builders/src/step-source-registration.test.ts implementation pending"
    )
}
