use workflow_builders_tdd::swc_plugin::{SwcMode, SwcPluginBuildOptions, TransformOutput};

use super::support::{
    TestRoot, build, options, passthrough, set_entries_to_bundle, set_side_effect_entries,
    side_effect_package, virtual_options, write_file,
};

const SIDE_EFFECT_CODE: &str = "globalThis.__registered = globalThis.__registered || [];\n\
     globalThis.__registered.push(\"my-pkg\");";

#[test]
fn preserves_bare_imports_of_side_effect_entries_without_entries_to_bundle() {
    let root = TestRoot::new("side-effect-bare-preserved");
    let entry = side_effect_package(&root, "my-side-effect-pkg", SIDE_EFFECT_CODE);
    let mut options = virtual_options(
        &root,
        SwcMode::Workflow,
        "virtual-entry.js",
        "import 'my-side-effect-pkg';",
    );
    set_side_effect_entries(&mut options, &[&entry]);
    passthrough(&mut options, &entry);

    let observation = build(options);
    assert!(observation.errors.is_empty());
    assert!(observation.output.contains("__registered"));
}

#[test]
fn drops_bare_imports_when_side_effect_entries_is_absent() {
    let root = TestRoot::new("side-effect-bare-dropped");
    let entry = side_effect_package(&root, "my-dropped-pkg", SIDE_EFFECT_CODE);
    let mut options = virtual_options(
        &root,
        SwcMode::Workflow,
        "virtual-entry.js",
        "import 'my-dropped-pkg';",
    );
    passthrough(&mut options, &entry);

    let observation = build(options);
    assert!(observation.errors.is_empty());
    assert!(!observation.output.contains("__registered"));
}

#[test]
fn produces_no_ignored_bare_import_warnings_for_side_effect_entries() {
    let root = TestRoot::new("side-effect-no-warning");
    let entry = side_effect_package(&root, "my-warned-pkg", SIDE_EFFECT_CODE);
    let mut options = virtual_options(
        &root,
        SwcMode::Workflow,
        "virtual-entry.js",
        "import 'my-warned-pkg';",
    );
    set_side_effect_entries(&mut options, &[&entry]);
    passthrough(&mut options, &entry);

    let observation = build(options);
    assert!(observation.errors.is_empty());
    assert!(
        observation
            .warnings
            .iter()
            .all(|warning| warning.id != "ignored-bare-import")
    );
}

#[test]
fn preserves_bare_imports_with_entries_to_bundle_and_side_effect_entries_together() {
    let root = TestRoot::new("side-effect-with-bundle-list");
    let entry = side_effect_package(&root, "my-bundled-pkg", SIDE_EFFECT_CODE);
    let step = root.write(
        "src/step.ts",
        "import 'my-bundled-pkg';\nexport const POST = () => {};",
    );
    let mut options = options(&root, SwcMode::Step, vec![step.clone()]);
    options.outdir = Some(root.join("out"));
    set_entries_to_bundle(&mut options, &[&step, &entry]);
    set_side_effect_entries(&mut options, &[&entry]);
    passthrough(&mut options, &step);
    passthrough(&mut options, &entry);

    let observation = build(options);
    assert!(observation.errors.is_empty());
    assert!(observation.output.contains("__registered"));
}

#[test]
fn preserves_local_side_effect_entries_under_side_effects_false_package_metadata() {
    let root = TestRoot::new("side-effect-workspace-file");
    let package_root = root.join("packages/shared");
    write_file(
        &package_root.join("package.json"),
        r#"{"name":"@myorg/shared","version":"1.0.0","sideEffects":false,"main":"index.js"}"#,
    );
    let entry = package_root.join("index.js");
    write_file(&entry, "globalThis.__sharedRegistered = true;");
    let mut options = virtual_options(
        &root,
        SwcMode::Workflow,
        "virtual-entry.js",
        "import './packages/shared/index.js';",
    );
    set_side_effect_entries(&mut options, &[&entry]);
    passthrough(&mut options, &entry);

    let observation = build(options);
    assert!(observation.errors.is_empty());
    assert!(observation.output.contains("__sharedRegistered"));
}

#[test]
fn does_not_override_side_effect_metadata_for_unlisted_modules() {
    let root = TestRoot::new("side-effect-exact-scope");
    let listed = side_effect_package(&root, "entry-pkg", "globalThis.__entryRegistered = true;");
    let unlisted = side_effect_package(
        &root,
        "non-entry-pkg",
        "globalThis.__nonEntryRegistered = true;",
    );
    let mut options = virtual_options(
        &root,
        SwcMode::Workflow,
        "virtual-entry.js",
        "import 'entry-pkg';\nimport 'non-entry-pkg';",
    );
    set_side_effect_entries(&mut options, &[&listed]);
    passthrough(&mut options, &listed);
    passthrough(&mut options, &unlisted);

    let observation = build(options);
    assert!(observation.errors.is_empty());
    assert!(observation.output.contains("__entryRegistered"));
    assert!(!observation.output.contains("__nonEntryRegistered"));
}

fn project_root_fixture(
    label: &str,
) -> (
    TestRoot,
    std::path::PathBuf,
    std::path::PathBuf,
    std::path::PathBuf,
) {
    let root = TestRoot::new(label);
    let app_root = root.join("apps/chat");
    let package_root = root.join("packages/vade");
    std::fs::create_dir_all(&app_root).unwrap();
    let workflow = package_root.join("src/internal/message/workflow/handle-message.ts");
    write_file(
        &workflow,
        "export async function handleMessageWorkflow(message) {\n\
           \"use workflow\";\n\
           return message;\n\
         }\n",
    );
    (root, app_root, package_root, workflow)
}

fn project_options(
    package_root: std::path::PathBuf,
    workflow: std::path::PathBuf,
) -> SwcPluginBuildOptions {
    let source = std::fs::read_to_string(&workflow).unwrap();
    let mut options =
        SwcPluginBuildOptions::new(SwcMode::Workflow, package_root, vec![workflow.clone()]);
    options
        .transform_outputs
        .insert(workflow, TransformOutput::passthrough(&source));
    options
}

#[test]
fn passes_explicit_project_root_to_the_transform() {
    let (_root, app_root, package_root, workflow) = project_root_fixture("project-root-explicit");
    let mut options = project_options(package_root, workflow.clone());
    options.project_root = Some(app_root.clone());

    let observation = build(options);
    assert!(observation.errors.is_empty());
    assert_eq!(observation.transform_calls.len(), 1);
    let call = &observation.transform_calls[0];
    assert_eq!(
        call.filename,
        "src/internal/message/workflow/handle-message.ts"
    );
    assert_eq!(call.absolute_path, workflow);
    assert_eq!(call.project_root, app_root);
    assert_eq!(call.module_specifier_root, app_root);
    assert!(call.source.contains("\"use workflow\""));
}

#[test]
fn passes_module_specifier_root_separately_from_project_root() {
    let (root, app_root, package_root, workflow) =
        project_root_fixture("module-specifier-root-explicit");
    let tracing_root = root.0.clone();
    let mut options = project_options(package_root, workflow.clone());
    options.project_root = Some(tracing_root.clone());
    options.module_specifier_root = Some(app_root.clone());

    let observation = build(options);
    assert!(observation.errors.is_empty());
    assert_eq!(observation.transform_calls.len(), 1);
    let call = &observation.transform_calls[0];
    assert_eq!(
        call.filename,
        "src/internal/message/workflow/handle-message.ts"
    );
    assert_eq!(call.absolute_path, workflow);
    assert_eq!(call.project_root, tracing_root);
    assert_eq!(call.module_specifier_root, app_root);
}

#[test]
fn defaults_project_and_module_specifier_roots_to_abs_working_dir() {
    let (_root, _app_root, package_root, workflow) = project_root_fixture("project-root-default");
    let options = project_options(package_root.clone(), workflow.clone());

    let observation = build(options);
    assert!(observation.errors.is_empty());
    assert_eq!(observation.transform_calls.len(), 1);
    let call = &observation.transform_calls[0];
    assert_eq!(
        call.filename,
        "src/internal/message/workflow/handle-message.ts"
    );
    assert_eq!(call.absolute_path, workflow);
    assert_eq!(call.project_root, package_root);
    assert_eq!(call.module_specifier_root, package_root);
}
