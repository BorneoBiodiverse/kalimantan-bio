//! Relationship scoring and pair similarity calculation.
//!
//! Provides pure functions to:
//! - Score attribute evidence vectors against validated weights.
//! - Calculate relationship scores between pairs of species.

use std::collections::HashSet;

use kalimantanbio_shared::{
    core::Species, error::ValidationError, scoring::combine_weighted_scores,
    validation::validate_weights_sum_to_one,
};

use crate::evidence::{set_evidence, taxonomy_evidence, EVIDENCE_ORDER};
use crate::types::{AttributeEvidence, AttributeKind, RelationshipScore, ScoreWeights};

/// Menggabungkan bukti menjadi skor total dan coverage.
///
/// Prasyarat: `weights` sudah divalidasi (finite, non-negatif, jumlah `1.0`)
/// dan `evidence` memuat paling banyak satu entri per [`AttributeKind`] dalam
/// urutan kanonik. Dimensi tanpa bukti atau dengan `similarity` `None` tidak
/// menambah skor maupun coverage. Bila terjadi duplikat, kemunculan pertama
/// yang dipakai.
pub(crate) fn score_evidence(
    evidence: &[AttributeEvidence],
    weights: &ScoreWeights,
) -> RelationshipScore {
    let mut contributions: Vec<(f64, f64)> = Vec::with_capacity(EVIDENCE_ORDER.len());
    let mut coverage = 0.0_f64;

    for kind in EVIDENCE_ORDER {
        let similarity = evidence
            .iter()
            .find(|item| item.attribute == kind)
            .and_then(|item| item.similarity);

        let Some(similarity) = similarity else {
            continue;
        };

        let weight = weights.weight(kind);
        contributions.push((similarity, weight));
        coverage += weight;
    }

    RelationshipScore {
        total: combine_weighted_scores(&contributions),
        coverage,
        evidence: evidence.to_vec(),
    }
}

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
/// let weights = ScoreWeights::default();
///
/// let score = calculate_relationship(&a, &b, &habitats_a, &habitats_b, &weights)?;
/// assert!((score.total - 1.0).abs() < 1e-9);
/// # Ok::<(), kalimantanbio_shared::error::ValidationError>(())
/// ```
pub fn calculate_relationship(
    a: &Species,
    b: &Species,
    habitats_a: &HashSet<String>,
    habitats_b: &HashSet<String>,
    weights: &ScoreWeights,
) -> Result<RelationshipScore, ValidationError> {
    let weights_slice = [weights.taxonomy, weights.habitat, weights.characteristic];
    validate_weights_sum_to_one(&weights_slice, 1e-9)?;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_fixtures::default_weights;

    fn assert_close(actual: f64, expected: f64) {
        assert!(
            (actual - expected).abs() < 1e-9,
            "expected {expected}, got {actual}"
        );
    }

    #[test]
    fn score_evidence_matches_scenario_one() {
        let weights = default_weights();

        let evidence_ab = vec![
            AttributeEvidence {
                attribute: AttributeKind::Taxonomy,
                similarity: Some(1.0),
                shared_values: vec!["genus:genus-a".to_string()],
            },
            AttributeEvidence {
                attribute: AttributeKind::Habitat,
                similarity: Some(0.5),
                shared_values: vec!["wetland".to_string()],
            },
            AttributeEvidence {
                attribute: AttributeKind::Characteristic,
                similarity: Some(1.0),
                shared_values: vec!["woody".to_string(), "evergreen".to_string()],
            },
        ];
        let score_ab = score_evidence(&evidence_ab, &weights);
        assert_close(score_ab.total, 0.85);
        assert_close(score_ab.coverage, 1.0);

        let evidence_ac = vec![
            AttributeEvidence {
                attribute: AttributeKind::Taxonomy,
                similarity: Some(0.6),
                shared_values: vec!["family:family-a".to_string()],
            },
            AttributeEvidence {
                attribute: AttributeKind::Habitat,
                similarity: Some(0.5),
                shared_values: vec!["lowland".to_string()],
            },
            AttributeEvidence {
                attribute: AttributeKind::Characteristic,
                similarity: Some(0.5),
                shared_values: vec!["woody".to_string()],
            },
        ];
        let score_ac = score_evidence(&evidence_ac, &weights);
        assert_close(score_ac.total, 0.55);
        assert_close(score_ac.coverage, 1.0);
    }

    #[test]
    fn score_evidence_matches_scenario_two() {
        let weights = default_weights();

        let evidence_ae = vec![
            AttributeEvidence {
                attribute: AttributeKind::Taxonomy,
                similarity: Some(1.0),
                shared_values: vec!["genus:genus-a".to_string()],
            },
            AttributeEvidence {
                attribute: AttributeKind::Habitat,
                similarity: None,
                shared_values: Vec::new(),
            },
            AttributeEvidence {
                attribute: AttributeKind::Characteristic,
                similarity: None,
                shared_values: Vec::new(),
            },
        ];
        let score_ae = score_evidence(&evidence_ae, &weights);
        assert_close(score_ae.total, 0.50);
        assert_close(score_ae.coverage, 0.50);

        let evidence_af = vec![
            AttributeEvidence {
                attribute: AttributeKind::Taxonomy,
                similarity: None,
                shared_values: Vec::new(),
            },
            AttributeEvidence {
                attribute: AttributeKind::Habitat,
                similarity: None,
                shared_values: Vec::new(),
            },
            AttributeEvidence {
                attribute: AttributeKind::Characteristic,
                similarity: None,
                shared_values: Vec::new(),
            },
        ];
        let score_af = score_evidence(&evidence_af, &weights);
        assert_close(score_af.total, 0.0);
        assert_close(score_af.coverage, 0.0);
    }

    #[test]
    fn score_evidence_missing_dimension_does_not_add_coverage() {
        let weights = default_weights();
        let evidence = vec![AttributeEvidence {
            attribute: AttributeKind::Taxonomy,
            similarity: Some(1.0),
            shared_values: Vec::new(),
        }];

        let score = score_evidence(&evidence, &weights);

        assert_close(score.total, 0.5);
        assert_close(score.coverage, 0.5);
    }

    #[test]
    fn score_evidence_zero_weight_contributes_nothing() {
        let weights = ScoreWeights {
            taxonomy: 0.0,
            habitat: 0.6,
            characteristic: 0.4,
        };
        let evidence = vec![AttributeEvidence {
            attribute: AttributeKind::Taxonomy,
            similarity: Some(1.0),
            shared_values: Vec::new(),
        }];

        let score = score_evidence(&evidence, &weights);

        assert_close(score.total, 0.0);
        assert_close(score.coverage, 0.0);
    }

    #[test]
    fn score_evidence_duplicate_dimension_uses_first_occurrence() {
        let weights = default_weights();
        let evidence = vec![
            AttributeEvidence {
                attribute: AttributeKind::Taxonomy,
                similarity: Some(1.0),
                shared_values: Vec::new(),
            },
            AttributeEvidence {
                attribute: AttributeKind::Taxonomy,
                similarity: Some(0.0),
                shared_values: Vec::new(),
            },
        ];

        let score = score_evidence(&evidence, &weights);

        assert_close(score.total, 0.5);
        assert_close(score.coverage, 0.5);
    }
}
