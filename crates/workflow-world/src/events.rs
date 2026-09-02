use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::event_metadata::event_data_ref_fields;
use crate::shared::ResolveData;

/// All event types understood by the current World protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    RunCreated,
    RunStarted,
    RunCompleted,
    RunFailed,
    RunCancelled,
    AttrSet,
    StepCreated,
    StepCompleted,
    StepFailed,
    StepRetrying,
    StepStarted,
    HookCreated,
    HookReceived,
    HookDisposed,
    HookConflict,
    WaitCreated,
    WaitCompleted,
    Noop,
}

impl EventType {
    /// Parses a canonical event type string.
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "run_created" => Some(Self::RunCreated),
            "run_started" => Some(Self::RunStarted),
            "run_completed" => Some(Self::RunCompleted),
            "run_failed" => Some(Self::RunFailed),
            "run_cancelled" => Some(Self::RunCancelled),
            "attr_set" => Some(Self::AttrSet),
            "step_created" => Some(Self::StepCreated),
            "step_completed" => Some(Self::StepCompleted),
            "step_failed" => Some(Self::StepFailed),
            "step_retrying" => Some(Self::StepRetrying),
            "step_started" => Some(Self::StepStarted),
            "hook_created" => Some(Self::HookCreated),
            "hook_received" => Some(Self::HookReceived),
            "hook_disposed" => Some(Self::HookDisposed),
            "hook_conflict" => Some(Self::HookConflict),
            "wait_created" => Some(Self::WaitCreated),
            "wait_completed" => Some(Self::WaitCompleted),
            "noop" => Some(Self::Noop),
            _ => None,
        }
    }

    /// Canonical wire name.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RunCreated => "run_created",
            Self::RunStarted => "run_started",
            Self::RunCompleted => "run_completed",
            Self::RunFailed => "run_failed",
            Self::RunCancelled => "run_cancelled",
            Self::AttrSet => "attr_set",
            Self::StepCreated => "step_created",
            Self::StepCompleted => "step_completed",
            Self::StepFailed => "step_failed",
            Self::StepRetrying => "step_retrying",
            Self::StepStarted => "step_started",
            Self::HookCreated => "hook_created",
            Self::HookReceived => "hook_received",
            Self::HookDisposed => "hook_disposed",
            Self::HookConflict => "hook_conflict",
            Self::WaitCreated => "wait_created",
            Self::WaitCompleted => "wait_completed",
            Self::Noop => "noop",
        }
    }

    /// Whether callers may submit this event to `events.create`.
    pub const fn is_user_creatable(self) -> bool {
        !matches!(self, Self::HookConflict | Self::Noop)
    }

    pub const fn is_run_event(self) -> bool {
        matches!(
            self,
            Self::RunCreated
                | Self::RunStarted
                | Self::RunCompleted
                | Self::RunFailed
                | Self::RunCancelled
        )
    }

    pub const fn is_terminal_run_event(self) -> bool {
        matches!(
            self,
            Self::RunCompleted | Self::RunFailed | Self::RunCancelled
        )
    }

    pub const fn is_step_event(self) -> bool {
        matches!(
            self,
            Self::StepCreated
                | Self::StepCompleted
                | Self::StepFailed
                | Self::StepRetrying
                | Self::StepStarted
        )
    }

    pub const fn is_terminal_step_event(self) -> bool {
        matches!(self, Self::StepCompleted | Self::StepFailed)
    }

    pub const fn is_hook_lifecycle_event(self) -> bool {
        matches!(
            self,
            Self::HookCreated | Self::HookReceived | Self::HookDisposed
        )
    }

    pub const fn requires_existing_hook(self) -> bool {
        matches!(self, Self::HookReceived | Self::HookDisposed)
    }

    pub const fn is_wait_event(self) -> bool {
        matches!(self, Self::WaitCreated | Self::WaitCompleted)
    }

    pub const fn is_child_entity_creation_event(self) -> bool {
        matches!(
            self,
            Self::StepCreated | Self::HookCreated | Self::WaitCreated
        )
    }
}

/// Returns whether a raw event type belongs to the run lifecycle.
pub fn is_run_event_type(value: &str) -> bool {
    EventType::parse(value).is_some_and(EventType::is_run_event)
}

/// Returns whether a raw event type is terminal for a run.
pub fn is_terminal_run_event_type(value: &str) -> bool {
    EventType::parse(value).is_some_and(EventType::is_terminal_run_event)
}

/// Returns whether a raw event type belongs to the step lifecycle.
pub fn is_step_event_type(value: &str) -> bool {
    EventType::parse(value).is_some_and(EventType::is_step_event)
}

/// Returns whether a raw event type is terminal for a step.
pub fn is_terminal_step_event_type(value: &str) -> bool {
    EventType::parse(value).is_some_and(EventType::is_terminal_step_event)
}

/// Returns whether a raw event type belongs to a hook lifecycle.
pub fn is_hook_lifecycle_event_type(value: &str) -> bool {
    EventType::parse(value).is_some_and(EventType::is_hook_lifecycle_event)
}

/// Returns whether a raw hook event requires an existing hook.
pub fn is_hook_event_requiring_existence(value: &str) -> bool {
    EventType::parse(value).is_some_and(EventType::requires_existing_hook)
}

/// Returns whether a raw event type belongs to the wait lifecycle.
pub fn is_wait_event_type(value: &str) -> bool {
    EventType::parse(value).is_some_and(EventType::is_wait_event)
}

/// Returns whether a raw event type creates a child entity.
pub fn is_child_entity_creation_event_type(value: &str) -> bool {
    EventType::parse(value).is_some_and(EventType::is_child_entity_creation_event)
}

/// Includes lazy `step_started` requests that create their step on demand.
pub fn is_child_entity_creation_event(event: &Value) -> bool {
    let Some(event_type) = event.get("eventType").and_then(Value::as_str) else {
        return false;
    };
    if is_child_entity_creation_event_type(event_type) {
        return true;
    }
    if event_type != "step_started" {
        return false;
    }

    let Some(event_data) = event.get("eventData").and_then(Value::as_object) else {
        return false;
    };
    event_data.get("stepName").is_some_and(Value::is_string) && event_data.contains_key("input")
}

/// Strips opaque payload fields when data resolution is disabled.
pub fn strip_event_data_refs(mut event: Value, resolve_data: ResolveData) -> Value {
    if resolve_data != ResolveData::None {
        return event;
    }

    let event_type = event
        .get("eventType")
        .and_then(Value::as_str)
        .map(str::to_owned);
    let Some(event_object) = event.as_object_mut() else {
        return event;
    };
    let Some(event_data) = event_object.get("eventData") else {
        return event;
    };

    // JavaScript treats arrays as objects here, but removes primitive or null
    // eventData completely before consulting the per-event payload mapping.
    if !event_data.is_object() && !event_data.is_array() {
        event_object.remove("eventData");
        return event;
    }

    let Some(event_type) = event_type else {
        return event;
    };
    let ref_fields = event_data_ref_fields(&event_type);
    if ref_fields.is_empty() {
        return event;
    }

    let event_data = event_object
        .remove("eventData")
        .expect("eventData was checked above");
    let mut stripped = object_spread(event_data);
    for field in ref_fields {
        stripped.remove(*field);
    }
    if !stripped.is_empty() {
        event_object.insert("eventData".to_owned(), Value::Object(stripped));
    }
    event
}

fn object_spread(value: Value) -> Map<String, Value> {
    match value {
        Value::Object(object) => object,
        Value::Array(values) => values
            .into_iter()
            .enumerate()
            .map(|(index, value)| (index.to_string(), value))
            .collect(),
        _ => Map::new(),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn world_only_events_are_not_user_creatable() {
        assert!(!EventType::HookConflict.is_user_creatable());
        assert!(!EventType::Noop.is_user_creatable());
        assert!(EventType::StepCreated.is_user_creatable());
    }

    #[test]
    fn recognizes_lazy_step_creation_only_with_name_and_input() {
        assert!(is_child_entity_creation_event(&json!({
            "eventType": "step_started",
            "eventData": { "stepName": "work", "input": null }
        })));
        assert!(!is_child_entity_creation_event(&json!({
            "eventType": "step_started",
            "eventData": { "stepName": "work" }
        })));
    }

    #[test]
    fn strips_only_payload_refs_and_preserves_metadata() {
        let event = json!({
            "eventType": "step_completed",
            "eventData": { "stepName": "work", "result": [1, 2, 3] },
            "eventId": "evnt_1"
        });
        assert_eq!(
            strip_event_data_refs(event, ResolveData::None),
            json!({
                "eventType": "step_completed",
                "eventData": { "stepName": "work" },
                "eventId": "evnt_1"
            })
        );
    }
}
