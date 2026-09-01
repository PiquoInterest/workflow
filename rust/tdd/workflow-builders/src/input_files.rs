use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuilderTarget {
    Standalone,
    VercelBuildOutputApi,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputFilesConfig {
    pub build_target: BuilderTarget,
    pub working_dir: PathBuf,
    pub dirs: Vec<PathBuf>,
    pub diagnostics_dir: Option<PathBuf>,
    pub target_world: Option<String>,
}

pub fn get_input_files(config: &InputFilesConfig) -> Result<Vec<PathBuf>, String> {
    let _ = config;
    panic!("TDD RED: packages/builders/src/get-input-files.test.ts implementation pending")
}

pub fn ensure_swc_ignored(config: &InputFilesConfig) -> Result<(), String> {
    let _ = config;
    panic!("TDD RED: packages/builders/src/get-input-files.test.ts implementation pending")
}

pub fn get_diagnostics_manifest_path(config: &InputFilesConfig) -> Option<PathBuf> {
    let _ = config;
    panic!("TDD RED: packages/builders/src/get-input-files.test.ts implementation pending")
}
