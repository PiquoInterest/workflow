use workflow_world::event_metadata::{
    EntityEventClass, entity_event_class, event_data_payload_field, event_data_ref_fields,
    is_sealed_noop_event,
};

#[test]
fn classifies_mutually_exclusive_entity_events() {
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
fn singular_and_plural_payload_field_lookups_share_one_mapping() {
    assert_eq!(event_data_ref_fields("run_created"), &["input"]);
    assert_eq!(event_data_payload_field("run_created"), Some("input"));
    assert_eq!(event_data_ref_fields("step_completed"), &["result"]);
    assert_eq!(event_data_payload_field("step_completed"), Some("result"));
}

#[test]
fn unknown_and_inherited_property_names_have_no_metadata() {
    assert!(event_data_ref_fields("unknown").is_empty());
    assert_eq!(event_data_payload_field("unknown"), None);
    assert!(event_data_ref_fields("constructor").is_empty());
    assert_eq!(entity_event_class("toString"), None);
}

#[test]
fn sealed_log_filler_events_are_identified_without_loading_event_schemas() {
    assert!(is_sealed_noop_event("noop"));
    assert!(!is_sealed_noop_event("run_started"));
}
