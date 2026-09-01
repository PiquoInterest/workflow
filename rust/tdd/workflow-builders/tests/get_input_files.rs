use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use workflow_builders_tdd::input_files::{
    BuilderTarget, InputFilesConfig, ensure_swc_ignored, get_diagnostics_manifest_path,
    get_input_files,
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

fn write_file(root: &Path, relative_path: &str, content: &str) -> PathBuf {
    let full_path = root.join(relative_path);
    fs::create_dir_all(full_path.parent().unwrap()).unwrap();
    fs::write(&full_path, content).unwrap();
    full_path
}

fn standalone_config(root: &Path) -> InputFilesConfig {
    InputFilesConfig {
        build_target: BuilderTarget::Standalone,
        working_dir: root.to_path_buf(),
        dirs: vec![PathBuf::from("src")],
        diagnostics_dir: None,
        target_world: None,
    }
}

fn normalize(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn normalized_files(config: &InputFilesConfig) -> Vec<String> {
    get_input_files(config)
        .unwrap()
        .iter()
        .map(|path| normalize(path))
        .collect()
}

#[test]
fn discovers_files_inside_dot_prefixed_directories() {
    let root = TestRoot::new("get-input-files-hidden-directories");
    let src_dir = root.0.join("src");
    let hidden_step = write_file(&src_dir, ".hidden/step.ts", "'use step';");
    let hidden_workflow = write_file(&src_dir, ".config/workflow.ts", "'use workflow';");
    let regular_step = write_file(&src_dir, "regular/step.ts", "'use step';");

    let files = normalized_files(&standalone_config(&root.0));
    assert!(files.contains(&normalize(&hidden_step)));
    assert!(files.contains(&normalize(&hidden_workflow)));
    assert!(files.contains(&normalize(&regular_step)));
}

#[test]
fn discovers_dot_prefixed_files() {
    let root = TestRoot::new("get-input-files-hidden-files");
    let src_dir = root.0.join("src");
    let hidden = write_file(&src_dir, ".hidden-step.ts", "'use step';");
    let visible = write_file(&src_dir, "visible-step.ts", "'use step';");

    let files = normalized_files(&standalone_config(&root.0));
    assert!(files.contains(&normalize(&hidden)));
    assert!(files.contains(&normalize(&visible)));
}

#[test]
fn excludes_explicitly_ignored_dot_directories() {
    let root = TestRoot::new("get-input-files-ignored-directories");
    let src_dir = root.0.join("src");
    let ignored = [
        ".git/hooks/pre-commit.ts",
        ".next/server/page.ts",
        ".nuxt/workflow/steps.mjs",
        ".vercel/output/step.ts",
        ".svelte-kit/output/step.ts",
        ".workflow-data/state.ts",
        ".workflow-vitest/workflows.mjs",
        ".well-known/workflow/route.ts",
        ".swc/cache/plugin-output.ts",
        ".turbo/cache/build.ts",
        ".cache/babel/plugin.js",
        ".yarn/releases/yarn.cjs",
        ".pnpm-store/v3/files.ts",
        "node_modules/pkg/index.ts",
    ];
    let ignored_paths = ignored
        .iter()
        .map(|relative| write_file(&src_dir, relative, ""))
        .collect::<Vec<_>>();
    let custom = write_file(&src_dir, ".custom/step.ts", "'use step';");

    let files = normalized_files(&standalone_config(&root.0));
    for ignored_path in ignored_paths {
        assert!(!files.contains(&normalize(&ignored_path)));
    }
    assert!(files.contains(&normalize(&custom)));
}

#[test]
fn discovers_supported_extensions_inside_dot_directories() {
    let root = TestRoot::new("get-input-files-supported-extensions");
    let src_dir = root.0.join("src");
    let expected = [
        ".api/route.tsx",
        ".api/handler.mts",
        ".api/utils.js",
        ".api/config.cjs",
    ]
    .iter()
    .map(|relative| write_file(&src_dir, relative, ""))
    .collect::<Vec<_>>();

    let files = normalized_files(&standalone_config(&root.0));
    for expected_path in expected {
        assert!(files.contains(&normalize(&expected_path)));
    }
}

#[test]
fn ensure_swc_ignored_adds_the_project_entry_once() {
    let root = TestRoot::new("swc-gitignore-add");
    write_file(&root.0, ".gitignore", "node_modules\n");
    let config = standalone_config(&root.0);

    ensure_swc_ignored(&config).unwrap();
    ensure_swc_ignored(&config).unwrap();

    let gitignore = fs::read_to_string(root.0.join(".gitignore")).unwrap();
    let swc_entries = gitignore
        .lines()
        .filter(|line| line.trim() == "/.swc")
        .count();
    assert_eq!(gitignore, "node_modules\n/.swc\n");
    assert_eq!(swc_entries, 1);
}

#[test]
fn ensure_swc_ignored_preserves_existing_variants() {
    let root = TestRoot::new("swc-gitignore-preserve");
    write_file(&root.0, ".gitignore", "node_modules\n.swc/\n");
    let config = standalone_config(&root.0);

    ensure_swc_ignored(&config).unwrap();

    assert_eq!(
        fs::read_to_string(root.0.join(".gitignore")).unwrap(),
        "node_modules\n.swc/\n"
    );
}

#[test]
fn diagnostics_manifest_uses_an_explicit_directory() {
    let root = TestRoot::new("diagnostics-explicit");
    let mut config = standalone_config(&root.0);
    config.diagnostics_dir = Some(PathBuf::from(".next/diagnostics"));

    assert_eq!(
        get_diagnostics_manifest_path(&config),
        Some(root.0.join(".next/diagnostics/workflows-manifest.json"))
    );
}

#[test]
fn non_vercel_builders_do_not_emit_vercel_diagnostics() {
    let root = TestRoot::new("diagnostics-standalone");
    let mut config = standalone_config(&root.0);
    config.target_world = Some("vercel".to_owned());

    assert_eq!(get_diagnostics_manifest_path(&config), None);
}

#[test]
fn vercel_builder_falls_back_to_output_diagnostics() {
    let root = TestRoot::new("diagnostics-vercel");
    let config = InputFilesConfig {
        build_target: BuilderTarget::VercelBuildOutputApi,
        working_dir: root.0.clone(),
        dirs: vec![PathBuf::from("src")],
        diagnostics_dir: None,
        target_world: None,
    };

    assert_eq!(
        get_diagnostics_manifest_path(&config),
        Some(
            root.0
                .join(".vercel/output/diagnostics/workflows-manifest.json")
        )
    );
}
