use workflow_builders_tdd::swc_plugin::{ManifestEntry, ObserverPolicy, SwcMode};

use super::support::{
    TestRoot, build, build_result, normalized, observer_recording, options, passthrough, relative,
    set_entries_to_bundle, transformed,
};

#[test]
fn reports_authoritative_transform_results_to_an_optional_observer() {
    let root = TestRoot::new("observer-authoritative");
    let step = root.write("src/step.ts", "export const value = 42;");
    let source = "export const value = 42;";
    let transformed_source = format!("{source}\n/* transformed */");
    let mut options = options(&root, SwcMode::Step, vec![step.clone()]);
    set_entries_to_bundle(&mut options, &[&step]);
    observer_recording(&mut options);
    transformed(
        &mut options,
        &step,
        &transformed_source,
        vec![ManifestEntry::step(
            "src/step.ts",
            "value",
            "step//src/step//value",
        )],
    );

    let observation = build(options);
    assert_eq!(observation.observer_calls.len(), 1);
    let call = &observation.observer_calls[0];
    assert_eq!(call.mode, SwcMode::Step);
    assert_eq!(call.filename, "src/step.ts");
    assert_eq!(call.absolute_path, step);
    assert_eq!(call.source, source);
    assert_eq!(call.code, transformed_source);
    assert_eq!(
        call.manifest_entries,
        vec![ManifestEntry::step(
            "src/step.ts",
            "value",
            "step//src/step//value",
        )]
    );
}

#[test]
fn awaits_asynchronous_transform_observers_before_build_completion() {
    let root = TestRoot::new("observer-deferred");
    let step = root.write("src/step.ts", "export const value = 42;");
    let mut options = options(&root, SwcMode::Step, vec![step.clone()]);
    set_entries_to_bundle(&mut options, &[&step]);
    passthrough(&mut options, &step);
    options.observer = ObserverPolicy::Deferred;

    let observation = build(options);
    assert!(observation.observer_awaited_before_completion);
    assert_eq!(observation.observer_calls.len(), 1);
}

#[test]
fn fails_the_build_when_a_transform_observer_throws() {
    let root = TestRoot::new("observer-failure");
    let step = root.write("src/step.ts", "export const value = 42;");
    let mut options = options(&root, SwcMode::Step, vec![step.clone()]);
    set_entries_to_bundle(&mut options, &[&step]);
    passthrough(&mut options, &step);
    options.observer = ObserverPolicy::Fail("transform observer failed".to_owned());

    let error = build_result(options).unwrap_err();
    assert!(error.contains("transform observer failed"));
}

#[test]
fn fails_the_build_when_two_files_emit_the_same_step_id() {
    let root = TestRoot::new("duplicate-step");
    let first = root.write("src/confirmation.ts", "export const first = true;");
    let second = root.write("src/reschedule.ts", "export const second = true;");
    let id = "step//shared-package@1.0.0//sendMessage";
    let mut options = options(
        &root,
        SwcMode::Step,
        vec![first.clone(), second.clone()],
    );
    observer_recording(&mut options);
    transformed(
        &mut options,
        &first,
        &std::fs::read_to_string(&first).unwrap(),
        vec![ManifestEntry::step(
            &relative(&root.0, &first),
            "sendMessage",
            id,
        )],
    );
    transformed(
        &mut options,
        &second,
        &std::fs::read_to_string(&second).unwrap(),
        vec![ManifestEntry::step(
            &relative(&root.0, &second),
            "sendMessage",
            id,
        )],
    );

    let error = build_result(options).unwrap_err();
    assert!(error.contains("Duplicate workflow step ID"));
    assert_eq!(error.observation.observer_calls.len(), 1);
}

#[test]
fn deduplicates_identical_pnpm_peer_variant_copies_emitting_the_same_step_id() {
    let root = TestRoot::new("pnpm-identical");
    let source = "export const sendMessage = true;";
    let first = root.write(
        "node_modules/.pnpm/shared-package@1.0.0_peer-a@1.0.0/node_modules/shared-package/index.ts",
        source,
    );
    let second = root.write(
        "node_modules/.pnpm/shared-package@1.0.0_peer-b@2.0.0/node_modules/shared-package/index.ts",
        source,
    );
    let id = "step//shared-package@1.0.0//sendMessage";
    let mut options = options(
        &root,
        SwcMode::Step,
        vec![first.clone(), second.clone()],
    );
    observer_recording(&mut options);
    transformed(
        &mut options,
        &first,
        source,
        vec![ManifestEntry::step(&normalized(&first), "sendMessage", id)],
    );
    transformed(
        &mut options,
        &second,
        source,
        vec![ManifestEntry::step(&normalized(&second), "sendMessage", id)],
    );

    let observation = build(options);
    assert!(observation.errors.is_empty());
    assert_eq!(observation.observer_calls.len(), 2);
    assert_eq!(observation.manifest_entries.len(), 2);
    assert!(
        observation
            .manifest_entries
            .iter()
            .all(|entry| entry.id() == id)
    );
}

#[test]
fn fails_the_build_when_pnpm_style_copies_with_different_contents_emit_the_same_step_id() {
    let root = TestRoot::new("pnpm-divergent");
    let first = root.write(
        "node_modules/.pnpm/shared-package@1.0.0_peer-a@1.0.0/node_modules/shared-package/index.ts",
        "export const sendMessage = 1;",
    );
    let second = root.write(
        "node_modules/.pnpm/shared-package@1.0.0_peer-b@2.0.0/node_modules/shared-package/index.ts",
        "export const sendMessage = 2;",
    );
    let id = "step//shared-package@1.0.0//sendMessage";
    let mut options = options(
        &root,
        SwcMode::Step,
        vec![first.clone(), second.clone()],
    );
    transformed(
        &mut options,
        &first,
        &std::fs::read_to_string(&first).unwrap(),
        vec![ManifestEntry::step(&normalized(&first), "sendMessage", id)],
    );
    transformed(
        &mut options,
        &second,
        &std::fs::read_to_string(&second).unwrap(),
        vec![ManifestEntry::step(&normalized(&second), "sendMessage", id)],
    );

    let error = build_result(options).unwrap_err();
    assert!(error.contains("Duplicate workflow step ID"));
}

#[test]
fn fails_the_build_when_two_files_emit_the_same_workflow_id() {
    let root = TestRoot::new("duplicate-workflow");
    let first = root.write("src/confirmation.ts", "export const first = true;");
    let second = root.write("src/reschedule.ts", "export const second = true;");
    let id = "workflow//shared-package@1.0.0//sendMessage";
    let mut options = options(
        &root,
        SwcMode::Workflow,
        vec![first.clone(), second.clone()],
    );
    transformed(
        &mut options,
        &first,
        &std::fs::read_to_string(&first).unwrap(),
        vec![ManifestEntry::workflow(
            &relative(&root.0, &first),
            "sendMessage",
            id,
        )],
    );
    transformed(
        &mut options,
        &second,
        &std::fs::read_to_string(&second).unwrap(),
        vec![ManifestEntry::workflow(
            &relative(&root.0, &second),
            "sendMessage",
            id,
        )],
    );

    let error = build_result(options).unwrap_err();
    assert!(error.contains("Duplicate workflow ID"));
}
