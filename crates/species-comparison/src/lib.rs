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

use std::collections::{HashMap, HashSet};

use kalimantanbio_shared::core::{Observation, Species};
use kalimantanbio_shared::taxonomy::build_taxonomy_path;
use kalimantanbio_shared::ValidationError;

use crate::types::{
    ComparisonQuery, ComparisonResult, DataError, ModuleError, QueryError, ScoringWeights,
    SimilarityScore, SpeciesAttributeRow,
};

/// Minimum number of species a comparison must request.
pub(crate) const MIN_SPECIES: usize = 2;

/// Maximum number of species a comparison may request (this is the bound the
/// module passes to the shared `validate_id_list`).
pub(crate) const MAX_SPECIES: usize = 10;

/// Reject a malformed comparison query before any species data is fetched.
///
/// The counting and duplicate rules come from the shared validation contract
/// (`kalimantanbio_shared::validation::validate_id_list`), so this module never
/// re-implements them. The shared error is translated into the module's own
/// private [`QueryError`], which callers of the public API never see.
///
/// # Arguments
///
/// * `query` - The comparison query to validate.
///
/// # Returns
///
/// `Ok(())` for a query holding 2 to 10 distinct species ids, otherwise the
/// matching [`QueryError`].
fn validate_query(query: &ComparisonQuery) -> Result<(), QueryError> {
    match kalimantanbio_shared::validation::validate_id_list(
        &query.species_ids,
        MIN_SPECIES,
        MAX_SPECIES,
    ) {
        Ok(()) => Ok(()),
        Err(ValidationError::TooFewItems { .. }) => Err(QueryError::TooFewSpecies),
        Err(ValidationError::TooManyItems { .. }) => Err(QueryError::TooManySpecies),
        Err(ValidationError::DuplicateIds) => Err(QueryError::DuplicateId),
        Err(other) => unreachable!("validate_id_list cannot return {other}"),
    }
}

/// Resolve the requested species ids against the provided dataset.
///
/// Reads only from the slice it is given — no implicit fetching, no database
/// access (planning §19.4). The order of the result follows the order of
/// `ids`, so the comparison matrix stays in the order the caller asked for.
///
/// # Arguments
///
/// * `ids` - The species ids to resolve, already validated by [`validate_query`].
/// * `all_species` - The species dataset to look the ids up in.
///
/// # Returns
///
/// Borrowed references to the matched species, or [`DataError::SpeciesNotFound`]
/// carrying the first id that has no match.
fn fetch_species_by_ids<'a>(
    ids: &[u64],
    all_species: &'a [Species],
) -> Result<Vec<&'a Species>, DataError> {
    ids.iter()
        .map(|id| {
            all_species
                .iter()
                .find(|species| species.id == *id)
                .ok_or(DataError::SpeciesNotFound(*id))
        })
        .collect()
}

/// Collect every observation record belonging to one species.
///
/// An empty result is **not** an error: a species may legitimately have no
/// observations yet, and downstream distribution scoring already treats an
/// empty regency set as a score of `0.0` (planning §15 edge cases).
///
/// # Arguments
///
/// * `species_id` - The species whose observations are wanted.
/// * `all_observations` - The observation dataset to filter.
///
/// # Returns
///
/// Borrowed references to the matching observations, possibly empty.
fn fetch_observations_for_species<'a>(
    species_id: u64,
    all_observations: &'a [Observation],
) -> Vec<&'a Observation> {
    all_observations
        .iter()
        .filter(|observation| observation.species_id == species_id)
        .collect()
}

/// Collect the distinct regency (kabupaten) ids from one species' observations.
///
/// Several observations usually fall in the same regency, so the result is a
/// set: ten sightings spread over two regencies collapse to a two-element set
/// such as `{6201, 6202}`. That set is exactly what the geographic (Jaccard)
/// dimension of pair scoring consumes.
///
/// # Arguments
///
/// * `observations` - The observations belonging to a single species.
///
/// # Returns
///
/// The distinct regency ids, empty when the species has no observations.
fn build_kabupaten_set(observations: &[&Observation]) -> HashSet<u64> {
    observations
        .iter()
        .map(|observation| observation.kabupaten_id)
        .collect()
}

/// Map every selected species to the set of regencies it was observed in.
///
/// The pair-scoring stage needs one regency set per species; building them all
/// in one pass keeps the pipeline from rescanning the observation dataset for
/// every comparison.
///
/// # Arguments
///
/// * `species_list` - The species selected by the (already validated) query.
/// * `all_observations` - The full observation dataset.
///
/// # Returns
///
/// A map from species id to its distinct regency ids. A species with no
/// observations maps to an empty set rather than being omitted, so callers can
/// look it up without a fallback branch.
fn build_observations_map(
    species_list: &[&Species],
    all_observations: &[Observation],
) -> HashMap<u64, HashSet<u64>> {
    species_list
        .iter()
        .map(|species| {
            let observations = fetch_observations_for_species(species.id, all_observations);
            (species.id, build_kabupaten_set(&observations))
        })
        .collect()
}

/// Builds a display row for one species.
///
/// The taxonomy breadcrumb comes from the shared renderer. It is display data,
/// not Module 3's parent-to-children `build_taxonomy_map`.
fn build_attribute_row(species: &Species, observations: &[&Observation]) -> SpeciesAttributeRow {
    let mut kabupaten_list: Vec<String> = observations
        .iter()
        .map(|observation| observation.kabupaten_name.clone())
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    kabupaten_list.sort();

    SpeciesAttributeRow {
        species_id: species.id,
        scientific_name: species.scientific_name.clone(),
        common_name: species.common_name.clone(),
        taxonomy_path: build_taxonomy_path(&species.taxonomy),
        species_type: species.species_type.clone(),
        iucn: species.iucn.clone(),
        cites: species.cites.clone(),
        p106: species.p106.clone(),
        observation_count: species.observation_count,
        kabupaten_list,
        is_verified: species.is_verified,
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;

    /// The agreed shared fixture, so the module is tested against the same
    /// dataset every other module uses (planning: fixtures live in shared).
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

    /// A throwaway observation, so a test can state the exact regency set it
    /// wants instead of depending on the shared fixture's numbers.
    fn observation_for(species_id: u64, kabupaten_id: u64) -> Observation {
        Observation {
            id: 0,
            species_id,
            latitude: 0.0,
            longitude: 0.0,
            date: String::from("2025-01-01"),
            count: 1,
            is_verified: true,
            source_method: String::from("test"),
            kabupaten_id,
            kabupaten_name: String::from("test regency"),
            province_name: String::from("test province"),
        }
    }

    #[test]
    fn fewer_than_two_ids_is_too_few() {
        let query = ComparisonQuery {
            species_ids: vec![1],
        };

        assert_eq!(validate_query(&query), Err(QueryError::TooFewSpecies));
    }

    #[test]
    fn duplicate_ids_are_rejected() {
        // Two ids satisfies the minimum count, so only the duplicate rule fires.
        let query = ComparisonQuery {
            species_ids: vec![1, 1],
        };

        assert_eq!(validate_query(&query), Err(QueryError::DuplicateId));
    }

    #[test]
    fn more_than_ten_ids_is_too_many() {
        let query = ComparisonQuery {
            species_ids: (1..=11).collect(),
        };

        assert_eq!(validate_query(&query), Err(QueryError::TooManySpecies));
    }

    #[test]
    fn a_valid_query_passes() {
        let query = ComparisonQuery {
            species_ids: vec![1, 2, 3],
        };

        assert_eq!(validate_query(&query), Ok(()));
    }

    #[test]
    fn query_errors_map_to_the_public_module_error() {
        let public: ModuleError = QueryError::TooFewSpecies.into();

        assert_eq!(
            public,
            ModuleError::InvalidQuery(String::from("too few species (minimum 2)"))
        );
    }

    #[test]
    fn all_requested_ids_are_resolved_in_query_order() {
        let species = fixture_species();
        let found = fetch_species_by_ids(&[3, 1], &species).expect("both ids exist");

        assert_eq!(
            found.iter().map(|s| s.id).collect::<Vec<u64>>(),
            vec![3, 1],
            "result must follow the query order, not the dataset order"
        );
    }

    #[test]
    fn a_missing_id_reports_the_exact_id() {
        let species = fixture_species();
        let error = fetch_species_by_ids(&[1, 99_999], &species).unwrap_err();

        assert_eq!(error, DataError::SpeciesNotFound(99_999));
    }

    #[test]
    fn a_species_without_observations_yields_an_empty_vec() {
        let observations = fixture_observations();
        let found = fetch_observations_for_species(5, &observations); // Species 5 (Rafflesia arnoldii) has no observation rows at all.

        assert!(found.is_empty(), "expected no observations, got {found:?}");
    }

    #[test]
    fn observations_are_filtered_to_the_requested_species() {
        let observations = fixture_observations();
        let found = fetch_observations_for_species(1, &observations);

        assert_eq!(
            found.len(),
            2,
            "species 1 has two observations in the fixture"
        );
        assert!(
            found.iter().all(|o| o.species_id == 1),
            "no other species may leak into the result"
        );
    }

    #[test]
    fn data_errors_map_to_the_public_module_error() {
        let public: ModuleError = DataError::SpeciesNotFound(7).into();

        assert_eq!(public, ModuleError::SpeciesNotFound(7));
    }

    #[test]
    fn repeated_regencies_collapse_into_one_entry() {
        let first = observation_for(1, 6201);
        let second = observation_for(1, 6201);
        let third = observation_for(1, 6202);
        let observations = vec![&first, &second, &third];

        let set = build_kabupaten_set(&observations);

        assert_eq!(
            set,
            HashSet::from([6201, 6202]),
            "three observations over two regencies must yield two entries"
        );
    }

    #[test]
    fn no_observations_yields_an_empty_set() {
        let set = build_kabupaten_set(&[]);

        assert!(set.is_empty(), "expected an empty set, got {set:?}");
    }

    #[test]
    fn fixture_species_map_to_their_regency_sets() {
        let species = fixture_species();
        let observations = fixture_observations();
        let selected = fetch_species_by_ids(&[1, 3, 5], &species).expect("all three ids exist");

        let map = build_observations_map(&selected, &observations);

        assert_eq!(
            map.get(&1),
            Some(&HashSet::from([6201, 6202])),
            "species 1 was recorded in Kotawaringin Barat and Seruyan"
        );
        assert_eq!(
            map.get(&3),
            Some(&HashSet::from([6201, 6271])),
            "species 3 was recorded in Kotawaringin Barat and Palangka Raya"
        );
        assert_eq!(
            map.get(&5),
            Some(&HashSet::new()),
            "species 5 has no observations, so it maps to an empty set"
        );
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
