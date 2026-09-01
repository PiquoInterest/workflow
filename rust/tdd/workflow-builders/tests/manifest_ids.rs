use std::collections::BTreeMap;

use workflow_builders_tdd::manifest_ids::{
    ManifestEntry, ManifestIdRegistry, WorkflowManifest, hash_manifest_source,
    merge_workflow_manifest,
};

const PEER_A_FILE: &str =
    "node_modules/.pnpm/pkg@1.0.0_peer-a@1.0.0/node_modules/pkg/dist/steps.js";
const PEER_B_FILE: &str =
    "node_modules/.pnpm/pkg@1.0.0_peer-b@2.0.0/node_modules/pkg/dist/steps.js";
const STEP_ID: &str = "step//pkg/steps@1.0.0//doWork";
const WORKFLOW_ID: &str = "workflow//pkg/steps@1.0.0//orchestrate";

fn step_manifest(file_path: &str, name: &str, step_id: &str) -> WorkflowManifest {
    WorkflowManifest {
        steps: BTreeMap::from([(
            file_path.to_owned(),
            BTreeMap::from([(
                name.to_owned(),
                ManifestEntry {
                    symbol_name: name.to_owned(),
                    id: step_id.to_owned(),
                },
            )]),
        )]),
        ..WorkflowManifest::default()
    }
}

fn workflow_manifest(file_path: &str, name: &str, workflow_id: &str) -> WorkflowManifest {
    WorkflowManifest {
        workflows: BTreeMap::from([(
            file_path.to_owned(),
            BTreeMap::from([(
                name.to_owned(),
                ManifestEntry {
                    symbol_name: name.to_owned(),
                    id: workflow_id.to_owned(),
                },
            )]),
        )]),
        ..WorkflowManifest::default()
    }
}

#[test]
fn rejects_same_step_id_from_different_contents() {
    let mut registry = ManifestIdRegistry::default();
    let mut target = WorkflowManifest::default();
    let first_hash = hash_manifest_source("export const doWork = 1;");
    let second_hash = hash_manifest_source("export const doWork = 2;");

    merge_workflow_manifest(
        &mut target,
        &step_manifest(PEER_A_FILE, "doWork", STEP_ID),
        &mut registry,
        Some(&first_hash),
    )
    .unwrap();
    let error = merge_workflow_manifest(
        &mut target,
        &step_manifest(PEER_B_FILE, "doWork", STEP_ID),
        &mut registry,
        Some(&second_hash),
    )
    .unwrap_err();

    assert!(error.contains("Duplicate workflow step ID"));
}

#[test]
fn deduplicates_identical_peer_copies_of_the_same_step() {
    let mut registry = ManifestIdRegistry::default();
    let mut target = WorkflowManifest::default();
    let hash = hash_manifest_source("export const doWork = 1;");

    merge_workflow_manifest(
        &mut target,
        &step_manifest(PEER_A_FILE, "doWork", STEP_ID),
        &mut registry,
        Some(&hash),
    )
    .unwrap();
    merge_workflow_manifest(
        &mut target,
        &step_manifest(PEER_B_FILE, "doWork", STEP_ID),
        &mut registry,
        Some(&hash),
    )
    .unwrap();

    assert_eq!(registry.step_ids[STEP_ID].file_path, PEER_A_FILE);
    assert_eq!(
        target.steps.keys().cloned().collect::<Vec<_>>(),
        vec![PEER_A_FILE.to_owned(), PEER_B_FILE.to_owned()]
    );
}

#[test]
fn deduplicates_identical_peer_copies_of_the_same_workflow() {
    let mut registry = ManifestIdRegistry::default();
    let mut target = WorkflowManifest::default();
    let hash = hash_manifest_source("export const orchestrate = 1;");

    merge_workflow_manifest(
        &mut target,
        &workflow_manifest(PEER_A_FILE, "orchestrate", WORKFLOW_ID),
        &mut registry,
        Some(&hash),
    )
    .unwrap();
    merge_workflow_manifest(
        &mut target,
        &workflow_manifest(PEER_B_FILE, "orchestrate", WORKFLOW_ID),
        &mut registry,
        Some(&hash),
    )
    .unwrap();

    assert_eq!(registry.workflow_ids[WORKFLOW_ID].file_path, PEER_A_FILE);
}

#[test]
fn rejects_same_workflow_id_from_different_contents() {
    let mut registry = ManifestIdRegistry::default();
    let mut target = WorkflowManifest::default();
    let first_hash = hash_manifest_source("a");
    let second_hash = hash_manifest_source("b");

    merge_workflow_manifest(
        &mut target,
        &workflow_manifest(PEER_A_FILE, "orchestrate", WORKFLOW_ID),
        &mut registry,
        Some(&first_hash),
    )
    .unwrap();
    let error = merge_workflow_manifest(
        &mut target,
        &workflow_manifest(PEER_B_FILE, "orchestrate", WORKFLOW_ID),
        &mut registry,
        Some(&second_hash),
    )
    .unwrap_err();

    assert!(error.contains("Duplicate workflow ID"));
}

#[test]
fn rejects_duplicate_ids_when_content_hashes_are_unavailable() {
    let mut registry = ManifestIdRegistry::default();
    let mut target = WorkflowManifest::default();

    merge_workflow_manifest(
        &mut target,
        &step_manifest(PEER_A_FILE, "doWork", STEP_ID),
        &mut registry,
        None,
    )
    .unwrap();
    let error = merge_workflow_manifest(
        &mut target,
        &step_manifest(PEER_B_FILE, "doWork", STEP_ID),
        &mut registry,
        None,
    )
    .unwrap_err();

    assert!(error.contains("Duplicate workflow step ID"));
}

#[test]
fn rejects_identical_contents_when_symbol_names_differ() {
    let mut registry = ManifestIdRegistry::default();
    let mut target = WorkflowManifest::default();
    let hash = hash_manifest_source("shared");

    merge_workflow_manifest(
        &mut target,
        &step_manifest(PEER_A_FILE, "doWork", STEP_ID),
        &mut registry,
        Some(&hash),
    )
    .unwrap();
    let error = merge_workflow_manifest(
        &mut target,
        &step_manifest(PEER_B_FILE, "doOtherWork", STEP_ID),
        &mut registry,
        Some(&hash),
    )
    .unwrap_err();

    assert!(error.contains("Duplicate workflow step ID"));
}

#[test]
fn allows_remerging_the_same_file() {
    let mut registry = ManifestIdRegistry::default();
    let mut target = WorkflowManifest::default();
    let manifest = step_manifest(PEER_A_FILE, "doWork", STEP_ID);
    let hash = hash_manifest_source("export const doWork = 1;");

    merge_workflow_manifest(&mut target, &manifest, &mut registry, Some(&hash)).unwrap();
    merge_workflow_manifest(&mut target, &manifest, &mut registry, Some(&hash)).unwrap();
}
