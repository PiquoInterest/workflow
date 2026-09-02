use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use workflow_builders_tdd::discovery::{
    DiscoverEntriesOptions, DiscoveryTransformMode, discover_entries,
};

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

fn write_file(path: &Path, contents: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, contents).unwrap();
}

fn normalize(path: &Path) -> String {
    path.canonicalize()
        .unwrap()
        .to_string_lossy()
        .replace('\\', "/")
}

fn workflow_fixture(root: &Path) -> (PathBuf, PathBuf, PathBuf) {
    let app_root = root.join("apps/chat");
    let package_root = root.join("packages/vade");
    let workflow_file = package_root.join("src/internal/message/workflow/handle-message.ts");
    fs::create_dir_all(&app_root).unwrap();
    write_file(
        &workflow_file,
        r#"export async function handleMessageWorkflow(message) {
  "use workflow";

  return message;
}
"#,
    );
    (app_root, package_root, workflow_file)
}

#[test]
fn uses_the_explicit_project_root_during_discovery_transforms() {
    let root = TestRoot::new("discover-explicit-root");
    let (app_root, package_root, workflow_file) = workflow_fixture(&root.0);
    let normalized_workflow = normalize(&workflow_file);

    let observation = discover_entries(&DiscoverEntriesOptions {
        entry_points: vec![workflow_file],
        abs_working_dir: package_root,
        project_root: Some(app_root.clone()),
    })
    .unwrap();

    assert_eq!(
        observation.state.discovered_workflows,
        BTreeSet::from([normalized_workflow.clone()])
    );
    assert_eq!(observation.transform_calls.len(), 1);
    let call = &observation.transform_calls[0];
    assert_eq!(call.filename, normalized_workflow);
    assert!(call.source.contains("\"use workflow\""));
    assert_eq!(call.mode, DiscoveryTransformMode::Detect);
    assert_eq!(call.input_filename, call.filename);
    assert_eq!(call.project_root, app_root);
}

#[test]
fn defaults_discovery_transforms_to_abs_working_dir() {
    let root = TestRoot::new("discover-working-dir");
    let (_, package_root, workflow_file) = workflow_fixture(&root.0);
    let normalized_workflow = normalize(&workflow_file);

    let observation = discover_entries(&DiscoverEntriesOptions {
        entry_points: vec![workflow_file],
        abs_working_dir: package_root.clone(),
        project_root: None,
    })
    .unwrap();

    assert_eq!(
        observation.state.discovered_workflows,
        BTreeSet::from([normalized_workflow.clone()])
    );
    assert_eq!(observation.transform_calls.len(), 1);
    let call = &observation.transform_calls[0];
    assert_eq!(call.filename, normalized_workflow);
    assert!(call.source.contains("\"use workflow\""));
    assert_eq!(call.mode, DiscoveryTransformMode::Detect);
    assert_eq!(call.input_filename, call.filename);
    assert_eq!(call.project_root, package_root);
}

#[test]
fn tracks_import_parents_through_bare_specifiers() {
    let root = TestRoot::new("discover-bare-import");
    let entry_file = root.0.join("entry.ts");
    let package_dir = root.0.join("node_modules/bare-pkg");
    let package_index = package_dir.join("index.js");
    let serde_file = package_dir.join("serde.js");

    write_file(
        &package_dir.join("package.json"),
        r#"{"name":"bare-pkg","main":"index.js"}"#,
    );
    write_file(&package_index, "export { Foo } from './serde.js';\n");
    write_file(&serde_file, "export class Foo {}\n");
    write_file(
        &entry_file,
        "import { Foo } from 'bare-pkg';\nconsole.log(Foo);\n",
    );

    let observation = discover_entries(&DiscoverEntriesOptions {
        entry_points: vec![entry_file.clone()],
        abs_working_dir: root.0.clone(),
        project_root: None,
    })
    .unwrap();

    let entry = normalize(&entry_file);
    let package_index = normalize(&package_index);
    let serde_file = normalize(&serde_file);

    assert!(
        observation
            .import_parents
            .get(&entry)
            .is_some_and(|children| children.contains(&package_index))
    );
    assert!(
        observation
            .import_parents
            .get(&package_index)
            .is_some_and(|children| children.contains(&serde_file))
    );
    assert!(observation.parent_has_child(&entry, &serde_file, &BTreeSet::new()));
}
