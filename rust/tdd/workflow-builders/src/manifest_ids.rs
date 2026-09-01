use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManifestEntry {
    pub symbol_name: String,
    pub id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WorkflowManifest {
    pub steps: BTreeMap<String, BTreeMap<String, ManifestEntry>>,
    pub workflows: BTreeMap<String, BTreeMap<String, ManifestEntry>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManifestEntryLocation {
    pub file_path: String,
    pub symbol_name: String,
    pub source_hash: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ManifestIdRegistry {
    pub step_ids: BTreeMap<String, ManifestEntryLocation>,
    pub workflow_ids: BTreeMap<String, ManifestEntryLocation>,
}

pub fn hash_manifest_source(source: &str) -> String {
    source.to_owned()
}

pub fn merge_workflow_manifest(
    target: &mut WorkflowManifest,
    incoming: &WorkflowManifest,
    registry: &mut ManifestIdRegistry,
    source_hash: Option<&str>,
) -> Result<(), String> {
    let _ = (target, incoming, registry, source_hash);
    panic!("TDD RED: packages/builders/src/manifest-ids.test.ts implementation pending")
}
