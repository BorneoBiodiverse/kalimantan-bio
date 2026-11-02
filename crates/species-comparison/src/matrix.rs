//! Stage 2: Matrix assembly and column extraction.
//!
//! Provides table-level comparison matrix construction and vertical
//! attribute-column extraction for downstream pairwise and multi-species
//! analytics.

use std::collections::{BTreeSet, HashMap};

use kalimantanbio_shared::core::{Observation, Species};
use kalimantanbio_shared::taxonomy::build_taxonomy_path;

use crate::types::SpeciesAttributeRow;

/// Builds a display row for one species.
///
/// Bundles species taxonomy path, conservation classifications, and observation
/// occurrences into a [`SpeciesAttributeRow`].
///
/// # Arguments
///
/// * `species` - The target species record.
/// * `observations` - All observation records associated with the species.
///
/// # Returns
///
/// A fully populated [`SpeciesAttributeRow`].
pub(crate) fn build_attribute_row(
    species: &Species,
    observations: &[&Observation],
) -> SpeciesAttributeRow {
    let kabupaten_list: Vec<String> = observations
        .iter()
        .map(|obs| obs.kabupaten_name.as_str())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .map(String::from)
        .collect();

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

/// Assemble the comparison matrix for all selected species.
///
/// Pre-groups observations in a single pass ($O(M)$) and maps over the species
/// list in query order to assemble a [`SpeciesAttributeRow`] for each species.
///
/// # Arguments
///
/// * `species_list` - The selected species in validated query order.
/// * `all_observations` - The complete observations dataset.
///
/// # Returns
///
/// A vector of attribute rows corresponding 1:1 with `species_list`.
///
/// # Example
///
/// ```rust,ignore
/// let matrix = build_attribute_matrix(&species_refs, &all_observations);
/// assert_eq!(matrix.len(), species_refs.len());
/// ```
pub(crate) fn build_attribute_matrix(
    species_list: &[&Species],
    all_observations: &[Observation],
) -> Vec<SpeciesAttributeRow> {
    let mut observations_by_species: HashMap<u64, Vec<&Observation>> = HashMap::new();
    for obs in all_observations {
        observations_by_species
            .entry(obs.species_id)
            .or_default()
            .push(obs);
    }

    species_list
        .iter()
        .map(|species| {
            let observations = observations_by_species
                .get(&species.id)
                .map(|v| v.as_slice())
                .unwrap_or(&[]);
            build_attribute_row(species, observations)
        })
        .collect()
}

/// Extract a single column of attribute values across all rows in the matrix.
///
/// Used by downstream analysis functions that need column-oriented values
/// rather than full row bundles (e.g. comparing all IUCN statuses, species types,
/// or individual taxonomy ranks).
///
/// # Arguments
///
/// * `matrix` - The comparison attribute matrix.
/// * `key` - The attribute key to extract. Supported keys (case-insensitive):
///   - Conservation & metadata: `"iucn"`, `"cites"`, `"p106"`, `"species_type"`,
///     `"scientific_name"`, `"common_name"`, `"taxonomy_path"`, `"species_id"`,
///     `"observation_count"`, `"is_verified"`
///   - Canonical taxonomy ranks (parsed from `taxonomy_path`): `"kingdom"`,
///     `"phylum"` / `"phylum_division"`, `"class"`, `"order"`, `"family"`, `"genus"`
///
/// # Returns
///
/// A vector of attribute value strings for each row in the matrix. If `key` is
/// unrecognized, returns an empty vector without panicking.
///
/// # Example
///
/// ```rust,ignore
/// let iucn_statuses = extract_attribute_values(&matrix, "iucn");
/// assert_eq!(iucn_statuses.len(), matrix.len());
/// ```
pub(crate) fn extract_attribute_values(matrix: &[SpeciesAttributeRow], key: &str) -> Vec<String> {
    let normalized = key.trim().to_ascii_lowercase();
    matrix
        .iter()
        .map(|row| extract_row_attribute(row, &normalized))
        .collect::<Option<Vec<_>>>()
        .unwrap_or_default()
}

/// Maps canonical rank names to their 0-based depth index in `taxonomy_path`.
fn taxonomy_rank_index(key: &str) -> Option<usize> {
    match key {
        "kingdom" => Some(0),
        "phylum" | "phylum_division" => Some(1),
        "class" => Some(2),
        "order" => Some(3),
        "family" => Some(4),
        "genus" => Some(5),
        _ => None,
    }
}

/// Extract a single attribute value from one row, or `None` if the key is unknown.
fn extract_row_attribute(row: &SpeciesAttributeRow, key: &str) -> Option<String> {
    if let Some(rank_idx) = taxonomy_rank_index(key) {
        return Some(parse_taxonomy_rank(&row.taxonomy_path, rank_idx).unwrap_or_default());
    }

    match key {
        "iucn" => Some(row.iucn.clone()),
        "cites" => Some(row.cites.clone()),
        "p106" => Some(row.p106.clone()),
        "species_type" => Some(row.species_type.clone()),
        "scientific_name" => Some(row.scientific_name.clone()),
        "common_name" => Some(row.common_name.clone()),
        "taxonomy_path" => Some(row.taxonomy_path.clone()),
        "species_id" => Some(row.species_id.to_string()),
        "observation_count" => Some(row.observation_count.to_string()),
        "is_verified" => Some(row.is_verified.to_string()),
        _ => None,
    }
}

/// Parse a specific taxonomic rank segment from a rendered `taxonomy_path`.
///
/// Assumes standard `"Kingdom > Phylum > Class > Order > Family > Genus"` delimiter format.
fn parse_taxonomy_rank(taxonomy_path: &str, rank_index: usize) -> Option<String> {
    taxonomy_path
        .split(" > ")
        .nth(rank_index)
        .map(|s| s.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn matrix_length_equals_species_count() {
        let species = fixture_species();
        let observations = fixture_observations();
        let selected = crate::fetch_species_by_ids(&[1, 2, 3], &species).expect("ids exist");

        let matrix = build_attribute_matrix(&selected, &observations);

        assert_eq!(matrix.len(), 3);
        assert_eq!(matrix[0].species_id, 1);
        assert_eq!(matrix[1].species_id, 2);
        assert_eq!(matrix[2].species_id, 3);
    }

    #[test]
    fn matrix_preserves_arbitrary_query_order() {
        let species = fixture_species();
        let observations = fixture_observations();
        let selected = crate::fetch_species_by_ids(&[3, 1], &species).expect("ids exist");

        let matrix = build_attribute_matrix(&selected, &observations);

        assert_eq!(
            matrix.iter().map(|r| r.species_id).collect::<Vec<_>>(),
            vec![3, 1]
        );
    }

    #[test]
    fn matrix_populates_row_fields_accurately() {
        let species = fixture_species();
        let observations = fixture_observations();
        let selected = crate::fetch_species_by_ids(&[1], &species).expect("id 1 exists");

        let matrix = build_attribute_matrix(&selected, &observations);
        assert_eq!(matrix.len(), 1);

        let row = &matrix[0];
        assert_eq!(row.species_id, 1);
        assert_eq!(row.scientific_name, "Pongo pygmaeus");
        assert_eq!(row.common_name, "Orangutan Kalimantan");
        assert_eq!(row.species_type, "Endemic");
        assert_eq!(row.iucn, "CR");
        assert_eq!(row.cites, "Appendix I");
        assert_eq!(row.p106, "Dilindungi");
        assert_eq!(row.is_verified, true);
        assert_eq!(row.kabupaten_list, vec!["Kotawaringin Barat", "Seruyan"]);
    }

    #[test]
    fn matrix_species_with_zero_observations_has_empty_kabupaten_list() {
        let species = fixture_species();
        let observations = fixture_observations();
        let selected = crate::fetch_species_by_ids(&[5], &species).expect("id 5 exists");

        let matrix = build_attribute_matrix(&selected, &observations);
        assert_eq!(matrix.len(), 1);
        assert!(matrix[0].kabupaten_list.is_empty());
    }

    #[test]
    fn extract_attribute_values_known_columns() {
        let species = fixture_species();
        let observations = fixture_observations();
        let selected = crate::fetch_species_by_ids(&[1, 2, 3], &species).expect("ids exist");
        let matrix = build_attribute_matrix(&selected, &observations);

        let iucn_col = extract_attribute_values(&matrix, "iucn");
        assert_eq!(iucn_col, vec!["CR", "EN", "CR"]);

        let type_col = extract_attribute_values(&matrix, "species_type");
        assert_eq!(type_col, vec!["Endemic", "Native", "Native"]);

        let names = extract_attribute_values(&matrix, "scientific_name");
        assert_eq!(
            names,
            vec!["Pongo pygmaeus", "Panthera tigris", "Pongo abelii"]
        );
    }

    #[test]
    fn extract_attribute_values_case_insensitive_key() {
        let species = fixture_species();
        let observations = fixture_observations();
        let selected = crate::fetch_species_by_ids(&[1], &species).expect("id 1 exists");
        let matrix = build_attribute_matrix(&selected, &observations);

        assert_eq!(extract_attribute_values(&matrix, "IUCN"), vec!["CR"]);
        assert_eq!(
            extract_attribute_values(&matrix, "Species_Type"),
            vec!["Endemic"]
        );
    }

    #[test]
    fn extract_attribute_values_taxonomy_ranks() {
        let species = fixture_species();
        let observations = fixture_observations();
        // Species 1 (Pongo pygmaeus): Animalia > Chordata > Mammalia > Primates > Hominidae > Pongo
        // Species 2 (Panthera tigris): Animalia > Chordata > Mammalia > Carnivora > Felidae > Panthera
        // Species 3 (Pongo abelii): Animalia > Chordata > Mammalia > Primates > Hominidae > Pongo
        let selected = crate::fetch_species_by_ids(&[1, 2, 3], &species).expect("ids exist");
        let matrix = build_attribute_matrix(&selected, &observations);

        assert_eq!(
            extract_attribute_values(&matrix, "kingdom"),
            vec!["Animalia", "Animalia", "Animalia"]
        );
        assert_eq!(
            extract_attribute_values(&matrix, "phylum"),
            vec!["Chordata", "Chordata", "Chordata"]
        );
        assert_eq!(
            extract_attribute_values(&matrix, "phylum_division"),
            vec!["Chordata", "Chordata", "Chordata"]
        );
        assert_eq!(
            extract_attribute_values(&matrix, "class"),
            vec!["Mammalia", "Mammalia", "Mammalia"]
        );
        assert_eq!(
            extract_attribute_values(&matrix, "order"),
            vec!["Primates", "Carnivora", "Primates"]
        );
        assert_eq!(
            extract_attribute_values(&matrix, "family"),
            vec!["Hominidae", "Felidae", "Hominidae"]
        );
        assert_eq!(
            extract_attribute_values(&matrix, "genus"),
            vec!["Pongo", "Panthera", "Pongo"]
        );
    }

    #[test]
    fn extract_attribute_values_unknown_key_returns_empty_without_panic() {
        let species = fixture_species();
        let observations = fixture_observations();
        let selected = crate::fetch_species_by_ids(&[1, 2], &species).expect("ids exist");
        let matrix = build_attribute_matrix(&selected, &observations);

        let result = extract_attribute_values(&matrix, "unknown_field");
        assert!(result.is_empty());
    }

    #[test]
    fn extract_attribute_values_on_empty_matrix() {
        let empty_matrix: Vec<SpeciesAttributeRow> = Vec::new();
        let result = extract_attribute_values(&empty_matrix, "iucn");
        assert!(result.is_empty());
    }
}
