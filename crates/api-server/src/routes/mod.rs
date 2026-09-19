//! Route handlers for the unified Axum API server.
//!
//! Each handler composes the public `pub fn` interface of a module crate.

pub mod comparison;
pub mod knowledge;
pub mod relationship;
pub mod search;
pub mod taxonomy;