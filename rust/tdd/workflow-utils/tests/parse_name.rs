use workflow_utils_tdd::{
    ParsedName, format_step_name, format_workflow_name, parse_class_name, parse_step_name,
    parse_workflow_name, step_display_name, workflow_display_name,
};

fn parsed(short_name: &str, module_specifier: &str, function_name: &str) -> ParsedName {
    ParsedName {
        short_name: short_name.to_owned(),
        module_specifier: module_specifier.to_owned(),
        function_name: function_name.to_owned(),
    }
}

#[test]
fn parses_a_valid_workflow_name_with_relative_path() {
    assert_eq!(
        parse_workflow_name("workflow//./src/workflows/order//handleOrder"),
        Some(parsed(
            "handleOrder",
            "./src/workflows/order",
            "handleOrder"
        ))
    );
}

#[test]
fn parses_a_valid_workflow_name_with_module_specifier() {
    assert_eq!(
        parse_workflow_name("workflow//mypackage@1.0.0//handleOrder"),
        Some(parsed("handleOrder", "mypackage@1.0.0", "handleOrder"))
    );
}

#[test]
fn parses_a_valid_workflow_name_with_scoped_module_specifier() {
    assert_eq!(
        parse_workflow_name("workflow//@myorg/tasks@2.0.0//processOrder"),
        Some(parsed("processOrder", "@myorg/tasks@2.0.0", "processOrder"))
    );
}

#[test]
fn parses_workflow_names_with_nested_function_names() {
    assert_eq!(
        parse_workflow_name("workflow//./src/app//nested//function//name"),
        Some(parsed("name", "./src/app", "nested//function//name"))
    );
}

#[test]
fn rejects_invalid_workflow_names() {
    assert_eq!(parse_workflow_name("invalid"), None);
    assert_eq!(parse_workflow_name("workflow//"), None);
    assert_eq!(parse_workflow_name("step//path//fn"), None);
}

#[test]
fn accepts_a_workflow_name_with_an_empty_function_part() {
    assert_eq!(
        parse_workflow_name("workflow//./path//"),
        Some(parsed("", "./path", ""))
    );
}

#[test]
fn uses_the_module_name_for_relative_default_workflow_exports() {
    assert_eq!(
        parse_workflow_name("workflow//./src/jobs/order//default"),
        Some(parsed("order", "./src/jobs/order", "default"))
    );
}

#[test]
fn uses_the_package_name_for_default_workflow_exports() {
    assert_eq!(
        parse_workflow_name("workflow//mypackage@1.0.0//default"),
        Some(parsed("mypackage", "mypackage@1.0.0", "default"))
    );
}

#[test]
fn uses_the_scoped_package_name_for_default_workflow_exports() {
    assert_eq!(
        parse_workflow_name("workflow//@myorg/tasks@2.0.0//default"),
        Some(parsed("tasks", "@myorg/tasks@2.0.0", "default"))
    );
}

#[test]
fn parses_a_valid_step_name_with_relative_path() {
    assert_eq!(
        parse_step_name("step//./src/workflows/order//processOrder"),
        Some(parsed(
            "processOrder",
            "./src/workflows/order",
            "processOrder"
        ))
    );
}

#[test]
fn parses_a_valid_step_name_with_module_specifier() {
    assert_eq!(
        parse_step_name("step//mypackage@1.0.0//processOrder"),
        Some(parsed("processOrder", "mypackage@1.0.0", "processOrder"))
    );
}

#[test]
fn parses_a_step_name_with_nested_path() {
    assert_eq!(
        parse_step_name("step//./app/api/generate/route//handleStep"),
        Some(parsed(
            "handleStep",
            "./app/api/generate/route",
            "handleStep"
        ))
    );
}

#[test]
fn rejects_invalid_step_names() {
    assert_eq!(parse_step_name("invalid"), None);
    assert_eq!(parse_step_name("step//"), None);
    assert_eq!(parse_step_name("workflow//path//fn"), None);
}

#[test]
fn accepts_a_step_name_with_an_empty_function_part() {
    assert_eq!(
        parse_step_name("step//./path//"),
        Some(parsed("", "./path", ""))
    );
}

#[test]
fn parses_builtin_step_names() {
    assert_eq!(
        parse_step_name("step//builtin//__builtin_fetch"),
        Some(parsed("__builtin_fetch", "builtin", "__builtin_fetch"))
    );
}

#[test]
fn parses_a_nested_step_in_a_workflow() {
    assert_eq!(
        parse_step_name("step//./src/jobs/order//processOrder/innerStep"),
        Some(parsed(
            "innerStep",
            "./src/jobs/order",
            "processOrder/innerStep"
        ))
    );
}

#[test]
fn parses_a_static_method_step() {
    assert_eq!(
        parse_step_name("step//./src/jobs/order//MyClass.staticMethod"),
        Some(parsed(
            "MyClass.staticMethod",
            "./src/jobs/order",
            "MyClass.staticMethod"
        ))
    );
}

#[test]
fn parses_an_instance_method_step() {
    assert_eq!(
        parse_step_name("step//./src/jobs/order//MyClass#instanceMethod"),
        Some(parsed(
            "MyClass#instanceMethod",
            "./src/jobs/order",
            "MyClass#instanceMethod"
        ))
    );
}

#[test]
fn parses_a_valid_class_id_with_relative_path() {
    assert_eq!(
        parse_class_name("class//./src/models/point//Point"),
        Some(parsed("Point", "./src/models/point", "Point"))
    );
}

#[test]
fn parses_a_valid_class_id_with_module_specifier() {
    assert_eq!(
        parse_class_name("class//point@0.0.1//Point"),
        Some(parsed("Point", "point@0.0.1", "Point"))
    );
}

#[test]
fn parses_a_class_id_with_scoped_module_specifier() {
    assert_eq!(
        parse_class_name("class//@myorg/models@1.2.3//UserData"),
        Some(parsed("UserData", "@myorg/models@1.2.3", "UserData"))
    );
}

#[test]
fn parses_a_class_id_with_nested_path() {
    assert_eq!(
        parse_class_name("class//./workflows/user-signup//UserData"),
        Some(parsed("UserData", "./workflows/user-signup", "UserData"))
    );
}

#[test]
fn rejects_invalid_class_ids() {
    assert_eq!(parse_class_name("invalid"), None);
    assert_eq!(parse_class_name("class//"), None);
    assert_eq!(parse_class_name("step//path//fn"), None);
    assert_eq!(parse_class_name("workflow//path//fn"), None);
}

#[test]
fn formats_a_relative_step_as_short_name_and_module_specifier() {
    assert_eq!(
        format_step_name("step//./workflows/1_simple//add"),
        "add (./workflows/1_simple)"
    );
}

#[test]
fn formats_a_relative_workflow_as_short_name_and_module_specifier() {
    assert_eq!(
        format_workflow_name("workflow//./workflows/1_simple//simple"),
        "simple (./workflows/1_simple)"
    );
}

#[test]
fn formats_a_step_with_module_specifier() {
    assert_eq!(
        format_step_name("step//@myorg/tasks@2.0.0//processOrder"),
        "processOrder (@myorg/tasks@2.0.0)"
    );
}

#[test]
fn formats_nested_functions_using_the_leaf_name() {
    assert_eq!(
        format_step_name("step//./workflows/order//processOrder/innerStep"),
        "innerStep (./workflows/order)"
    );
}

#[test]
fn formatting_falls_back_to_the_unrecognized_raw_name() {
    assert_eq!(format_step_name("something-weird"), "something-weird");
    assert_eq!(
        format_workflow_name("step//wrong-tag//fn"),
        "step//wrong-tag//fn"
    );
}

#[test]
fn display_names_return_short_names_for_raw_machine_names() {
    assert_eq!(
        workflow_display_name("workflow//./src/jobs/order//processOrder"),
        "processOrder"
    );
    assert_eq!(
        step_display_name("step//./src/jobs/order//chargeCard"),
        "chargeCard"
    );
}

#[test]
fn display_names_return_short_names_for_module_specifiers() {
    assert_eq!(
        workflow_display_name("workflow//@myorg/shared@1.2.3//sync"),
        "sync"
    );
}

#[test]
fn display_names_recover_functions_from_queue_sanitized_names() {
    assert_eq!(
        workflow_display_name("workflow----src-jobs-order--processOrder"),
        "processOrder"
    );
    assert_eq!(
        step_display_name("step----src-jobs-order--chargeCard"),
        "chargeCard"
    );
}

#[test]
fn sanitized_nested_functions_use_the_leaf_name() {
    assert_eq!(
        step_display_name("step----src-jobs-order--processOrder-innerStep"),
        "innerStep"
    );
}

#[test]
fn sanitized_default_exports_map_to_the_module_short_name() {
    assert_eq!(
        workflow_display_name("workflow----src-jobs-order--default"),
        "order"
    );
    assert_eq!(
        step_display_name("step----src-jobs-order--default"),
        "order"
    );
    assert_eq!(
        workflow_display_name("workflow----src-jobs-order--__default"),
        "order"
    );
}

#[test]
fn sanitized_dollar_names_degrade_to_the_trailing_segment() {
    assert_eq!(
        step_display_name("step----src-jobs-order--process-Order"),
        "Order"
    );
}

#[test]
fn display_names_fall_back_to_unrecognized_input() {
    assert_eq!(workflow_display_name("my-plain-name"), "my-plain-name");
    assert_eq!(
        step_display_name("workflow--wrong-tag--fn"),
        "workflow--wrong-tag--fn"
    );
}
