use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use workflow_utils_tdd::{
    DataDirContext, POSSIBLE_WORKFLOW_DATA_PATHS, WorkflowDataDirResult, find_workflow_data_dir,
};

static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(0);

struct TempDir {
    path: PathBuf,
}

impl TempDir {
    fn new() -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let id = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "workflow-data-dir-tdd-{}-{nanos}-{id}",
            std::process::id()
        ));
        fs::create_dir_all(&path).unwrap();
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
    fs::create_dir_all(&home).unwrap();
    DataDirContext {
        cwd: root.to_path_buf(),
        home,
    }
}

fn find(input: &Path, context: &DataDirContext) -> WorkflowDataDirResult {
    find_workflow_data_dir(&input.to_string_lossy(), context)
}

fn assert_found(result: &WorkflowDataDirResult, project_dir: &Path, data_dir: &Path) {
    assert_eq!(result.project_dir, project_dir);
    assert_eq!(result.data_dir.as_deref(), Some(data_dir));
    assert!(!result.short_name.is_empty());
    assert_eq!(result.error, None);
}

#[test]
fn finds_next_workflow_data_in_the_current_directory() {
    let temp = TempDir::new();
    let data = temp.path().join(".next/workflow-data");
    fs::create_dir_all(&data).unwrap();
    let result = find(temp.path(), &context(temp.path()));
    assert_found(&result, temp.path(), &data);
}

#[test]
fn finds_hidden_workflow_data_in_the_current_directory() {
    let temp = TempDir::new();
    let data = temp.path().join(".workflow-data");
    fs::create_dir_all(&data).unwrap();
    let result = find(temp.path(), &context(temp.path()));
    assert_found(&result, temp.path(), &data);
}

#[test]
fn finds_plain_workflow_data_in_the_current_directory() {
    let temp = TempDir::new();
    let data = temp.path().join("workflow-data");
    fs::create_dir_all(&data).unwrap();
    let result = find(temp.path(), &context(temp.path()));
    assert_found(&result, temp.path(), &data);
}

#[test]
fn prefers_next_workflow_data_over_the_other_candidates() {
    let temp = TempDir::new();
    for candidate in POSSIBLE_WORKFLOW_DATA_PATHS {
        fs::create_dir_all(temp.path().join(candidate)).unwrap();
    }
    let result = find(temp.path(), &context(temp.path()));
    assert_eq!(
        result.data_dir,
        Some(temp.path().join(".next/workflow-data"))
    );
}

#[test]
fn detects_next_workflow_data_when_it_is_the_input_directory() {
    let temp = TempDir::new();
    let project = temp.path().join("myproject");
    let data = project.join(".next/workflow-data");
    fs::create_dir_all(&data).unwrap();
    let result = find(&data, &context(temp.path()));
    assert_found(&result, &project, &data);
}

#[test]
fn detects_hidden_workflow_data_when_it_is_the_input_directory() {
    let temp = TempDir::new();
    let project = temp.path().join("myproject");
    let data = project.join(".workflow-data");
    fs::create_dir_all(&data).unwrap();
    let result = find(&data, &context(temp.path()));
    assert_found(&result, &project, &data);
}

#[test]
fn detects_plain_workflow_data_when_it_is_the_input_directory() {
    let temp = TempDir::new();
    let project = temp.path().join("myproject");
    let data = project.join("workflow-data");
    fs::create_dir_all(&data).unwrap();
    let result = find(&data, &context(temp.path()));
    assert_found(&result, &project, &data);
}

#[test]
fn walks_up_to_find_workflow_data_in_a_parent() {
    let temp = TempDir::new();
    let project = temp.path().join("myproject");
    let data = project.join(".next/workflow-data");
    let child = project.join("src/components");
    fs::create_dir_all(&data).unwrap();
    fs::create_dir_all(&child).unwrap();
    let result = find(&child, &context(temp.path()));
    assert_found(&result, &project, &data);
}

#[test]
fn finds_workflow_data_several_levels_up() {
    let temp = TempDir::new();
    let project = temp.path().join("myproject");
    let data = project.join(".workflow-data");
    let child = project.join("src/app/api/workflows");
    fs::create_dir_all(&data).unwrap();
    fs::create_dir_all(&child).unwrap();
    let result = find(&child, &context(temp.path()));
    assert_found(&result, &project, &data);
}

#[test]
fn returns_no_data_directory_when_the_directory_is_empty() {
    let temp = TempDir::new();
    let result = find(temp.path(), &context(temp.path()));
    assert_eq!(result.data_dir, None);
    assert_eq!(result.project_dir, temp.path());
    assert!(!result.short_name.is_empty());
}

#[test]
fn returns_no_data_directory_when_only_unrelated_directories_exist() {
    let temp = TempDir::new();
    fs::create_dir_all(temp.path().join("src")).unwrap();
    fs::create_dir_all(temp.path().join("node_modules")).unwrap();
    let result = find(temp.path(), &context(temp.path()));
    assert_eq!(result.data_dir, None);
    assert_eq!(result.project_dir, temp.path());
}

#[test]
fn handles_relative_paths() {
    let temp = TempDir::new();
    let project = temp.path().join("relative-test");
    let data = project.join(".workflow-data");
    fs::create_dir_all(&data).unwrap();
    let result = find_workflow_data_dir("relative-test", &context(temp.path()));
    assert_found(&result, &project, &data);
}

#[test]
fn handles_absolute_paths() {
    let temp = TempDir::new();
    let project = temp.path().join("absolute-test");
    let data = project.join(".next/workflow-data");
    fs::create_dir_all(&data).unwrap();
    let result = find(&project, &context(temp.path()));
    assert_found(&result, &project, &data);
}

#[test]
fn expands_tilde_paths_against_the_injected_home_directory() {
    let temp = TempDir::new();
    let ctx = context(temp.path());
    let project = ctx.home.join(".workflow-test");
    let data = project.join(".workflow-data");
    fs::create_dir_all(&data).unwrap();
    let result = find_workflow_data_dir("~/.workflow-test", &ctx);
    assert_found(&result, &project, &data);
}

#[test]
fn returns_absolute_paths_for_relative_input() {
    let temp = TempDir::new();
    let project = temp.path().join("paths-test");
    let data = project.join("workflow-data");
    fs::create_dir_all(&data).unwrap();
    let result = find_workflow_data_dir("paths-test", &context(temp.path()));
    assert!(result.project_dir.is_absolute());
    assert!(result.data_dir.as_ref().unwrap().is_absolute());
}

#[test]
fn normalizes_dot_and_parent_segments() {
    let temp = TempDir::new();
    let project = temp.path().join("normalize-test");
    let data = project.join(".workflow-data");
    let subdir = project.join("subdir");
    fs::create_dir_all(&data).unwrap();
    fs::create_dir_all(&subdir).unwrap();
    let weird = subdir.join(".././.");
    let result = find(&weird, &context(temp.path()));
    assert_found(&result, &project, &data);
}

#[test]
fn short_name_contains_the_last_two_folder_names() {
    let temp = TempDir::new();
    let project = temp.path().join("code/myproject");
    let data = project.join(".workflow-data");
    fs::create_dir_all(&data).unwrap();
    let result = find(&project, &context(temp.path()));
    assert_eq!(result.short_name, "code/myproject");
}

#[test]
fn short_name_handles_shallow_paths() {
    let temp = TempDir::new();
    let project = temp.path().join("myproject");
    let data = project.join(".workflow-data");
    fs::create_dir_all(&data).unwrap();
    let result = find(&project, &context(temp.path()));
    let parts: Vec<_> = result.short_name.split('/').collect();
    assert!(parts.len() <= 2);
    assert_eq!(parts.last().copied(), Some("myproject"));
}

#[test]
fn short_name_handles_deeply_nested_projects() {
    let temp = TempDir::new();
    let project = temp.path().join("a/b/c/d/project");
    let data = project.join(".workflow-data");
    fs::create_dir_all(&data).unwrap();
    let result = find(&project, &context(temp.path()));
    assert_eq!(result.short_name, "d/project");
}

#[test]
fn handles_non_existent_directories_gracefully() {
    let temp = TempDir::new();
    let missing = temp.path().join("this/path/does/not/exist");
    let result = find(&missing, &context(temp.path()));
    assert_eq!(result.data_dir, None);
    assert_eq!(result.error.as_deref(), Some("Folder does not exist"));
}

#[test]
fn empty_input_defaults_to_the_injected_current_directory() {
    let temp = TempDir::new();
    let result = find_workflow_data_dir("", &context(temp.path()));
    assert_eq!(result.project_dir, temp.path());
    assert!(!result.short_name.is_empty());
}

#[test]
fn handles_paths_with_trailing_separators() {
    let temp = TempDir::new();
    let project = temp.path().join("trailing");
    let data = project.join(".workflow-data");
    fs::create_dir_all(&data).unwrap();
    let input = format!(
        "{}{separator}",
        project.display(),
        separator = std::path::MAIN_SEPARATOR
    );
    let result = find_workflow_data_dir(&input, &context(temp.path()));
    assert_found(&result, &project, &data);
}

#[test]
fn handles_a_missing_path_that_looks_like_a_workflow_data_directory() {
    let temp = TempDir::new();
    let missing = temp.path().join("fake/.next/workflow-data");
    let result = find(&missing, &context(temp.path()));
    assert_eq!(result.data_dir, None);
    assert_eq!(result.error.as_deref(), Some("Folder does not exist"));
}
