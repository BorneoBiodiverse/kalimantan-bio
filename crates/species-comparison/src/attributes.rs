//! Stage 3: Shared and unique attribute analysis.
//!
//! Provides multi-species attribute comparison: identifying shared attributes across
//! all species, discovering unique attributes possessed by a single species, and
//! extracting key distinguishing characteristics sorted by analytical priority.

use std::collections::HashMap;

use crate::matrix::extract_attribute_values;
use crate::types::SpeciesAttributeRow;

/// Canonical candidate attribute keys inspected for comparative analysis.
const COMPARISON_KEYS: &[&str] = &[
    "kingdom",
    "phylum",
    "class",
    "order",
    "family",
    "genus",
    "species_type",
    "iucn",
    "cites",
    "p106",
];

/// Returns priority score for distinguishing characteristics ordering.
/// Higher value indicates higher priority (Planning Doc §6 priority:
/// iucn > species_type > family > order > class > phylum).
fn distinguishing_priority(key: &str) -> usize {
    match key {
        "iucn" => 10,
        "species_type" => 9,
        "family" => 8,
        "order" => 7,
        "class" => 6,
        "phylum" => 5,
        "kingdom" => 4,
        "genus" => 3,
        "cites" => 2,
        "p106" => 1,
        _ => 0,
    }
}

/// Check whether all values in a slice are non-empty and identical.
///
/// Returns `true` if and only if the slice is non-empty, the first element
/// is non-empty, and every subsequent element matches the first.
///
/// # Arguments
///
/// * `values` - The attribute value slice to test.
///
/// # Returns
///
/// `true` if all values are non-empty and identical; `false` otherwise.
///
/// # Example
///
/// ```rust,ignore
/// let values = vec!["Animalia".to_string(), "Animalia".to_string()];
/// assert!(is_attribute_shared(&values));
/// ```
pub(crate) fn is_attribute_shared(values: &[String]) -> bool {
    if values.is_empty() || values[0].is_empty() {
        return false;
    }
    values.windows(2).all(|w| w[0] == w[1])
}

/// Find attributes that have identical values across all compared species.
///
/// Scans canonical taxonomy and conservation/status attributes. When all species
/// share the same non-empty value, the attribute is recorded in `"key=value"` format.
///
/// # Arguments
///
/// * `matrix` - The comparison attribute matrix.
///
/// # Returns
///
/// A vector of shared attribute strings in `"key=value"` format (e.g. `"kingdom=Animalia"`).
///
/// # Example
///
/// ```rust,ignore
/// let shared = find_shared_attributes(&matrix);
/// assert!(shared.contains(&"kingdom=Animalia".to_string()));
/// ```
pub(crate) fn find_shared_attributes(matrix: &[SpeciesAttributeRow]) -> Vec<String> {
    if matrix.is_empty() {
        return Vec::new();
    }

    let mut shared = Vec::new();
    for &key in COMPARISON_KEYS {
        let values = extract_attribute_values(matrix, key);
        if is_attribute_shared(&values) {
            shared.push(format!("{key}={}", values[0]));
        }
    }
    shared
}

/// Find attributes whose value is uniquely possessed by exactly one species in the comparison.
///
/// For each attribute column, counts value frequencies across the matrix. If a value
/// occurs exactly once (frequency == 1), it is reported as unique to that species in
/// `"species_id:key=value"` format, preserving matrix row order.
///
/// # Arguments
///
/// * `matrix` - The comparison attribute matrix.
///
/// # Returns
///
/// A vector of unique attribute strings in `"species_id:key=value"` format
/// (e.g. `"1:species_type=Endemic"`).
///
/// # Example
///
/// ```rust,ignore
/// let unique = find_unique_attributes(&matrix);
/// ```
pub(crate) fn find_unique_attributes(matrix: &[SpeciesAttributeRow]) -> Vec<String> {
    if matrix.len() < 2 {
        return Vec::new();
    }

    let mut unique = Vec::new();

    for &key in COMPARISON_KEYS {
        let values = extract_attribute_values(matrix, key);
        if values.len() != matrix.len() {
            continue;
        }

        // Count frequency of each distinct value in this column
        let mut frequencies: HashMap<&str, usize> = HashMap::new();
        for val in &values {
            if !val.is_empty() && val != "-" {
                *frequencies.entry(val.as_str()).or_default() += 1;
            }
        }

        // Record entries that appear exactly once
        for (row, val) in matrix.iter().zip(&values) {
            if frequencies.get(val.as_str()).copied() == Some(1) {
                unique.push(format!("{}:{key}={val}", row.species_id));
            }
        }
    }

    unique
}

/// Identify attributes that exhibit variation across species, sorted by diagnostic priority.
///
/// Attributes that vary across the compared species are formatted as
/// `"{key}: {val1} vs {val2} vs ..."` in matrix row order, prioritized by
/// analytical importance: `iucn` > `species_type` > `family` > `order` > `class` > `phylum`
/// (Planning Doc §6).
///
/// # Arguments
///
/// * `matrix` - The comparison attribute matrix.
///
/// # Returns
///
/// A vector of distinguishing characteristic strings.
///
/// # Example
///
/// ```rust,ignore
/// let diffs = find_distinguishing_characteristics(&matrix);
/// assert!(diffs.iter().any(|d| d.starts_with("iucn:")));
/// ```
pub(crate) fn find_distinguishing_characteristics(matrix: &[SpeciesAttributeRow]) -> Vec<String> {
    if matrix.len() < 2 {
        return Vec::new();
    }

    let mut characteristics: Vec<(&str, usize, String)> = Vec::new();

    for &key in COMPARISON_KEYS {
        let values = extract_attribute_values(matrix, key);
        if values.len() == matrix.len() && !is_attribute_shared(&values) {
            // Only consider distinguishing if there is actual variation
            let formatted = values.join(" vs ");
            let priority = distinguishing_priority(key);
            characteristics.push((key, priority, format!("{key}: {formatted}")));
        }
    }

    // Sort by priority descending
    characteristics.sort_by_key(|item| std::cmp::Reverse(item.1));

    characteristics
        .into_iter()
        .map(|(_, _, desc)| desc)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use kalimantanbio_shared::core::{Observation, Species};

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

    fn test_matrix_for_ids(ids: &[u64]) -> Vec<SpeciesAttributeRow> {
        let species = fixture_species();
        let observations = fixture_observations();
        let selected = crate::fetch_species_by_ids(ids, &species).expect("test ids exist");
        crate::matrix::build_attribute_matrix(&selected, &observations)
    }

    #[test]
    fn is_attribute_shared_empty_slice_returns_false() {
        let empty: Vec<String> = Vec::new();
        assert!(!is_attribute_shared(&empty));
    }

    #[test]
    fn is_attribute_shared_empty_string_returns_false() {
        let values = vec![String::new(), String::new()];
        assert!(!is_attribute_shared(&values));
    }

    #[test]
    fn is_attribute_shared_single_element_returns_true() {
        let values = vec!["Animalia".to_string()];
        assert!(is_attribute_shared(&values));
    }

    #[test]
    fn is_attribute_shared_identical_values_returns_true() {
        let values = vec![
            "Chordata".to_string(),
            "Chordata".to_string(),
            "Chordata".to_string(),
        ];
        assert!(is_attribute_shared(&values));
    }

    #[test]
    fn is_attribute_shared_differing_values_returns_false() {
        let values = vec!["Endemic".to_string(), "Native".to_string()];
        assert!(!is_attribute_shared(&values));
    }

    #[test]
    fn find_shared_attributes_orangutan_pair() {
        // Species 1 (Pongo pygmaeus) & Species 3 (Pongo abelii) share:
        // kingdom=Animalia, phylum=Chordata, class=Mammalia, order=Primates,
        // family=Hominidae, genus=Pongo, iucn=CR, cites=Appendix I, p106=Dilindungi
        let matrix = test_matrix_for_ids(&[1, 3]);
        let shared = find_shared_attributes(&matrix);

        assert!(shared.contains(&"kingdom=Animalia".to_string()));
        assert!(shared.contains(&"phylum=Chordata".to_string()));
        assert!(shared.contains(&"class=Mammalia".to_string()));
        assert!(shared.contains(&"order=Primates".to_string()));
        assert!(shared.contains(&"family=Hominidae".to_string()));
        assert!(shared.contains(&"genus=Pongo".to_string()));
        assert!(shared.contains(&"iucn=CR".to_string()));
        assert!(shared.contains(&"cites=Appendix I".to_string()));
        assert!(shared.contains(&"p106=Dilindungi".to_string()));
        // Species type differs (Endemic vs Native), so it must NOT be shared
        assert!(!shared.iter().any(|s| s.starts_with("species_type=")));
    }

    #[test]
    fn find_shared_attributes_empty_matrix() {
        let empty_matrix: Vec<SpeciesAttributeRow> = Vec::new();
        assert!(find_shared_attributes(&empty_matrix).is_empty());
    }

    #[test]
    fn find_unique_attributes_identifies_singleton_values() {
        // Species 1 (Endemic) and Species 3 (Native): both have unique species_type values in this pair
        let matrix = test_matrix_for_ids(&[1, 3]);
        let unique = find_unique_attributes(&matrix);

        assert!(unique.contains(&"1:species_type=Endemic".to_string()));
        assert!(unique.contains(&"3:species_type=Native".to_string()));
        // Shared taxonomy (e.g. genus Pongo) must not appear in unique attributes
        assert!(!unique.iter().any(|u| u.contains("genus=")));
    }

    #[test]
    fn find_unique_attributes_three_species_with_duplicate_and_unique() {
        // Species 1: Endemic, iucn CR
        // Species 2: Native, iucn EN
        // Species 3: Native, iucn CR
        // Here, Native is shared by 2 and 3 (frequency 2), while Endemic is unique to Species 1 (frequency 1)
        // For IUCN: CR is shared by 1 and 3 (frequency 2), while EN is unique to Species 2 (frequency 1)
        let matrix = test_matrix_for_ids(&[1, 2, 3]);
        let unique = find_unique_attributes(&matrix);

        assert!(unique.contains(&"1:species_type=Endemic".to_string()));
        assert!(!unique.contains(&"2:species_type=Native".to_string()));
        assert!(!unique.contains(&"3:species_type=Native".to_string()));
        assert!(unique.contains(&"2:iucn=EN".to_string()));
    }

    #[test]
    fn find_unique_attributes_ignores_unknown_dash() {
        let mut row1 = test_matrix_for_ids(&[1]).remove(0);
        let mut row2 = test_matrix_for_ids(&[2]).remove(0);
        row1.cites = "-".to_string();
        row2.cites = "Appendix II".to_string();

        let matrix = vec![row1, row2];
        let unique = find_unique_attributes(&matrix);

        // "-" must not be reported as a unique attribute
        assert!(!unique.iter().any(|u| u.ends_with("=-")));
        assert!(unique.contains(&"2:cites=Appendix II".to_string()));
    }

    #[test]
    fn find_distinguishing_characteristics_prioritizes_iucn_and_species_type() {
        // Species 1 (Pongo pygmaeus: CR, Endemic, Carnivora/Primates differ from Panthera tigris)
        // Species 2 (Panthera tigris: EN, Native)
        let matrix = test_matrix_for_ids(&[1, 2]);
        let diffs = find_distinguishing_characteristics(&matrix);

        assert!(!diffs.is_empty());
        // iucn must be first (priority 10)
        assert_eq!(diffs[0], "iucn: CR vs EN");
        // species_type must be second (priority 9)
        assert_eq!(diffs[1], "species_type: Endemic vs Native");
        // Order differences follow
        assert!(diffs.contains(&"order: Primates vs Carnivora".to_string()));
        assert!(diffs.contains(&"family: Hominidae vs Felidae".to_string()));
        assert!(diffs.contains(&"genus: Pongo vs Panthera".to_string()));
        // Shared kingdom (Animalia) must NOT appear in distinguishing characteristics
        assert!(!diffs.iter().any(|d| d.starts_with("kingdom:")));
    }

    #[test]
    fn find_distinguishing_characteristics_single_or_empty_row_returns_empty() {
        let matrix = test_matrix_for_ids(&[1]);
        assert!(find_distinguishing_characteristics(&matrix).is_empty());

        let empty: Vec<SpeciesAttributeRow> = Vec::new();
        assert!(find_distinguishing_characteristics(&empty).is_empty());
    }
}
