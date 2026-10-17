//! Stage 3 — Relevance Scoring.
//!
//! Evaluates candidate species matching filter criteria and calculates a composite
//! relevance score using weighted attribute matching.

use kalimantanbio_shared::core::Species;

use crate::types::{QueryFilters, ScoringWeights};

/// Scores the text relevance of a species against query keywords.
///
/// Gives higher priority to matches found in scientific or common names
/// compared to matches occurring only within free-text descriptions.
///
/// # Arguments
///
/// * `species` - The candidate species record.
/// * `filters` - The active query filters containing keywords.
///
/// # Returns
///
/// A normalized text match score between 0.0 and 1.0.
pub(crate) fn score_text_match(species: &Species, filters: &QueryFilters) -> f64 {
    todo!("species-search phase 3: score_text_match")
}

/// Scores the taxonomic relevance of a species.
///
/// Evaluates matching precision across taxonomic levels.
///
/// # Arguments
///
/// * `species` - The candidate species record.
/// * `filters` - The active query filters containing taxonomic constraints.
///
/// # Returns
///
/// A normalized taxonomy score between 0.0 and 1.0.
pub(crate) fn score_taxonomy_match(species: &Species, filters: &QueryFilters) -> f64 {
    todo!("species-search phase 3: score_taxonomy_match")
}

/// Scores conservation status alignment between query and species attributes.
///
/// # Arguments
///
/// * `species` - The candidate species record.
/// * `filters` - The active query filters containing conservation constraints.
///
/// # Returns
///
/// A normalized conservation status score between 0.0 and 1.0.
pub(crate) fn score_conservation_match(species: &Species, filters: &QueryFilters) -> f64 {
    todo!("species-search phase 3: score_conservation_match")
}

/// Computes the overall composite relevance score for a species.
///
/// Combines text, taxonomy, and conservation match scores according to
/// the supplied [`ScoringWeights`].
///
/// # Arguments
///
/// * `species` - The candidate species record to score.
/// * `filters` - Active query filters.
/// * `weights` - Configuration determining the relative importance of each score component.
///
/// # Returns
///
/// A composite relevance score between 0.0 and 1.0.
pub(crate) fn score_species(
    species: &Species,
    filters: &QueryFilters,
    weights: &ScoringWeights,
) -> f64 {
    todo!("species-search phase 3: score_species")
}
