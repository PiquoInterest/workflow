#![forbid(unsafe_code)]

//! Rust source of truth for the Workflow deterministic runtime.
//!
//! TypeScript remains the compatibility oracle during migration. A module is
//! promoted into this crate only after its translated Rust tests have first
//! been observed failing in the expected-RED lane.

pub mod capabilities;
pub mod runtime;
