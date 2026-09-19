#![warn(missing_docs)]
//! Module 2 — Species Relationship Explorer.
//!
//! Discovers and explains relationships between species: related-species
//! discovery, relationship scoring, explanations, and network data generation.
//!
//! Public API: `explore_relationships` and `calculate_relationship`. Internal
//! helpers (evidence extraction, scoring, explanation, network building) stay
//! private.
//!
//! > **Status: interface scaffold.** Bodies are stubs pending implementation.

pub mod types;

use std::collections::HashSet;

use kalimantanbio_shared::{core::Species, error::ValidationError};

use crate::types::{ExplorerError, ExplorerResult, RelationshipQuery, RelationshipScore,
    ScoreWeights};

pub use crate::types::SpeciesId;

/// Calculate the relationship score between two species.
///
/// Combines taxonomy, habitat, and characteristic evidence into a single score
/// using validated weights.
///
/// # Arguments
///
/// * `a` - First species.
/// * `b` - Second species.
/// * `habitats_a` - Habitat labels of the first species.
/// * `habitats_b` - Habitat labels of the second species.
/// * `weights` - Weighted contribution of each evidence basis.
///
/// # Returns
///
/// The relationship score, or a validation error when weights are invalid.
///
/// # Example
///
/// ```rust,ignore
/// let score = calculate_relationship(&a, &b, &habitats_a, &habitats_b, &weights)?;
/// ```
pub fn calculate_relationship(
    a: &Species,
    b: &Species,
    habitats_a: &HashSet<String>,
    habitats_b: &HashSet<String>,
    weights: &ScoreWeights,
) -> Result<RelationshipScore, ValidationError> {
    todo!("species-relationships phase 5: calculate_relationship")
}

/// Explore all relationships around a center species.
///
/// Pipeline: prepare → validate → discover candidates → score → explain →
/// rank → build network → assemble result.
///
/// # Arguments
///
/// * `input` - The full species dataset.
/// * `query` - Center species, minimum score, and limit.
/// * `weights` - Scoring weights.
///
/// # Returns
///
/// Related species with scores, explanations, and network data.
///
/// # Example
///
/// ```rust,ignore
/// let result = explore_relationships(&species, &query, &weights)?;
/// ```
pub fn explore_relationships(
    input: &[Species],
    query: &RelationshipQuery,
    weights: &ScoreWeights,
) -> Result<ExplorerResult, ExplorerError> {
    todo!("species-relationships phase 5: explore_relationships")
}