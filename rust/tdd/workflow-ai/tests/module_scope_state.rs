use workflow_ai_tdd::scan_module_scope_state;

#[test]
fn reports_nothing_for_workflow_ai() {
    assert_eq!(scan_module_scope_state("packages/ai"), Vec::<String>::new());
}
