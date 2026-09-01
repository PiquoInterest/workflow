use workflow_core_tdd::agent_e2e::{
    AgentScenario, CapturedFinishEvent, CapturedStepResult, run_agent_scenario,
};

#[test]
fn fires_constructor_and_method_on_step_finish_callbacks_in_order() {
    let observation = run_agent_scenario(AgentScenario::OnStepFinish);
    assert_eq!(observation.call_sources, vec!["constructor", "method"]);
    assert_eq!(
        observation.captured_step_result,
        Some(CapturedStepResult {
            text: "hello".to_owned(),
            finish_reason: "stop".to_owned(),
        })
    );
    assert_eq!(observation.step_count, 1);
}

#[test]
fn fires_constructor_and_method_on_finish_callbacks_in_order() {
    let observation = run_agent_scenario(AgentScenario::OnFinish);
    assert_eq!(observation.call_sources, vec!["constructor", "method"]);
    assert_eq!(
        observation.captured_finish_event,
        Some(CapturedFinishEvent {
            text: "hello from finish".to_owned(),
            finish_reason: "stop".to_owned(),
            steps_length: 1,
            has_messages: true,
            has_total_usage: true,
        })
    );
}

#[test]
fn constructor_prepare_step_runs_for_each_model_step() {
    let observation = run_agent_scenario(AgentScenario::ConstructorPrepareStep);
    assert_eq!(observation.step_count, 2);
    assert_eq!(observation.prepare_step_call_count, 2);
    assert_eq!(observation.prepare_step_numbers, vec![0, 1]);
}

#[test]
fn stream_prepare_step_overrides_constructor_prepare_step() {
    let observation = run_agent_scenario(AgentScenario::StreamPrepareStepOverride);
    assert_eq!(observation.prepare_step_sources, vec!["stream"]);
}
