use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use workflow_builders_tdd::resolve_project_root;

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
        Self(path)
    }
}

impl Drop for TestRoot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn prefers_the_workspace_root_over_app_lockfiles() {
    let root = TestRoot::new("project-root");
    let app_root = root.0.join("apps/web");
    fs::create_dir_all(&app_root).unwrap();
    fs::write(root.0.join("pnpm-workspace.yaml"), "packages: []\n").unwrap();
    fs::write(app_root.join("package-lock.json"), "{}\n").unwrap();

    assert_eq!(resolve_project_root(&app_root), root.0);
}
