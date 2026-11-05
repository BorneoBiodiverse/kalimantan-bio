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
pub(crate) mod validation;

use std::collections::HashSet;

use kalimantanbio_shared::core::{Observation, Species};

pub(crate) use crate::summary::generate_summary;
pub use crate::types::{
    ComparisonQuery, ComparisonResult, ModuleError, ScoringWeights, SimilarityScore,
    SpeciesAttributeRow,
};
pub(crate) use crate::validation::{
    build_kabupaten_set, build_observations_map, fetch_observations_for_species,
    fetch_species_by_ids, validate_query, MAX_SPECIES, MIN_SPECIES,
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
pub fn compare_species(
    _query: &ComparisonQuery,
    _species: &[Species],
    _observations: &[Observation],
) -> Result<ComparisonResult, ModuleError> {
    todo!("species-comparison phase 5: compare_species")
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
}
