#![warn(missing_docs)]
//! Shared library for the KalimantanBio workspace.
//!
//! Provides the standardized domain types (`Species`, `Taxonomy`, `Observation`),
//! common utility functions (taxonomy, collections, scoring, text, validation,
//! stats), a database access layer, and shared error types.
//!
//! All modules depend on this crate. Module crates depend only on this crate.
//!
//! > **Status: interface scaffold.** Every `pub fn` is declared with rustdoc;
//! > bodies are stubs (`todo!`) pending implementation. See `.github/AGENTS.md`.

pub mod collections;
pub mod core;
pub mod db;
pub mod error;
pub mod scoring;
pub mod stats;
pub mod taxonomy;
pub mod text;
pub mod validation;

pub use error::{DatabaseError, ValidationError};