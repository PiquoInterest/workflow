#![forbid(unsafe_code)]

pub mod check_data_dir;

pub use check_data_dir::{
    DataDirContext, POSSIBLE_WORKFLOW_DATA_PATHS, WorkflowDataDirResult, find_workflow_data_dir,
};
