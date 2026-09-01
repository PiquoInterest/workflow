use std::fmt::{self, Display, Formatter};

pub const MANIFEST_VERSION: &str = "1.0.0";
pub const TDD_RED_MARKER: &str =
    "TDD RED: packages/core/e2e/manifest.test.ts implementation pending";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManifestProject {
    NextWebpack,
    NextTurbopack,
    Nitro,
    Vite,
    SvelteKit,
    Nuxt,
    Hono,
    Express,
}

pub const ALL_MANIFEST_PROJECTS: [ManifestProject; 8] = [
    ManifestProject::NextWebpack,
    ManifestProject::NextTurbopack,
    ManifestProject::Nitro,
    ManifestProject::Vite,
    ManifestProject::SvelteKit,
    ManifestProject::Nuxt,
    ManifestProject::Hono,
    ManifestProject::Express,
];

pub const DOT_DIRECTORY_PROJECTS: [ManifestProject; 2] =
    [ManifestProject::NextWebpack, ManifestProject::NextTurbopack];

impl ManifestProject {
    pub const fn app_name(self) -> &'static str {
        match self {
            Self::NextWebpack => "nextjs-webpack",
            Self::NextTurbopack => "nextjs-turbopack",
            Self::Nitro => "nitro",
            Self::Vite => "vite",
            Self::SvelteKit => "sveltekit",
            Self::Nuxt => "nuxt",
            Self::Hono => "hono",
            Self::Express => "express",
        }
    }

    pub const fn manifest_path(self) -> &'static str {
        match self {
            Self::NextWebpack | Self::NextTurbopack => "app/.well-known/workflow/v1/manifest.json",
            Self::SvelteKit => "src/routes/.well-known/workflow/v1/manifest.json",
            Self::Nitro | Self::Vite | Self::Nuxt | Self::Hono | Self::Express => {
                "node_modules/.nitro/workflow/manifest.json"
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConditionalBranch {
    Then,
    Else,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NodeMetadata {
    pub loop_id: Option<String>,
    pub loop_is_await: Option<bool>,
    pub conditional_id: Option<String>,
    pub conditional_branch: Option<ConditionalBranch>,
    pub parallel_group_id: Option<String>,
    pub parallel_method: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManifestNode {
    pub id: String,
    pub node_type: String,
    pub label: String,
    pub node_kind: String,
    pub step_id: Option<String>,
    pub metadata: Option<NodeMetadata>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManifestEdge {
    pub id: String,
    pub source: String,
    pub target: String,
    pub edge_type: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ManifestGraph {
    pub nodes: Vec<ManifestNode>,
    pub edges: Vec<ManifestEdge>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManifestStep {
    pub name: String,
    pub step_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManifestStepFile {
    pub path: String,
    pub steps: Vec<ManifestStep>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManifestWorkflow {
    pub name: String,
    pub workflow_id: String,
    pub graph: ManifestGraph,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManifestWorkflowFile {
    pub path: String,
    pub workflows: Vec<ManifestWorkflow>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Manifest {
    pub version: String,
    pub steps: Vec<ManifestStepFile>,
    pub workflows: Vec<ManifestWorkflowFile>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManifestValidationError {
    pub message: String,
}

impl Display for ManifestValidationError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

fn invalid(message: impl Into<String>) -> ManifestValidationError {
    ManifestValidationError {
        message: message.into(),
    }
}

fn require_text(value: &str, field: &str) -> Result<(), ManifestValidationError> {
    if value.is_empty() {
        return Err(invalid(format!("manifest {field} must not be empty")));
    }
    Ok(())
}

pub fn validate_graph(graph: &ManifestGraph) -> Result<(), ManifestValidationError> {
    for node in &graph.nodes {
        require_text(&node.id, "node id")?;
        require_text(&node.node_type, "node type")?;
        require_text(&node.label, "node label")?;
        require_text(&node.node_kind, "node kind")?;
    }
    for edge in &graph.edges {
        require_text(&edge.id, "edge id")?;
        require_text(&edge.source, "edge source")?;
        require_text(&edge.target, "edge target")?;
    }
    if !graph.nodes.is_empty() {
        let has_start = graph
            .nodes
            .iter()
            .any(|node| node.node_type == "workflowStart");
        let has_end = graph
            .nodes
            .iter()
            .any(|node| node.node_type == "workflowEnd");
        if !has_start || !has_end {
            return Err(invalid(
                "non-empty workflow graphs require workflowStart and workflowEnd nodes",
            ));
        }
    }
    Ok(())
}

pub fn validate_manifest(manifest: &Manifest) -> Result<(), ManifestValidationError> {
    if manifest.version != MANIFEST_VERSION {
        return Err(invalid(format!(
            "manifest version must be {MANIFEST_VERSION}"
        )));
    }
    if manifest.steps.is_empty() {
        return Err(invalid("manifest steps must contain at least one file"));
    }
    if manifest.workflows.is_empty() {
        return Err(invalid("manifest workflows must contain at least one file"));
    }

    for file in &manifest.steps {
        if file.path.contains("builtins.js") {
            continue;
        }
        for step in &file.steps {
            if !step.step_id.contains("step//") || !step.step_id.contains(&step.name) {
                return Err(invalid(format!("step {} has an invalid stepId", step.name)));
            }
        }
    }
    for file in &manifest.workflows {
        for workflow in &file.workflows {
            if !workflow.workflow_id.contains("workflow//")
                || !workflow.workflow_id.contains(&workflow.name)
            {
                return Err(invalid(format!(
                    "workflow {} has an invalid workflowId",
                    workflow.name
                )));
            }
            validate_graph(&workflow.graph)?;
        }
    }
    Ok(())
}

pub fn find_workflow<'a>(
    manifest: &'a Manifest,
    workflow_name: &str,
) -> Option<&'a ManifestWorkflow> {
    manifest
        .workflows
        .iter()
        .flat_map(|file| file.workflows.iter())
        .find(|workflow| workflow.name == workflow_name)
}

pub fn step_nodes(graph: &ManifestGraph) -> Vec<&ManifestNode> {
    graph
        .nodes
        .iter()
        .filter(|node| node.step_id.is_some())
        .collect()
}

pub fn validate_dot_directory_step(manifest: &Manifest) -> Result<(), ManifestValidationError> {
    let file = manifest
        .steps
        .iter()
        .find(|file| {
            file.path.contains(".well-known/agent") || file.path.contains("well-known/agent")
        })
        .ok_or_else(|| invalid("manifest is missing a .well-known/agent step file"))?;
    let step = file
        .steps
        .iter()
        .find(|step| step.name == "wellKnownAgentStep")
        .ok_or_else(|| invalid("manifest is missing wellKnownAgentStep"))?;
    if !step.step_id.contains("wellKnownAgentStep") {
        return Err(invalid("wellKnownAgentStep has an invalid stepId"));
    }
    Ok(())
}

pub fn validate_dot_directory_workflow(manifest: &Manifest) -> Result<(), ManifestValidationError> {
    let file = manifest
        .workflows
        .iter()
        .find(|file| {
            file.path.contains(".well-known/agent") || file.path.contains("well-known/agent")
        })
        .ok_or_else(|| invalid("manifest is missing a .well-known/agent workflow file"))?;
    let workflow = file
        .workflows
        .iter()
        .find(|workflow| workflow.name == "wellKnownAgentWorkflow")
        .ok_or_else(|| invalid("manifest is missing wellKnownAgentWorkflow"))?;
    if !workflow.workflow_id.contains("wellKnownAgentWorkflow") {
        return Err(invalid("wellKnownAgentWorkflow has an invalid workflowId"));
    }
    Ok(())
}

fn has_step_name(nodes: &[&ManifestNode], name: &str) -> bool {
    nodes
        .iter()
        .filter_map(|node| node.step_id.as_deref())
        .any(|step_id| step_id.contains(name))
}

pub fn validate_single_statement_if(
    workflow: &ManifestWorkflow,
) -> Result<(), ManifestValidationError> {
    let nodes = step_nodes(&workflow.graph);
    if nodes.is_empty()
        || !has_step_name(&nodes, "singleStmtStepA")
        || !has_step_name(&nodes, "singleStmtStepB")
    {
        return Err(invalid("single-statement if is missing expected steps"));
    }
    let has_conditional = nodes.iter().any(|node| {
        node.metadata
            .as_ref()
            .and_then(|metadata| metadata.conditional_id.as_ref())
            .is_some()
    });
    let has_then = nodes.iter().any(|node| {
        node.metadata
            .as_ref()
            .and_then(|metadata| metadata.conditional_branch)
            == Some(ConditionalBranch::Then)
    });
    let has_else = nodes.iter().any(|node| {
        node.metadata
            .as_ref()
            .and_then(|metadata| metadata.conditional_branch)
            == Some(ConditionalBranch::Else)
    });
    if !has_conditional || !has_then || !has_else {
        return Err(invalid(
            "single-statement if is missing conditional branch metadata",
        ));
    }
    Ok(())
}

fn validate_single_statement_loop(
    workflow: &ManifestWorkflow,
    required_steps: &[&str],
) -> Result<(), ManifestValidationError> {
    let nodes = step_nodes(&workflow.graph);
    if nodes.is_empty()
        || required_steps
            .iter()
            .any(|name| !has_step_name(&nodes, name))
    {
        return Err(invalid("single-statement loop is missing expected steps"));
    }
    if !nodes.iter().any(|node| {
        node.metadata
            .as_ref()
            .and_then(|metadata| metadata.loop_id.as_ref())
            .is_some()
    }) {
        return Err(invalid("single-statement loop is missing loop metadata"));
    }
    if !workflow
        .graph
        .edges
        .iter()
        .any(|edge| edge.edge_type.as_deref() == Some("loop"))
    {
        return Err(invalid("single-statement loop is missing a loop edge"));
    }
    Ok(())
}

pub fn validate_single_statement_while(
    workflow: &ManifestWorkflow,
) -> Result<(), ManifestValidationError> {
    validate_single_statement_loop(workflow, &["singleStmtStepA"])
}

pub fn validate_single_statement_for(
    workflow: &ManifestWorkflow,
) -> Result<(), ManifestValidationError> {
    validate_single_statement_loop(workflow, &["singleStmtStepB", "singleStmtStepC"])
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManifestReadFailure {
    NotFound,
    Io(String),
    Parse(String),
}

pub fn gate_manifest_read(
    result: Result<Manifest, ManifestReadFailure>,
) -> Result<Option<Manifest>, ManifestReadFailure> {
    match result {
        Ok(manifest) => Ok(Some(manifest)),
        Err(ManifestReadFailure::NotFound) => Ok(None),
        Err(error @ ManifestReadFailure::Io(_)) | Err(error @ ManifestReadFailure::Parse(_)) => {
            Err(error)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManifestScenario {
    Structure,
    DotDirectoryStep,
    DotDirectoryWorkflow,
    SingleStatementIf,
    SingleStatementWhile,
    SingleStatementFor,
}

/// Reads the future Rust-generated manifest for one workbench project.
///
/// Missing manifests preserve the source suite's optional-project skip. Parse
/// and I/O failures must remain errors instead of silently becoming skips.
pub fn probe_manifest(
    project: ManifestProject,
    scenario: ManifestScenario,
) -> Result<Option<Manifest>, ManifestReadFailure> {
    let _ = (project, scenario);
    panic!("{TDD_RED_MARKER}")
}
