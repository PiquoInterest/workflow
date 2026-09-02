use workflow_builders_tdd::{StepRegistrationFixture, create_step_registrations};

#[test]
fn imports_serde_only_files_for_step_context_class_registration() {
    let output = create_step_registrations(StepRegistrationFixture::SerdeOnlyFile).unwrap();

    assert!(
        output
            .generated_source
            .contains("import \"workflow/internal/builtins\";")
    );
    assert!(
        output
            .generated_source
            .contains("import \"../src/step.ts\";")
    );
    assert!(
        output
            .generated_source
            .contains("import \"../src/serde.ts\";")
    );
    assert!(output.class_files.iter().any(|path| path == "src/serde.ts"));
}

#[test]
fn deduplicates_identical_pnpm_peer_variant_package_copies() {
    let output =
        create_step_registrations(StepRegistrationFixture::IdenticalPnpmPeerCopies).unwrap();

    assert_eq!(output.manifest_step_ids.len(), 2);
    assert!(
        output
            .manifest_step_ids
            .iter()
            .all(|step_id| { step_id == "step//step-pkg@1.0.0//runPackagedStep" })
    );
}

#[test]
fn rejects_pnpm_style_package_copies_whose_implementations_differ() {
    let error =
        create_step_registrations(StepRegistrationFixture::DivergentPnpmPeerCopies).unwrap_err();
    assert!(error.contains("Duplicate workflow step ID"));
}
