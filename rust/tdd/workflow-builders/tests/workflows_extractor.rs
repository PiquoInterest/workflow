use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use workflow_builders_tdd::workflows_extractor::{WorkflowGraphNode, extract_workflow_graphs};

struct TestRoot(PathBuf);

impl TestRoot {
    fn new(label: &str) -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path =
            std::env::temp_dir().join(format!("workflow-{label}-{}-{nonce}", std::process::id()));
        fs::create_dir_all(&path).unwrap();
        Self(path.canonicalize().unwrap())
    }
}

impl Drop for TestRoot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn parses_template_literals_with_unicode_escape_identifiers() {
    let root = TestRoot::new("workflow-extractor-unicode");
    let bundle_path = root.0.join("workflow-bundle.js");
    fs::write(
        &bundle_path,
        [
            "const workflowCode = `",
            "function workflow() {",
            "  var DEBURR_MAP = new Map(Object.entries({\\\\u00C6: \"Ae\"}));",
            "  return DEBURR_MAP;",
            "}",
            "workflow.workflowId = \"workflow//./input.js//workflow\";",
            "`;",
        ]
        .join("\n"),
    )
    .unwrap();

    let extraction = extract_workflow_graphs(&bundle_path).unwrap();
    assert!(extraction.diagnostics.is_empty());
    assert_eq!(
        extraction.workflows["./input.js"]["workflow"].workflow_id,
        "workflow//./input.js//workflow"
    );
}

#[test]
fn extracts_step_nodes_when_proxies_include_pure_annotations() {
    let root = TestRoot::new("workflow-extractor-pure-step");
    let bundle_path = root.0.join("workflow-bundle.js");
    fs::write(
        &bundle_path,
        [
            "var stepOne = globalThis[/* @__PURE__ */ Symbol.for(\"WORKFLOW_USE_STEP\")](\"step//./input.ts//stepOne\");",
            "async function testWorkflow(input) {",
            "  const output = await stepOne(input);",
            "  return output;",
            "}",
            "testWorkflow.workflowId = \"workflow//./input.ts//testWorkflow\";",
        ]
        .join("\n"),
    )
    .unwrap();

    let extraction = extract_workflow_graphs(&bundle_path).unwrap();
    let workflow = &extraction.workflows["./input.ts"]["testWorkflow"];
    assert_eq!(workflow.workflow_id, "workflow//./input.ts//testWorkflow");
    assert!(workflow.graph.nodes.iter().any(|node| {
        matches!(
            node,
            WorkflowGraphNode::Step { label, step_id }
                if label == "stepOne" && step_id == "step//./input.ts//stepOne"
        )
    }));
}
