#![forbid(unsafe_code)]

//! Rust source of truth for the Workflow deterministic runtime.
//!
//! TypeScript remains the compatibility oracle during migration. Production
//! behavior moves into this crate only after a translated test suite proves the
//! same contract, plus any explicitly documented security correction.

pub mod attribute_changes;
pub mod capabilities;
pub mod replay_payload_cache;
pub mod runtime;
