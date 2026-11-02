//! Public error types for the Species Relationship Explorer.

use serde::Serialize;
use thiserror::Error;

use crate::types::SpeciesId;

/// Errors reported by the exploration pipeline.
#[derive(Error, Debug, Clone, PartialEq, Eq, Serialize)]
pub enum ExplorerError {
    /// The query is invalid.
    #[error("invalid query: {0}")]
    InvalidQuery(String),
    /// The weights are invalid.
    #[error("invalid weights: weights must be non-negative, finite, and sum to 1.0")]
    InvalidWeights,
    /// The dataset is invalid.
    #[error("invalid dataset: {0}")]
    InvalidDataset(String),
    /// The center species was not found.
    #[error("species not found: {0}")]
    SpeciesNotFound(SpeciesId),
}
