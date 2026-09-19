#![warn(missing_docs)]
//! Module 4 — Comparative Species Explorer.
//!
//! Enables multi-species comparison: shared/unique attribute analysis,
//! similarity scoring, distinguishing-characteristic identification, and
//! comparative summaries.
//!
//! Public API: `compare_species` and `calculate_pair_similarity`. Internal
//! helpers stay private.
//!
//! > **Status: interface scaffold.** Bodies are stubs pending implementation.

pub mod types;

use std::collections::HashSet;

use kalimantanbio_shared::core::{Observation, Species};

use crate::types::{ComparisonQuery, ComparisonResult, ModuleError, ScoringWeights,
    SimilarityScore};

/// Compute the similarity between two species across all dimensions.
///
/// # Arguments
///
/// * `a` - First species.
/// * `b` - Second species.
/// * `kabupaten_a` - Regent regions where the first species is recorded.
/// * `kabupaten_b` - Regent regions where the second species is recorded.
/// * `weights` - Scoring weights per dimension.
///
/// # Returns
///
/// The per-dimension and combined similarity scores.
///
/// # Example
///
/// ```rust,ignore
/// let score = calculate_pair_similarity(&a, &b, &kabupaten_a, &kabupaten_b, &weights);
/// ```
pub fn calculate_pair_similarity(
    a: &Species,
    b: &Species,
    kabupaten_a: &HashSet<u64>,
    kabupaten_b: &HashSet<u64>,
    weights: &ScoringWeights,
) -> SimilarityScore {
    todo!("species-comparison phase 5: calculate_pair_similarity")
}

/// Compare multiple species and produce the full comparison report.
///
/// Pipeline: retrieve → validate → build attribute matrix → analyze shared /
/// unique attributes → score similarity → summarize.
///
/// # Arguments
///
/// * `query` - A validated comparison query (list of species ids).
/// * `species` - The species dataset.
/// * `observations` - The observation dataset for distribution comparison.
///
/// # Returns
///
/// The comparison report, or a module error.
///
/// # Example
///
/// ```rust,ignore
/// let report = compare_species(&query, &species, &observations)?;
/// ```
pub fn compare_species(
    query: &ComparisonQuery,
    species: &[Species],
    observations: &[Observation],
) -> Result<ComparisonResult, ModuleError> {
    todo!("species-comparison phase 5: compare_species")
}