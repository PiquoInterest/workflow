use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{ValidationError, ValidationResult};

/// Serialized workflow data at the Rust boundary.
///
/// Modern event-sourced runs transport opaque binary devalue bytes. Legacy
/// spec-version 1 runs may still contain arbitrary JSON values.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SerializedData {
    /// Opaque binary devalue payload used by spec version 2 and newer.
    Binary { bytes: Vec<u8> },
    /// Legacy JSON payload used by spec version 1 and unstamped legacy runs.
    Legacy { value: Value },
}

impl SerializedData {
    /// Returns the binary bytes when this is a modern payload.
    pub fn as_binary(&self) -> Option<&[u8]> {
        match self {
            Self::Binary { bytes } => Some(bytes),
            Self::Legacy { .. } => None,
        }
    }
}

/// Validates payload representation against the run's protocol version.
///
/// The TypeScript compatibility schema currently unions `Uint8Array` with
/// `z.any()`, which makes the binary branch non-enforcing. Rust keeps legacy
/// reads but does not let modern writes silently downgrade to arbitrary JSON.
pub fn validate_serialized_data_for_spec(
    spec_version: Option<u32>,
    data: &SerializedData,
) -> ValidationResult<()> {
    let legacy = spec_version.is_none_or(|version| version <= 1);
    if legacy || matches!(data, SerializedData::Binary { .. }) {
        return Ok(());
    }

    Err(ValidationError::new(
        "modern_payload_must_be_binary",
        format!(
            "Serialized data for spec version {} must be binary",
            spec_version.unwrap_or_default()
        ),
    ))
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn legacy_runs_accept_json_or_binary() {
        let legacy = SerializedData::Legacy {
            value: json!({ "old": true }),
        };
        let binary = SerializedData::Binary {
            bytes: vec![1, 2, 3],
        };

        assert!(validate_serialized_data_for_spec(None, &legacy).is_ok());
        assert!(validate_serialized_data_for_spec(Some(1), &legacy).is_ok());
        assert!(validate_serialized_data_for_spec(Some(1), &binary).is_ok());
    }

    #[test]
    fn modern_runs_require_binary_payloads() {
        let legacy = SerializedData::Legacy {
            value: json!("mangled-to-json"),
        };
        let binary = SerializedData::Binary {
            bytes: vec![1, 2, 3],
        };

        assert!(validate_serialized_data_for_spec(Some(2), &binary).is_ok());
        let error = validate_serialized_data_for_spec(Some(2), &legacy).unwrap_err();
        assert_eq!(error.code(), "modern_payload_must_be_binary");
    }
}
