use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use workflow_builders_tdd::fast_discovery::{
    FastDiscoveryConfig, create_route_import_specifier, discover_fast_entries,
};

struct TestRoot(PathBuf);

impl TestRoot {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "workflow-fast-discovery-rust-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir_all(&path).unwrap();
        Self(path.canonicalize().unwrap())
    }

    fn join(&self, relative: &str) -> PathBuf {
        self.0.join(relative)
    }
}

impl Drop for TestRoot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn normalize(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn write_file(path: &Path, contents: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, contents).unwrap();
}

fn config(root: &TestRoot) -> FastDiscoveryConfig {
    FastDiscoveryConfig {
        working_dir: root.0.clone(),
        discover_workflows_in_node_modules: true,
        tsconfig_path: None,
    }
}

fn discover(
    root: &TestRoot,
    inputs: &[PathBuf],
    discover_node_modules: bool,
    tsconfig_path: Option<PathBuf>,
) -> workflow_builders_tdd::fast_discovery::FastDiscoveryObservation {
    discover_fast_entries(
        &FastDiscoveryConfig {
            discover_workflows_in_node_modules: discover_node_modules,
            tsconfig_path,
            ..config(root)
        },
        inputs,
        &root.join("out"),
    )
    .unwrap()
}

#[test]
fn discovers_transitive_relative_step_imports_and_tracks_the_parent_chain() {
    let root = TestRoot::new();
    let entry = root.join("src/entry.ts");
    let workflow = root.join("src/workflow.ts");
    let step = root.join("src/step.ts");
    write_file(&entry, "import './workflow';\n");
    write_file(
        &workflow,
        "import { doStep } from './step';\nvoid doStep;\n",
    );
    write_file(
        &step,
        "export async function doStep() {\n  'use step';\n  return 1;\n}\n",
    );

    let observation = discover(&root, &[entry.clone()], true, None);
    assert_eq!(
        observation.discovered_steps,
        BTreeSet::from([normalize(&step)])
    );
    assert!(observation.parent_has_child(&normalize(&entry), &normalize(&step)));
}

#[test]
fn discovers_relative_js_imports_whose_basename_includes_step() {
    let root = TestRoot::new();
    let entry = root.join("src/entry.ts");
    let workflow = root.join("src/hello.step.js");
    write_file(&entry, "import './hello.step';\n");
    write_file(
        &workflow,
        "export async function run() {\n  'use workflow';\n  return 'ok';\n}\n",
    );

    let observation = discover(&root, &[entry.clone()], true, None);
    assert_eq!(
        observation.discovered_workflows,
        BTreeSet::from([normalize(&workflow)])
    );
    assert!(observation.parent_has_child(&normalize(&entry), &normalize(&workflow)));
}

#[test]
fn discovers_workflow_files_reached_through_an_imported_package_reexport() {
    let root = TestRoot::new();
    let entry = root.join("src/entry.ts");
    let package_root = root.join("node_modules/workflow-pkg");
    let package_index = package_root.join("index.js");
    let package_workflow = package_root.join("workflow.js");
    write_file(&entry, "import { run } from 'workflow-pkg';\nvoid run;\n");
    write_file(
        &package_root.join("package.json"),
        r#"{"name":"workflow-pkg","version":"1.0.0","main":"index.js","dependencies":{"workflow":"^1.0.0"}}"#,
    );
    write_file(&package_index, "export { run } from './workflow.js';\n");
    write_file(
        &package_workflow,
        "export async function run() {\n  \"use workflow\";\n  return \"ok\";\n}\n",
    );

    let observation = discover(&root, &[entry], true, None);
    assert_eq!(
        observation.discovered_workflows,
        BTreeSet::from([normalize(&package_workflow)])
    );
    assert!(
        observation.parent_has_child(&normalize(&package_index), &normalize(&package_workflow))
    );
}

#[test]
fn does_not_descend_into_node_modules_when_disabled() {
    let root = TestRoot::new();
    let entry = root.join("src/entry.ts");
    let local_workflow = root.join("src/local-workflow.ts");
    let package_root = root.join("node_modules/workflow-pkg");
    let package_index = package_root.join("index.js");
    let package_workflow = package_root.join("workflow.js");
    let package_step = package_root.join("step.js");
    write_file(
        &entry,
        "import './local-workflow';\nimport { run } from 'workflow-pkg';\nvoid run;\n",
    );
    write_file(
        &local_workflow,
        "export async function localRun() {\n  'use workflow';\n  return 'ok';\n}\n",
    );
    write_file(
        &package_root.join("package.json"),
        r#"{"name":"workflow-pkg","version":"1.0.0","main":"index.js","dependencies":{"workflow":"^1.0.0"}}"#,
    );
    write_file(
        &package_index,
        "export { run } from './workflow.js';\nexport { doWork } from './step.js';\n",
    );
    write_file(
        &package_workflow,
        "export async function run() {\n  \"use workflow\";\n  return \"ok\";\n}\n",
    );
    write_file(
        &package_step,
        "export async function doWork() {\n  \"use step\";\n  return \"done\";\n}\n",
    );

    let observation = discover(&root, &[entry.clone()], false, None);
    assert_eq!(
        observation.discovered_workflows,
        BTreeSet::from([normalize(&local_workflow)])
    );
    assert!(
        !observation
            .discovered_workflows
            .contains(&normalize(&package_workflow))
    );
    assert!(
        !observation
            .discovered_steps
            .contains(&normalize(&package_step))
    );
    for file in [&package_index, &package_workflow, &package_step] {
        assert!(!observation.discovered_files.contains(&normalize(file)));
    }
    assert!(!observation.parent_has_child(&normalize(&entry), &normalize(&package_index)));
}

#[test]
fn seeded_node_modules_entries_still_resolve_their_own_subtree() {
    let root = TestRoot::new();
    let entry = root.join("src/entry.ts");
    let package_root = root.join("node_modules/seeded-pkg");
    let package_index = package_root.join("index.js");
    let package_workflow = package_root.join("workflow.js");
    write_file(&entry, "export const noop = 1;\n");
    write_file(
        &package_root.join("package.json"),
        r#"{"name":"seeded-pkg","version":"1.0.0","main":"index.js","dependencies":{"workflow":"^1.0.0"}}"#,
    );
    write_file(&package_index, "export { run } from './workflow.js';\n");
    write_file(
        &package_workflow,
        "export async function run() {\n  \"use workflow\";\n  return \"ok\";\n}\n",
    );

    let observation = discover(&root, &[entry, package_index.clone()], false, None);
    assert!(
        observation
            .discovered_workflows
            .contains(&normalize(&package_workflow))
    );
    assert!(
        observation.parent_has_child(&normalize(&package_index), &normalize(&package_workflow))
    );
}

#[test]
fn discovers_files_reached_through_tsconfig_path_aliases() {
    let root = TestRoot::new();
    let entry = root.join("src/entry.ts");
    let registry = root.join("src/_workflows.ts");
    let workflow = root.join("src/workflows/workflow.ts");
    let tsconfig = root.join("tsconfig.json");
    write_file(
        &tsconfig,
        r#"{"compilerOptions":{"paths":{"@/*":["./src/*"]}}}"#,
    );
    write_file(&entry, "import { allWorkflows } from '@/_workflows';\n");
    write_file(
        &registry,
        "import * as workflow from './workflows/workflow';\nexport const allWorkflows = { workflow };\n",
    );
    write_file(
        &workflow,
        "export async function run() {\n  'use workflow';\n  return 'ok';\n}\n",
    );

    let observation = discover(&root, &[entry.clone()], true, Some(tsconfig));
    assert_eq!(
        observation.discovered_workflows,
        BTreeSet::from([normalize(&workflow)])
    );
    assert!(observation.parent_has_child(&normalize(&entry), &normalize(&workflow)));
}

#[test]
fn discovers_dotted_files_reached_through_tsconfig_path_aliases() {
    let root = TestRoot::new();
    let entry = root.join("src/entry.ts");
    let registry = root.join("src/workflows/hello.index.ts");
    let workflow = root.join("src/workflows/hello.ts");
    let tsconfig = root.join("tsconfig.json");
    write_file(
        &tsconfig,
        r#"{"compilerOptions":{"paths":{"@/*":["./src/*"]}}}"#,
    );
    write_file(
        &entry,
        "import { helloWorkflow } from '@/workflows/hello.index';\n",
    );
    write_file(&registry, "export { helloWorkflow } from './hello';\n");
    write_file(
        &workflow,
        "export async function helloWorkflow() {\n  'use workflow';\n  return 'ok';\n}\n",
    );

    let observation = discover(&root, &[entry], true, Some(tsconfig));
    assert_eq!(
        observation.discovered_workflows,
        BTreeSet::from([normalize(&workflow)])
    );
}

#[test]
fn ignores_non_source_files_reached_through_tsconfig_aliases() {
    let root = TestRoot::new();
    let entry = root.join("src/entry.ts");
    let asset = root.join("src/styles/app.css");
    let tsconfig = root.join("tsconfig.json");
    write_file(
        &tsconfig,
        r#"{"compilerOptions":{"paths":{"@/*":["./src/*"]}}}"#,
    );
    write_file(&entry, "import '@/styles/app.css';\n");
    write_file(&asset, "'use workflow';\n");

    let observation = discover(&root, &[entry.clone()], true, Some(tsconfig));
    assert!(observation.discovered_workflows.is_empty());
    assert_eq!(
        observation.discovered_files,
        BTreeSet::from([normalize(&entry)])
    );
}

#[test]
fn discovers_path_aliases_inherited_through_tsconfig_extends() {
    let root = TestRoot::new();
    let entry = root.join("src/entry.ts");
    let workflow = root.join("src/workflows/workflow.ts");
    let base_tsconfig = root.join("tsconfig.base.json");
    let tsconfig = root.join("tsconfig.json");
    write_file(
        &base_tsconfig,
        r#"{"compilerOptions":{"baseUrl":".","paths":{"@base/*":["./src/*"]}}}"#,
    );
    write_file(&tsconfig, r#"{"extends":"./tsconfig.base.json"}"#);
    write_file(&entry, "import { run } from '@base/workflows/workflow';\n");
    write_file(
        &workflow,
        "export async function run() {\n  'use workflow';\n  return 'ok';\n}\n",
    );

    let observation = discover(&root, &[entry], true, Some(tsconfig));
    assert_eq!(
        observation.discovered_workflows,
        BTreeSet::from([normalize(&workflow)])
    );
}

#[test]
fn discovers_path_aliases_with_multiple_wildcards() {
    let root = TestRoot::new();
    let entry = root.join("src/entry.ts");
    let workflow = root.join("src/features/billing/flows/charge.ts");
    let tsconfig = root.join("tsconfig.json");
    write_file(
        &tsconfig,
        r#"{"compilerOptions":{"paths":{"@feature/*/workflow/*":["./src/features/*/flows/*"]}}}"#,
    );
    write_file(
        &entry,
        "import { charge } from '@feature/billing/workflow/charge';\n",
    );
    write_file(
        &workflow,
        "export async function charge() {\n  'use workflow';\n  return 'ok';\n}\n",
    );

    let observation = discover(&root, &[entry], true, Some(tsconfig));
    assert_eq!(
        observation.discovered_workflows,
        BTreeSet::from([normalize(&workflow)])
    );
}

#[test]
fn ignores_imports_that_only_appear_inside_comments() {
    let root = TestRoot::new();
    let entry = root.join("src/entry.ts");
    let registry = root.join("src/_workflows.ts");
    let workflow = root.join("src/workflows/simple.ts");
    write_file(&entry, "import './_workflows';\n");
    write_file(
        &registry,
        "// import * as simple from './workflows/simple';\n\nexport const allWorkflows = {\n  'workflows/simple.ts': simple,\n} as const;\n",
    );
    write_file(
        &workflow,
        "export async function simple() {\n  'use workflow';\n}\n",
    );

    let observation = discover(&root, &[entry.clone()], true, None);
    assert!(observation.discovered_workflows.is_empty());
    assert_eq!(
        observation.discovered_files,
        BTreeSet::from([normalize(&entry), normalize(&registry)])
    );
}

#[test]
fn does_not_treat_regex_literals_as_comments() {
    let root = TestRoot::new();
    let entry = root.join("src/entry.ts");
    let registry = root.join("src/_workflows.ts");
    let workflow = root.join("src/workflows/simple.ts");
    write_file(&entry, "import './_workflows';\n");
    write_file(
        &registry,
        "const commentStartChars = /[/*]/;\nconst protocol = /https?:\\/\\//;\nimport './workflows/simple';\n",
    );
    write_file(
        &workflow,
        "export async function simple() {\n  'use workflow';\n}\n",
    );

    let observation = discover(&root, &[entry], true, None);
    assert_eq!(
        observation.discovered_workflows,
        BTreeSet::from([normalize(&workflow)])
    );
}

#[test]
fn uses_nearest_nested_jsconfig_aliases_in_monorepo_packages() {
    let root = TestRoot::new();
    let package_root = root.join("packages/app");
    let entry = package_root.join("src/entry.js");
    let workflow = package_root.join("src/workflow.js");
    write_file(
        &root.join("tsconfig.json"),
        r#"{"compilerOptions":{"paths":{"@root/*":["./root/*"]}}}"#,
    );
    write_file(
        &package_root.join("jsconfig.json"),
        r##"{"compilerOptions":{"paths":{"#/*":["./src/*"]}}}"##,
    );
    write_file(&entry, "import { run } from '#/workflow';\n");
    write_file(
        &workflow,
        "export async function run() {\n  \"use workflow\";\n  return \"ok\";\n}\n",
    );

    let observation = discover(&root, &[entry], true, None);
    assert_eq!(
        observation.discovered_workflows,
        BTreeSet::from([normalize(&workflow)])
    );
}

#[test]
fn serde_registration_requires_static_serde_methods() {
    let root = TestRoot::new();
    let entry = root.join("src/entry.ts");
    let reducer = root.join("src/reducer.ts");
    let serde = root.join("src/serde.ts");
    write_file(&entry, "import './reducer';\nimport './serde';\n");
    write_file(
        &reducer,
        "import { WORKFLOW_SERIALIZE } from '@workflow/serde';\n\nexport function reducer(value: unknown) {\n  return value?.constructor?.[WORKFLOW_SERIALIZE];\n}\n",
    );
    write_file(
        &serde,
        "import { WORKFLOW_SERIALIZE as WS } from '@workflow/serde';\n\nexport class Value {\n  static classId = 'Value';\n  static [WS](value: Value) {\n    return value;\n  }\n}\n",
    );

    let observation = discover(&root, &[entry], true, None);
    assert_eq!(
        observation.discovered_serde_files,
        BTreeSet::from([normalize(&serde)])
    );
}

#[test]
fn categorizes_step_workflow_and_serde_usage_independently() {
    let root = TestRoot::new();
    let entry = root.join("src/entry.ts");
    let step = root.join("src/step.ts");
    let workflow = root.join("src/workflow.ts");
    let serde = root.join("src/serde.ts");
    write_file(
        &entry,
        "import './step';\nimport './workflow';\nimport './serde';\n",
    );
    write_file(
        &step,
        "export async function runStep() {\n  'use step';\n  return 'ok';\n}\n",
    );
    write_file(
        &workflow,
        "export async function runWorkflow() {\n  'use workflow';\n  return 'ok';\n}\n",
    );
    write_file(
        &serde,
        "export class Value {\n  static classId = 'Value';\n  static [Symbol.for('workflow-serialize')](value: Value) {\n    return value;\n  }\n}\n",
    );

    let observation = discover(&root, &[entry], true, None);
    assert_eq!(
        observation.discovered_steps,
        BTreeSet::from([normalize(&step)])
    );
    assert_eq!(
        observation.discovered_workflows,
        BTreeSet::from([normalize(&workflow)])
    );
    assert_eq!(
        observation.discovered_serde_files,
        BTreeSet::from([normalize(&serde)])
    );
}

#[test]
fn ignores_serde_examples_that_only_appear_inside_comments() {
    let root = TestRoot::new();
    let entry = root.join("src/entry.ts");
    let docs = root.join("src/docs.ts");
    write_file(&entry, "import './docs';\n");
    write_file(
        &docs,
        "/**\n * import { WORKFLOW_SERIALIZE } from '@workflow/serde';\n *\n * class Example {\n *   static [WORKFLOW_SERIALIZE](value) {\n *     return value;\n *   }\n * }\n */\nexport const WORKFLOW_SERIALIZE = Symbol.for('workflow-serialize');\n",
    );

    let observation = discover(&root, &[entry], true, None);
    assert!(observation.discovered_serde_files.is_empty());
}

#[test]
fn relativizes_nested_package_step_registration_imports() {
    let root = TestRoot::new();
    let route_dir = root.join("app/.well-known/workflow/v1");
    let direct = root.join("node_modules/direct-pkg/step.js");
    let nested = root.join("node_modules/parent-pkg/node_modules/nested-pkg/step.js");
    write_file(
        &root.join("package.json"),
        r#"{"dependencies":{"direct-pkg":"1.0.0"}}"#,
    );
    write_file(
        &root.join("node_modules/direct-pkg/package.json"),
        r#"{"name":"direct-pkg","version":"1.0.0","exports":{"./step":"./step.js"}}"#,
    );
    write_file(&direct, "export const step = true;\n");
    write_file(
        &root.join("node_modules/parent-pkg/node_modules/nested-pkg/package.json"),
        r#"{"name":"nested-pkg","version":"1.0.0","exports":{"./step":"./step.js"}}"#,
    );
    write_file(&nested, "export const step = true;\n");

    assert_eq!(
        create_route_import_specifier(&direct, &route_dir, &root.0).unwrap(),
        "direct-pkg/step"
    );
    assert_eq!(
        create_route_import_specifier(&nested, &route_dir, &root.0).unwrap(),
        "../../../../node_modules/parent-pkg/node_modules/nested-pkg/step.js"
    );
}
