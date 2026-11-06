//! Evidence extraction for species relationships.
//!
//! Provides pure functions to:
//! - Extract taxonomy similarity evidence across taxonomic ranks.
//! - Extract set similarity evidence for habitats and characteristics.
//! - Assemble complete evidence vectors for pairs of species.

use std::collections::HashSet;

use kalimantanbio_shared::{
    collections::{jaccard_similarity, set_intersection},
    core::{Species, Taxonomy},
    taxonomy::calculate_taxonomy_similarity,
};

use crate::types::{AttributeEvidence, AttributeKind};

/// Canonical order of evidence evaluation.
pub(crate) const EVIDENCE_ORDER: [AttributeKind; 3] = [
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_fixtures::{set, taxonomy};

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
            vec![
                "genus:genus-a".to_string(),
                "family:family-a".to_string(),
                "order:order-a".to_string()
            ]
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
            vec!["family:family-a".to_string(), "order:order-a".to_string()]
        );
    }

    #[test]
    fn taxonomy_evidence_order_match_only() {
        let a = taxonomy("order-a", "family-a", "genus-a");
        let b = taxonomy("order-a", "family-b", "genus-b");

        let evidence = taxonomy_evidence(&a, &b);

        assert_eq!(evidence.similarity, Some(0.3));
        assert_eq!(evidence.shared_values, vec!["order:order-a".to_string()]);
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
        let a = taxonomy("order-a", "family-a", "");
        let b = taxonomy("order-a", "family-a", "genus-b");

        let evidence = taxonomy_evidence(&a, &b);

        assert_eq!(evidence.similarity, Some(0.6));
        assert_eq!(
            evidence.shared_values,
            vec!["family:family-a".to_string(), "order:order-a".to_string()]
        );
    }

    #[test]
    fn taxonomy_evidence_is_symmetric() {
        let a = taxonomy("order-a", "family-a", "genus-a");
        let b = taxonomy("order-a", "family-a", "genus-b");

        assert_eq!(taxonomy_evidence(&a, &b), taxonomy_evidence(&b, &a));
    }

    #[test]
    fn set_evidence_computes_jaccard_and_shared_values() {
        let a = set(&["wetland", "lowland"]);
        let b = set(&["wetland", "montane"]);

        let evidence = set_evidence(AttributeKind::Habitat, &a, &b);

        assert_eq!(evidence.attribute, AttributeKind::Habitat);
        assert!((evidence.similarity.unwrap() - (1.0 / 3.0)).abs() < 1e-9);
        assert_eq!(evidence.shared_values, vec!["wetland".to_string()]);
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
        let populated = set(&["wetland"]);
        let empty = HashSet::new();

        let left_empty = set_evidence(AttributeKind::Habitat, &empty, &populated);
        let right_empty = set_evidence(AttributeKind::Habitat, &populated, &empty);
        let both_empty = set_evidence(AttributeKind::Habitat, &empty, &empty);

        assert_eq!(left_empty.similarity, None);
        assert_eq!(right_empty.similarity, None);
        assert_eq!(both_empty.similarity, None);
        assert!(left_empty.shared_values.is_empty());
        assert!(right_empty.shared_values.is_empty());
        assert!(both_empty.shared_values.is_empty());
    }

    #[test]
    fn set_evidence_shared_values_are_sorted() {
        let a = set(&["wetland", "forest", "alpine"]);
        let b = set(&["forest", "alpine", "wetland"]);

        let evidence = set_evidence(AttributeKind::Habitat, &a, &b);

        assert_eq!(
            evidence.shared_values,
            vec![
                "alpine".to_string(),
                "forest".to_string(),
                "wetland".to_string()
            ]
        );
    }

    #[test]
    fn collect_evidence_returns_one_evidence_per_dimension_in_order() {
        let a = crate::test_fixtures::scenario_one().remove(0);
        let b = crate::test_fixtures::scenario_one().remove(1);

        let evidence = collect_evidence(&a, &b);

        assert_eq!(evidence.len(), 3);
        assert_eq!(evidence[0].attribute, AttributeKind::Taxonomy);
        assert_eq!(evidence[1].attribute, AttributeKind::Habitat);
        assert_eq!(evidence[2].attribute, AttributeKind::Characteristic);
        assert!(evidence_of(&evidence, AttributeKind::Taxonomy)
            .similarity
            .is_some());
    }
}
