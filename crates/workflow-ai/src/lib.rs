#![forbid(unsafe_code)]

use std::fmt::Write as _;

/// A side-effect-free representation of JavaScript values accepted by
/// `getErrorMessage`.
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

/// Normalizes an unknown error value using the TypeScript contract.
///
/// Error instances expose their message, strings are returned unchanged,
/// nullish values use the stable fallback, and all remaining values are
/// serialized with JavaScript-compatible JSON rules.
#[must_use]
pub fn get_error_message(value: &ErrorValue) -> String {
    match value {
        ErrorValue::Error(message)
        | ErrorValue::CustomError(message)
        | ErrorValue::String(message) => message.clone(),
        ErrorValue::Null | ErrorValue::Undefined => "unknown error".to_owned(),
        value => serialize_json(value, JsonPosition::TopLevel)
            .unwrap_or_else(|| "unknown error".to_owned()),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum JsonPosition {
    TopLevel,
    ObjectProperty,
    ArrayElement,
}

fn serialize_json(value: &ErrorValue, position: JsonPosition) -> Option<String> {
    match value {
        ErrorValue::Error(_) | ErrorValue::CustomError(_) => Some("{}".to_owned()),
        ErrorValue::String(value) => Some(quote_json_string(value)),
        ErrorValue::Object(entries) => {
            let mut serialized = String::from("{");
            let mut first = true;
            for (key, value) in entries {
                let Some(value) = serialize_json(value, JsonPosition::ObjectProperty) else {
                    continue;
                };
                if !first {
                    serialized.push(',');
                }
                first = false;
                serialized.push_str(&quote_json_string(key));
                serialized.push(':');
                serialized.push_str(&value);
            }
            serialized.push('}');
            Some(serialized)
        }
        ErrorValue::Null => Some("null".to_owned()),
        ErrorValue::Undefined => match position {
            JsonPosition::TopLevel | JsonPosition::ObjectProperty => None,
            JsonPosition::ArrayElement => Some("null".to_owned()),
        },
        ErrorValue::Number(value) => Some(value.to_string()),
        ErrorValue::Bool(value) => Some(value.to_string()),
        ErrorValue::Array(values) => {
            let mut serialized = String::from("[");
            for (index, value) in values.iter().enumerate() {
                if index != 0 {
                    serialized.push(',');
                }
                serialized.push_str(
                    &serialize_json(value, JsonPosition::ArrayElement)
                        .unwrap_or_else(|| "null".to_owned()),
                );
            }
            serialized.push(']');
            Some(serialized)
        }
    }
}

fn quote_json_string(value: &str) -> String {
    let mut quoted = String::with_capacity(value.len() + 2);
    quoted.push('"');
    for character in value.chars() {
        match character {
            '"' => quoted.push_str("\\\""),
            '\\' => quoted.push_str("\\\\"),
            '\u{0008}' => quoted.push_str("\\b"),
            '\u{000C}' => quoted.push_str("\\f"),
            '\n' => quoted.push_str("\\n"),
            '\r' => quoted.push_str("\\r"),
            '\t' => quoted.push_str("\\t"),
            character if character <= '\u{001F}' => {
                write!(&mut quoted, "\\u{:04x}", u32::from(character))
                    .expect("writing to a String cannot fail");
            }
            character => quoted.push(character),
        }
    }
    quoted.push('"');
    quoted
}

#[cfg(test)]
mod tests {
    use super::{ErrorValue, get_error_message};

    #[test]
    fn object_undefined_properties_are_omitted_and_array_values_become_null() {
        let value = ErrorValue::Object(vec![
            ("omit".to_owned(), ErrorValue::Undefined),
            (
                "keep".to_owned(),
                ErrorValue::Array(vec![ErrorValue::Undefined, ErrorValue::Null]),
            ),
        ]);
        assert_eq!(get_error_message(&value), r#"{"keep":[null,null]}"#);
    }

    #[test]
    fn strings_and_object_keys_are_json_escaped() {
        let value = ErrorValue::Object(vec![(
            "line\n\"key".to_owned(),
            ErrorValue::String("tab\tbackslash\\nul\0".to_owned()),
        )]);
        assert_eq!(
            get_error_message(&value),
            r#"{"line\n\"key":"tab\tbackslash\\nul\u0000"}"#
        );
    }
}
