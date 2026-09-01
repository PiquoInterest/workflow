use workflow_core_tdd::runtime_import::import_runtime;

#[test]
fn runtime_import_does_not_load_vercel_functions() {
    let observation = import_runtime();
    assert!(observation.runtime_defined);
    assert!(
        !observation
            .loaded_platform_modules
            .iter()
            .any(|module| module == "@vercel/functions")
    );
}
