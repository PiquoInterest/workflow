use std::error::Error;
use std::fmt::{self, Display, Formatter};

pub const ATTRIBUTE_KEY_MAX_LENGTH: usize = 256;
pub const ATTRIBUTE_VALUE_MAX_BYTES: usize = 256;
pub const ATTRIBUTE_MAX_PER_RUN: usize = 64;

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

#[derive(Debug, Clone, PartialEq)]
pub enum AttributeInput {
    Null,
    Array(Vec<String>),
    String(String),
    Number(f64),
    Record(Vec<AttributeField>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct NormalizeAttributeOptions {
    pub allow_reserved_attributes: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttributeChange {
    pub key: String,
    pub value: Option<String>,
}

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

/// Converts a JavaScript-style record into validated ordered attribute changes.
pub fn normalize_attribute_changes(
    input: AttributeInput,
    options: NormalizeAttributeOptions,
) -> Result<Vec<AttributeChange>, FatalError> {
    let _ = (input, options);
    panic!("TDD RED: packages/core/src/attribute-changes.test.ts implementation pending")
}
