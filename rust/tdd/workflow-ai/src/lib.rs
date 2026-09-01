#![forbid(unsafe_code)]

use std::marker::PhantomData;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WeatherTool;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WeatherTools {
    pub get_weather: WeatherTool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DurableAgent<Tools> {
    pub model: String,
    pub tools: Tools,
}

pub trait AgentTools {
    type Tools;
}

impl<Tools> AgentTools for DurableAgent<Tools> {
    type Tools = Tools;
}

pub type InferDurableAgentTools<Agent> = <Agent as AgentTools>::Tools;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiMessage<Metadata, Tools> {
    pub metadata: Metadata,
    pub tools: PhantomData<Tools>,
}

pub type InferDurableAgentUiMessage<Agent, Metadata> =
    UiMessage<Metadata, InferDurableAgentTools<Agent>>;

pub fn durable_agent_type_contract() {
    panic!(
        "TDD RED: packages/ai/src/agent/durable-agent-types.test.ts implementation pending"
    )
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorValue {
    Error(String),
    CustomError(String),
    String(String),
    Object(Vec<(String, ErrorValue)>),
    Null,
    Undefined,
    Number(i64),
    Bool(bool),
    Array(Vec<ErrorValue>),
}

pub fn get_error_message(value: &ErrorValue) -> String {
    let _ = value;
    panic!("TDD RED: packages/ai/src/get-error-message.test.ts implementation pending")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IteratorCase {
    GeneratorToStream,
    BrowserMacrotaskYield,
    NonBrowserNoYield,
    AbortAfterFirst,
    AlreadyAborted,
    GeneratorError,
    StreamToIterator,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct StreamObservation {
    pub values: Vec<i32>,
    pub done: bool,
    pub macrotask_yields: usize,
    pub error: Option<String>,
}

pub fn exercise_stream_iterator(case: IteratorCase) -> StreamObservation {
    let _ = case;
    panic!("TDD RED: packages/ai/src/stream-iterator.test.ts implementation pending")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChatRepairCase {
    RawInterleavedText,
    RepairedInterleavedText,
    DuplicateTextTail,
    WellFormedMultiStep,
    RawInterleavedReasoning,
    RepairedInterleavedReasoning,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ChatRepairObservation {
    pub consumer_error: Option<String>,
    pub text: String,
    pub reasoning: String,
}

pub fn exercise_chat_repair(case: ChatRepairCase) -> ChatRepairObservation {
    let _ = case;
    panic!(
        "TDD RED: packages/ai/src/workflow-chat-transport.stream-repair.test.ts implementation pending"
    )
}

pub fn scan_module_scope_state(package_path: &str) -> Vec<String> {
    let _ = package_path;
    panic!("TDD RED: packages/ai/src/module-scope-state.test.ts implementation pending")
}
