#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct WorkflowPatternDetection {
    pub has_use_workflow: bool,
    pub has_use_step: bool,
    pub has_serde_import: bool,
    pub has_serde_symbol: bool,
    pub has_directive: bool,
    pub has_serde: bool,
}

pub fn matches_use_workflow(source: &str) -> bool {
    let _ = source;
    panic!("TDD RED: packages/builders/src/transform-utils.test.ts implementation pending")
}

pub fn matches_use_step(source: &str) -> bool {
    let _ = source;
    panic!("TDD RED: packages/builders/src/transform-utils.test.ts implementation pending")
}

pub fn matches_workflow_serde_import(source: &str) -> bool {
    let _ = source;
    panic!("TDD RED: packages/builders/src/transform-utils.test.ts implementation pending")
}

pub fn matches_workflow_serde_symbol(source: &str) -> bool {
    let _ = source;
    panic!("TDD RED: packages/builders/src/transform-utils.test.ts implementation pending")
}

pub fn matches_workflow_serde_computed_property(source: &str) -> bool {
    let _ = source;
    panic!("TDD RED: packages/builders/src/transform-utils.test.ts implementation pending")
}

pub fn detect_workflow_patterns(source: &str) -> WorkflowPatternDetection {
    let _ = source;
    panic!("TDD RED: packages/builders/src/transform-utils.test.ts implementation pending")
}

pub fn should_transform_file(path: &str, patterns: WorkflowPatternDetection) -> bool {
    let _ = (path, patterns);
    panic!("TDD RED: packages/builders/src/transform-utils.test.ts implementation pending")
}
