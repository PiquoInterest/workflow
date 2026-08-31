use std::path::PathBuf;

pub const POSSIBLE_WORKFLOW_DATA_PATHS: &[&str] =
    &[".next/workflow-data", ".workflow-data", "workflow-data"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataDirContext {
    pub cwd: PathBuf,
    pub home: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkflowDataDirResult {
    pub data_dir: Option<PathBuf>,
    pub project_dir: PathBuf,
    pub short_name: String,
    pub error: Option<String>,
}

#[must_use]
pub fn find_workflow_data_dir(input: &str, context: &DataDirContext) -> WorkflowDataDirResult {
    let _ = (input, context);
    panic!("TDD RED: packages/utils/src/check-data-dir.test.ts implementation pending")
}
