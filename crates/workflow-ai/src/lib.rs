#![forbid(unsafe_code)]

use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Weak};

const MAX_SERIALIZATION_DEPTH: usize = 64;

/// A callable-shaped diagnostic value used to prove that normalization never
/// executes object-controlled hooks such as JavaScript's `toJSON`.
#[derive(Debug, Clone)]
pub struct CallableProbe {
    name: String,
    calls: Arc<AtomicUsize>,
}

impl CallableProbe {
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            calls: Arc::new(AtomicUsize::new(0)),
        }
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub fn call_count(&self) -> usize {
        self.calls.load(Ordering::SeqCst)
    }

    pub fn invoke(&self) {
        let _ = self.calls.fetch_add(1, Ordering::SeqCst);
    }
}

impl PartialEq for CallableProbe {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && Arc::ptr_eq(&self.calls, &other.calls)
    }
}

impl Eq for CallableProbe {}

/// A shareable object node used for object graphs that can contain cycles.
#[derive(Debug)]
pub struct SharedErrorObject {
    entries: Vec<(String, ErrorValue)>,
}

impl SharedErrorObject {
    #[must_use]
    pub fn new(entries: Vec<(String, ErrorValue)>) -> Self {
        Self { entries }
    }

    #[must_use]
    pub fn entries(&self) -> &[(String, ErrorValue)] {
        &self.entries
    }
}

/// A side-effect-free representation of JavaScript values accepted by
/// `getErrorMessage`.
#[derive(Debug, Clone)]
pub enum ErrorValue {
    Error(String),
    CustomError(String),
    String(String),
    Object(Vec<(String, ErrorValue)>),
    SharedObject(Arc<SharedErrorObject>),
    ObjectReference(Weak<SharedErrorObject>),
    Callable(CallableProbe),
    BigInt(String),
    Null,
    Undefined,
    Number(i64),
    Bool(bool),
    Array(Vec<ErrorValue>),
}

impl PartialEq for ErrorValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Error(left), Self::Error(right))
            | (Self::CustomError(left), Self::CustomError(right))
            | (Self::String(left), Self::String(right))
            | (Self::BigInt(left), Self::BigInt(right)) => left == right,
            (Self::Object(left), Self::Object(right)) => left == right,
            (Self::SharedObject(left), Self::SharedObject(right)) => Arc::ptr_eq(left, right),
            (Self::ObjectReference(left), Self::ObjectReference(right)) => left.ptr_eq(right),
            (Self::Callable(left), Self::Callable(right)) => left == right,
            (Self::Null, Self::Null) | (Self::Undefined, Self::Undefined) => true,
            (Self::Number(left), Self::Number(right)) => left == right,
            (Self::Bool(left), Self::Bool(right)) => left == right,
            (Self::Array(left), Self::Array(right)) => left == right,
            _ => false,
        }
    }
}

impl Eq for ErrorValue {}

/// Normalizes an unknown error value using the TypeScript compatibility
/// contract while intentionally hardening unsafe JSON serialization edges.
///
/// Error instances expose their message, strings are returned unchanged,
/// nullish values use the stable fallback, and all remaining values are
/// serialized with JavaScript-compatible JSON rules. Callable object fields
/// are represented as inert data, cycles are labeled, and BigInts receive a
/// stable diagnostic representation instead of executing or throwing.
#[must_use]
pub fn get_error_message(value: &ErrorValue) -> String {
    match value {
        ErrorValue::Error(message)
        | ErrorValue::CustomError(message)
        | ErrorValue::String(message) => message.clone(),
        ErrorValue::Null | ErrorValue::Undefined => "unknown error".to_owned(),
        value => {
            let mut state = SerializationState::default();
            serialize_json(value, JsonPosition::TopLevel, &mut state, 0)
                .unwrap_or_else(|| "unknown error".to_owned())
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum JsonPosition {
    TopLevel,
    ObjectProperty,
    ArrayElement,
}

#[derive(Debug, Default)]
struct SerializationState {
    active_shared_objects: BTreeSet<usize>,
}

fn serialize_json(
    value: &ErrorValue,
    position: JsonPosition,
    state: &mut SerializationState,
    depth: usize,
) -> Option<String> {
    match value {
        ErrorValue::Error(_) | ErrorValue::CustomError(_) => Some("{}".to_owned()),
        ErrorValue::String(value) => Some(quote_json_string(value)),
        ErrorValue::Object(entries) => serialize_object(entries, position, state, depth),
        ErrorValue::SharedObject(object) => serialize_shared_object(object, position, state, depth),
        ErrorValue::ObjectReference(reference) => match reference.upgrade() {
            Some(object) => serialize_shared_object(&object, position, state, depth),
            None => Some(render_diagnostic("[Released reference]", position)),
        },
        ErrorValue::Callable(callable) => Some(render_diagnostic(
            &format!("[Function {}]", escape_inline_diagnostic(callable.name())),
            position,
        )),
        ErrorValue::BigInt(value) => Some(render_diagnostic(
            &canonical_bigint(value).unwrap_or_else(|| "[Invalid BigInt]".to_owned()),
            position,
        )),
        ErrorValue::Null => Some("null".to_owned()),
        ErrorValue::Undefined => match position {
            JsonPosition::TopLevel | JsonPosition::ObjectProperty => None,
            JsonPosition::ArrayElement => Some("null".to_owned()),
        },
        ErrorValue::Number(value) => Some(value.to_string()),
        ErrorValue::Bool(value) => Some(value.to_string()),
        ErrorValue::Array(values) => serialize_array(values, position, state, depth),
    }
}

fn serialize_object(
    entries: &[(String, ErrorValue)],
    position: JsonPosition,
    state: &mut SerializationState,
    depth: usize,
) -> Option<String> {
    if depth >= MAX_SERIALIZATION_DEPTH {
        return Some(render_diagnostic("[Max depth exceeded]", position));
    }

    let mut serialized = String::from("{");
    let mut first = true;
    for (key, value) in entries {
        let Some(value) = serialize_json(value, JsonPosition::ObjectProperty, state, depth + 1)
        else {
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

fn serialize_shared_object(
    object: &Arc<SharedErrorObject>,
    position: JsonPosition,
    state: &mut SerializationState,
    depth: usize,
) -> Option<String> {
    if depth >= MAX_SERIALIZATION_DEPTH {
        return Some(render_diagnostic("[Max depth exceeded]", position));
    }

    let identity = Arc::as_ptr(object) as usize;
    if !state.active_shared_objects.insert(identity) {
        return Some(render_diagnostic("[Circular]", position));
    }

    let serialized = serialize_object(object.entries(), position, state, depth);
    state.active_shared_objects.remove(&identity);
    serialized
}

fn serialize_array(
    values: &[ErrorValue],
    position: JsonPosition,
    state: &mut SerializationState,
    depth: usize,
) -> Option<String> {
    if depth >= MAX_SERIALIZATION_DEPTH {
        return Some(render_diagnostic("[Max depth exceeded]", position));
    }

    let mut serialized = String::from("[");
    for (index, value) in values.iter().enumerate() {
        if index != 0 {
            serialized.push(',');
        }
        serialized.push_str(
            &serialize_json(value, JsonPosition::ArrayElement, state, depth + 1)
                .unwrap_or_else(|| "null".to_owned()),
        );
    }
    serialized.push(']');
    Some(serialized)
}

fn render_diagnostic(value: &str, position: JsonPosition) -> String {
    match position {
        JsonPosition::TopLevel => value.to_owned(),
        JsonPosition::ObjectProperty | JsonPosition::ArrayElement => quote_json_string(value),
    }
}

fn canonical_bigint(value: &str) -> Option<String> {
    let (negative, digits) = match value.strip_prefix('-') {
        Some(digits) => (true, digits),
        None => (false, value),
    };
    if digits.is_empty() || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }

    let digits = digits.trim_start_matches('0');
    let digits = if digits.is_empty() { "0" } else { digits };
    let sign = if negative && digits != "0" { "-" } else { "" };
    Some(format!("{sign}{digits}n"))
}

fn escape_inline_diagnostic(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\u{0008}' => escaped.push_str("\\b"),
            '\u{000C}' => escaped.push_str("\\f"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            character if character <= '\u{001F}' => {
                write!(&mut escaped, "\\u{:04x}", u32::from(character))
                    .expect("writing to a String cannot fail");
            }
            character => escaped.push(character),
        }
    }
    escaped
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

    #[test]
    fn bigint_literals_are_canonicalized() {
        assert_eq!(
            get_error_message(&ErrorValue::BigInt("-00042".to_owned())),
            "-42n"
        );
        assert_eq!(
            get_error_message(&ErrorValue::BigInt("-000".to_owned())),
            "0n"
        );
        assert_eq!(
            get_error_message(&ErrorValue::BigInt("1;drop".to_owned())),
            "[Invalid BigInt]"
        );
    }
}
