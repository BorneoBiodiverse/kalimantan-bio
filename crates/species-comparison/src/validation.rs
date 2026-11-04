//! Stage 1: Comparison query validation and data retrieval.
//!
//! Validates species comparison queries against bounds (2 to 10 distinct species)
//! and resolves species datasets and observations into working lookups.

use std::collections::{HashMap, HashSet};

use kalimantanbio_shared::core::{Observation, Species};
use kalimantanbio_shared::ValidationError;

use crate::types::{ComparisonQuery, DataError, QueryError};

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
pub(crate) fn validate_query(query: &ComparisonQuery) -> Result<(), QueryError> {
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
pub(crate) fn fetch_species_by_ids<'a>(
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
pub(crate) fn fetch_observations_for_species(
    species_id: u64,
    all_observations: &[Observation],
) -> Vec<&Observation> {
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
pub(crate) fn build_kabupaten_set(observations: &[&Observation]) -> HashSet<u64> {
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
pub(crate) fn build_observations_map(
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ModuleError;

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
        let one = ComparisonQuery {
            species_ids: vec![1],
        };
        assert_eq!(validate_query(&one), Err(QueryError::TooFewSpecies));
    }

    #[test]
    fn duplicate_ids_are_rejected() {
        let dup = ComparisonQuery {
            species_ids: vec![1, 1],
        };
        assert_eq!(validate_query(&dup), Err(QueryError::DuplicateId));
    }

    #[test]
    fn more_than_ten_ids_is_too_many() {
        let eleven = ComparisonQuery {
            species_ids: (1..=11).collect(),
        };
        assert_eq!(validate_query(&eleven), Err(QueryError::TooManySpecies));
    }

    #[test]
    fn a_valid_query_passes() {
        let valid = ComparisonQuery {
            species_ids: vec![1, 2, 3],
        };
        assert_eq!(validate_query(&valid), Ok(()));
    }

    #[test]
    fn query_errors_map_to_the_public_module_error() {
        let invalid = ComparisonQuery {
            species_ids: vec![1],
        };
        let err: ModuleError = validate_query(&invalid).unwrap_err().into();
        assert!(matches!(err, ModuleError::InvalidQuery(_)));
    }

    #[test]
    fn all_requested_ids_are_resolved_in_query_order() {
        let species = fixture_species();
        let ids = vec![3, 1, 2];
        let found = fetch_species_by_ids(&ids, &species).expect("known ids must be found");
        let resolved_ids: Vec<u64> = found.iter().map(|s| s.id).collect();
        assert_eq!(resolved_ids, ids);
    }

    #[test]
    fn a_missing_id_reports_the_exact_id() {
        let species = fixture_species();
        let ids = vec![1, 999_999];
        assert_eq!(
            fetch_species_by_ids(&ids, &species),
            Err(DataError::SpeciesNotFound(999_999))
        );
    }

    #[test]
    fn a_species_without_observations_yields_an_empty_vec() {
        let observations = fixture_observations();
        let matched = fetch_observations_for_species(999_999, &observations);
        assert!(matched.is_empty());
    }

    #[test]
    fn observations_are_filtered_to_the_requested_species() {
        let observations = vec![
            observation_for(1, 6201),
            observation_for(2, 6202),
            observation_for(1, 6203),
        ];
        let matched = fetch_observations_for_species(1, &observations);
        let regencies: Vec<u64> = matched.iter().map(|obs| obs.kabupaten_id).collect();
        assert_eq!(regencies, vec![6201, 6203]);
    }

    #[test]
    fn data_errors_map_to_the_public_module_error() {
        let err: ModuleError = DataError::SpeciesNotFound(42).into();
        assert!(matches!(err, ModuleError::SpeciesNotFound(42)));
    }

    #[test]
    fn repeated_regencies_collapse_into_one_entry() {
        let observations = vec![
            observation_for(1, 6201),
            observation_for(1, 6201),
            observation_for(1, 6202),
        ];
        let refs: Vec<&Observation> = observations.iter().collect();
        let regencies = build_kabupaten_set(&refs);
        assert_eq!(regencies, HashSet::from([6201, 6202]));
    }

    #[test]
    fn no_observations_yields_an_empty_set() {
        let regencies = build_kabupaten_set(&[]);
        assert!(regencies.is_empty());
    }

    #[test]
    fn fixture_species_map_to_their_regency_sets() {
        let species = fixture_species();
        let observations = fixture_observations();

        let query = ComparisonQuery {
            species_ids: vec![1, 2],
        };
        let selected =
            fetch_species_by_ids(&query.species_ids, &species).expect("ids 1 and 2 exist");

        let map = build_observations_map(&selected, &observations);

        assert_eq!(map.len(), 2);
        assert!(map.contains_key(&1));
        assert!(map.contains_key(&2));
        assert!(!map.get(&1).unwrap().is_empty());
        assert!(!map.get(&2).unwrap().is_empty());
    }
}
