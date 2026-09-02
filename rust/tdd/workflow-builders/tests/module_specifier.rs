use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use workflow_builders_tdd::module_specifier::{
    ImportPath, ModuleSpecifierResolution, get_import_path, resolve_module_specifier,
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

fn import_path(import_path: &str, is_package: bool) -> ImportPath {
    ImportPath {
        import_path: import_path.to_owned(),
        is_package,
    }
}

fn module_specifier(value: Option<&str>) -> ModuleSpecifierResolution {
    ModuleSpecifierResolution {
        module_specifier: value.map(str::to_owned),
    }
}

#[test]
fn uses_package_subpath_when_source_matches_an_export() {
    let root = TestRoot::new("module-export-subpath");
    let project_root = root.0.join("apps/chat");
    let package_dir = root.0.join("packages/agent");
    let file_path = package_dir.join("src/server.ts");
    write_file(
        &project_root.join("package.json"),
        r#"{"name":"chat","dependencies":{"@internal/agent":"workspace:*"}}"#,
    );
    write_file(
        &package_dir.join("package.json"),
        r#"{"name":"@internal/agent","version":"1.0.0","exports":{"./server":"./src/server.ts"}}"#,
    );
    write_file(&file_path, "'use step';\n");

    assert_eq!(
        get_import_path(&file_path, &project_root).unwrap(),
        import_path("@internal/agent/server", true)
    );
}

#[test]
fn falls_back_to_relative_import_when_export_does_not_point_to_source() {
    let root = TestRoot::new("module-export-mismatch");
    let project_root = root.0.join("apps/chat");
    let package_dir = root.0.join("packages/agent");
    let file_path = package_dir.join("src/server.ts");
    write_file(
        &project_root.join("package.json"),
        r#"{"name":"chat","dependencies":{"@internal/agent":"workspace:*"}}"#,
    );
    write_file(
        &package_dir.join("package.json"),
        r#"{"name":"@internal/agent","version":"1.0.0","exports":{"./server":"./dist/server.js"}}"#,
    );
    write_file(&file_path, "'use step';\n");

    assert_eq!(
        get_import_path(&file_path, &project_root).unwrap(),
        import_path("../../packages/agent/src/server.ts", false)
    );
}

#[test]
fn uses_package_root_for_root_exports() {
    let root = TestRoot::new("module-root-export");
    let project_root = root.0.join("apps/chat");
    let package_dir = root.0.join("packages/agent");
    let file_path = package_dir.join("src/index.ts");
    write_file(
        &project_root.join("package.json"),
        r#"{"name":"chat","dependencies":{"@internal/agent":"workspace:*"}}"#,
    );
    write_file(
        &package_dir.join("package.json"),
        r#"{"name":"@internal/agent","version":"1.0.0","exports":{".":"./src/index.ts"}}"#,
    );
    write_file(&file_path, "'use workflow';\n");

    assert_eq!(
        get_import_path(&file_path, &project_root).unwrap(),
        import_path("@internal/agent", true)
    );
}

#[test]
fn uses_package_root_when_module_points_to_the_file() {
    let root = TestRoot::new("module-field");
    let project_root = root.0.join("apps/chat");
    let package_dir = root.0.join("packages/agent");
    let file_path = package_dir.join("src/index.mjs");
    write_file(
        &project_root.join("package.json"),
        r#"{"name":"chat","dependencies":{"@internal/agent":"workspace:*"}}"#,
    );
    write_file(
        &package_dir.join("package.json"),
        r#"{"name":"@internal/agent","version":"1.0.0","module":"./src/index.mjs","main":"./dist/index.cjs"}"#,
    );
    write_file(&file_path, "'use workflow';\n");

    assert_eq!(
        get_import_path(&file_path, &project_root).unwrap(),
        import_path("@internal/agent", true)
    );
}

#[test]
fn uses_package_root_for_conditional_root_exports() {
    let root = TestRoot::new("module-conditional-export");
    let project_root = root.0.join("apps/chat");
    let package_dir = root.0.join("packages/agent");
    let file_path = package_dir.join("src/index.js");
    write_file(
        &project_root.join("package.json"),
        r#"{"name":"chat","dependencies":{"@internal/agent":"workspace:*"}}"#,
    );
    write_file(
        &package_dir.join("package.json"),
        r#"{"name":"@internal/agent","version":"1.0.0","exports":{".":{"import":"./src/index.mjs","default":"./src/index.js"}}}"#,
    );
    write_file(&file_path, "'use workflow';\n");

    assert_eq!(
        get_import_path(&file_path, &project_root).unwrap(),
        import_path("@internal/agent", true)
    );
}

#[test]
fn falls_back_to_relative_import_for_deep_files_without_exports() {
    let root = TestRoot::new("module-deep-relative");
    let project_root = root.0.join("apps/chat");
    let package_dir = root.0.join("packages/agent");
    let file_path = package_dir.join("lib/tools/dynamic/workflow.ts");
    write_file(
        &project_root.join("package.json"),
        r#"{"name":"chat","dependencies":{"@internal/agent":"workspace:*"}}"#,
    );
    write_file(
        &package_dir.join("package.json"),
        r#"{"name":"@internal/agent","version":"1.0.0"}"#,
    );
    write_file(&file_path, "'use workflow';\n");

    assert_eq!(
        get_import_path(&file_path, &project_root).unwrap(),
        import_path("../../packages/agent/lib/tools/dynamic/workflow.ts", false)
    );
}

#[test]
fn uses_package_root_when_main_points_to_the_file() {
    let root = TestRoot::new("module-main-field");
    let project_root = root.0.join("apps/chat");
    let package_dir = root.0.join("packages/agent");
    let file_path = package_dir.join("src/index.ts");
    write_file(
        &project_root.join("package.json"),
        r#"{"name":"chat","dependencies":{"@internal/agent":"workspace:*"}}"#,
    );
    write_file(
        &package_dir.join("package.json"),
        r#"{"name":"@internal/agent","version":"1.0.0","main":"./src/index.ts"}"#,
    );
    write_file(&file_path, "'use step';\n");

    assert_eq!(
        get_import_path(&file_path, &project_root).unwrap(),
        import_path("@internal/agent", true)
    );
}

#[test]
fn uses_package_subpath_for_direct_node_modules_dependencies() {
    let root = TestRoot::new("module-direct-node-modules");
    let project_root = root.0.join("apps/chat");
    let package_dir = project_root.join("node_modules/@workflow/core");
    let file_path = package_dir.join("dist/serialization.js");
    write_file(
        &project_root.join("package.json"),
        r#"{"name":"chat","dependencies":{"@workflow/core":"1.0.0"}}"#,
    );
    write_file(
        &package_dir.join("package.json"),
        r#"{"name":"@workflow/core","version":"1.0.0","exports":{"./serialization":"./dist/serialization.js"}}"#,
    );
    write_file(&file_path, "'use workflow';\n");

    assert_eq!(
        get_import_path(&file_path, &project_root).unwrap(),
        import_path("@workflow/core/serialization", true)
    );
}

#[test]
fn falls_back_to_relative_import_for_transitive_node_modules_dependencies() {
    let root = TestRoot::new("module-transitive-node-modules");
    let project_root = root.0.join("apps/chat");
    let package_dir = project_root.join("node_modules/@workflow/core");
    let file_path = package_dir.join("dist/serialization.js");
    write_file(
        &project_root.join("package.json"),
        r#"{"name":"chat","dependencies":{"workflow":"1.0.0"}}"#,
    );
    write_file(
        &package_dir.join("package.json"),
        r#"{"name":"@workflow/core","version":"1.0.0","exports":{"./serialization":"./dist/serialization.js"}}"#,
    );
    write_file(&file_path, "'use workflow';\n");

    assert_eq!(
        get_import_path(&file_path, &project_root).unwrap(),
        import_path("./node_modules/@workflow/core/dist/serialization.js", false)
    );
}

#[test]
fn treats_project_package_files_as_local() {
    let root = TestRoot::new("module-project-package");
    let project_root = root.0.join("packages/vade");
    let file_path = project_root.join("src/internal/message/workflow/handle-message.ts");
    write_file(
        &project_root.join("package.json"),
        r#"{"name":"vade","version":"0.0.0"}"#,
    );
    write_file(&file_path, "'use workflow';\n");

    assert_eq!(
        resolve_module_specifier(&file_path, &project_root).unwrap(),
        module_specifier(None)
    );
}

#[test]
fn uses_package_specifier_for_workspace_root_entrypoint_ids() {
    let root = TestRoot::new("module-workspace-entrypoint");
    let project_root = root.0.join("apps/chat");
    let package_dir = root.0.join("packages/vade");
    let file_path = package_dir.join("src/index.ts");
    write_file(
        &project_root.join("package.json"),
        r#"{"name":"chat","dependencies":{"vade":"workspace:*"}}"#,
    );
    write_file(
        &package_dir.join("package.json"),
        r#"{"name":"vade","version":"0.0.0","main":"./src/index.ts"}"#,
    );
    write_file(&file_path, "'use workflow';\n");

    assert_eq!(
        resolve_module_specifier(&file_path, &project_root).unwrap(),
        module_specifier(Some("vade@0.0.0"))
    );
}

#[test]
fn non_exported_workspace_files_have_no_package_module_specifier() {
    let root = TestRoot::new("module-workspace-non-exported");
    let project_root = root.0.join("apps/chat");
    let package_dir = root.0.join("packages/vade");
    let file_path = package_dir.join("src/internal/message/workflow/handle-message.ts");
    write_file(
        &project_root.join("package.json"),
        r#"{"name":"chat","dependencies":{"vade":"workspace:*"}}"#,
    );
    write_file(
        &package_dir.join("package.json"),
        r#"{"name":"vade","version":"0.0.0"}"#,
    );
    write_file(&file_path, "'use workflow';\n");

    assert_eq!(
        resolve_module_specifier(&file_path, &project_root).unwrap(),
        module_specifier(None)
    );
}

#[test]
fn preserves_export_subpaths_when_source_files_back_dist_exports() {
    let root = TestRoot::new("module-dist-backed-source");
    let project_root = root.0.join("apps/chat");
    let package_dir = root.0.join("packages/agent");
    let file_path = package_dir.join("src/server.ts");
    write_file(
        &project_root.join("package.json"),
        r#"{"name":"chat","dependencies":{"@internal/agent":"workspace:*"}}"#,
    );
    write_file(
        &package_dir.join("package.json"),
        r#"{"name":"@internal/agent","version":"1.0.0","exports":{"./server":"./dist/server.js"}}"#,
    );
    write_file(&file_path, "'use step';\n");

    assert_eq!(
        resolve_module_specifier(&file_path, &project_root).unwrap(),
        module_specifier(Some("@internal/agent/server@1.0.0"))
    );
}
