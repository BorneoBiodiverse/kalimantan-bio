//! Integration tests for the Species Relationship Explorer pipeline.

mod common;

use common::{default_weights, scenario_one, scenario_two, set};
use species_relationships::calculate_relationship;
use species_relationships::error::ExplorerError;
use species_relationships::explore_relationships;
use species_relationships::types::{
    AttributeEvidence, AttributeKind, RelationshipQuery, RelationshipScore, ScoreWeights,
};

fn assert_close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() < 1e-9,
        "expected {expected}, got {actual}"
    );
}

fn evidence_of(score: &RelationshipScore, kind: AttributeKind) -> &AttributeEvidence {
    score
        .evidence
        .iter()
        .find(|item| item.attribute == kind)
        .expect("evidence dimension must exist")
}

#[test]
fn calculates_scenario_one_pair_scores() {
    let species = scenario_one();
    let a = &species[0];
    let b = &species[1];
    let c = &species[2];
    let d = &species[3];
    let weights = default_weights();

    let score_ab = calculate_relationship(a, b, &a.habitats, &b.habitats, &weights).unwrap();
    assert_close(score_ab.total, 0.85);
    assert_close(score_ab.coverage, 1.0);
    assert_eq!(
        evidence_of(&score_ab, AttributeKind::Taxonomy).shared_values,
        vec![
            "genus:genus-a".to_string(),
            "family:family-a".to_string(),
            "order:order-a".to_string(),
        ]
    );

    let score_ac = calculate_relationship(a, c, &a.habitats, &c.habitats, &weights).unwrap();
    assert_close(score_ac.total, 0.55);
    assert_close(score_ac.coverage, 1.0);

    let score_ad = calculate_relationship(a, d, &a.habitats, &d.habitats, &weights).unwrap();
    assert_close(score_ad.total, 0.0);
    assert_close(score_ad.coverage, 1.0);
}

#[test]
fn calculates_scenario_two_pair_scores() {
    let species = scenario_two();
    let a = &species[0];
    let e = &species[1];
    let f = &species[2];
    let weights = default_weights();

    let score_ae = calculate_relationship(a, e, &a.habitats, &e.habitats, &weights).unwrap();
    assert_close(score_ae.total, 0.50);
    assert_close(score_ae.coverage, 0.50);
    assert_eq!(
        evidence_of(&score_ae, AttributeKind::Habitat).similarity,
        None
    );

    let score_af = calculate_relationship(a, f, &a.habitats, &f.habitats, &weights).unwrap();
    assert_close(score_af.total, 0.0);
    assert_close(score_af.coverage, 0.0);
}

#[test]
fn rejects_invalid_weights() {
    let species = scenario_one();
    let bad = ScoreWeights {
        taxonomy: 0.5,
        habitat: 0.5,
        characteristic: 0.5,
    };
    let result = calculate_relationship(
        &species[0],
        &species[1],
        &species[0].habitats,
        &species[1].habitats,
        &bad,
    );
    assert!(result.is_err());
}

#[test]
fn explicit_habitats_override_species_fields() {
    let species = scenario_one();
    let a = &species[0];
    let b = &species[1];
    let weights = default_weights();

    let disjoint = set(&["montane"]);
    let shared = calculate_relationship(a, b, &a.habitats, &b.habitats, &weights).unwrap();
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
fn pipeline_rejects_invalid_query_before_discovery() {
    let query = RelationshipQuery {
        species_id: 0,
        min_score: 0.0,
        limit: 10,
        required_basis: None,
    };
    let result = explore_relationships(&scenario_one(), &query, &default_weights());
    assert!(matches!(result, Err(ExplorerError::InvalidQuery(_))));
}

#[test]
fn pipeline_rejects_invalid_weights_before_discovery() {
    let query = RelationshipQuery {
        species_id: 1,
        min_score: 0.0,
        limit: 10,
        required_basis: None,
    };
    let bad = ScoreWeights {
        taxonomy: 0.5,
        habitat: 0.5,
        characteristic: 0.5,
    };
    let result = explore_relationships(&scenario_one(), &query, &bad);
    assert!(matches!(result, Err(ExplorerError::InvalidWeights)));
}

#[test]
fn pipeline_reports_unknown_center_before_discovery() {
    let query = RelationshipQuery {
        species_id: 999,
        min_score: 0.0,
        limit: 10,
        required_basis: None,
    };
    let result = explore_relationships(&scenario_one(), &query, &default_weights());
    assert!(matches!(result, Err(ExplorerError::SpeciesNotFound(999))));
}

#[test]
fn pipeline_reports_invalid_dataset_before_discovery() {
    let mut dataset = scenario_one();
    dataset.push(dataset[0].clone());
    let query = RelationshipQuery {
        species_id: 1,
        min_score: 0.0,
        limit: 10,
        required_basis: None,
    };
    let result = explore_relationships(&dataset, &query, &default_weights());
    assert!(matches!(result, Err(ExplorerError::InvalidDataset(_))));
}

#[test]
fn pipeline_composes_discovery_into_network() {
    let query = RelationshipQuery {
        species_id: 1,
        min_score: 0.0,
        limit: 10,
        required_basis: None,
    };
    let result = explore_relationships(&scenario_one(), &query, &default_weights())
        .expect("valid query over scenario one must succeed");

    assert_eq!(result.center_id, 1);
    assert_eq!(result.scoring_version, "relationship-v1");
    assert!(!result.related.is_empty(), "scenario one must rank peers");
    assert_eq!(result.network.nodes.len(), result.related.len() + 1);
    assert_eq!(result.network.edges.len(), result.related.len());
    assert_eq!(
        result
            .network
            .nodes
            .iter()
            .filter(|node| node.is_center)
            .count(),
        1
    );
    assert!(result
        .network
        .nodes
        .iter()
        .any(|node| node.id == 1 && node.is_center));
    assert!(result
        .network
        .edges
        .iter()
        .all(|edge| edge.source == 1 && edge.target != 1));
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
