/// Global JavaScript symbol-registry key used for custom serialization.
pub const WORKFLOW_SERIALIZE: &str = "workflow-serialize";

/// Global JavaScript symbol-registry key used for custom deserialization.
pub const WORKFLOW_DESERIALIZE: &str = "workflow-deserialize";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_are_stable_protocol_values() {
        assert_eq!(WORKFLOW_SERIALIZE, "workflow-serialize");
        assert_eq!(WORKFLOW_DESERIALIZE, "workflow-deserialize");
    }
}
