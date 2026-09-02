#![forbid(unsafe_code)]

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use workflow_utils::{DataDirContext, find_workflow_data_dir};

static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(0);

struct TempDir {
    path: PathBuf,
}

impl TempDir {
    fn new() -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock must be after the Unix epoch")
            .as_nanos();
        let id = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "workflow-utils-security-{}-{nanos}-{id}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect("temporary test directory must be created");
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn context(root: &Path) -> DataDirContext {
    let home = root.join("home");
    fs::create_dir_all(&home).expect("test home must be created");
    DataDirContext {
        cwd: root.to_path_buf(),
        home,
    }
}

#[test]
fn rejects_a_regular_file_passed_as_a_workflow_data_directory() {
    let temp = TempDir::new();
    let candidate = temp.path().join(".workflow-data");
    fs::write(&candidate, b"not a directory").expect("candidate file must be created");

    let result = find_workflow_data_dir(&candidate.to_string_lossy(), &context(temp.path()));

    assert_eq!(result.data_dir, None);
    assert_eq!(result.error.as_deref(), Some("Folder does not exist"));
}

#[test]
fn ignores_a_regular_file_at_a_candidate_path() {
    let temp = TempDir::new();
    let project = temp.path().join("project");
    fs::create_dir_all(&project).expect("project directory must be created");
    fs::write(project.join(".workflow-data"), b"not a directory")
        .expect("candidate file must be created");

    let result = find_workflow_data_dir(&project.to_string_lossy(), &context(temp.path()));

    assert_eq!(result.project_dir, project);
    assert_eq!(result.data_dir, None);
    assert_eq!(result.error, None);
}

#[test]
fn rejects_dot_suffix_lookalikes() {
    let temp = TempDir::new();
    let lookalike = temp.path().join("not.workflow-data");
    fs::create_dir_all(&lookalike).expect("lookalike directory must be created");

    let result = find_workflow_data_dir(&lookalike.to_string_lossy(), &context(temp.path()));

    assert_eq!(result.project_dir, lookalike);
    assert_eq!(result.data_dir, None);
}

#[test]
fn rejects_name_suffix_lookalikes() {
    let temp = TempDir::new();
    let lookalike = temp.path().join("not-workflow-data");
    fs::create_dir_all(&lookalike).expect("lookalike directory must be created");

    let result = find_workflow_data_dir(&lookalike.to_string_lossy(), &context(temp.path()));

    assert_eq!(result.project_dir, lookalike);
    assert_eq!(result.data_dir, None);
}

#[test]
fn treats_user_qualified_tilde_names_as_literal_relative_paths() {
    let temp = TempDir::new();
    let project = temp.path().join("~service");
    let data_dir = project.join(".workflow-data");
    fs::create_dir_all(&data_dir).expect("literal tilde project must be created");

    let result = find_workflow_data_dir("~service", &context(temp.path()));

    assert_eq!(result.project_dir, project);
    assert_eq!(result.data_dir, Some(data_dir));
}
