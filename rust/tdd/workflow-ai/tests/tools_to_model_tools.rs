use workflow_ai_tdd::{
    FunctionModelTool, FunctionToolDefinition, ModelTool, ModelValue, ProviderModelTool,
    ProviderToolDefinition, ToolDefinition, tools_to_model_tools,
};

fn object(entries: &[(&str, ModelValue)]) -> ModelValue {
    ModelValue::Object(
        entries
            .iter()
            .map(|(key, value)| ((*key).to_owned(), value.clone()))
            .collect(),
    )
}

fn input_schema(field: &str) -> ModelValue {
    object(&[
        ("type", ModelValue::String("object".to_owned())),
        (
            "properties",
            object(&[(field, ModelValue::String("string".to_owned()))]),
        ),
    ])
}

fn function_tool(description: &str, field: &str) -> FunctionToolDefinition {
    FunctionToolDefinition {
        description: Some(description.to_owned()),
        input_schema: input_schema(field),
        strict: None,
        input_examples: None,
        provider_options: None,
    }
}

fn provider_tool(id: &str, args: Option<ModelValue>) -> ProviderToolDefinition {
    ProviderToolDefinition {
        id: id.to_owned(),
        args,
        input_schema: object(&[
            ("type", ModelValue::String("object".to_owned())),
            ("properties", object(&[])),
        ]),
    }
}

#[test]
fn serializes_function_tools_with_description_and_input_schema() {
    let definition = function_tool("Get the weather", "city");
    let expected_schema = definition.input_schema.clone();
    let result = tools_to_model_tools(vec![(
        "weather".to_owned(),
        ToolDefinition::Function(definition),
    )]);

    assert_eq!(result.len(), 1);
    assert_eq!(
        result[0],
        ModelTool::Function(FunctionModelTool {
            name: "weather".to_owned(),
            description: Some("Get the weather".to_owned()),
            input_schema: expected_schema,
            strict: None,
            input_examples: None,
            provider_options: None,
        })
    );
}

#[test]
fn preserves_provider_tool_type_id_and_args() {
    let args = object(&[("maxUses", ModelValue::Number(5))]);
    let result = tools_to_model_tools(vec![(
        "webSearch".to_owned(),
        ToolDefinition::Provider(provider_tool(
            "anthropic.web_search",
            Some(args.clone()),
        )),
    )]);

    assert_eq!(
        result,
        vec![ModelTool::Provider(ProviderModelTool {
            id: "anthropic.web_search".to_owned(),
            name: "webSearch".to_owned(),
            args,
        })]
    );
}

#[test]
fn handles_mixed_function_and_provider_tools() {
    let result = tools_to_model_tools(vec![
        (
            "weather".to_owned(),
            ToolDefinition::Function(function_tool("Get the weather", "city")),
        ),
        (
            "webSearch".to_owned(),
            ToolDefinition::Provider(provider_tool(
                "anthropic.web_search",
                Some(object(&[])),
            )),
        ),
    ]);

    assert_eq!(result.len(), 2);
    assert!(matches!(
        &result[0],
        ModelTool::Function(FunctionModelTool { name, .. }) if name == "weather"
    ));
    assert_eq!(
        result[1],
        ModelTool::Provider(ProviderModelTool {
            id: "anthropic.web_search".to_owned(),
            name: "webSearch".to_owned(),
            args: object(&[]),
        })
    );
}

#[test]
fn defaults_provider_tool_args_to_an_empty_object() {
    let result = tools_to_model_tools(vec![(
        "codeExec".to_owned(),
        ToolDefinition::Provider(provider_tool("anthropic.code_execution", None)),
    )]);

    assert_eq!(
        result[0],
        ModelTool::Provider(ProviderModelTool {
            id: "anthropic.code_execution".to_owned(),
            name: "codeExec".to_owned(),
            args: object(&[]),
        })
    );
}

#[test]
fn forwards_strict_true() {
    let mut definition = function_tool("Get weather", "location");
    definition.strict = Some(true);
    let result = tools_to_model_tools(vec![(
        "weather".to_owned(),
        ToolDefinition::Function(definition),
    )]);

    assert!(matches!(
        &result[0],
        ModelTool::Function(FunctionModelTool {
            strict: Some(true),
            ..
        })
    ));
}

#[test]
fn forwards_strict_false() {
    let mut definition = function_tool("Get weather", "location");
    definition.strict = Some(false);
    let result = tools_to_model_tools(vec![(
        "weather".to_owned(),
        ToolDefinition::Function(definition),
    )]);

    assert!(matches!(
        &result[0],
        ModelTool::Function(FunctionModelTool {
            strict: Some(false),
            ..
        })
    ));
}

#[test]
fn omits_strict_when_not_set() {
    let result = tools_to_model_tools(vec![(
        "weather".to_owned(),
        ToolDefinition::Function(function_tool("Get weather", "location")),
    )]);

    assert!(matches!(
        &result[0],
        ModelTool::Function(FunctionModelTool { strict: None, .. })
    ));
}

#[test]
fn forwards_input_examples() {
    let examples = vec![object(&[(
        "input",
        object(&[("location", ModelValue::String("Tokyo".to_owned()))]),
    )])];
    let mut definition = function_tool("Get weather", "location");
    definition.input_examples = Some(examples.clone());
    let result = tools_to_model_tools(vec![(
        "weather".to_owned(),
        ToolDefinition::Function(definition),
    )]);

    assert!(matches!(
        &result[0],
        ModelTool::Function(FunctionModelTool {
            input_examples: Some(actual),
            ..
        }) if actual == &examples
    ));
}

#[test]
fn omits_input_examples_when_not_set() {
    let result = tools_to_model_tools(vec![(
        "weather".to_owned(),
        ToolDefinition::Function(function_tool("Get weather", "location")),
    )]);

    assert!(matches!(
        &result[0],
        ModelTool::Function(FunctionModelTool {
            input_examples: None,
            ..
        })
    ));
}

#[test]
fn forwards_provider_options() {
    let provider_options = object(&[(
        "openai",
        object(&[("parallel_tool_calls", ModelValue::Bool(false))]),
    )]);
    let mut definition = function_tool("Get weather", "location");
    definition.provider_options = Some(provider_options.clone());
    let result = tools_to_model_tools(vec![(
        "weather".to_owned(),
        ToolDefinition::Function(definition),
    )]);

    assert!(matches!(
        &result[0],
        ModelTool::Function(FunctionModelTool {
            provider_options: Some(actual),
            ..
        }) if actual == &provider_options
    ));
}

#[test]
fn handles_dynamic_tools_as_function_tools() {
    let result = tools_to_model_tools(vec![(
        "dynamic".to_owned(),
        ToolDefinition::Dynamic(function_tool("A dynamic tool", "input")),
    )]);

    assert!(matches!(
        &result[0],
        ModelTool::Function(FunctionModelTool {
            name,
            description: Some(description),
            ..
        }) if name == "dynamic" && description == "A dynamic tool"
    ));
}

#[test]
fn returns_empty_array_for_empty_tools() {
    assert_eq!(tools_to_model_tools(Vec::new()), Vec::<ModelTool>::new());
}
