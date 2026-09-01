use workflow_core_tdd::module_scope_state::scan_core_module_scope_state;

#[test]
fn reports_no_prohibited_module_scope_state_for_core() {
    assert_eq!(scan_core_module_scope_state(), Vec::new());
}
