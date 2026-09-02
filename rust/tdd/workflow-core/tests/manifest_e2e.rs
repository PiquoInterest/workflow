use workflow_core_tdd::manifest_e2e::{
    ALL_MANIFEST_PROJECTS, ConditionalBranch, DOT_DIRECTORY_PROJECTS, Manifest, ManifestEdge,
    ManifestGraph, ManifestNode, ManifestProject, ManifestReadFailure, ManifestScenario,
    ManifestStep, ManifestStepFile, ManifestWorkflow, ManifestWorkflowFile, gate_manifest_read,
    probe_manifest, validate_dot_directory_step, validate_dot_directory_workflow,
    validate_manifest, validate_single_statement_for, validate_single_statement_if,
    validate_single_statement_while,
};

fn node(id: &str, node_type: &str, step_id: Option<&str>) -> ManifestNode {
    ManifestNode {
        id: id.to_owned(),
        node_type: node_type.to_owned(),
        label: id.to_owned(),
        node_kind: node_type.to_owned(),
        step_id: step_id.map(str::to_owned),
        metadata: None,
    }
}

fn graph_with_steps(step_names: &[&str]) -> ManifestGraph {
    let mut nodes = vec![
        node("start", "workflowStart", None),
        node("end", "workflowEnd", None),
    ];
    nodes.extend(step_names.iter().enumerate().map(|(index, name)| {
        node(
            &format!("step-{index}"),
            "step",
            Some(&format!("step//fixture//{name}")),
        )
    }));
    ManifestGraph {
        nodes,
        edges: vec![ManifestEdge {
            id: "edge-1".to_owned(),
            source: "start".to_owned(),
            target: "end".to_owned(),
            edge_type: None,
        }],
    }
}

fn valid_manifest() -> Manifest {
    Manifest {
        version: "1.0.0".to_owned(),
        steps: vec![ManifestStepFile {
            path: "workflows/simple.ts".to_owned(),
            steps: vec![ManifestStep {
                name: "simpleStep".to_owned(),
                step_id: "step//workflows/simple//simpleStep".to_owned(),
            }],
        }],
        workflows: vec![ManifestWorkflowFile {
            path: "workflows/simple.ts".to_owned(),
            workflows: vec![ManifestWorkflow {
                name: "simple".to_owned(),
                workflow_id: "workflow//workflows/simple//simple".to_owned(),
                graph: graph_with_steps(&["simpleStep"]),
            }],
        }],
    }
}

fn validate_probe(project: ManifestProject, scenario: ManifestScenario) {
    let Some(manifest) =
        probe_manifest(project, scenario).expect("manifest read must not be hidden")
    else {
        return;
    };
    match scenario {
        ManifestScenario::Structure => validate_manifest(&manifest).unwrap(),
        ManifestScenario::DotDirectoryStep => validate_dot_directory_step(&manifest).unwrap(),
        ManifestScenario::DotDirectoryWorkflow => {
            validate_dot_directory_workflow(&manifest).unwrap()
        }
        ManifestScenario::SingleStatementIf => {
            if let Some(workflow) =
                workflow_core_tdd::manifest_e2e::find_workflow(&manifest, "single_statement_if")
            {
                validate_single_statement_if(workflow).unwrap();
            }
        }
        ManifestScenario::SingleStatementWhile => {
            if let Some(workflow) =
                workflow_core_tdd::manifest_e2e::find_workflow(&manifest, "single_statement_while")
            {
                validate_single_statement_while(workflow).unwrap();
            }
        }
        ManifestScenario::SingleStatementFor => {
            if let Some(workflow) =
                workflow_core_tdd::manifest_e2e::find_workflow(&manifest, "single_statement_for")
            {
                validate_single_statement_for(workflow).unwrap();
            }
        }
    }
}

#[test]
fn maps_every_project_to_the_exact_manifest_path() {
    let paths = ALL_MANIFEST_PROJECTS
        .iter()
        .map(|project| (project.app_name(), project.manifest_path()))
        .collect::<Vec<_>>();
    assert_eq!(
        paths,
        vec![
            (
                "nextjs-webpack",
                "app/.well-known/workflow/v1/manifest.json"
            ),
            (
                "nextjs-turbopack",
                "app/.well-known/workflow/v1/manifest.json",
            ),
            ("nitro", "node_modules/.nitro/workflow/manifest.json"),
            ("vite", "node_modules/.nitro/workflow/manifest.json"),
            (
                "sveltekit",
                "src/routes/.well-known/workflow/v1/manifest.json",
            ),
            ("nuxt", "node_modules/.nitro/workflow/manifest.json"),
            ("hono", "node_modules/.nitro/workflow/manifest.json"),
            ("express", "node_modules/.nitro/workflow/manifest.json"),
        ]
    );
}

#[test]
fn skips_only_missing_manifests_and_fails_closed_on_parse_or_io_errors() {
    assert_eq!(
        gate_manifest_read(Err(ManifestReadFailure::NotFound)),
        Ok(None)
    );
    assert_eq!(
        gate_manifest_read(Err(ManifestReadFailure::Parse("bad json".to_owned()))),
        Err(ManifestReadFailure::Parse("bad json".to_owned()))
    );
    assert_eq!(
        gate_manifest_read(Err(ManifestReadFailure::Io("permission denied".to_owned()))),
        Err(ManifestReadFailure::Io("permission denied".to_owned()))
    );
}

#[test]
fn validates_manifest_ids_and_non_empty_graph_boundaries() {
    let manifest = valid_manifest();
    assert!(validate_manifest(&manifest).is_ok());

    let mut wrong_version = manifest.clone();
    wrong_version.version = "2.0.0".to_owned();
    assert!(validate_manifest(&wrong_version).is_err());

    let mut bad_step = manifest.clone();
    bad_step.steps[0].steps[0].step_id = "not-a-step-id".to_owned();
    assert!(validate_manifest(&bad_step).is_err());

    let mut missing_end = manifest;
    missing_end.workflows[0].workflows[0]
        .graph
        .nodes
        .retain(|node| node.node_type != "workflowEnd");
    assert!(validate_manifest(&missing_end).is_err());
}

#[test]
fn validates_dot_directory_and_single_statement_metadata_contracts() {
    let mut manifest = valid_manifest();
    manifest.steps.push(ManifestStepFile {
        path: "app/.well-known/agent/v1/steps.ts".to_owned(),
        steps: vec![ManifestStep {
            name: "wellKnownAgentStep".to_owned(),
            step_id: "step//.well-known/agent//wellKnownAgentStep".to_owned(),
        }],
    });
    manifest.workflows.push(ManifestWorkflowFile {
        path: "app/.well-known/agent/v1/steps.ts".to_owned(),
        workflows: vec![ManifestWorkflow {
            name: "wellKnownAgentWorkflow".to_owned(),
            workflow_id: "workflow//.well-known/agent//wellKnownAgentWorkflow".to_owned(),
            graph: graph_with_steps(&[]),
        }],
    });
    assert!(validate_dot_directory_step(&manifest).is_ok());
    assert!(validate_dot_directory_workflow(&manifest).is_ok());

    let mut if_workflow = ManifestWorkflow {
        name: "single_statement_if".to_owned(),
        workflow_id: "workflow//fixture//single_statement_if".to_owned(),
        graph: graph_with_steps(&["singleStmtStepA", "singleStmtStepB"]),
    };
    if_workflow.graph.nodes[2].metadata = Some(workflow_core_tdd::manifest_e2e::NodeMetadata {
        conditional_id: Some("condition-1".to_owned()),
        conditional_branch: Some(ConditionalBranch::Then),
        ..Default::default()
    });
    if_workflow.graph.nodes[3].metadata = Some(workflow_core_tdd::manifest_e2e::NodeMetadata {
        conditional_id: Some("condition-1".to_owned()),
        conditional_branch: Some(ConditionalBranch::Else),
        ..Default::default()
    });
    assert!(validate_single_statement_if(&if_workflow).is_ok());

    let mut while_workflow = ManifestWorkflow {
        name: "single_statement_while".to_owned(),
        workflow_id: "workflow//fixture//single_statement_while".to_owned(),
        graph: graph_with_steps(&["singleStmtStepA"]),
    };
    while_workflow.graph.nodes[2].metadata = Some(workflow_core_tdd::manifest_e2e::NodeMetadata {
        loop_id: Some("loop-1".to_owned()),
        ..Default::default()
    });
    while_workflow.graph.edges.push(ManifestEdge {
        id: "loop-edge".to_owned(),
        source: "step-0".to_owned(),
        target: "step-0".to_owned(),
        edge_type: Some("loop".to_owned()),
    });
    assert!(validate_single_statement_while(&while_workflow).is_ok());

    let mut for_workflow = ManifestWorkflow {
        name: "single_statement_for".to_owned(),
        workflow_id: "workflow//fixture//single_statement_for".to_owned(),
        graph: graph_with_steps(&["singleStmtStepB", "singleStmtStepC"]),
    };
    for node in &mut for_workflow.graph.nodes[2..] {
        node.metadata = Some(workflow_core_tdd::manifest_e2e::NodeMetadata {
            loop_id: Some("loop-2".to_owned()),
            ..Default::default()
        });
    }
    for_workflow.graph.edges.push(ManifestEdge {
        id: "loop-edge".to_owned(),
        source: "step-1".to_owned(),
        target: "step-0".to_owned(),
        edge_type: Some("loop".to_owned()),
    });
    assert!(validate_single_statement_for(&for_workflow).is_ok());
}

macro_rules! project_case {
    ($name:ident, $project:expr, $scenario:expr) => {
        #[test]
        fn $name() {
            validate_probe($project, $scenario);
        }
    };
}

project_case!(
    next_webpack_manifest_structure,
    ManifestProject::NextWebpack,
    ManifestScenario::Structure
);
project_case!(
    next_turbopack_manifest_structure,
    ManifestProject::NextTurbopack,
    ManifestScenario::Structure
);
project_case!(
    nitro_manifest_structure,
    ManifestProject::Nitro,
    ManifestScenario::Structure
);
project_case!(
    vite_manifest_structure,
    ManifestProject::Vite,
    ManifestScenario::Structure
);
project_case!(
    sveltekit_manifest_structure,
    ManifestProject::SvelteKit,
    ManifestScenario::Structure
);
project_case!(
    nuxt_manifest_structure,
    ManifestProject::Nuxt,
    ManifestScenario::Structure
);
project_case!(
    hono_manifest_structure,
    ManifestProject::Hono,
    ManifestScenario::Structure
);
project_case!(
    express_manifest_structure,
    ManifestProject::Express,
    ManifestScenario::Structure
);

project_case!(
    next_webpack_dot_directory_step,
    DOT_DIRECTORY_PROJECTS[0],
    ManifestScenario::DotDirectoryStep
);
project_case!(
    next_turbopack_dot_directory_step,
    DOT_DIRECTORY_PROJECTS[1],
    ManifestScenario::DotDirectoryStep
);
project_case!(
    next_webpack_dot_directory_workflow,
    DOT_DIRECTORY_PROJECTS[0],
    ManifestScenario::DotDirectoryWorkflow
);
project_case!(
    next_turbopack_dot_directory_workflow,
    DOT_DIRECTORY_PROJECTS[1],
    ManifestScenario::DotDirectoryWorkflow
);

project_case!(
    next_webpack_single_if,
    ManifestProject::NextWebpack,
    ManifestScenario::SingleStatementIf
);
project_case!(
    next_turbopack_single_if,
    ManifestProject::NextTurbopack,
    ManifestScenario::SingleStatementIf
);
project_case!(
    nitro_single_if,
    ManifestProject::Nitro,
    ManifestScenario::SingleStatementIf
);
project_case!(
    vite_single_if,
    ManifestProject::Vite,
    ManifestScenario::SingleStatementIf
);
project_case!(
    sveltekit_single_if,
    ManifestProject::SvelteKit,
    ManifestScenario::SingleStatementIf
);
project_case!(
    nuxt_single_if,
    ManifestProject::Nuxt,
    ManifestScenario::SingleStatementIf
);
project_case!(
    hono_single_if,
    ManifestProject::Hono,
    ManifestScenario::SingleStatementIf
);
project_case!(
    express_single_if,
    ManifestProject::Express,
    ManifestScenario::SingleStatementIf
);

project_case!(
    next_webpack_single_while,
    ManifestProject::NextWebpack,
    ManifestScenario::SingleStatementWhile
);
project_case!(
    next_turbopack_single_while,
    ManifestProject::NextTurbopack,
    ManifestScenario::SingleStatementWhile
);
project_case!(
    nitro_single_while,
    ManifestProject::Nitro,
    ManifestScenario::SingleStatementWhile
);
project_case!(
    vite_single_while,
    ManifestProject::Vite,
    ManifestScenario::SingleStatementWhile
);
project_case!(
    sveltekit_single_while,
    ManifestProject::SvelteKit,
    ManifestScenario::SingleStatementWhile
);
project_case!(
    nuxt_single_while,
    ManifestProject::Nuxt,
    ManifestScenario::SingleStatementWhile
);
project_case!(
    hono_single_while,
    ManifestProject::Hono,
    ManifestScenario::SingleStatementWhile
);
project_case!(
    express_single_while,
    ManifestProject::Express,
    ManifestScenario::SingleStatementWhile
);

project_case!(
    next_webpack_single_for,
    ManifestProject::NextWebpack,
    ManifestScenario::SingleStatementFor
);
project_case!(
    next_turbopack_single_for,
    ManifestProject::NextTurbopack,
    ManifestScenario::SingleStatementFor
);
project_case!(
    nitro_single_for,
    ManifestProject::Nitro,
    ManifestScenario::SingleStatementFor
);
project_case!(
    vite_single_for,
    ManifestProject::Vite,
    ManifestScenario::SingleStatementFor
);
project_case!(
    sveltekit_single_for,
    ManifestProject::SvelteKit,
    ManifestScenario::SingleStatementFor
);
project_case!(
    nuxt_single_for,
    ManifestProject::Nuxt,
    ManifestScenario::SingleStatementFor
);
project_case!(
    hono_single_for,
    ManifestProject::Hono,
    ManifestScenario::SingleStatementFor
);
project_case!(
    express_single_for,
    ManifestProject::Express,
    ManifestScenario::SingleStatementFor
);
