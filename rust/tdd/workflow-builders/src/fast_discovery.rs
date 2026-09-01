use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FastDiscoveryConfig {
    pub working_dir: PathBuf,
    pub discover_workflows_in_node_modules: bool,
    pub tsconfig_path: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FastDiscoveryObservation {
    pub discovered_steps: BTreeSet<String>,
    pub discovered_workflows: BTreeSet<String>,
    pub discovered_serde_files: BTreeSet<String>,
    pub discovered_files: BTreeSet<String>,
    pub import_parents: BTreeMap<String, BTreeSet<String>>,
}

impl FastDiscoveryObservation {
    pub fn parent_has_child(&self, parent: &str, child_to_find: &str) -> bool {
        let mut visited = BTreeSet::new();
        let mut queue = VecDeque::from([parent.to_owned()]);

        while let Some(current) = queue.pop_front() {
            if !visited.insert(current.clone()) {
                continue;
            }
            let Some(children) = self.import_parents.get(&current) else {
                continue;
            };
            for child in children {
                if child == child_to_find {
                    return true;
                }
                queue.push_back(child.clone());
            }
        }

        false
    }
}

pub fn discover_fast_entries(
    config: &FastDiscoveryConfig,
    inputs: &[PathBuf],
    outdir: &Path,
) -> Result<FastDiscoveryObservation, String> {
    let _ = (config, inputs, outdir);
    panic!("TDD RED: packages/builders/src/fast-discovery.test.ts implementation pending")
}

pub fn create_route_import_specifier(
    file: &Path,
    route_dir: &Path,
    project_root: &Path,
) -> Result<String, String> {
    let _ = (file, route_dir, project_root);
    panic!("TDD RED: packages/builders/src/fast-discovery.test.ts implementation pending")
}
