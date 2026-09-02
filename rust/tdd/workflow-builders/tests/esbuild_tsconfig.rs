use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use workflow_builders_tdd::get_esbuild_tsconfig_options;

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
fn returns_tsconfig_for_regular_files() {
    let root = TestRoot::new("tsconfig-regular");
    let app = root.0.join("app");
    fs::create_dir_all(&app).unwrap();
    let tsconfig = app.join("tsconfig.json");
    fs::write(
        &tsconfig,
        r#"{"compilerOptions":{"paths":{"@/*":["./*"]}}}"#,
    )
    .unwrap();

    let options = get_esbuild_tsconfig_options(&tsconfig);
    assert_eq!(options.tsconfig.as_deref(), Some(tsconfig.as_path()));
    assert!(options.tsconfig_raw.is_none());
}

#[cfg(not(windows))]
#[test]
fn uses_raw_config_for_symlinks_so_aliases_resolve_from_the_working_directory() {
    use std::os::unix::fs::symlink;

    let root = TestRoot::new("tsconfig-symlink");
    let working_dir = root.0.join("webpack-app");
    let source_dir = root.0.join("turbopack-app");
    fs::create_dir_all(&working_dir).unwrap();
    fs::create_dir_all(&source_dir).unwrap();

    let source_config = source_dir.join("tsconfig.json");
    fs::write(
        &source_config,
        r#"{"compilerOptions":{"paths":{"@/*":["./*"]}}}"#,
    )
    .unwrap();
    let linked_config = working_dir.join("tsconfig.json");
    symlink(&source_config, &linked_config).unwrap();

    let options = get_esbuild_tsconfig_options(&linked_config);
    assert!(options.tsconfig.is_none());
    assert!(options.tsconfig_raw.is_some());
    assert_eq!(
        options.alias_base.as_deref(),
        Some(working_dir.as_path()),
        "aliases from a symlinked config must resolve in the consuming app"
    );
}
