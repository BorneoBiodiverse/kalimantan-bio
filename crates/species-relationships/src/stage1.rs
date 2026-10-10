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

use crate::types::ExplorerError;

/// Membersihkan spasi berlebih pada teks (trim dan merapatkan spasi ganda)
/// tanpa mengubah kapitalisasi huruf.
pub(crate) fn sanitize_name(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Menormalisasi label: merapikan spasi dan mengubah ke huruf kecil.
pub(crate) fn normalize_label(s: &str) -> String {
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
}
