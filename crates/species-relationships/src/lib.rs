#![warn(missing_docs)]
//! Module 2 — Species Relationship Explorer.
//!
//! Discovers and explains relationships between species: related-species
//! discovery, relationship scoring, explanations, and network data generation.
//!
//! Public API: `explore_relationships` and `calculate_relationship`. Internal
//! helpers (evidence extraction, scoring, explanation, network building) stay
//! private.
//!
//! > **Status: partial implementation.** Stage 2 (evidence extraction and
//! > relationship scoring) is implemented; the exploration pipeline and the
//! > network stages are pending.

pub(crate) mod stage1;
pub(crate) mod stage2;
pub(crate) mod stage4;
pub mod types;

#[cfg(test)]
pub(crate) mod test_fixtures;

use std::collections::HashSet;

use kalimantanbio_shared::{
    core::Species, error::ValidationError, validation::validate_weights_sum_to_one,
};

use crate::stage2::{score_evidence, set_evidence, taxonomy_evidence};
use crate::types::{
    AttributeKind, ExplorerError, ExplorerResult, RelationshipQuery, RelationshipScore,
    ScoreWeights,
};

pub use crate::types::SpeciesId;

/// Calculate the relationship score between two species.
///
/// Combines taxonomy, habitat, and characteristic evidence into a single score
/// using validated weights (`S = w_t·T + w_h·H + w_c·C`). Dimensions whose
/// data is unavailable on either side (`None`) contribute neither score nor
/// coverage; comparable-but-different dimensions contribute `0.0`.
///
/// Taxonomy and characteristic evidence are read from the species records.
/// Habitat labels are taken from the explicit `habitats_a`/`habitats_b`
/// arguments so callers can score against a repository snapshot. All labels
/// are expected to be normalized (trimmed, lowercased, empty values removed)
/// as produced by the data preparation stage.
///
/// # Arguments
///
/// * `a` - First species (normalized).
/// * `b` - Second species (normalized).
/// * `habitats_a` - Normalized habitat labels of the first species.
/// * `habitats_b` - Normalized habitat labels of the second species.
/// * `weights` - Weighted contribution of each evidence basis; all values must
///   be finite and non-negative and sum to `1.0` within tolerance `1e-9`.
///
/// # Returns
///
/// The relationship score with exactly one evidence entry per dimension in the
/// order taxonomy, habitat, characteristic. `total` and `coverage` lie in
/// `0.0..=1.0` and are stored without rounding.
///
/// # Errors
///
/// Returns [`ValidationError::NonFiniteWeight`] for NaN/infinite weights,
/// [`ValidationError::NegativeWeight`] for negative weights, and
/// [`ValidationError::WeightsSumMismatch`] when the weights do not sum to
/// `1.0` within tolerance. Scoring is not performed when validation fails.
///
/// # Example
///
/// ```
/// use std::collections::HashSet;
///
/// use kalimantanbio_shared::core::Species;
/// use species_relationships::calculate_relationship;
/// use species_relationships::types::ScoreWeights;
///
/// # fn species(id: u64, genus: &str, habitat: &str) -> Species {
/// #     serde_json::from_value(serde_json::json!({
/// #         "id": id,
/// #         "scientific_name": "Species synthetica",
/// #         "common_name": "Synthetic species",
/// #         "description": "",
/// #         "species_type": "native",
/// #         "iucn": "-",
/// #         "cites": "-",
/// #         "p106": "-",
/// #         "is_verified": true,
/// #         "image_url": null,
/// #         "taxonomy": {
/// #             "kingdom": "animalia",
/// #             "kingdom_id": 1,
/// #             "phylum_division": "chordata",
/// #             "phylum_division_id": 2,
/// #             "class": "mammalia",
/// #             "class_id": 3,
/// #             "order": "primates",
/// #             "order_id": 4,
/// #             "family": "hominidae",
/// #             "family_id": 5,
/// #             "genus": genus,
/// #             "genus_id": 6
/// #         },
/// #         "habitats": [habitat],
/// #         "characteristics": ["woody"],
/// #         "observation_count": 0,
/// #         "recorded_individuals_total": 0,
/// #         "latest_observation_year": null,
/// #         "created_at": "2026-01-01T00:00:00Z",
/// #         "updated_at": "2026-01-01T00:00:00Z"
/// #     }))
/// #     .unwrap()
/// # }
/// let a = species(1, "pongo", "wetland");
/// let b = species(2, "pongo", "wetland");
/// let habitats_a: HashSet<String> = ["wetland".to_string()].into_iter().collect();
/// let habitats_b = habitats_a.clone();
///
/// let weights = ScoreWeights {
///     taxonomy: 0.5,
///     habitat: 0.3,
///     characteristic: 0.2,
/// };
///
/// let score = calculate_relationship(&a, &b, &habitats_a, &habitats_b, &weights)?;
/// assert!((score.total - 1.0).abs() < 1e-9);
/// # Ok::<(), kalimantanbio_shared::ValidationError>(())
/// ```
pub fn calculate_relationship(
    a: &Species,
    b: &Species,
    habitats_a: &HashSet<String>,
    habitats_b: &HashSet<String>,
    weights: &ScoreWeights,
) -> Result<RelationshipScore, ValidationError> {
    validate_weights_sum_to_one(
        &[weights.taxonomy, weights.habitat, weights.characteristic],
        1e-9,
    )?;

    let evidence = vec![
        taxonomy_evidence(&a.taxonomy, &b.taxonomy),
        set_evidence(AttributeKind::Habitat, habitats_a, habitats_b),
        set_evidence(
            AttributeKind::Characteristic,
            &a.characteristics,
            &b.characteristics,
        ),
    ];

    Ok(score_evidence(&evidence, weights))
}

/// Explore all relationships around a center species.
///
/// Pipeline: prepare → validate → discover candidates → score → explain →
/// rank → build network → assemble result.
///
/// # Arguments
///
/// * `input` - The full species dataset.
/// * `query` - Center species, minimum score, and limit.
/// * `weights` - Scoring weights.
///
/// # Returns
///
/// Related species with scores, explanations, and network data.
///
/// # Example
///
/// ```rust,ignore
/// let result = explore_relationships(&species, &query, &weights)?;
/// ```
pub fn explore_relationships(
    input: &[Species],
    query: &RelationshipQuery,
    weights: &ScoreWeights,
) -> Result<ExplorerResult, ExplorerError> {
    todo!("species-relationships phase 5: explore_relationships")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_fixtures::{default_weights, scenario_one, scenario_two};

    fn assert_close(actual: f64, expected: f64) {
        assert!(
            (actual - expected).abs() < 1e-9,
            "expected {expected}, got {actual}"
        );
    }

    fn evidence_of(
        score: &RelationshipScore,
        kind: AttributeKind,
    ) -> &crate::types::AttributeEvidence {
        score
            .evidence
            .iter()
            .find(|item| item.attribute == kind)
            .expect("evidence dimension must exist")
    }

    #[test]
    fn calculates_scenario_one_pair_scores() {
        let species = scenario_one();
        let weights = default_weights();

        let sp_b = calculate_relationship(
            &species[0],
            &species[1],
            &species[0].habitats,
            &species[1].habitats,
            &weights,
        )
        .unwrap();
        assert_close(sp_b.total, 0.85);
        assert_close(sp_b.coverage, 1.0);

        let sp_c = calculate_relationship(
            &species[0],
            &species[2],
            &species[0].habitats,
            &species[2].habitats,
            &weights,
        )
        .unwrap();
        assert_close(sp_c.total, 0.55);
        assert_close(sp_c.coverage, 1.0);

        let sp_d = calculate_relationship(
            &species[0],
            &species[3],
            &species[0].habitats,
            &species[3].habitats,
            &weights,
        )
        .unwrap();
        assert_close(sp_d.total, 0.0);
        assert_close(sp_d.coverage, 1.0);
    }

    #[test]
    fn calculates_scenario_two_pair_scores() {
        let species = scenario_two();
        let weights = default_weights();

        let sp_e = calculate_relationship(
            &species[0],
            &species[1],
            &species[0].habitats,
            &species[1].habitats,
            &weights,
        )
        .unwrap();
        assert_close(sp_e.total, 0.5);
        assert_close(sp_e.coverage, 0.5);

        let sp_f = calculate_relationship(
            &species[0],
            &species[2],
            &species[0].habitats,
            &species[2].habitats,
            &weights,
        )
        .unwrap();
        assert_close(sp_f.total, 0.0);
        assert_close(sp_f.coverage, 0.0);
    }

    #[test]
    fn rejects_invalid_weights() {
        let species = scenario_one();
        let a = &species[0];
        let b = &species[1];
        let habitats_a = &a.habitats;
        let habitats_b = &b.habitats;

        let sum_mismatch = ScoreWeights {
            taxonomy: 0.5,
            habitat: 0.4,
            characteristic: 0.2,
        };
        assert!(matches!(
            calculate_relationship(a, b, habitats_a, habitats_b, &sum_mismatch),
            Err(ValidationError::WeightsSumMismatch { .. })
        ));

        let negative = ScoreWeights {
            taxonomy: 1.1,
            habitat: -0.1,
            characteristic: 0.0,
        };
        assert!(matches!(
            calculate_relationship(a, b, habitats_a, habitats_b, &negative),
            Err(ValidationError::NegativeWeight)
        ));

        let non_finite = ScoreWeights {
            taxonomy: f64::NAN,
            habitat: 0.5,
            characteristic: 0.5,
        };
        assert!(matches!(
            calculate_relationship(a, b, habitats_a, habitats_b, &non_finite),
            Err(ValidationError::NonFiniteWeight)
        ));
    }

    #[test]
    fn explicit_habitats_override_species_fields() {
        let species = scenario_one();
        let a = &species[0];
        let b = &species[1];
        let weights = default_weights();

        let shared = calculate_relationship(a, b, &a.habitats, &b.habitats, &weights).unwrap();
        let disjoint: HashSet<String> = ["montane".to_string()].into_iter().collect();
        let overridden = calculate_relationship(a, b, &a.habitats, &disjoint, &weights).unwrap();

        assert_close(overridden.total, 0.7);
        assert!(overridden.total < shared.total);
    }

    #[test]
    fn score_is_symmetric() {
        let species = scenario_one();
        let weights = default_weights();

        let forward = calculate_relationship(
            &species[0],
            &species[2],
            &species[0].habitats,
            &species[2].habitats,
            &weights,
        )
        .unwrap();
        let backward = calculate_relationship(
            &species[2],
            &species[0],
            &species[2].habitats,
            &species[0].habitats,
            &weights,
        )
        .unwrap();

        assert_close(forward.total, backward.total);
        assert_close(forward.coverage, backward.coverage);
    }

    #[test]
    fn characteristic_evidence_comes_from_species_fields() {
        let species = scenario_one();
        let weights = default_weights();

        let disjoint = calculate_relationship(
            &species[0],
            &species[3],
            &species[0].habitats,
            &species[3].habitats,
            &weights,
        )
        .unwrap();
        assert_eq!(
            evidence_of(&disjoint, AttributeKind::Characteristic).similarity,
            Some(0.0)
        );

        let scenario = scenario_two();
        let unavailable = calculate_relationship(
            &scenario[0],
            &scenario[1],
            &scenario[0].habitats,
            &scenario[1].habitats,
            &weights,
        )
        .unwrap();
        assert_eq!(
            evidence_of(&unavailable, AttributeKind::Characteristic).similarity,
            None
        );
    }
}
