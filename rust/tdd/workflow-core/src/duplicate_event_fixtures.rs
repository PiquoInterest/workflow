#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DuplicateEventSpec {
    pub event_type: String,
    pub entity: Option<String>,
}

impl DuplicateEventSpec {
    pub fn new(event_type: &str, entity: Option<&str>) -> Self {
        Self {
            event_type: event_type.to_owned(),
            entity: entity.map(str::to_owned),
        }
    }
}

/// Returns the log indices a replay must classify as unclaimed duplicates.
pub fn ignored_duplicate_indices(events: &[DuplicateEventSpec]) -> Vec<usize> {
    let _ = events;
    panic!("TDD RED: packages/core/src/duplicate-event-fixtures.test.ts implementation pending")
}
