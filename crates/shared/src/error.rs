//! Shared error types.

use thiserror::Error;

/// Validation errors reported by shared validation helpers.
#[derive(Error, Debug)]
pub enum ValidationError {
    /// Too few items were provided.
    #[error("Too few items: found {found}, minimum {minimum}")]
    TooFewItems {
        /// Number of items provided.
        found: usize,
        /// Minimum number of items allowed.
        minimum: usize,
    },
    /// Too many items were provided.
    #[error("Too many items: found {found}, maximum {maximum}")]
    TooManyItems {
        /// Number of items provided.
        found: usize,
        /// Maximum number of items allowed.
        maximum: usize,
    },
    /// Duplicate identifiers were provided.
    #[error("Duplicate IDs found")]
    DuplicateIds,
    /// A score was not finite.
    #[error("Non-finite score")]
    NonFiniteScore,
    /// A score is outside the allowed range.
    #[error("Score {score} out of range [{min}, {max}]")]
    ScoreOutOfRange {
        /// The offending score.
        score: f64,
        /// Minimum allowed score.
        min: f64,
        /// Maximum allowed score.
        max: f64,
    },
    /// A weight was not finite.
    #[error("Non-finite weight")]
    NonFiniteWeight,
    /// A weight was negative.
    #[error("Negative weight")]
    NegativeWeight,
    /// Weights did not sum to the expected value.
    #[error("Weights sum to {sum}, expected {expected}")]
    WeightsSumMismatch {
        /// Actual sum of weights.
        sum: f64,
        /// Expected sum of weights.
        expected: f64,
    },
}

/// Database errors surfaced by the shared database layer.
#[derive(Error, Debug)]
pub enum DatabaseError {
    /// An underlying SQLx error.
    #[error("Database error: {0}")]
    SqlxError(#[from] sqlx::Error),
    /// A species with the given identifier was not found.
    #[error("Species not found: {0}")]
    SpeciesNotFound(u64),
}