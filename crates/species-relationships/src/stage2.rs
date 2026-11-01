//! Tahap 2 — Ekstraksi Bukti dan Relationship Scoring.
//!
//! Modul ini menyediakan fungsi murni untuk:
//! - Mengekstrak bukti kesamaan taksonomi (orde, famili, genus).
//! - Mengekstrak bukti kesamaan habitat dan karakteristik terstruktur.
//! - Menyusun tepat satu bukti per dimensi untuk sepasang spesies.
//! - Menggabungkan similarity dan coverage memakai bobot yang telah divalidasi.
//!
//! Semua fungsi mengasumsikan data telah dinormalisasi oleh Tahap 1
//! (`prepare_species`): label di-trim, huruf kecil, dan nilai kosong dihapus.
//! Himpunan kosong (`None`) berarti data tidak tersedia untuk dibandingkan,
//! bukan berarti spesies tidak memiliki atribut tersebut.

#![allow(dead_code)]

use std::collections::HashSet;

use kalimantanbio_shared::{
    collections::{jaccard_similarity, set_intersection},
    core::{Species, Taxonomy},
    scoring::combine_weighted_scores,
    taxonomy::calculate_taxonomy_similarity,
};

use crate::types::{AttributeEvidence, AttributeKind, RelationshipScore, ScoreWeights};

/// Urutan dimensi bukti yang selalu dipakai pipeline.
const EVIDENCE_ORDER: [AttributeKind; 3] = [
    AttributeKind::Taxonomy,
    AttributeKind::Habitat,
    AttributeKind::Characteristic,
];

/// Mengekstrak bukti kesamaan taksonomi dari dua taksonomi yang dinormalisasi.
///
/// Hanya tingkat orde, famili, dan genus yang dibandingkan. Tingkat dianggap
/// dapat dibandingkan bila kedua label tidak kosong. `similarity` bernilai
/// `None` bila tidak ada tingkat yang dapat dibandingkan; jika ada, nilainya
/// berasal dari [`calculate_taxonomy_similarity`] (genus `1.0`, famili `0.6`,
/// orde `0.3`, atau `0.0` bila seluruh tingkat yang dibandingkan berbeda).
///
/// `shared_values` memuat seluruh tingkat yang cocok (bukan yang berbeda),
/// berurutan dari terdalam ke terluar: `genus:{g}`, `family:{f}`, `order:{o}`.
pub(crate) fn taxonomy_evidence(a: &Taxonomy, b: &Taxonomy) -> AttributeEvidence {
    let levels = [
        ("genus", &a.genus, &b.genus),
        ("family", &a.family, &b.family),
        ("order", &a.order, &b.order),
    ];

    let comparable = levels
        .iter()
        .any(|(_, left, right)| !left.is_empty() && !right.is_empty());

    let shared_values = levels
        .iter()
        .filter(|(_, left, right)| !left.is_empty() && left == right)
        .map(|(name, left, _)| format!("{name}:{left}"))
        .collect();

    let similarity = if comparable {
        Some(calculate_taxonomy_similarity(a, b))
    } else {
        None
    };

    AttributeEvidence {
        attribute: AttributeKind::Taxonomy,
        similarity,
        shared_values,
    }
}

/// Mengekstrak bukti kesamaan himpunan untuk habitat atau karakteristik.
///
/// Bila salah satu himpunan kosong, data dianggap tidak tersedia dan
/// `similarity` bernilai `None`. Bila keduanya terisi, `similarity` adalah
/// Jaccard similarity dan `shared_values` memuat irisan yang terurut secara
/// leksikografis agar hasil deterministik.
pub(crate) fn set_evidence(
    kind: AttributeKind,
    a: &HashSet<String>,
    b: &HashSet<String>,
) -> AttributeEvidence {
    if a.is_empty() || b.is_empty() {
        return AttributeEvidence {
            attribute: kind,
            similarity: None,
            shared_values: Vec::new(),
        };
    }

    let mut shared_values: Vec<String> = set_intersection(a, b).into_iter().collect();
    shared_values.sort();

    AttributeEvidence {
        attribute: kind,
        similarity: Some(jaccard_similarity(a, b)),
        shared_values,
    }
}

/// Menghasilkan tepat satu bukti per dimensi untuk sepasang spesies.
///
/// Urutan bukti selalu taksonomi, habitat, lalu karakteristik. Habitat dan
/// karakteristik dibaca dari himpunan kanonik pada [`Species`].
pub(crate) fn collect_evidence(a: &Species, b: &Species) -> Vec<AttributeEvidence> {
    vec![
        taxonomy_evidence(&a.taxonomy, &b.taxonomy),
        set_evidence(AttributeKind::Habitat, &a.habitats, &b.habitats),
        set_evidence(
            AttributeKind::Characteristic,
            &a.characteristics,
            &b.characteristics,
        ),
    ]
}

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

        let weight = weight_for(kind, weights);
        contributions.push((similarity, weight));
        coverage += weight;
    }

    RelationshipScore {
        total: combine_weighted_scores(&contributions),
        coverage,
        evidence: evidence.to_vec(),
    }
}

/// Mengambil bobot untuk satu dimensi bukti.
fn weight_for(kind: AttributeKind, weights: &ScoreWeights) -> f64 {
    match kind {
        AttributeKind::Taxonomy => weights.taxonomy,
        AttributeKind::Habitat => weights.habitat,
        AttributeKind::Characteristic => weights.characteristic,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_fixtures::{default_weights, scenario_one, scenario_two, set, taxonomy};

    fn assert_close(actual: f64, expected: f64) {
        assert!(
            (actual - expected).abs() < 1e-9,
            "expected {expected}, got {actual}"
        );
    }

    fn evidence_of(evidence: &[AttributeEvidence], kind: AttributeKind) -> &AttributeEvidence {
        evidence
            .iter()
            .find(|item| item.attribute == kind)
            .expect("evidence dimension must exist")
    }

    #[test]
    fn taxonomy_evidence_uses_deepest_match() {
        let a = taxonomy("order-a", "family-a", "genus-a");
        let b = taxonomy("order-a", "family-a", "genus-a");

        let evidence = taxonomy_evidence(&a, &b);

        assert_eq!(evidence.attribute, AttributeKind::Taxonomy);
        assert_eq!(evidence.similarity, Some(1.0));
        assert_eq!(
            evidence.shared_values,
            vec!["genus:genus-a", "family:family-a", "order:order-a"]
        );
    }

    #[test]
    fn taxonomy_evidence_family_match_without_genus() {
        let a = taxonomy("order-a", "family-a", "genus-a");
        let b = taxonomy("order-a", "family-a", "genus-b");

        let evidence = taxonomy_evidence(&a, &b);

        assert_eq!(evidence.similarity, Some(0.6));
        assert_eq!(
            evidence.shared_values,
            vec!["family:family-a", "order:order-a"]
        );
    }

    #[test]
    fn taxonomy_evidence_order_match_only() {
        let a = taxonomy("order-a", "family-a", "genus-a");
        let b = taxonomy("order-a", "family-b", "genus-b");

        let evidence = taxonomy_evidence(&a, &b);

        assert_eq!(evidence.similarity, Some(0.3));
        assert_eq!(evidence.shared_values, vec!["order:order-a"]);
    }

    #[test]
    fn taxonomy_evidence_comparable_but_different() {
        let a = taxonomy("order-a", "family-a", "genus-a");
        let b = taxonomy("order-b", "family-b", "genus-b");

        let evidence = taxonomy_evidence(&a, &b);

        assert_eq!(evidence.similarity, Some(0.0));
        assert!(evidence.shared_values.is_empty());
    }

    #[test]
    fn taxonomy_evidence_incomparable_levels_are_none() {
        let a = taxonomy("", "", "");
        let b = taxonomy("", "", "");

        let evidence = taxonomy_evidence(&a, &b);

        assert_eq!(evidence.similarity, None);
        assert!(evidence.shared_values.is_empty());
    }

    #[test]
    fn taxonomy_evidence_unknown_genus_does_not_claim_genus_differs() {
        let a = taxonomy("order-a", "family-a", "genus-a");
        let b = taxonomy("order-a", "family-a", "");

        let evidence = taxonomy_evidence(&a, &b);

        assert_eq!(evidence.similarity, Some(0.6));
        assert_eq!(
            evidence.shared_values,
            vec!["family:family-a", "order:order-a"]
        );
    }

    #[test]
    fn taxonomy_evidence_is_symmetric() {
        let a = taxonomy("order-a", "family-a", "genus-a");
        let b = taxonomy("order-a", "family-b", "genus-b");

        let forward = taxonomy_evidence(&a, &b);
        let backward = taxonomy_evidence(&b, &a);

        assert_eq!(forward.similarity, backward.similarity);
        assert_eq!(forward.shared_values, backward.shared_values);
    }

    #[test]
    fn set_evidence_computes_jaccard_and_shared_values() {
        let a = set(&["wetland", "lowland"]);
        let b = set(&["wetland"]);

        let evidence = set_evidence(AttributeKind::Habitat, &a, &b);

        assert_eq!(evidence.attribute, AttributeKind::Habitat);
        assert_eq!(evidence.similarity, Some(0.5));
        assert_eq!(evidence.shared_values, vec!["wetland"]);
    }

    #[test]
    fn set_evidence_disjoint_is_zero_not_none() {
        let a = set(&["wetland"]);
        let b = set(&["montane"]);

        let evidence = set_evidence(AttributeKind::Habitat, &a, &b);

        assert_eq!(evidence.similarity, Some(0.0));
        assert!(evidence.shared_values.is_empty());
    }

    #[test]
    fn set_evidence_empty_side_is_unavailable() {
        let a = set(&["wetland"]);
        let empty = set(&[]);

        assert_eq!(
            set_evidence(AttributeKind::Habitat, &a, &empty).similarity,
            None
        );
        assert_eq!(
            set_evidence(AttributeKind::Habitat, &empty, &a).similarity,
            None
        );
        assert_eq!(
            set_evidence(AttributeKind::Habitat, &empty, &empty).similarity,
            None
        );
    }

    #[test]
    fn set_evidence_shared_values_are_sorted() {
        let a = set(&["wetland", "lowland"]);
        let b = set(&["lowland", "wetland"]);

        let evidence = set_evidence(AttributeKind::Habitat, &a, &b);

        assert_eq!(evidence.shared_values, vec!["lowland", "wetland"]);
    }

    #[test]
    fn collect_evidence_returns_one_evidence_per_dimension_in_order() {
        let species = scenario_one();
        let evidence = collect_evidence(&species[0], &species[1]);

        assert_eq!(evidence.len(), 3);
        assert_eq!(evidence[0].attribute, AttributeKind::Taxonomy);
        assert_eq!(evidence[1].attribute, AttributeKind::Habitat);
        assert_eq!(evidence[2].attribute, AttributeKind::Characteristic);
        assert_eq!(evidence[0].similarity, Some(1.0));
        assert_eq!(evidence[1].similarity, Some(0.5));
        assert_eq!(evidence[2].similarity, Some(1.0));
    }

    #[test]
    fn score_evidence_matches_scenario_one() {
        let species = scenario_one();
        let weights = default_weights();

        let sp_b = score_evidence(&collect_evidence(&species[0], &species[1]), &weights);
        assert_close(sp_b.total, 0.85);
        assert_close(sp_b.coverage, 1.0);

        let sp_c = score_evidence(&collect_evidence(&species[0], &species[2]), &weights);
        assert_close(sp_c.total, 0.55);
        assert_close(sp_c.coverage, 1.0);

        let sp_d = score_evidence(&collect_evidence(&species[0], &species[3]), &weights);
        assert_close(sp_d.total, 0.0);
        assert_close(sp_d.coverage, 1.0);
    }

    #[test]
    fn score_evidence_matches_scenario_two() {
        let species = scenario_two();
        let weights = default_weights();

        let sp_e = score_evidence(&collect_evidence(&species[0], &species[1]), &weights);
        assert_close(sp_e.total, 0.5);
        assert_close(sp_e.coverage, 0.5);
        assert_eq!(
            evidence_of(&sp_e.evidence, AttributeKind::Habitat).similarity,
            None
        );
        assert_eq!(
            evidence_of(&sp_e.evidence, AttributeKind::Characteristic).similarity,
            None
        );

        let sp_f = score_evidence(&collect_evidence(&species[0], &species[2]), &weights);
        assert_close(sp_f.total, 0.0);
        assert_close(sp_f.coverage, 0.0);
    }

    #[test]
    fn score_evidence_missing_dimension_does_not_add_coverage() {
        let a = taxonomy("order-a", "family-a", "genus-a");
        let b = taxonomy("order-a", "family-a", "genus-a");
        let evidence = vec![taxonomy_evidence(&a, &b)];

        let score = score_evidence(&evidence, &default_weights());

        assert_close(score.total, 0.5);
        assert_close(score.coverage, 0.5);
    }

    #[test]
    fn score_evidence_zero_weight_contributes_nothing() {
        let species = scenario_one();
        let weights = ScoreWeights {
            taxonomy: 1.0,
            habitat: 0.0,
            characteristic: 0.0,
        };

        let score = score_evidence(&collect_evidence(&species[0], &species[1]), &weights);

        assert_close(score.total, 1.0);
        assert_close(score.coverage, 1.0);
    }

    #[test]
    fn score_evidence_duplicate_dimension_uses_first_occurrence() {
        let a = taxonomy("order-a", "family-a", "genus-a");
        let b = taxonomy("order-a", "family-a", "genus-a");
        let mut first = taxonomy_evidence(&a, &b);
        first.similarity = Some(1.0);
        let mut second = first.clone();
        second.similarity = Some(0.0);
        let evidence = vec![first, second];

        let score = score_evidence(&evidence, &default_weights());

        assert_close(score.total, 0.5);
        assert_close(score.coverage, 0.5);
    }
}
