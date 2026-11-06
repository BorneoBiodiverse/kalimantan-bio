#![warn(missing_docs)]
//! Module 4 — Comparative Species Explorer.
//!
//! Enables multi-species comparison: shared/unique attribute analysis,
//! similarity scoring, distinguishing-characteristic identification, and
//! comparative summaries.
//!
//! Public API: `compare_species` and `calculate_pair_similarity`. Internal
//! helpers stay private.

pub(crate) mod attributes;
pub(crate) mod matrix;
pub(crate) mod scoring;
pub(crate) mod summary;
pub mod types;
pub use crate::types::{
    ComparisonQuery, ComparisonResult, ModuleError, ScoringWeights, SimilarityScore,
    SpeciesAttributeRow,
};
pub(crate) mod validation;

use std::collections::HashSet;

use kalimantanbio_shared::core::{Observation, Species};

pub(crate) use crate::validation::{
    build_observations_map, fetch_observations_for_species, fetch_species_by_ids, validate_query,
    MAX_SPECIES, MIN_SPECIES,
};

/// Compute the similarity between two species across all dimensions.
///
/// Combines taxonomy, conservation status, species type, and distribution
/// scores according to the supplied weights.
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
/// let score = calculate_pair_similarity(&a, &b, &reg_a, &reg_b, &weights);
/// ```
pub fn calculate_pair_similarity(
    a: &Species,
    b: &Species,
    kabupaten_a: &HashSet<u64>,
    kabupaten_b: &HashSet<u64>,
    weights: &ScoringWeights,
) -> SimilarityScore {
    crate::scoring::calculate_similarity_score(a, b, kabupaten_a, kabupaten_b, weights)
}

/// Run a multi-species comparison query.
///
/// Validates the query, builds the attribute matrix, determines shared and
/// unique attributes, computes pairwise similarities, and produces a summary.
///
/// # Arguments
///
/// * `query` - The comparison query containing 2 to 10 species IDs.
/// * `species` - The full species dataset.
/// * `observations` - The observation dataset.
///
/// # Returns
///
/// A complete [`ComparisonResult`] or a [`ModuleError`] on validation failure.
///
/// # Example
///
/// ```rust,ignore
/// let result = compare_species(&query, &species, &observations)?;
/// ```
/// Run a multi-species comparison query.
///
/// Validates the query, builds the attribute matrix, determines shared and
/// unique attributes, computes pairwise similarities, and produces a summary.
///
/// # Arguments
///
/// * `query` - The comparison query containing 2 to 10 species IDs.
/// * `species` - The full species dataset.
/// * `observations` - The observation dataset.
///
/// # Returns
///
/// A complete [`ComparisonResult`] or a [`ModuleError`] on validation failure.
///
/// # Example
///
/// ```rust,ignore
/// let result = compare_species(&query, &species, &observations)?;
/// ```
pub fn compare_species(
    query: &ComparisonQuery,
    species: &[Species],
    observations: &[Observation],
) -> Result<ComparisonResult, ModuleError> {
    // 1. Validate the query (2-10 distinct species IDs).
    validate_query(query)?;

    // 2. Resolve the requested species IDs against the dataset, preserving order.
    let selected = fetch_species_by_ids(&query.species_ids, species)?;

    // 3. Build the regency-set map keyed by species ID.
    let obs_map = build_observations_map(&selected, observations);

    // 4. Build the attribute matrix in query order.
    let matrix = crate::matrix::build_attribute_matrix(&selected, observations);

    // 5. Shared / unique / distinguishing attributes.
    let shared = crate::attributes::find_shared_attributes(&matrix);
    let unique = crate::attributes::find_unique_attributes(&matrix);
    let distinguishing = crate::attributes::find_distinguishing_characteristics(&matrix);

    // 6. Pairwise similarity scores.
    let weights = ScoringWeights::default();
    let scores = crate::scoring::calculate_all_pairs(&selected, &obs_map, &weights);

    // 7. Human-readable summary.
    let summary =
        crate::summary::build_summary(&matrix, &shared, &unique, &distinguishing, &scores);

    Ok(ComparisonResult {
        query: query.clone(),
        attribute_matrix: matrix,
        shared_attributes: shared,
        unique_attributes: unique,
        distinguishing_characteristics: distinguishing,
        similarity_scores: scores,
        summary,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::matrix::build_attribute_row;

    fn fixture_species() -> Vec<Species> {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../shared/fixtures/species.json"
        );
        let raw = std::fs::read_to_string(path).expect("species fixture must be readable");
        serde_json::from_str(&raw).expect("species fixture must deserialize")
    }

    fn fixture_observations() -> Vec<Observation> {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../shared/fixtures/observations.json"
        );
        let raw = std::fs::read_to_string(path).expect("observations fixture must be readable");
        serde_json::from_str(&raw).expect("observations fixture must deserialize")
    }

    #[test]
    fn calculate_pair_similarity_delegates_to_scoring() {
        let species = fixture_species();
        let a = &species[0];
        let b = &species[1];
        let weights = ScoringWeights::default();
        let kab_a = HashSet::from([6201, 6202]);
        let kab_b = HashSet::from([6201]);

        let score = calculate_pair_similarity(a, b, &kab_a, &kab_b, &weights);
        assert_eq!(score.species_a_id, a.id);
        assert_eq!(score.species_b_id, b.id);
        assert!(score.total_score >= 0.0 && score.total_score <= 1.0);
    }

    #[test]
    fn the_pongo_pair_shares_exactly_one_regency() {
        let species = fixture_species();
        let observations = fixture_observations();
        let selected = fetch_species_by_ids(&[1, 3], &species).expect("both ids exist");

        let map = build_observations_map(&selected, &observations);
        let overlap: HashSet<u64> = map[&1].intersection(&map[&3]).copied().collect();

        assert_eq!(
            overlap,
            HashSet::from([6201]),
            "the Jaccard dimension needs a partial overlap, not an identical set"
        );
    }

    #[test]
    fn attribute_row_uses_shared_taxonomy_path_for_pongo() {
        let species = fixture_species();
        let observations = fixture_observations();

        let pongo = species
            .iter()
            .find(|s| s.id == 1)
            .expect("species 1 (Pongo pygmaeus) must exist");

        let pongo_observations = fetch_observations_for_species(pongo.id, &observations);
        let row = build_attribute_row(pongo, &pongo_observations);

        assert_eq!(
            row.taxonomy_path,
            "Animalia > Chordata > Mammalia > Primates > Hominidae > Pongo"
        );
    }

    #[test]
    fn compare_species_full_pipeline_on_fixture_pair() {
        let species = fixture_species();
        let observations = fixture_observations();

        let query = ComparisonQuery {
            species_ids: vec![1, 3],
        };
        let result = compare_species(&query, &species, &observations).expect("comparison succeeds");

        assert_eq!(result.query.species_ids, vec![1, 3]);
        assert_eq!(result.attribute_matrix.len(), 2);
        assert_eq!(result.attribute_matrix[0].species_id, 1);
        assert_eq!(result.attribute_matrix[1].species_id, 3);
        assert_eq!(result.similarity_scores.len(), 1);
        assert!(result.similarity_scores[0].total_score >= 0.0);
        assert!(result.similarity_scores[0].total_score <= 1.0);
        assert!(!result.summary.is_empty());
    }

    #[test]
    fn compare_species_rejects_unknown_species_id() {
        let species = fixture_species();
        let observations = fixture_observations();

        let query = ComparisonQuery {
            species_ids: vec![1, 999],
        };
        let err = compare_species(&query, &species, &observations).unwrap_err();
        assert!(matches!(err, ModuleError::SpeciesNotFound(999)));
    }

    #[test]
    fn compare_species_rejects_too_few_species() {
        let species = fixture_species();
        let observations = fixture_observations();

        let query = ComparisonQuery {
            species_ids: vec![1],
        };
        let err = compare_species(&query, &species, &observations).unwrap_err();
        assert!(matches!(err, ModuleError::InvalidQuery(_)));
    }
}
