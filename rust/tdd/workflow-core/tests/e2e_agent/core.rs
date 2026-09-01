use workflow_core_tdd::agent_e2e::{AgentScenario, run_agent_scenario};

#[test]
fn returns_a_basic_text_response() {
    let observation = run_agent_scenario(AgentScenario::BasicText {
        prompt: "hello world".to_owned(),
    });
    assert_eq!(observation.step_count, 1);
    assert_eq!(
        observation.last_step_text.as_deref(),
        Some("Echo: hello world")
    );
}

#[test]
fn executes_a_single_function_tool_call() {
    let observation = run_agent_scenario(AgentScenario::SingleToolCall { left: 3, right: 7 });
    assert_eq!(observation.step_count, 2);
    assert_eq!(observation.last_step_text.as_deref(), Some("The sum is 10"));
}

#[test]
fn executes_multiple_sequential_tool_calls() {
    let observation = run_agent_scenario(AgentScenario::MultipleSequentialTools);
    assert_eq!(observation.step_count, 4);
    assert_eq!(observation.last_step_text.as_deref(), Some("All done!"));
}

#[test]
fn lets_the_model_recover_after_a_tool_error() {
    let observation = run_agent_scenario(AgentScenario::ToolErrorRecovery);
    assert_eq!(observation.step_count, 2);
    assert_eq!(
        observation.last_step_text.as_deref(),
        Some("Tool failed but I recovered.")
    );
}

#[test]
fn passes_string_instructions_to_the_model() {
    let observation = run_agent_scenario(AgentScenario::StringInstructions);
    assert_eq!(observation.step_count, 1);
    assert_eq!(observation.last_step_text.as_deref(), Some("ok"));
}

#[test]
fn completes_within_the_configured_timeout() {
    let observation = run_agent_scenario(AgentScenario::Timeout);
    assert_eq!(observation.step_count, 1);
    assert_eq!(observation.last_step_text.as_deref(), Some("fast response"));
}
