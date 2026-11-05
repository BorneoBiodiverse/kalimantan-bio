//! Stage 4: Similarity scoring and dimensional calculation.
//!
//! Provides pairwise similarity calculations across four dimensions:
//! - Taxonomy (shared helper `calculate_taxonomy_similarity`)
//! - Conservation status (IUCN, CITES, P106 equality & missing-data averaging)
//! - Species type (normalized text equality)
//! - Distribution (geographic Jaccard similarity across regency sets)

use std::collections::{HashMap, HashSet};

use kalimantanbio_shared::core::Species;

use kalimantanbio_shared::scoring::combine_weighted_scores;

use crate::types::{ScoringWeights, SimilarityScore};

/// Score taxonomic similarity using the shared taxonomy helper.
///
/// This wrapper introduces no local scoring scale and does not compare
/// the display-only taxonomy breadcrumb.
///
/// # Arguments
///
/// * `a` - First species.
/// * `b` - Second species.
///
/// # Returns
///
/// The similarity score returned by the shared taxonomy helper.
///
/// # Example
///
/// ```rust,ignore
/// let score = score_taxonomy(&species_a, &species_b);
/// ```
pub(crate) fn score_taxonomy(a: &Species, b: &Species) -> f64 {
    kalimantanbio_shared::taxonomy::calculate_taxonomy_similarity(&a.taxonomy, &b.taxonomy)
}

/// Score geographic overlap using the shared Jaccard helper.
///
/// Delegates to [`kalimantanbio_shared::collections::jaccard_similarity`];
/// this module owns no separate set-similarity formula.
///
/// # Arguments
///
/// * `kabupaten_a` - Distinct regency ids for the first species.
/// * `kabupaten_b` - Distinct regency ids for the second species.
///
/// # Returns
///
/// A score in `0.0..=1.0`. If either set is empty, returns `0.0`.
///
/// # Example
///
/// ```rust,ignore
/// let a = HashSet::from([6201, 6202]);
/// let b = HashSet::from([6201]);
/// assert_eq!(score_distribution(&a, &b), 0.5);
/// ```
pub(crate) fn score_distribution(kabupaten_a: &HashSet<u64>, kabupaten_b: &HashSet<u64>) -> f64 {
    kalimantanbio_shared::collections::jaccard_similarity(kabupaten_a, kabupaten_b)
}

/// Average conservation-status similarity across IUCN, CITES, and P106.
///
/// Each field follows the agreed order: identical values score `1.0`,
/// otherwise a `"-"` on either side scores `0.5`, otherwise `0.0`.
/// Equality is checked first, so `"-"` compared with `"-"` scores `1.0`.
/// Values are compared exactly, without text normalization.
///
/// # Arguments
///
/// * `a` - First species.
/// * `b` - Second species.
///
/// # Returns
///
/// The arithmetic mean of the three field scores, in `0.0..=1.0`.
///
/// # Example
///
/// ```rust,ignore
/// // Identical conservation fields yield a full match.
/// assert_eq!(score_conservation(&species, &species), 1.0);
/// ```
pub(crate) fn score_conservation(a: &Species, b: &Species) -> f64 {
    [
        (a.iucn.as_str(), b.iucn.as_str()),
        (a.cites.as_str(), b.cites.as_str()),
        (a.p106.as_str(), b.p106.as_str()),
    ]
    .into_iter()
    .map(|(left, right)| {
        if left == right {
            1.0
        } else if left == "-" || right == "-" {
            0.5
        } else {
            0.0
        }
    })
    .sum::<f64>()
        / 3.0
}

/// Score equality of species types after shared text normalization.
///
/// Uses [`kalimantanbio_shared::text::normalize_text`] on both values so
/// case and extra whitespace do not affect equality. No local normalizer
/// or special missing-value rule is applied: two normalized empty strings
/// are equal, while an empty string and a nonempty string are not.
///
/// # Arguments
///
/// * `a` - First species.
/// * `b` - Second species.
///
/// # Returns
///
/// `1.0` when the normalized types are equal, otherwise `0.0`.
///
/// # Example
///
/// ```rust,ignore
/// // a.species_type = " Endemic ", b.species_type = "endemic"
/// assert_eq!(score_species_type(&a, &b), 1.0);
/// ```
pub(crate) fn score_species_type(a: &Species, b: &Species) -> f64 {
    if kalimantanbio_shared::text::normalize_text(&a.species_type)
        == kalimantanbio_shared::text::normalize_text(&b.species_type)
    {
        1.0
    } else {
        0.0
    }
}

/// Combine all four similarity dimensions for one species pair.
///
/// # Arguments
///
/// * `a`, `b` - Species to compare.
/// * `obs_a`, `obs_b` - Regency IDs for each species.
/// * `weights` - Finite, non-negative weights summing to one.
///   Pass `ScoringWeights::default()` for standard Module 4 scoring.
///
/// # Returns
///
/// Pair identifiers, individual dimension scores, and their weighted total.
///
/// # Example
///
/// ```rust,ignore
/// let weights = ScoringWeights::default();
/// let score = calculate_similarity_score(&a, &b, &obs_a, &obs_b, &weights);
/// ```
pub(crate) fn calculate_similarity_score(
    a: &Species,
    b: &Species,
    obs_a: &HashSet<u64>,
    obs_b: &HashSet<u64>,
    weights: &ScoringWeights,
) -> SimilarityScore {
    let taxonomy_score = score_taxonomy(a, b);
    let conservation_score = score_conservation(a, b);
    let type_score = score_species_type(a, b);
    let distribution_score = score_distribution(obs_a, obs_b);

    let total_score = combine_weighted_scores(&[
        (taxonomy_score, weights.taxonomy),
        (conservation_score, weights.conservation),
        (type_score, weights.species_type),
        (distribution_score, weights.distribution),
    ]);

    SimilarityScore {
        species_a_id: a.id,
        species_b_id: b.id,
        taxonomy_score,
        conservation_score,
        type_score,
        distribution_score,
        total_score,
    }
}

/// Calculate scores for every unique pair in a validated species selection.
///
/// # Arguments
///
/// * `species_list` - Selected species with distinct IDs, in query order.
/// * `observations_map` - Regency sets keyed by species ID.
///   Missing entries are treated as empty sets.
/// * `weights` - Validated scoring weights.
///
/// # Returns
///
/// One score for each pair of positions, without self-pairs or reversed pairs.
/// Empty and singleton selections produce no pairs.
///
/// # Example
///
/// ```rust,ignore
/// let scores = calculate_all_pairs(&selected, &map, &ScoringWeights::default());
/// ```
pub(crate) fn calculate_all_pairs(
    species_list: &[&Species],
    observations_map: &HashMap<u64, HashSet<u64>>,
    weights: &ScoringWeights,
) -> Vec<SimilarityScore> {
    species_list
        .iter()
        .enumerate()
        .flat_map(|(index, a)| {
            species_list.iter().skip(index + 1).map(move |b| {
                let empty = HashSet::new();
                let obs_a = observations_map.get(&a.id).unwrap_or(&empty);
                let obs_b = observations_map.get(&b.id).unwrap_or(&empty);
                calculate_similarity_score(a, b, obs_a, obs_b, weights)
            })
        })
        .collect()
}

/// Find the highest-scoring pair involving a target species.
///
/// # Arguments
///
/// * `target_id` - Species ID to look up on either side of each pair.
/// * `scores` - Pair scores with finite totals.
///
/// # Returns
///
/// The highest-scoring matching pair, or `None` when none exists.
/// Equal totals select the last matching pair in input order.
///
/// # Example
///
/// ```rust,ignore
/// let best = find_most_similar(target_id, &scores);
/// ```
pub(crate) fn find_most_similar(
    target_id: u64,
    scores: &[SimilarityScore],
) -> Option<SimilarityScore> {
    scores
        .iter()
        .filter(|score| score.species_a_id == target_id || score.species_b_id == target_id)
        .max_by(|left, right| left.total_score.total_cmp(&right.total_score))
        .cloned()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_species() -> Vec<Species> {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../shared/fixtures/species.json"
        );
        let raw = std::fs::read_to_string(path).expect("species fixture must be readable");
        serde_json::from_str(&raw).expect("species fixture must deserialize")
    }

    fn species_with_conservation(values: [&str; 3]) -> Species {
        let mut species = fixture_species().remove(0);
        species.iucn = values[0].to_string();
        species.cites = values[1].to_string();
        species.p106 = values[2].to_string();
        species
    }

    #[test]
    fn conservation_identical_known_fields_score_one() {
        let a = species_with_conservation(["CR", "Appendix I", "Dilindungi"]);
        let b = species_with_conservation(["CR", "Appendix I", "Dilindungi"]);
        assert_eq!(score_conservation(&a, &b), 1.0);
    }

    #[test]
    fn conservation_different_known_fields_score_zero() {
        let a = species_with_conservation(["CR", "Appendix I", "Dilindungi"]);
        let b = species_with_conservation(["VU", "Appendix II", "Tidak Dilindungi"]);
        assert_eq!(score_conservation(&a, &b), 0.0);
    }

    #[test]
    fn conservation_unknown_against_known_scores_half_in_either_order() {
        let known = species_with_conservation(["CR", "Appendix I", "Dilindungi"]);
        let unknown = species_with_conservation(["-", "-", "-"]);
        assert_eq!(score_conservation(&known, &unknown), 0.5);
        assert_eq!(score_conservation(&unknown, &known), 0.5);
    }

    #[test]
    fn conservation_identical_unknown_fields_follow_equality_first() {
        let a = species_with_conservation(["-", "-", "-"]);
        let b = species_with_conservation(["-", "-", "-"]);
        assert_eq!(score_conservation(&a, &b), 1.0);
    }

    #[test]
    fn conservation_mixed_field_scores_are_averaged() {
        let a = species_with_conservation(["CR", "Appendix I", "Dilindungi"]);
        let b = species_with_conservation(["CR", "-", "Tidak Dilindungi"]);
        let expected = (1.0 + 0.5 + 0.0) / 3.0;
        assert_eq!(score_conservation(&a, &b), expected);
    }

    #[test]
    fn conservation_each_field_contributes_one_third() {
        let base = species_with_conservation(["CR", "Appendix I", "Dilindungi"]);

        let iucn_mismatch = species_with_conservation(["VU", "Appendix I", "Dilindungi"]);
        assert_eq!(score_conservation(&base, &iucn_mismatch), 2.0 / 3.0);

        let cites_mismatch = species_with_conservation(["CR", "Appendix II", "Dilindungi"]);
        assert_eq!(score_conservation(&base, &cites_mismatch), 2.0 / 3.0);

        let p106_mismatch = species_with_conservation(["CR", "Appendix I", "Tidak Dilindungi"]);
        assert_eq!(score_conservation(&base, &p106_mismatch), 2.0 / 3.0);
    }

    #[test]
    fn species_type_scores_normalized_equality() {
        let species = fixture_species();
        let mut a = species[0].clone();
        let mut b = species[1].clone();
        let cases = [
            ("Endemic", "Endemic", 1.0),
            ("Endemic", "eNdEmIc", 1.0),
            (" \tEndemic\n", "endemic", 1.0),
            ("  Pongo  PYGMAEUS ", "pongo pygmaeus", 1.0),
            ("Endemic", "Native", 0.0),
            ("", "", 1.0),
            (" \t\n", "", 1.0),
            ("", "Endemic", 0.0),
        ];

        cases.iter().for_each(|&(type_a, type_b, expected)| {
            a.species_type = type_a.to_string();
            b.species_type = type_b.to_string();
            let original_a = a.clone();
            let original_b = b.clone();

            assert_eq!(
                score_species_type(&a, &b),
                expected,
                "species types {type_a:?} vs {type_b:?}"
            );
            assert_eq!(score_species_type(&b, &a), expected);
            assert_eq!(a, original_a);
            assert_eq!(b, original_b);
        });
    }

    #[test]
    fn distribution_identical_sets_score_one() {
        let set = HashSet::from([6201, 6202]);
        assert_eq!(score_distribution(&set, &set), 1.0);
    }

    #[test]
    fn distribution_disjoint_sets_score_zero() {
        let a = HashSet::from([6201]);
        let b = HashSet::from([6202]);
        assert_eq!(score_distribution(&a, &b), 0.0);
    }

    #[test]
    fn distribution_half_overlap_scores_half() {
        let a = HashSet::from([6201, 6202]);
        let b = HashSet::from([6201]);
        assert_eq!(score_distribution(&a, &b), 0.5);
    }

    #[test]
    fn distribution_both_empty_score_zero() {
        let empty = HashSet::new();
        assert_eq!(score_distribution(&empty, &empty), 0.0);
    }

    #[test]
    fn distribution_one_empty_scores_zero_in_either_order() {
        let populated = HashSet::from([6201]);
        let empty = HashSet::new();
        assert_eq!(score_distribution(&populated, &empty), 0.0);
        assert_eq!(score_distribution(&empty, &populated), 0.0);
    }

    #[test]
    fn taxonomy_same_genus_score_one() {
        let species = fixture_species();
        let a = species.iter().find(|s| s.id == 1).expect("Pongo pygmaeus");
        let b = species.iter().find(|s| s.id == 3).expect("Pongo abelii");

        assert_eq!(score_taxonomy(a, b), 1.0);
    }

    #[test]
    fn taxonomy_same_family_only_scores_point_six() {
        let species = fixture_species();
        let a = species.iter().find(|s| s.id == 2).expect("Panthera tigris");
        let b = species.iter().find(|s| s.id == 4).expect("Neofelis diardi");

        assert_eq!(score_taxonomy(a, b), 0.6);
    }

    #[test]
    fn taxonomy_no_similarity_scores_zero() {
        let species = fixture_species();
        let a = species.iter().find(|s| s.id == 1).expect("Pongo pygmaeus");
        let b = species
            .iter()
            .find(|s| s.id == 5)
            .expect("Rafflesia arnoldii");

        assert_eq!(score_taxonomy(a, b), 0.0);
    }

    #[test]
    fn similarity_combines_dimensions_with_default_weights() {
        let species = fixture_species();
        let a = species.iter().find(|s| s.id == 1).expect("Pongo pygmaeus");
        let b = species.iter().find(|s| s.id == 3).expect("Pongo abelii");
        let obs_a = HashSet::from([6201, 6202]);
        let obs_b = HashSet::from([6201]);
        let score = calculate_similarity_score(a, b, &obs_a, &obs_b, &ScoringWeights::default());
        assert_eq!((score.species_a_id, score.species_b_id), (1, 3));
        assert_eq!(score.taxonomy_score, 1.0);
        assert_eq!(score.conservation_score, 1.0);
        assert_eq!(score.type_score, 0.0);
        assert_eq!(score.distribution_score, 0.5);
        let expected = 1.0 * 0.40 + 1.0 * 0.30 + 0.0 * 0.15 + 0.5 * 0.15;
        assert!((score.total_score - expected).abs() < 1e-9);
    }

    #[test]
    fn all_pairs_for_three_species_are_unique_and_ordered() {
        let species = fixture_species();
        let selected: Vec<&Species> = [1, 2, 3]
            .iter()
            .map(|id| species.iter().find(|s| s.id == *id).expect("fixture ID"))
            .collect();
        let scores = calculate_all_pairs(&selected, &HashMap::new(), &ScoringWeights::default());
        let pairs: Vec<_> = scores
            .iter()
            .map(|score| (score.species_a_id, score.species_b_id))
            .collect();
        assert_eq!(pairs, vec![(1, 2), (1, 3), (2, 3)]);
        assert!(scores.iter().all(|score| score.distribution_score == 0.0));
    }
    #[test]
    fn all_pairs_empty_or_single_species_returns_empty() {
        let species = fixture_species();
        let map = HashMap::new();
        let weights = ScoringWeights::default();
        assert!(calculate_all_pairs(&[], &map, &weights).is_empty());
        assert!(calculate_all_pairs(&[&species[0]], &map, &weights).is_empty());
    }
    #[test]
    fn most_similar_selects_best_matching_pair_on_either_side() {
        let species = fixture_species();
        let selected: Vec<&Species> = [1, 2, 3]
            .iter()
            .map(|id| species.iter().find(|s| s.id == *id).expect("fixture ID"))
            .collect();
        let scores = calculate_all_pairs(&selected, &HashMap::new(), &ScoringWeights::default());
        for target in [1, 3] {
            let best = find_most_similar(target, &scores).expect("matching pair");
            assert_eq!((best.species_a_id, best.species_b_id), (1, 3));
        }
        // Pair (1, 3) is strongest overall, but must not be returned for target 2.
        let best_for_two = find_most_similar(2, &scores).expect("matching pair");
        assert_eq!(
            (best_for_two.species_a_id, best_for_two.species_b_id),
            (2, 3)
        );
        assert!(find_most_similar(999, &scores).is_none());
        assert!(find_most_similar(1, &[]).is_none());
    }
}
