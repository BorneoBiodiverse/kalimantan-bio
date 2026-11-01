//! Fixture sintetis untuk unit test Modul 2 (data fiktif, §15).
//!
//! Semua nama dan atribut pada modul ini sengaja dibuat-buat dan hanya
//! dipakai untuk memverifikasi aturan scoring, bukan data biologis nyata.

#![allow(dead_code)]

use std::collections::HashSet;

use kalimantanbio_shared::core::{Species, Taxonomy};

use crate::types::ScoreWeights;

/// Mengubah daftar label menjadi himpunan label yang telah dinormalisasi.
pub(crate) fn set(values: &[&str]) -> HashSet<String> {
    values.iter().map(|value| (*value).to_string()).collect()
}

/// Membuat taksonomi sintetis dengan label orde, famili, dan genus.
pub(crate) fn taxonomy(order: &str, family: &str, genus: &str) -> Taxonomy {
    Taxonomy {
        kingdom: "kingdom-a".to_string(),
        kingdom_id: 1,
        phylum_division: "phylum-a".to_string(),
        phylum_division_id: 2,
        class: "class-a".to_string(),
        class_id: 3,
        order: order.to_string(),
        order_id: 4,
        family: family.to_string(),
        family_id: 5,
        genus: genus.to_string(),
        genus_id: 6,
    }
}

/// Membuat record [`Species`] sintetis dengan atribut bukti yang diberikan.
pub(crate) fn species(
    id: u64,
    scientific_name: &str,
    order: &str,
    family: &str,
    genus: &str,
    habitats: &[&str],
    characteristics: &[&str],
) -> Species {
    Species {
        id,
        scientific_name: scientific_name.to_string(),
        common_name: format!("common {id}"),
        description: String::new(),
        species_type: "native".to_string(),
        iucn: "-".to_string(),
        cites: "-".to_string(),
        p106: "-".to_string(),
        is_verified: true,
        image_url: None,
        taxonomy: taxonomy(order, family, genus),
        habitats: set(habitats),
        characteristics: set(characteristics),
        observation_count: 0,
        recorded_individuals_total: 0,
        latest_observation_year: None,
        created_at: "2026-01-01T00:00:00Z".to_string(),
        updated_at: "2026-01-01T00:00:00Z".to_string(),
    }
}

/// Bobot scoring default §5: taksonomi `0.50`, habitat `0.30`, karakteristik `0.20`.
pub(crate) fn default_weights() -> ScoreWeights {
    ScoreWeights {
        taxonomy: 0.5,
        habitat: 0.3,
        characteristic: 0.2,
    }
}

/// Skenario 1 §15: empat spesies dengan data lengkap pada semua dimensi.
///
/// Urutan: `sp-a` (pusat), `sp-b`, `sp-c`, `sp-d`.
pub(crate) fn scenario_one() -> Vec<Species> {
    vec![
        species(
            1,
            "Species Alpha",
            "order-a",
            "family-a",
            "genus-a",
            &["wetland", "lowland"],
            &["woody", "evergreen"],
        ),
        species(
            2,
            "Species Beta",
            "order-a",
            "family-a",
            "genus-a",
            &["wetland"],
            &["woody", "evergreen"],
        ),
        species(
            3,
            "Species Gamma",
            "order-a",
            "family-a",
            "genus-b",
            &["lowland"],
            &["woody"],
        ),
        species(
            4,
            "Species Delta",
            "order-b",
            "family-b",
            "genus-c",
            &["montane"],
            &["herbaceous"],
        ),
    ]
}

/// Skenario 2 §15: taksonomi lengkap tetapi habitat/karakteristik kosong.
///
/// Urutan: `sp-a` (pusat), `sp-e`, `sp-f` (semua taksonomi kosong).
pub(crate) fn scenario_two() -> Vec<Species> {
    vec![
        species(
            1,
            "Species Alpha",
            "order-a",
            "family-a",
            "genus-a",
            &["wetland"],
            &["woody"],
        ),
        species(
            5,
            "Species Epsilon",
            "order-a",
            "family-a",
            "genus-a",
            &[],
            &[],
        ),
        species(6, "Species Zeta", "", "", "", &[], &[]),
    ]
}
