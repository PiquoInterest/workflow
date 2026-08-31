use std::error::Error;
use std::fmt::{self, Display, Formatter};

/// A stable validation failure returned at the Rust World boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    code: String,
    message: String,
}

impl ValidationError {
    /// Creates a validation error with a machine-readable code.
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }

    /// Machine-readable error code.
    pub fn code(&self) -> &str {
        &self.code
    }

    /// Human-readable error message.
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl Display for ValidationError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl Error for ValidationError {}

/// Result type used by World contract validation.
pub type ValidationResult<T> = Result<T, ValidationError>;
