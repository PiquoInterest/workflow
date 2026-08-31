use serde::{Deserialize, Serialize};

/// Event classes that a replay records once per entity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntityEventClass {
    AttrSet,
    StepCreated,
    StepStarted,
    StepRetrying,
    StepTerminal,
    WaitCreated,
    WaitCompleted,
    HookCreated,
    HookDisposed,
    RunStarted,
}

impl EntityEventClass {
    /// Canonical wire name used by the TypeScript implementation.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::AttrSet => "attr_set",
            Self::StepCreated => "step_created",
            Self::StepStarted => "step_started",
            Self::StepRetrying => "step_retrying",
            Self::StepTerminal => "step_terminal",
            Self::WaitCreated => "wait_created",
            Self::WaitCompleted => "wait_completed",
            Self::HookCreated => "hook_created",
            Self::HookDisposed => "hook_disposed",
            Self::RunStarted => "run_started",
        }
    }

    /// Whether consuming an event of this class closes its entity.
    pub const fn is_terminal(self) -> bool {
        matches!(
            self,
            Self::AttrSet | Self::StepTerminal | Self::WaitCompleted | Self::HookDisposed
        )
    }
}

/// Entity key used for classes tracked once per run.
pub const RUN_ENTITY_KEY: &str = "";

/// The class and entity under which replay tracks one event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EntityEventClassification {
    pub event_class: EntityEventClass,
    pub entity: String,
}

/// Whether an event is a sealed-log filler occupying an abandoned slot.
pub fn is_sealed_noop_event(event_type: &str) -> bool {
    event_type == "noop"
}

/// Maps an event type to its replay entity class.
pub fn entity_event_class(event_type: &str) -> Option<EntityEventClass> {
    match event_type {
        "attr_set" => Some(EntityEventClass::AttrSet),
        "step_created" => Some(EntityEventClass::StepCreated),
        "step_started" => Some(EntityEventClass::StepStarted),
        "step_retrying" => Some(EntityEventClass::StepRetrying),
        "step_completed" | "step_failed" => Some(EntityEventClass::StepTerminal),
        "wait_created" => Some(EntityEventClass::WaitCreated),
        "wait_completed" => Some(EntityEventClass::WaitCompleted),
        "hook_created" => Some(EntityEventClass::HookCreated),
        "hook_disposed" => Some(EntityEventClass::HookDisposed),
        "run_started" => Some(EntityEventClass::RunStarted),
        _ => None,
    }
}

/// Classifies an event, excluding events that have no stable entity key.
pub fn classify_entity_event(
    event_type: &str,
    correlation_id: Option<&str>,
) -> Option<EntityEventClassification> {
    let event_class = entity_event_class(event_type)?;
    if event_class == EntityEventClass::RunStarted {
        return Some(EntityEventClassification {
            event_class,
            entity: RUN_ENTITY_KEY.to_owned(),
        });
    }

    correlation_id
        .filter(|value| !value.is_empty())
        .map(|entity| EntityEventClassification {
            event_class,
            entity: entity.to_owned(),
        })
}

/// Opaque payload fields removed when referenced data is not resolved.
pub fn event_data_ref_fields(event_type: &str) -> &'static [&'static str] {
    match event_type {
        "run_created" | "run_started" => &["input"],
        "run_completed" => &["output"],
        "run_failed" => &["error"],
        "step_created" | "step_started" => &["input"],
        "step_completed" => &["result"],
        "step_failed" | "step_retrying" => &["error"],
        "hook_created" => &["metadata"],
        "hook_received" => &["payload"],
        _ => &[],
    }
}

/// First opaque payload field, when the event type has one.
pub fn event_data_payload_field(event_type: &str) -> Option<&'static str> {
    event_data_ref_fields(event_type).first().copied()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_mutually_exclusive_step_outcomes() {
        assert_eq!(
            entity_event_class("step_completed"),
            Some(EntityEventClass::StepTerminal)
        );
        assert_eq!(
            entity_event_class("step_failed"),
            Some(EntityEventClass::StepTerminal)
        );
        assert_eq!(entity_event_class("run_completed"), None);
    }

    #[test]
    fn unknown_and_prototype_property_names_have_no_metadata() {
        assert!(event_data_ref_fields("unknown").is_empty());
        assert!(event_data_ref_fields("constructor").is_empty());
        assert_eq!(entity_event_class("toString"), None);
    }

    #[test]
    fn step_written_attributes_do_not_collapse_into_a_run_class() {
        assert_eq!(classify_entity_event("attr_set", None), None);
        assert_eq!(
            classify_entity_event("attr_set", Some("attr_A")),
            Some(EntityEventClassification {
                event_class: EntityEventClass::AttrSet,
                entity: "attr_A".to_owned(),
            })
        );
    }
}
