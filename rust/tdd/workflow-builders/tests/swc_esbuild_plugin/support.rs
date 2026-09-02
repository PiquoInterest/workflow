use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use workflow_builders_tdd::swc_plugin::{
    ManifestEntry, ObserverPolicy, SwcMode, SwcPluginBuildFailure, SwcPluginBuildObservation,
    SwcPluginBuildOptions, TransformOutput, VirtualEntry, build_with_swc_plugin,
};

pub struct TestRoot(pub PathBuf);

impl TestRoot {
    pub fn new(label: &str) -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "workflow-swc-plugin-rust-{label}-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir_all(&path).unwrap();
        Self(path.canonicalize().unwrap())
    }

    pub fn join(&self, relative: &str) -> PathBuf {
        self.0.join(relative)
    }

    pub fn write(&self, relative: &str, contents: &str) -> PathBuf {
        let path = self.join(relative);
        write_file(&path, contents);
        path
    }
}

impl Drop for TestRoot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

pub fn write_file(path: &Path, contents: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, contents).unwrap();
}

pub fn read_file(path: &Path) -> String {
    fs::read_to_string(path).unwrap()
}

pub fn normalized(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

pub fn relative(root: &Path, path: &Path) -> String {
    normalized(path.strip_prefix(root).unwrap())
}

pub fn options(
    root: &TestRoot,
    mode: SwcMode,
    entry_points: Vec<PathBuf>,
) -> SwcPluginBuildOptions {
    SwcPluginBuildOptions::new(mode, root.0.clone(), entry_points)
}

pub fn passthrough(options: &mut SwcPluginBuildOptions, path: &Path) {
    let source = read_file(path);
    options
        .transform_outputs
        .insert(path.to_path_buf(), TransformOutput::passthrough(&source));
}

pub fn transformed(
    options: &mut SwcPluginBuildOptions,
    path: &Path,
    code: &str,
    entries: Vec<ManifestEntry>,
) {
    options.transform_outputs.insert(
        path.to_path_buf(),
        TransformOutput {
            code: code.to_owned(),
            manifest_entries: entries,
        },
    );
}

pub fn build(options: SwcPluginBuildOptions) -> SwcPluginBuildObservation {
    build_with_swc_plugin(&options).unwrap()
}

pub fn build_result(
    options: SwcPluginBuildOptions,
) -> Result<SwcPluginBuildObservation, Box<SwcPluginBuildFailure>> {
    build_with_swc_plugin(&options)
}

pub fn virtual_options(
    root: &TestRoot,
    mode: SwcMode,
    source_file: &str,
    source: &str,
) -> SwcPluginBuildOptions {
    let mut options = SwcPluginBuildOptions::new(mode, root.0.clone(), Vec::new());
    options.virtual_entry = Some(VirtualEntry {
        source_file: source_file.to_owned(),
        resolve_dir: root.0.clone(),
        source: source.to_owned(),
    });
    options
}

pub fn set_entries_to_bundle(options: &mut SwcPluginBuildOptions, paths: &[&Path]) {
    options.entries_to_bundle = paths
        .iter()
        .map(|path| (*path).to_path_buf())
        .collect::<BTreeSet<_>>();
}

pub fn set_side_effect_entries(options: &mut SwcPluginBuildOptions, paths: &[&Path]) {
    options.side_effect_entries = paths
        .iter()
        .map(|path| (*path).to_path_buf())
        .collect::<BTreeSet<_>>();
}

pub fn side_effect_package(root: &TestRoot, package_name: &str, source: &str) -> PathBuf {
    let package_dir = root.join(&format!("node_modules/{package_name}"));
    write_file(
        &package_dir.join("package.json"),
        &format!(
            r#"{{"name":"{package_name}","version":"1.0.0","sideEffects":false,"main":"index.js"}}"#
        ),
    );
    let entry = package_dir.join("index.js");
    write_file(&entry, source);
    entry
}

pub fn observer_recording(options: &mut SwcPluginBuildOptions) {
    options.observer = ObserverPolicy::Record;
}
