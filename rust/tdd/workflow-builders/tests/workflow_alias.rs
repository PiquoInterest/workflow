use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use workflow_builders_tdd::workflow_alias::resolve_workflow_alias_relative_path;

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

fn write_workflow(path: &Path) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, "'use workflow';\n").unwrap();
}

#[cfg(unix)]
fn symlink_dir(target: &Path, link: &Path) {
    std::os::unix::fs::symlink(target, link).unwrap();
}

#[cfg(windows)]
fn symlink_dir(target: &Path, link: &Path) {
    std::os::windows::fs::symlink_dir(target, link).unwrap();
}

#[test]
fn maps_workflows_directory_files_to_workflows_aliases() {
    let root = TestRoot::new("workflow-alias-workflows");
    let working_dir = root.0.join("project");
    let file_path = working_dir.join("workflows/foo.ts");
    write_workflow(&file_path);

    assert_eq!(
        resolve_workflow_alias_relative_path(&file_path, &working_dir).unwrap(),
        Some("workflows/foo.ts".to_owned())
    );
}

#[test]
fn maps_src_workflows_files_to_src_workflows_aliases() {
    let root = TestRoot::new("workflow-alias-src-workflows");
    let working_dir = root.0.join("project");
    let file_path = working_dir.join("src/workflows/foo.ts");
    write_workflow(&file_path);

    assert_eq!(
        resolve_workflow_alias_relative_path(&file_path, &working_dir).unwrap(),
        Some("src/workflows/foo.ts".to_owned())
    );
}

#[test]
fn ignores_files_outside_workflow_paths() {
    let root = TestRoot::new("workflow-alias-outside");
    let working_dir = root.0.join("project");
    let file_path = working_dir.join("lib/foo.ts");
    write_workflow(&file_path);

    assert_eq!(
        resolve_workflow_alias_relative_path(&file_path, &working_dir).unwrap(),
        None
    );
}

#[test]
fn ignores_same_basename_when_the_realpath_is_external() {
    let root = TestRoot::new("workflow-alias-realpath");
    let working_dir = root.0.join("project");
    let internal_file = working_dir.join("workflows/foo.ts");
    let external_file = root.0.join("external/workflows/foo.ts");
    write_workflow(&internal_file);
    write_workflow(&external_file);

    assert_eq!(
        resolve_workflow_alias_relative_path(&external_file, &working_dir).unwrap(),
        None
    );
}

#[test]
fn maps_symlinked_app_files_to_app_aliases() {
    let root = TestRoot::new("workflow-alias-symlink");
    let working_dir = root.0.join("project");
    let external_app_dir = root.0.join("external/app");
    let external_file = external_app_dir.join(".well-known/agent/v1/steps.ts");
    write_workflow(&external_file);
    fs::create_dir_all(&working_dir).unwrap();
    symlink_dir(&external_app_dir, &working_dir.join("app"));

    assert_eq!(
        resolve_workflow_alias_relative_path(&external_file, &working_dir).unwrap(),
        Some("app/.well-known/agent/v1/steps.ts".to_owned())
    );
}
