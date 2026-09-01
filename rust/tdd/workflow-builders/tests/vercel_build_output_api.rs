use workflow_builders_tdd::build_vercel_esm_fixture;

#[test]
fn bundled_cjs_dependencies_receive_exactly_one_esm_dirname_shim_per_function() {
    let observation = build_vercel_esm_fixture();

    assert_eq!(observation.step_result, "dirs-ok");
    assert_eq!(observation.combined_file_url_import_count, 1);
    assert_eq!(observation.combined_dirname_definition_count, 1);
    assert_eq!(observation.webhook_file_url_import_count, 1);
    assert_eq!(observation.webhook_dirname_definition_count, 1);
}
