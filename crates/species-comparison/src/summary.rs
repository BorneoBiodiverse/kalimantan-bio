//! Tahap 5 — Comparative Summary Generation.
//!
//! Provides deterministic, human-readable summary generation from comparison
//! outputs: shared attributes, unique/distinguishing characteristics, and
//! similarity scores.

use crate::types::{ComparisonResult, SimilarityScore, SpeciesAttributeRow};

/// Builds a deterministic, human-readable summary paragraph.
///
/// Output names similarities first, then differences, then the best matching pair.
///
/// # Arguments
///
/// * `matrix` - Attribute rows for looking up species scientific names.
/// * `shared` - Shared attributes in `"key=value"` format.
/// * `unique` - Unique attributes in `"species_id:key=value"` format.
/// * `distinguishing` - Distinguishing characteristics in `"{key}: {v1} vs {v2}"` format.
/// * `scores` - Pairwise similarity scores.
///
/// # Returns
///
/// A formatted paragraph string.
pub(crate) fn build_summary(
    matrix: &[SpeciesAttributeRow],
    shared: &[String],
    unique: &[String],
    distinguishing: &[String],
    scores: &[SimilarityScore],
) -> String {
    let mut parts = Vec::new();

    // 1. Similarities first
    if !shared.is_empty() {
        parts.push(format!("Persamaan utama: {}.", shared.join(", ")));
    } else {
        parts.push(
            "Tidak ada atribut bersama yang identik antar spesies yang dibandingkan.".to_string(),
        );
    }

    // 2. Differences second (prioritize distinguishing characteristics, fallback to unique)
    if !distinguishing.is_empty() {
        parts.push(format!("Perbedaan utama: {}.", distinguishing.join("; ")));
    } else if !unique.is_empty() {
        parts.push(format!("Karakteristik unik: {}.", unique.join(", ")));
    } else {
        parts.push("Tidak ditemukan perbedaan atribut yang signifikan.".to_string());
    }

    // 3. Best matching pair last
    if let Some(best) = scores.iter().max_by(|a, b| {
        a.total_score
            .total_cmp(&b.total_score)
            .then_with(|| b.species_a_id.cmp(&a.species_a_id))
            .then_with(|| b.species_b_id.cmp(&a.species_b_id))
    }) {
        let name_of = |id: u64| -> Option<&str> {
            matrix
                .iter()
                .find(|row| row.species_id == id)
                .map(|row| row.scientific_name.as_str())
                .filter(|name| !name.is_empty())
        };

        let formatted_pair = match (name_of(best.species_a_id), name_of(best.species_b_id)) {
            (Some(name_a), Some(name_b)) => format!(
                "Pasangan paling mirip adalah {name_a} (ID {}) dan {name_b} (ID {}) dengan skor kemiripan {:.2}.",
                best.species_a_id, best.species_b_id, best.total_score
            ),
            _ => format!(
                "Pasangan paling mirip adalah Spesies {} dan Spesies {} dengan skor kemiripan {:.2}.",
                best.species_a_id, best.species_b_id, best.total_score
            ),
        };
        parts.push(formatted_pair);
    } else {
        parts.push("Tidak ada skor pasangan yang dihitung.".to_string());
    }

    parts.join(" ")
}

/// Generate a human-readable comparison summary from a [`ComparisonResult`].
///
/// # Arguments
///
/// * `result` - The populated comparison result.
///
/// # Returns
///
/// The formatted summary paragraph string.
pub(crate) fn generate_summary(result: &ComparisonResult) -> String {
    build_summary(
        &result.attribute_matrix,
        &result.shared_attributes,
        &result.unique_attributes,
        &result.distinguishing_characteristics,
        &result.similarity_scores,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attributes::{
        find_distinguishing_characteristics, find_shared_attributes, find_unique_attributes,
    };
    use crate::matrix::build_attribute_matrix;
    use crate::scoring::calculate_all_pairs;
    use crate::types::{ComparisonQuery, ScoringWeights};
    use crate::validation::{build_observations_map, fetch_species_by_ids};
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

    #[test]
    fn summary_names_similarities_differences_and_top_score_in_order() {
        let row_a = SpeciesAttributeRow {
            species_id: 1,
            scientific_name: "Pongo pygmaeus".to_string(),
            common_name: "Bornean Orangutan".to_string(),
            taxonomy_path: "Animalia > Chordata > Mammalia > Primates > Hominidae > Pongo"
                .to_string(),
            species_type: "Endemic".to_string(),
            iucn: "CR".to_string(),
            cites: "Appendix I".to_string(),
            p106: "Dilindungi".to_string(),
            observation_count: 5,
            kabupaten_list: vec!["Kotawaringin Barat".to_string()],
            is_verified: true,
        };
        let row_b = SpeciesAttributeRow {
            species_id: 3,
            scientific_name: "Pongo abelii".to_string(),
            common_name: "Sumatran Orangutan".to_string(),
            taxonomy_path: "Animalia > Chordata > Mammalia > Primates > Hominidae > Pongo"
                .to_string(),
            species_type: "Native".to_string(),
            iucn: "CR".to_string(),
            cites: "Appendix I".to_string(),
            p106: "Dilindungi".to_string(),
            observation_count: 2,
            kabupaten_list: vec!["Kotawaringin Barat".to_string()],
            is_verified: true,
        };

        let matrix = vec![row_a, row_b];
        let shared = vec![
            "kingdom=Animalia".to_string(),
            "genus=Pongo".to_string(),
            "iucn=CR".to_string(),
        ];
        let unique = vec![
            "1:species_type=Endemic".to_string(),
            "3:species_type=Native".to_string(),
        ];
        let distinguishing = vec!["species_type: Endemic vs Native".to_string()];
        let scores = vec![SimilarityScore {
            species_a_id: 1,
            species_b_id: 3,
            taxonomy_score: 1.0,
            conservation_score: 1.0,
            type_score: 0.0,
            distribution_score: 1.0,
            total_score: 0.85,
        }];

        let summary = build_summary(&matrix, &shared, &unique, &distinguishing, &scores);

        // 1. Must start with similarities
        assert!(summary.starts_with("Persamaan utama: kingdom=Animalia, genus=Pongo, iucn=CR."));
        // 2. Must contain differences
        assert!(summary.contains("Perbedaan utama: species_type: Endemic vs Native."));
        // 3. Must end with top score
        assert!(summary.ends_with(
            "Pasangan paling mirip adalah Pongo pygmaeus (ID 1) dan Pongo abelii (ID 3) dengan skor kemiripan 0.85."
        ));
    }

    #[test]
    fn summary_handles_empty_fields_gracefully() {
        let summary = build_summary(&[], &[], &[], &[], &[]);

        assert_eq!(
            summary,
            "Tidak ada atribut bersama yang identik antar spesies yang dibandingkan. Tidak ditemukan perbedaan atribut yang signifikan. Tidak ada skor pasangan yang dihitung."
        );
    }

    #[test]
    fn summary_snapshot_on_fixture_species() {
        let species = fixture_species();
        let observations = fixture_observations();

        let ids = [1, 3];
        let selected = fetch_species_by_ids(&ids, &species).expect("both ids exist");
        let obs_map = build_observations_map(&selected, &observations);
        let matrix = build_attribute_matrix(&selected, &observations);
        let shared = find_shared_attributes(&matrix);
        let unique = find_unique_attributes(&matrix);
        let distinguishing = find_distinguishing_characteristics(&matrix);
        let weights = ScoringWeights::default();
        let scores = calculate_all_pairs(&selected, &obs_map, &weights);

        let result = ComparisonResult {
            query: ComparisonQuery {
                species_ids: ids.to_vec(),
            },
            attribute_matrix: matrix,
            shared_attributes: shared,
            unique_attributes: unique,
            distinguishing_characteristics: distinguishing,
            similarity_scores: scores,
            summary: String::new(),
        };

        let summary = generate_summary(&result);

        // Byte-stable snapshot assertion for species [1, 3] (Pongo pygmaeus & Pongo abelii)
        let expected_snapshot = "Persamaan utama: kingdom=Animalia, phylum=Chordata, class=Mammalia, order=Primates, family=Hominidae, genus=Pongo, iucn=CR, cites=Appendix I, p106=Dilindungi. Perbedaan utama: species_type: Endemic vs Native. Pasangan paling mirip adalah Pongo pygmaeus (ID 1) dan Pongo abelii (ID 3) dengan skor kemiripan 0.75.";

        assert_eq!(summary, expected_snapshot);
    }
}
