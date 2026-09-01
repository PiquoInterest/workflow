use std::error::Error;
use std::fmt::{self, Display, Formatter};

pub use workflow_world::attributes::{
    ATTRIBUTE_KEY_MAX_LENGTH, ATTRIBUTE_MAX_PER_RUN, ATTRIBUTE_VALUE_MAX_BYTES, AttributeChange,
};
use workflow_world::attributes::{AttributeValidationOptions, validate_attribute_changes};

/// One enumerable entry from the JavaScript attribute record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttributeField {
    pub key: String,
    pub value: Option<String>,
}

impl AttributeField {
    pub fn new(key: impl Into<String>, value: Option<impl Into<String>>) -> Self {
        Self {
            key: key.into(),
            value: value.map(Into::into),
        }
    }
}

/// Runtime values accepted at the JavaScript compatibility boundary.
#[derive(Debug, Clone, PartialEq)]
pub enum AttributeInput {
    Null,
    Array(Vec<String>),
    String(String),
    Number(f64),
    Record(Vec<AttributeField>),
}

/// Framework-level callers may explicitly opt in to the reserved namespace.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct NormalizeAttributeOptions {
    pub allow_reserved_attributes: bool,
}

/// Fatal compatibility error raised before an invalid attribute event is created.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FatalError {
    pub message: String,
}

impl Display for FatalError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for FatalError {}

fn plain_object_error(actual_type: &str) -> FatalError {
    FatalError {
        message: format!("setAttributes requires a plain object, got {actual_type}"),
    }
}

/// Converts a JavaScript-style record into validated ordered attribute changes.
///
/// Validation is deliberately delegated to the Rust World contract so key
/// lengths use JavaScript UTF-16 semantics, values use UTF-8 wire bytes, and
/// the per-run and reserved-namespace rules cannot drift between layers.
pub fn normalize_attribute_changes(
    input: AttributeInput,
    options: NormalizeAttributeOptions,
) -> Result<Vec<AttributeChange>, FatalError> {
    let fields = match input {
        AttributeInput::Record(fields) => fields,
        AttributeInput::Null => return Err(plain_object_error("null")),
        AttributeInput::Array(_) => return Err(plain_object_error("array")),
        AttributeInput::String(_) => return Err(plain_object_error("string")),
        AttributeInput::Number(_) => return Err(plain_object_error("number")),
    };

    let changes = fields
        .into_iter()
        .map(|field| AttributeChange {
            key: field.key,
            value: field.value,
        })
        .collect::<Vec<_>>();

    if changes.is_empty() {
        return Ok(changes);
    }

    validate_attribute_changes(
        &changes,
        &AttributeValidationOptions {
            existing_keys: None,
            allow_reserved_attributes: options.allow_reserved_attributes,
        },
    )
    .map_err(|error| FatalError {
        message: error.message().to_owned(),
    })?;

    Ok(changes)
}
