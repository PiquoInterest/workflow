use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use serde::de::Error as DeError;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

use crate::{ValidationError, ValidationResult};

/// Prefix reserved for framework-owned attributes.
pub const RESERVED_ATTRIBUTE_KEY_PREFIX: char = '$';
/// Attribute that links a run to the root of its run tree.
pub const ROOT_RUN_ID_ATTRIBUTE: &str = "$rootRunId";
/// Attribute that links a run to its direct parent.
pub const PARENT_RUN_ID_ATTRIBUTE: &str = "$parentRunId";
/// Maximum key length, measured with JavaScript UTF-16 semantics.
pub const ATTRIBUTE_KEY_MAX_LENGTH: usize = 256;
/// Maximum UTF-8 byte length for an attribute value.
pub const ATTRIBUTE_VALUE_MAX_BYTES: usize = 256;
/// Maximum materialized attributes on one run.
pub const ATTRIBUTE_MAX_PER_RUN: usize = 64;

/// A single run-attribute change. `None` removes the key.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AttributeChange {
    pub key: String,
    pub value: Option<String>,
}

impl<'de> Deserialize<'de> for AttributeChange {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RawAttributeChange {
            key: String,
            value: Value,
        }

        let raw = RawAttributeChange::deserialize(deserializer)?;
        let value = match raw.value {
            Value::Null => None,
            Value::String(value) => Some(value),
            other => {
                return Err(D::Error::custom(format!(
                    "Attribute value must be a string or null, got {}",
                    json_typeof(&other)
                )));
            }
        };

        Ok(Self {
            key: raw.key,
            value,
        })
    }
}

/// Context needed to calculate exact post-merge constraints.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AttributeValidationOptions {
    /// Existing keys make the post-merge count exact.
    pub existing_keys: Option<BTreeSet<String>>,
    /// Reserved `$` keys are available only to framework-level callers.
    pub allow_reserved_attributes: bool,
}

/// Counts a Rust string the same way JavaScript's `String.length` does.
pub fn javascript_string_length(value: &str) -> usize {
    value.encode_utf16().count()
}

fn push_json_unicode_escape(output: &mut String, unit: u16) {
    write!(output, "\\u{unit:04x}").expect("writing to a String cannot fail");
}

/// Serializes a JavaScript UTF-16 prefix with `JSON.stringify` string semantics.
///
/// Rust strings cannot contain lone surrogates, but JavaScript `slice` can
/// create one by cutting a supplementary code point in half. Iterating the
/// UTF-16 units directly preserves that diagnostic behavior exactly.
fn json_stringify_utf16_prefix(value: &str, max_units: Option<usize>) -> String {
    let mut output = String::from('"');
    let mut units = value
        .encode_utf16()
        .take(max_units.unwrap_or(usize::MAX))
        .peekable();

    while let Some(unit) = units.next() {
        match unit {
            b'"' as u16 => output.push_str("\\\""),
            b'\\' as u16 => output.push_str("\\\\"),
            0x0008 => output.push_str("\\b"),
            0x0009 => output.push_str("\\t"),
            0x000a => output.push_str("\\n"),
            0x000c => output.push_str("\\f"),
            0x000d => output.push_str("\\r"),
            0x0000..=0x001f => push_json_unicode_escape(&mut output, unit),
            0xd800..=0xdbff => {
                let Some(low) = units.peek().copied().filter(|low| (0xdc00..=0xdfff).contains(low))
                else {
                    push_json_unicode_escape(&mut output, unit);
                    continue;
                };
                units.next();
                let code_point = 0x1_0000
                    + ((u32::from(unit) - 0xd800) << 10)
                    + (u32::from(low) - 0xdc00);
                output.push(
                    char::from_u32(code_point)
                        .expect("paired UTF-16 surrogates always form a Unicode scalar value"),
                );
            }
            0xdc00..=0xdfff => push_json_unicode_escape(&mut output, unit),
            _ => output.push(
                char::from_u32(u32::from(unit))
                    .expect("non-surrogate UTF-16 units are Unicode scalar values"),
            ),
        }
    }

    output.push('"');
    output
}

/// Validates an attribute key.
pub fn validate_attribute_key(key: &str, allow_reserved_attributes: bool) -> ValidationResult<()> {
    let key_length = javascript_string_length(key);
    if key_length == 0 {
        return Err(ValidationError::new(
            "attribute_key_empty",
            "Attribute key must not be empty",
        ));
    }
    if key_length > ATTRIBUTE_KEY_MAX_LENGTH {
        let preview = json_stringify_utf16_prefix(key, Some(32));
        return Err(ValidationError::new(
            "attribute_key_too_long",
            format!(
                "Attribute key length {key_length} exceeds limit {ATTRIBUTE_KEY_MAX_LENGTH}: {preview}…"
            ),
        ));
    }
    if !allow_reserved_attributes && key.starts_with(RESERVED_ATTRIBUTE_KEY_PREFIX) {
        let encoded_key = json_stringify_utf16_prefix(key, None);
        return Err(ValidationError::new(
            "attribute_key_reserved",
            format!(
                "Attribute key {encoded_key} starts with reserved prefix \"{RESERVED_ATTRIBUTE_KEY_PREFIX}\" — that namespace is reserved for framework/library code. Set {{ allowReservedAttributes: true }} only if your caller is framework-level."
            ),
        ));
    }
    Ok(())
}

/// Validates an attribute value using its UTF-8 wire size.
pub fn validate_attribute_value(value: Option<&str>) -> ValidationResult<()> {
    let Some(value) = value else {
        return Ok(());
    };

    let byte_length = value.len();
    if byte_length > ATTRIBUTE_VALUE_MAX_BYTES {
        return Err(ValidationError::new(
            "attribute_value_too_long",
            format!(
                "Attribute value byte length {byte_length} exceeds limit {ATTRIBUTE_VALUE_MAX_BYTES}"
            ),
        ));
    }
    Ok(())
}

/// Validates constraints that apply across a batch of individually valid changes.
pub fn validate_attribute_batch_constraints(
    changes: &[AttributeChange],
    existing_keys: Option<&BTreeSet<String>>,
) -> ValidationResult<()> {
    let mut seen_keys = BTreeSet::new();
    let mut post_merge_count = existing_keys.map_or(0_i64, |keys| keys.len() as i64);

    for change in changes {
        if !seen_keys.insert(change.key.clone()) {
            let encoded_key = json_stringify_utf16_prefix(&change.key, None);
            return Err(ValidationError::new(
                "attribute_key_duplicate",
                format!("Attribute key {encoded_key} appears more than once in the same batch"),
            ));
        }

        match (&change.value, existing_keys) {
            (None, Some(keys)) if keys.contains(&change.key) => post_merge_count -= 1,
            (Some(_), None) => post_merge_count += 1,
            (Some(_), Some(keys)) if !keys.contains(&change.key) => post_merge_count += 1,
            _ => {}
        }
    }

    if post_merge_count > ATTRIBUTE_MAX_PER_RUN as i64 {
        return Err(ValidationError::new(
            "attribute_count_exceeded",
            format!(
                "Run attribute count would exceed limit {ATTRIBUTE_MAX_PER_RUN} (post-merge {post_merge_count})"
            ),
        ));
    }
    Ok(())
}

/// Validates an attribute batch exactly as the TypeScript contextual validator.
pub fn validate_attribute_changes(
    changes: &[AttributeChange],
    options: &AttributeValidationOptions,
) -> ValidationResult<()> {
    for change in changes {
        validate_attribute_key(&change.key, options.allow_reserved_attributes)?;
        validate_attribute_value(change.value.as_deref())?;
    }
    validate_attribute_batch_constraints(changes, options.existing_keys.as_ref())
}

/// Applies a batch without mutating the input map.
///
/// A map is used rather than a JavaScript-style object, so keys such as
/// `__proto__` remain ordinary data and cannot invoke prototype setters.
pub fn apply_attribute_changes(
    existing: Option<&BTreeMap<String, String>>,
    changes: &[AttributeChange],
) -> BTreeMap<String, String> {
    let mut next = existing.cloned().unwrap_or_default();
    for change in changes {
        match &change.value {
            Some(value) => {
                next.insert(change.key.clone(), value.clone());
            }
            None => {
                next.remove(&change.key);
            }
        }
    }
    next
}

fn json_typeof(value: &Value) -> &'static str {
    match value {
        Value::Null => "object",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) | Value::Object(_) => "object",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn change(key: &str, value: Option<&str>) -> AttributeChange {
        AttributeChange {
            key: key.to_owned(),
            value: value.map(ToOwned::to_owned),
        }
    }

    #[test]
    fn validates_utf8_bytes_not_characters() {
        validate_attribute_value(Some(&"💥".repeat(64))).unwrap();
        assert!(validate_attribute_value(Some(&"💥".repeat(65))).is_err());
    }

    #[test]
    fn matches_javascript_key_length() {
        assert_eq!(javascript_string_length("💥"), 2);
        assert!(validate_attribute_key(&"💥".repeat(128), true).is_ok());
        assert!(validate_attribute_key(&"💥".repeat(129), true).is_err());
    }

    #[test]
    fn matches_javascript_stringification_when_a_prefix_splits_a_surrogate_pair() {
        let key = format!("a{}", "💥".repeat(128));
        let error = validate_attribute_key(&key, true).unwrap_err();
        assert_eq!(
            error.message(),
            format!(
                "Attribute key length 257 exceeds limit 256: \"a{}\\ud83d\"…",
                "💥".repeat(15)
            )
        );
    }

    #[test]
    fn escapes_key_diagnostics_like_json_stringify() {
        assert_eq!(
            json_stringify_utf16_prefix("quote=\" slash=\\ newline=\n", None),
            "\"quote=\\\" slash=\\\\ newline=\\n\""
        );
    }

    #[test]
    fn rejects_duplicate_keys_and_exact_post_merge_overflow() {
        let duplicate = [change("phase", Some("init")), change("phase", Some("done"))];
        assert!(validate_attribute_changes(&duplicate, &Default::default()).is_err());

        let existing = (0..ATTRIBUTE_MAX_PER_RUN)
            .map(|index| format!("k{index}"))
            .collect();
        let options = AttributeValidationOptions {
            existing_keys: Some(existing),
            allow_reserved_attributes: false,
        };
        assert!(validate_attribute_changes(&[change("k0", Some("updated"))], &options).is_ok());
        assert!(validate_attribute_changes(&[change("new", Some("value"))], &options).is_err());
    }

    #[test]
    fn safely_round_trips_prototype_named_attributes() {
        let output = apply_attribute_changes(
            None,
            &[
                change("__proto__", Some("ordinary-data")),
                change("constructor", Some("also-data")),
            ],
        );
        assert_eq!(output.get("__proto__"), Some(&"ordinary-data".to_owned()));
        assert_eq!(output.get("constructor"), Some(&"also-data".to_owned()));
    }

    #[test]
    fn deserialization_requires_the_nullable_value_field() {
        assert!(serde_json::from_str::<AttributeChange>(r#"{"key":"phase"}"#).is_err());
        assert_eq!(
            serde_json::from_str::<AttributeChange>(r#"{"key":"phase","value":null}"#).unwrap(),
            change("phase", None)
        );
    }
}
