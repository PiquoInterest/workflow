use workflow_utils_tdd::{parse_class_name, parse_step_name, parse_workflow_name};

#[test]
fn parse_step_name_is_re_exported_from_the_crate_root() {
    let result = parse_step_name("step//./src/workflows/order//processOrder")
        .expect("valid step name must parse");
    assert_eq!(result.short_name, "processOrder");
}

#[test]
fn parse_workflow_name_is_re_exported_from_the_crate_root() {
    let result = parse_workflow_name(
        "workflow//./src/workflows/pulse//pulseRemoteWorkflow",
    )
    .expect("valid workflow name must parse");
    assert_eq!(result.short_name, "pulseRemoteWorkflow");
}

#[test]
fn parse_class_name_is_re_exported_from_the_crate_root() {
    let result = parse_class_name("class//./src/models/point//Point")
        .expect("valid class name must parse");
    assert_eq!(result.short_name, "Point");
}
