use workflow_errors_tdd::scan_module_scope_state;

#[test]
fn reports_nothing_for_workflow_errors() {
    assert_eq!(scan_module_scope_state("packages/errors"), Vec::<String>::new());
}
