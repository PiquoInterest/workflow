use crate::{ModelValue, ToolCallInput};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamTextIteratorCase {
    ProviderMetadata,
    NoProviderMetadata,
    ParallelProviderMetadata,
    MixedProviderMetadata,
    OpenAiItemId,
    OpenAiAllFields,
    MixedProviders,
    ReasoningBeforeToolCall,
    ReasoningProviderOptions,
    NoReasoning,
    PrepareSystemOnly,
    PrepareSystemAndMessages,
    ReplaceExistingSystem,
    UpdateSystemOnSubsequentStep,
    MalformedToolCallInput,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PromptMessage {
    System(String),
    User(String),
    Assistant(Vec<AssistantPart>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssistantPart {
    Reasoning {
        text: String,
        provider_options: Option<ModelValue>,
    },
    ToolCall {
        tool_call_id: String,
        tool_name: String,
        input: ToolCallInput,
        provider_options: Option<ModelValue>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct StreamTextIteratorObservation {
    pub first_done: bool,
    pub first_tool_call_count: usize,
    pub captured_prompts: Vec<Vec<PromptMessage>>,
}

pub fn exercise_stream_text_iterator(
    case: StreamTextIteratorCase,
) -> StreamTextIteratorObservation {
    let _ = case;
    panic!("TDD RED: packages/ai/src/agent/stream-text-iterator.test.ts implementation pending")
}
