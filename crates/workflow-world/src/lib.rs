#![forbid(unsafe_code)]

//! Rust source of truth for the Workflow World contract.
//!
//! The TypeScript packages remain compatibility oracles during migration.
//! Every module in this crate is either covered by a direct port of the
//! existing tests or by a differential test that executes the TypeScript and
//! Rust implementations with the same input.

pub mod attributes;
pub mod env;
pub mod error;
pub mod errors;
pub mod event_metadata;
pub mod events;
pub mod hooks;
pub mod queue;
pub mod runs;
pub mod serde_symbols;
pub mod serialization;
pub mod shared;
pub mod slot_identity;
pub mod spec_version;
pub mod time;
pub mod ulid;

pub use error::{ValidationError, ValidationResult};
