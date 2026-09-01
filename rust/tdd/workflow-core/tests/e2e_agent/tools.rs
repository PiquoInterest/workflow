use workflow_core_tdd::agent_e2e::{AgentScenario, run_agent_scenario};

#[test]
fn preserves_provider_tool_identity_across_step_boundaries() {
    let observation = run_agent_scenario(AgentScenario::ProviderTool);
    assert_eq!(observation.step_count, 2);
    assert_eq!(
        observation.last_step_text.as_deref(),
        Some("I found a result for you.")
    );
}

#[test]
fn supports_mixed_provider_and_function_tools() {
    let observation = run_agent_scenario(AgentScenario::MixedProviderAndFunctionTools {
        left: 3,
        right: 7,
    });
    assert_eq!(observation.step_count, 3);
    assert_eq!(observation.last_step_text.as_deref(), Some("The answer is 10"));
}

#[test]
fn passes_multimodal_tool_results_back_to_the_model() {
    let observation = run_agent_scenario(AgentScenario::MultimodalToolResult);
    assert_eq!(observation.step_count, 2);
    assert_eq!(observation.last_step_text.as_deref(), Some("I see the image"));
}
