use workflow_core_tdd::agent_e2e::{AgentScenario, run_agent_scenario};

#[test]
fn characterizes_the_typescript_experimental_on_start_gap() {
    let observation = run_agent_scenario(AgentScenario::ExperimentalOnStartGap);
    assert!(observation.call_sources.is_empty());
}

#[test]
fn characterizes_the_typescript_experimental_on_step_start_gap() {
    let observation = run_agent_scenario(AgentScenario::ExperimentalOnStepStartGap);
    assert!(observation.call_sources.is_empty());
}

#[test]
fn characterizes_the_typescript_experimental_on_tool_call_start_gap() {
    let observation = run_agent_scenario(AgentScenario::ExperimentalOnToolCallStartGap);
    assert!(observation.calls.is_empty());
}

#[test]
fn characterizes_the_typescript_experimental_on_tool_call_finish_gap() {
    let observation = run_agent_scenario(AgentScenario::ExperimentalOnToolCallFinishGap);
    assert!(observation.calls.is_empty());
    assert_eq!(observation.captured_finish_event, None);
}

#[test]
fn characterizes_the_typescript_prepare_call_gap() {
    let observation = run_agent_scenario(AgentScenario::PrepareCallGap);
    assert_eq!(observation.step_count, 1);
}

#[test]
fn characterizes_the_typescript_tool_approval_bypass() {
    let observation = run_agent_scenario(AgentScenario::ToolApprovalLegacyGap);
    assert_eq!(observation.step_count, 2);
    let approval = observation.approval.expect("approval observation");
    assert!(!approval.pending);
    assert_eq!(approval.tool_calls_count, 1);
    assert_eq!(approval.tool_results_count, 1);
    assert_eq!(approval.first_tool_call_name.as_deref(), Some("riskyTool"));
    assert!(approval.tool_executed);
}

#[test]
fn rust_target_pauses_needs_approval_tools_before_execution() {
    let observation = run_agent_scenario(AgentScenario::ToolApprovalSecureTarget);
    let approval = observation.approval.expect("approval observation");
    assert!(approval.pending);
    assert_eq!(approval.tool_calls_count, 1);
    assert_eq!(approval.tool_results_count, 0);
    assert_eq!(approval.first_tool_call_name.as_deref(), Some("riskyTool"));
    assert!(!approval.tool_executed);
}
