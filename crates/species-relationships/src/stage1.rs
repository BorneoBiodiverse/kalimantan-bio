//! Tahap 1 — Validasi dan Normalisasi Data.
//!
//! Modul ini menyediakan fungsi-fungsi murni (pure functions) untuk:
//! - Normalisasi representasi string dan atribut spesies.
//! - Validasi dataset spesies sebelum pemrosesan.
//! - Validasi parameter kueri eksplorasi relasi.
//! - Validasi bobot scoring relasi.
//! - Pencarian referensi spesies pusat dari snapshot dataset.

#![allow(dead_code)]

use std::collections::HashSet;

use kalimantanbio_shared::core::{Species, Taxonomy};

use crate::types::{ExplorerError, RelationshipQuery, ScoreWeights, SpeciesId};

/// Membersihkan spasi berlebih pada teks (trim dan merapatkan spasi ganda)
/// tanpa mengubah kapitalisasi huruf.
fn sanitize_name(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Menormalisasi label: merapikan spasi dan mengubah ke huruf kecil.
fn normalize_label(s: &str) -> String {
    sanitize_name(s).to_lowercase()
}

/// Membuat salinan record [`Species`] baru dengan label atribut yang di-trim,
/// dikonversi ke huruf kecil, dan spasi berlebih dirapikan.
///
/// ID spesies dan kapitalisasi tampilan [`Species::scientific_name`] tetap dipertahankan.
pub(crate) fn normalize_species(species: &Species) -> Species {
    Species {
        id: species.id,
        scientific_name: sanitize_name(&species.scientific_name),
        common_name: normalize_label(&species.common_name),
        description: species.description.trim().to_string(),
        species_type: normalize_label(&species.species_type),
        iucn: normalize_label(&species.iucn),
        cites: normalize_label(&species.cites),
        p106: normalize_label(&species.p106),
        is_verified: species.is_verified,
        image_url: species.image_url.clone(),
        taxonomy: Taxonomy {
            kingdom: normalize_label(&species.taxonomy.kingdom),
            kingdom_id: species.taxonomy.kingdom_id,
            phylum_division: normalize_label(&species.taxonomy.phylum_division),
            phylum_division_id: species.taxonomy.phylum_division_id,
            class: normalize_label(&species.taxonomy.class),
            class_id: species.taxonomy.class_id,
            order: normalize_label(&species.taxonomy.order),
            order_id: species.taxonomy.order_id,
            family: normalize_label(&species.taxonomy.family),
            family_id: species.taxonomy.family_id,
            genus: normalize_label(&species.taxonomy.genus),
            genus_id: species.taxonomy.genus_id,
        },
        observation_count: species.observation_count,
        recorded_individuals_total: species.recorded_individuals_total,
        latest_observation_year: species.latest_observation_year,
        created_at: species.created_at.clone(),
        updated_at: species.updated_at.clone(),
    }
}

/// Menyiapkan dan memvalidasi seluruh dataset spesies.
///
/// Memastikan:
/// - ID setiap spesies unik dan lebih besar dari 0.
/// - Nama ilmiah tidak kosong setelah normalisasi.
/// - Seluruh record telah dinormalisasi secara konsisten.
///
/// Dataset kosong dianggap valid dan menghasilkan vektor kosong.
pub(crate) fn prepare_species(input: &[Species]) -> Result<Vec<Species>, ExplorerError> {
    if input.is_empty() {
        return Ok(Vec::new());
    }

    // Periksa keunikan ID secara fungsional
    let mut seen_ids = HashSet::with_capacity(input.len());
    let has_duplicate = input.iter().any(|s| !seen_ids.insert(s.id));
    if has_duplicate {
        return Err(ExplorerError::InvalidDataset(
            "Duplicate species ID found in dataset".to_string(),
        ));
    }

    // Normalisasi dan validasi tiap record spesies
    input
        .iter()
        .map(|s| {
            if s.id == 0 {
                return Err(ExplorerError::InvalidDataset(
                    "Species ID must be greater than 0".to_string(),
                ));
            }

            let normalized = normalize_species(s);
            if normalized.scientific_name.is_empty() {
                return Err(ExplorerError::InvalidDataset(format!(
                    "Species ID {} has an empty scientific name",
                    s.id
                )));
            }

            Ok(normalized)
        })
        .collect()
}

/// Memvalidasi parameter kueri relasi [`RelationshipQuery`].
///
/// Aturan validasi:
/// - `species_id` harus > 0.
/// - `min_score` harus bernilai finite dan dalam rentang `0.0..=1.0`.
/// - `limit` harus dalam rentang `1..=50`.
pub(crate) fn validate_query(query: &RelationshipQuery) -> Result<(), ExplorerError> {
    if query.species_id == 0 {
        return Err(ExplorerError::InvalidQuery(
            "species_id must be greater than 0".to_string(),
        ));
    }

    if !query.min_score.is_finite() || query.min_score < 0.0 || query.min_score > 1.0 {
        return Err(ExplorerError::InvalidQuery(
            "min_score must be a finite number between 0.0 and 1.0".to_string(),
        ));
    }

    if query.limit < 1 || query.limit > 50 {
        return Err(ExplorerError::InvalidQuery(
            "limit must be between 1 and 50".to_string(),
        ));
    }

    Ok(())
}

/// Memvalidasi konfigurasi bobot [`ScoreWeights`].
///
/// Memastikan:
/// - Semua bobot bernilai finite dan non-negatif (`>= 0.0`).
/// - Jumlah total bobot bernilai `1.0` dengan batas toleransi `1e-9`.
pub(crate) fn validate_weights(weights: &ScoreWeights) -> Result<(), ExplorerError> {
    let weights_slice = [weights.taxonomy, weights.habitat, weights.characteristic];

    let all_valid = weights_slice
        .iter()
        .all(|&w| w.is_finite() && w >= 0.0);

    if !all_valid {
        return Err(ExplorerError::InvalidWeights);
    }

    let sum: f64 = weights_slice.iter().sum();
    if (sum - 1.0).abs() > 1e-9 {
        return Err(ExplorerError::InvalidWeights);
    }

    Ok(())
}

/// Mencari referensi spesies pusat berdasarkan ID pada snapshot dataset.
///
/// Menghasilkan `Ok(&Species)` jika ditemukan, atau `Err(ExplorerError::SpeciesNotFound)` jika tidak ada.
pub(crate) fn find_species(
    species: &[Species],
    id: SpeciesId,
) -> Result<&Species, ExplorerError> {
    species
        .iter()
        .find(|s| s.id == id)
        .ok_or(ExplorerError::SpeciesNotFound(id))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_taxonomy() -> Taxonomy {
        Taxonomy {
            kingdom: "  Plantae  ".to_string(),
            kingdom_id: 1,
            phylum_division: " Tracheophyta ".to_string(),
            phylum_division_id: 2,
            class: "  Magnoliopsida  ".to_string(),
            class_id: 3,
            order: " Malvales ".to_string(),
            order_id: 4,
            family: " Dipterocarpaceae ".to_string(),
            family_id: 5,
            genus: " Shorea ".to_string(),
            genus_id: 6,
        }
    }

    fn sample_species(id: u64, name: &str) -> Species {
        Species {
            id,
            scientific_name: name.to_string(),
            common_name: "  Mersawa   Kuning ".to_string(),
            description: " Pohon kanopi hutan tropis ".to_string(),
            species_type: " NATIVE ".to_string(),
            iucn: " CR ".to_string(),
            cites: " APPENDIX I ".to_string(),
            p106: " Dilindungi ".to_string(),
            is_verified: true,
            image_url: None,
            taxonomy: sample_taxonomy(),
            observation_count: 10,
            recorded_individuals_total: 25,
            latest_observation_year: Some(2023),
            created_at: "2026-01-01T00:00:00Z".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
        }
    }

    #[test]
    fn test_normalize_species() {
        let sp = sample_species(1, "   Shorea    leprosula   ");
        let norm = normalize_species(&sp);

        // ID tidak berubah
        assert_eq!(norm.id, 1);
        // Scientific name dirapikan spasinya tapi kapitalisasi dipertahankan
        assert_eq!(norm.scientific_name, "Shorea leprosula");
        // Label diubah ke lowercase dan spasi dirapikan
        assert_eq!(norm.common_name, "mersawa kuning");
        assert_eq!(norm.species_type, "native");
        assert_eq!(norm.iucn, "cr");
        assert_eq!(norm.cites, "appendix i");
        assert_eq!(norm.p106, "dilindungi");
        // Taksonomi dinormalisasi
        assert_eq!(norm.taxonomy.genus, "shorea");
        assert_eq!(norm.taxonomy.family, "dipterocarpaceae");
        assert_eq!(norm.taxonomy.order, "malvales");
    }

    #[test]
    fn test_prepare_species_empty() {
        let result = prepare_species(&[]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[test]
    fn test_prepare_species_valid() {
        let list = vec![
            sample_species(1, "Shorea leprosula"),
            sample_species(2, "Dipterocarpus grandiflorus"),
        ];
        let result = prepare_species(&list);
        assert!(result.is_ok());
        let prepared = result.unwrap();
        assert_eq!(prepared.len(), 2);
        assert_eq!(prepared[0].id, 1);
        assert_eq!(prepared[1].id, 2);
    }

    #[test]
    fn test_prepare_species_duplicate_id() {
        let list = vec![
            sample_species(1, "Shorea leprosula"),
            sample_species(1, "Shorea laevis"),
        ];
        let result = prepare_species(&list);
        assert!(matches!(result, Err(ExplorerError::InvalidDataset(_))));
    }

    #[test]
    fn test_prepare_species_empty_scientific_name() {
        let list = vec![sample_species(1, "   ")];
        let result = prepare_species(&list);
        assert!(matches!(result, Err(ExplorerError::InvalidDataset(_))));
    }

    #[test]
    fn test_prepare_species_zero_id() {
        let list = vec![sample_species(0, "Shorea leprosula")];
        let result = prepare_species(&list);
        assert!(matches!(result, Err(ExplorerError::InvalidDataset(_))));
    }

    #[test]
    fn test_validate_query_valid() {
        let query = RelationshipQuery {
            species_id: 1,
            min_score: 0.20,
            limit: 10,
            required_basis: None,
        };
        assert!(validate_query(&query).is_ok());
    }

    #[test]
    fn test_validate_query_zero_id() {
        let query = RelationshipQuery {
            species_id: 0,
            min_score: 0.20,
            limit: 10,
            required_basis: None,
        };
        assert!(matches!(validate_query(&query), Err(ExplorerError::InvalidQuery(_))));
    }

    #[test]
    fn test_validate_query_invalid_scores() {
        // Negatif
        let q_neg = RelationshipQuery {
            species_id: 1,
            min_score: -0.01,
            limit: 10,
            required_basis: None,
        };
        assert!(matches!(validate_query(&q_neg), Err(ExplorerError::InvalidQuery(_))));

        // Di atas 1.0
        let q_high = RelationshipQuery {
            species_id: 1,
            min_score: 1.05,
            limit: 10,
            required_basis: None,
        };
        assert!(matches!(validate_query(&q_high), Err(ExplorerError::InvalidQuery(_))));

        // NaN
        let q_nan = RelationshipQuery {
            species_id: 1,
            min_score: f64::NAN,
            limit: 10,
            required_basis: None,
        };
        assert!(matches!(validate_query(&q_nan), Err(ExplorerError::InvalidQuery(_))));
    }

    #[test]
    fn test_validate_query_invalid_limits() {
        let q_zero = RelationshipQuery {
            species_id: 1,
            min_score: 0.20,
            limit: 0,
            required_basis: None,
        };
        assert!(matches!(validate_query(&q_zero), Err(ExplorerError::InvalidQuery(_))));

        let q_over = RelationshipQuery {
            species_id: 1,
            min_score: 0.20,
            limit: 51,
            required_basis: None,
        };
        assert!(matches!(validate_query(&q_over), Err(ExplorerError::InvalidQuery(_))));
    }

    #[test]
    fn test_validate_weights_valid() {
        let weights = ScoreWeights {
            taxonomy: 0.50,
            habitat: 0.30,
            characteristic: 0.20,
        };
        assert!(validate_weights(&weights).is_ok());
    }

    #[test]
    fn test_validate_weights_negative() {
        let weights = ScoreWeights {
            taxonomy: -0.10,
            habitat: 0.60,
            characteristic: 0.50,
        };
        assert_eq!(validate_weights(&weights), Err(ExplorerError::InvalidWeights));
    }

    #[test]
    fn test_validate_weights_nan() {
        let weights = ScoreWeights {
            taxonomy: f64::NAN,
            habitat: 0.50,
            characteristic: 0.50,
        };
        assert_eq!(validate_weights(&weights), Err(ExplorerError::InvalidWeights));
    }

    #[test]
    fn test_validate_weights_sum_mismatch() {
        let weights = ScoreWeights {
            taxonomy: 0.50,
            habitat: 0.30,
            characteristic: 0.30, // Total 1.10
        };
        assert_eq!(validate_weights(&weights), Err(ExplorerError::InvalidWeights));
    }

    #[test]
    fn test_find_species() {
        let list = vec![
            sample_species(10, "Shorea leprosula"),
            sample_species(20, "Dipterocarpus grandiflorus"),
        ];

        let found = find_species(&list, 10);
        assert!(found.is_ok());
        assert_eq!(found.unwrap().id, 10);

        let not_found = find_species(&list, 99);
        assert_eq!(not_found, Err(ExplorerError::SpeciesNotFound(99)));
    }
}
