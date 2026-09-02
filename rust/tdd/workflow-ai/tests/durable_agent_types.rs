use std::marker::PhantomData;

use workflow_ai_tdd::{
    DurableAgent, InferDurableAgentTools, InferDurableAgentUiMessage, UiMessage, WeatherTool,
    WeatherTools, durable_agent_type_contract,
};

type WeatherAgent = DurableAgent<WeatherTools>;

#[derive(Debug, Clone, PartialEq, Eq)]
struct ThreadMetadata {
    thread_id: String,
}

#[test]
fn infers_tools_from_a_durable_agent() {
    durable_agent_type_contract();

    let tools: InferDurableAgentTools<WeatherAgent> = WeatherTools {
        get_weather: WeatherTool,
    };
    let _: WeatherTools = tools;
}

#[test]
fn exposes_configured_tools_on_the_agent_instance() {
    durable_agent_type_contract();

    let agent = DurableAgent {
        model: "test-model".to_owned(),
        tools: WeatherTools {
            get_weather: WeatherTool,
        },
    };
    let _: WeatherTools = agent.tools;
}

#[test]
fn infers_the_ui_message_type_from_a_durable_agent() {
    durable_agent_type_contract();

    let message: InferDurableAgentUiMessage<WeatherAgent, ThreadMetadata> = UiMessage {
        metadata: ThreadMetadata {
            thread_id: "thread_1".to_owned(),
        },
        tools: PhantomData,
    };
    let _: UiMessage<ThreadMetadata, WeatherTools> = message;
}
