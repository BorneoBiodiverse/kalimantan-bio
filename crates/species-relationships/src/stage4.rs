//! Tahap 4 — Pembentukan Data Species Network.
//!
//! Mengubah rekomendasi [`RelatedSpecies`] menjadi graf domain murni
//! ([`SpeciesNetwork`]) untuk API: satu node pusat + node tetangga unik +
//! edge pusat-tetangga. Tanpa koordinat layout, tanpa HTML/CSS.

#![allow(dead_code)]

use std::collections::HashSet;

use kalimantanbio_shared::core::Species;

use crate::types::{
    ExplorerResult, NetworkEdge, NetworkNode, RelatedSpecies, ScoreWeights, SpeciesId,
    SpeciesNetwork,
};

/// Versi aturan scoring yang dicatat pada hasil.
const SCORING_VERSION: &str = "relationship-v1";

/// Membuat node dengan ID stabil, label nama ilmiah, dan penanda pusat.
pub(crate) fn make_node(species: &Species, is_center: bool) -> NetworkNode {
    NetworkNode {
        id: species.id,
        label: species.scientific_name.clone(),
        is_center,
    }
}

/// Membuat edge pusat-tetangga berisi skor, coverage, bukti, dan penjelasan
/// yang sama dengan hasil discovery.
pub(crate) fn make_edge(center_id: SpeciesId, related: &RelatedSpecies) -> NetworkEdge {
    NetworkEdge {
        source: center_id,
        target: related.species.id,
        score: related.score.clone(),
        explanations: related.explanations.clone(),
    }
}

/// Menggabungkan satu node pusat, node tetangga unik, dan edge tanpa
/// self-loop atau duplikasi. Tidak ada edge antartetangga.
pub(crate) fn build_network(center: &Species, related: &[RelatedSpecies]) -> SpeciesNetwork {
    let mut nodes = Vec::with_capacity(related.len() + 1);
    let mut edges = Vec::with_capacity(related.len());
    let mut seen = HashSet::with_capacity(related.len() + 1);
    nodes.push(make_node(center, true));
    seen.insert(center.id);
    for item in related {
        let id = item.species.id;
        if id == center.id || !seen.insert(id) {
            continue;
        }
        nodes.push(make_node(&item.species, false));
        edges.push(make_edge(center.id, item));
    }
    SpeciesNetwork { nodes, edges }
}

/// Menyatukan daftar rekomendasi, graf, bobot, dan versi scoring.
pub(crate) fn assemble_result(
    center: &Species,
    related: Vec<RelatedSpecies>,
    weights: &ScoreWeights,
) -> ExplorerResult {
    ExplorerResult {
        center_id: center.id,
        network: build_network(center, &related),
        related,
        weights: weights.clone(),
        scoring_version: SCORING_VERSION.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{AttributeEvidence, AttributeKind, RelationshipScore};

    fn species(id: u64, name: &str) -> Species {
        serde_json::from_value(serde_json::json!({
            "id": id,
            "scientific_name": name,
            "common_name": "common",
            "description": "",
            "species_type": "native",
            "iucn": "-",
            "cites": "-",
            "p106": "-",
            "is_verified": true,
            "image_url": null,
            "taxonomy": {
                "kingdom": "k", "kingdom_id": 1,
                "phylum_division": "p", "phylum_division_id": 2,
                "class": "c", "class_id": 3,
                "order": "o", "order_id": 4,
                "family": "f", "family_id": 5,
                "genus": "g", "genus_id": 6
            },
            "observation_count": 0,
            "recorded_individuals_total": 0,
            "latest_observation_year": null,
            "created_at": "2026-01-01T00:00:00Z",
            "updated_at": "2026-01-01T00:00:00Z"
        }))
        .unwrap()
    }

    fn related(id: u64, total: f64) -> RelatedSpecies {
        RelatedSpecies {
            species: species(id, "Species Rel"),
            score: RelationshipScore {
                total,
                coverage: 1.0,
                evidence: vec![AttributeEvidence {
                    attribute: AttributeKind::Taxonomy,
                    similarity: Some(total),
                    shared_values: vec!["genus:g".to_string()],
                }],
            },
            explanations: vec!["shared genus".to_string()],
        }
    }

    fn weights() -> ScoreWeights {
        ScoreWeights {
            taxonomy: 0.5,
            habitat: 0.3,
            characteristic: 0.2,
        }
    }

    #[test]
    fn center_only_has_single_node() {
        let center = species(1, "Species Alpha");
        let network = build_network(&center, &[]);
        assert_eq!(network.nodes.len(), 1);
        assert!(network.nodes[0].is_center);
        assert!(network.edges.is_empty());
    }

    #[test]
    fn two_related_gives_three_nodes_two_edges() {
        let center = species(1, "Species Alpha");
        let related = vec![related(2, 0.85), related(3, 0.55)];
        let network = build_network(&center, &related);
        assert_eq!(network.nodes.len(), 3);
        assert_eq!(network.edges.len(), 2);
        assert_eq!(network.edges[0].source, 1);
        assert_eq!(network.edges[0].score.total, 0.85);
    }

    #[test]
    fn drops_self_loop_and_duplicates() {
        let center = species(1, "Species Alpha");
        let related = vec![related(1, 0.9), related(2, 0.8), related(2, 0.7)];
        let network = build_network(&center, &related);
        assert_eq!(network.nodes.len(), 2);
        assert_eq!(network.edges.len(), 1);
    }

    #[test]
    fn assemble_result_matches_plan_contract() {
        let center = species(1, "Species Alpha");
        let related = vec![related(2, 0.85)];
        let result = assemble_result(&center, related, &weights());
        assert_eq!(result.center_id, 1);
        assert_eq!(result.related.len(), 1);
        assert_eq!(result.network.nodes.len(), 2);
        assert_eq!(result.network.edges.len(), 1);
        assert_eq!(result.scoring_version, "relationship-v1");
    }
}
