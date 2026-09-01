use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiscoveryTransformMode {
    Detect,
    Transform,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveryTransformCall {
    pub filename: String,
    pub source: String,
    pub mode: DiscoveryTransformMode,
    pub input_filename: String,
    pub project_root: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DiscoveryState {
    pub discovered_steps: BTreeSet<String>,
    pub discovered_workflows: BTreeSet<String>,
    pub discovered_serde_files: BTreeSet<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoverEntriesOptions {
    pub entry_points: Vec<PathBuf>,
    pub abs_working_dir: PathBuf,
    pub project_root: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DiscoveryObservation {
    pub state: DiscoveryState,
    pub transform_calls: Vec<DiscoveryTransformCall>,
    pub import_parents: BTreeMap<String, BTreeSet<String>>,
}

impl DiscoveryObservation {
    pub fn parent_has_child(
        &self,
        parent: &str,
        child_to_find: &str,
        excluded_roots: &BTreeSet<String>,
    ) -> bool {
        parent_has_child(&self.import_parents, parent, child_to_find, excluded_roots)
    }
}

pub fn parent_has_child(
    import_parents: &BTreeMap<String, BTreeSet<String>>,
    parent: &str,
    child_to_find: &str,
    excluded_roots: &BTreeSet<String>,
) -> bool {
    let mut visited = BTreeSet::new();
    let mut queue = VecDeque::new();
    queue.push_back(parent.to_owned());

    while let Some(current) = queue.pop_front() {
        if !visited.insert(current.clone()) {
            continue;
        }
        let Some(children) = import_parents.get(&current) else {
            continue;
        };
        for child in children {
            if excluded_roots.contains(child) {
                continue;
            }
            if child == child_to_find {
                return true;
            }
            queue.push_back(child.clone());
        }
    }

    false
}

pub fn discover_entries(options: &DiscoverEntriesOptions) -> Result<DiscoveryObservation, String> {
    let _ = options;
    panic!(
        "TDD RED: packages/builders/src/discover-entries-esbuild-plugin.test.ts implementation pending"
    )
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalPackageWarningOptions {
    pub working_dir: PathBuf,
    pub inputs: Vec<PathBuf>,
    pub outdir: PathBuf,
    pub external_packages: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ExternalPackageWarningSession {
    pub warned_packages: BTreeSet<String>,
}

pub fn discover_with_external_package_warnings(
    session: &mut ExternalPackageWarningSession,
    options: &ExternalPackageWarningOptions,
) -> Result<Vec<String>, String> {
    let _ = (session, options);
    panic!("TDD RED: packages/builders/src/external-package-warning.test.ts implementation pending")
}
