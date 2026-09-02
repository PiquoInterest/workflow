use std::collections::BTreeMap;

use crate::ModelValue;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TelemetryValue {
    String(String),
    Number(i64),
    StringList(Vec<String>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TelemetryCase {
    DoStreamResponseAttributes,
    DoStreamToolCalls,
    DoStreamWithoutRecordedInputs,
    DoStreamWithoutRecordedOutputs,
    DoStreamReasoningAndCacheTokens,
    ExecuteToolResult,
    ExecuteToolWithoutRecordedOutputs,
    ExecuteToolSpanContext,
    StreamTextIteratorOptions,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TelemetryOptionsObservation {
    pub is_enabled: bool,
    pub function_id: Option<String>,
    pub record_inputs: Option<bool>,
    pub record_outputs: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TelemetryObservation {
    pub record_span_names: Vec<String>,
    pub initial_attributes: BTreeMap<String, TelemetryValue>,
    pub emitted_attributes: BTreeMap<String, TelemetryValue>,
    pub tool_span_attributes: BTreeMap<String, TelemetryValue>,
    pub tool_call_names: Vec<String>,
    pub tool_result: Option<ModelValue>,
    pub run_in_context_handles: Vec<String>,
    pub iterator_telemetry: Option<TelemetryOptionsObservation>,
}

pub fn exercise_telemetry(case: TelemetryCase) -> TelemetryObservation {
    let _ = case;
    panic!("TDD RED: packages/ai/src/agent/telemetry.test.ts implementation pending")
}
