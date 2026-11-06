//! Test fixtures and utilities for integration tests.

#![allow(dead_code)]

use std::collections::HashSet;

use kalimantanbio_shared::core::{Species, Taxonomy};
use species_relationships::types::ScoreWeights;

/// Helper to build a HashSet of strings.
pub fn set(values: &[&str]) -> HashSet<String> {
    values.iter().map(|value| (*value).to_string()).collect()
}

/// Helper to build a synthetic Taxonomy.
pub fn taxonomy(order: &str, family: &str, genus: &str) -> Taxonomy {
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

/// Helper to build a synthetic Species.
pub fn species(
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

/// Default scoring weights: taxonomy 0.50, habitat 0.30, characteristic 0.20.
pub fn default_weights() -> ScoreWeights {
    ScoreWeights::default()
}

/// Scenario 1: four species with complete data on all dimensions.
pub fn scenario_one() -> Vec<Species> {
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

/// Scenario 2: taxonomy present but habitat/characteristic empty.
pub fn scenario_two() -> Vec<Species> {
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
