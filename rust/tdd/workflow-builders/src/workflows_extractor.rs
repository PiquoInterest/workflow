use std::collections::BTreeMap;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkflowGraphNode {
    Step { label: String, step_id: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WorkflowGraph {
    pub nodes: Vec<WorkflowGraphNode>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtractedWorkflow {
    pub workflow_id: String,
    pub graph: WorkflowGraph,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WorkflowExtraction {
    pub workflows: BTreeMap<String, BTreeMap<String, ExtractedWorkflow>>,
    pub diagnostics: Vec<String>,
}

pub fn extract_workflow_graphs(bundle_path: &Path) -> Result<WorkflowExtraction, String> {
    let _ = bundle_path;
    panic!("TDD RED: packages/builders/src/workflows-extractor.test.ts implementation pending")
}
