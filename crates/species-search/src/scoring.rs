//! Stage 3 — Relevance Scoring.
//!
//! Evaluates candidate species matching filter criteria and calculates a composite
//! relevance score using weighted attribute matching.
//!
//! Every function in this stage is **pure**: the output depends only on the
//! arguments, nothing is mutated, and no I/O is performed. The composite score
//! is built by composing small scorers (`score_text_match`,
//! `score_taxonomy_match`, `score_conservation_match`) and handing the result to
//! the shared [`combine_weighted_scores`].

use kalimantanbio_shared::core::Species;
use kalimantanbio_shared::scoring::combine_weighted_scores;

use crate::types::{QueryFilters, ScoringWeights};

/// Score for a keyword that appears as a whole word in a scientific/common name.
const NAME_WORD_SCORE: f64 = 1.0;
/// Score for a keyword that only appears as a fragment of a name (e.g. "orang" in "orangutan").
const NAME_SUBSTRING_SCORE: f64 = 0.8;
/// Score for a keyword that only appears in the free-text description.
const DESCRIPTION_SCORE: f64 = 0.4;

/// Returns `true` when `word` is one of the alphanumeric words of `text_lower`.
fn contains_word(text_lower: &str, word: &str) -> bool {
    text_lower
        .split(|c: char| !c.is_alphanumeric())
        .any(|w| w == word)
}

/// Scores a single keyword against a species (0.0, 0.4, 0.8 or 1.0).
///
/// Names are weighted above the description, and a whole-word match is weighted
/// above a partial one.
fn score_keyword(species: &Species, keyword: &str) -> f64 {
    let keyword = keyword.to_lowercase();
    let names = [
        species.scientific_name.to_lowercase(),
        species.common_name.to_lowercase(),
    ];

    if names.iter().any(|name| contains_word(name, &keyword)) {
        NAME_WORD_SCORE
    } else if names.iter().any(|name| name.contains(&keyword)) {
        NAME_SUBSTRING_SCORE
    } else if species.description.to_lowercase().contains(&keyword) {
        DESCRIPTION_SCORE
    } else {
        0.0
    }
}

/// Mean of the constraints that were actually specified (`Some`), where `Some(true)`
/// counts as 1.0 and `Some(false)` as 0.0. Returns 0.0 when nothing was specified.
fn fraction_satisfied(checks: &[Option<bool>]) -> f64 {
    let (satisfied, total) = checks
        .iter()
        .flatten()
        .fold((0u32, 0u32), |(ok, n), &passed| (ok + u32::from(passed), n + 1));

    if total == 0 {
        0.0
    } else {
        f64::from(satisfied) / f64::from(total)
    }
}

/// Case-insensitive comparison of an optional filter value with a species field.
fn eq_filter(filter: &Option<String>, value: &str) -> Option<bool> {
    filter.as_ref().map(|wanted| wanted.eq_ignore_ascii_case(value))
}

/// Scores the text relevance of a species against query keywords.
///
/// Gives higher priority to matches found in scientific or common names
/// compared to matches occurring only within free-text descriptions.
/// Each keyword is scored independently (whole-word name match = 1.0,
/// partial name match = 0.8, description match = 0.4, no match = 0.0) and the
/// final score is the mean over all keywords.
///
/// # Arguments
///
/// * `species` - The candidate species record.
/// * `filters` - The active query filters containing keywords.
///
/// # Returns
///
/// A normalized text match score between 0.0 and 1.0. Returns 0.0 when the
/// query has no free-text keywords.
pub(crate) fn score_text_match(species: &Species, filters: &QueryFilters) -> f64 {
    if filters.free_text.is_empty() {
        return 0.0;
    }

    let total: f64 = filters
        .free_text
        .iter()
        .map(|keyword| score_keyword(species, keyword))
        .sum();

    total / filters.free_text.len() as f64
}

/// Scores the taxonomic relevance of a species.
///
/// Evaluates matching precision across taxonomic levels: the score is the
/// fraction of specified taxonomic constraints (`kingdom_id`, `class_id`) that
/// the species satisfies.
///
/// # Arguments
///
/// * `species` - The candidate species record.
/// * `filters` - The active query filters containing taxonomic constraints.
///
/// # Returns
///
/// A normalized taxonomy score between 0.0 and 1.0. Returns 0.0 when the query
/// has no taxonomic constraints.
pub(crate) fn score_taxonomy_match(species: &Species, filters: &QueryFilters) -> f64 {
    fraction_satisfied(&[
        filters
            .kingdom_id
            .map(|id| id == species.taxonomy.kingdom_id),
        filters.class_id.map(|id| id == species.taxonomy.class_id),
    ])
}

/// Scores conservation status alignment between query and species attributes.
///
/// The score is the fraction of specified conservation constraints (IUCN
/// category, CITES appendix, P.106 protection status) that the species
/// satisfies. Comparison is case-insensitive.
///
/// # Arguments
///
/// * `species` - The candidate species record.
/// * `filters` - The active query filters containing conservation constraints.
///
/// # Returns
///
/// A normalized conservation status score between 0.0 and 1.0. Returns 0.0 when
/// the query has no conservation constraints.
pub(crate) fn score_conservation_match(species: &Species, filters: &QueryFilters) -> f64 {
    fraction_satisfied(&[
        eq_filter(&filters.iucn, &species.iucn),
        eq_filter(&filters.cites, &species.cites),
        eq_filter(&filters.p106, &species.p106),
    ])
}

/// Computes the overall composite relevance score for a species.
///
/// Combines text, taxonomy, and conservation match scores according to
/// the supplied [`ScoringWeights`]. Only the components the query actually
/// uses take part: weights of unused components are dropped and the remaining
/// ones are re-normalized to sum to 1.0, so a query such as `"orangutan"` is
/// not capped at the text weight (0.5). The weighted sum itself is computed by
/// the shared `combine_weighted_scores`.
///
/// # Arguments
///
/// * `species` - The candidate species record to score.
/// * `filters` - Active query filters.
/// * `weights` - Configuration determining the relative importance of each score component.
///
/// # Returns
///
/// A composite relevance score between 0.0 and 1.0. Returns 0.0 when the query
/// has no scorable criteria at all.
pub(crate) fn score_species(
    species: &Species,
    filters: &QueryFilters,
    weights: &ScoringWeights,
) -> f64 {
    let has_text = !filters.free_text.is_empty();
    let has_taxonomy = filters.kingdom_id.is_some() || filters.class_id.is_some();
    let has_conservation =
        filters.iucn.is_some() || filters.cites.is_some() || filters.p106.is_some();

    // (is the component active?, its score, its weight)
    let components = [
        (has_text, score_text_match(species, filters), weights.text_weight),
        (
            has_taxonomy,
            score_taxonomy_match(species, filters),
            weights.taxonomy_weight,
        ),
        (
            has_conservation,
            score_conservation_match(species, filters),
            weights.conservation_weight,
        ),
    ];

    let active: Vec<(f64, f64)> = components
        .iter()
        .filter(|(is_active, _, _)| *is_active)
        .map(|&(_, score, weight)| (score, weight))
        .collect();

    let weight_sum: f64 = active.iter().map(|&(_, weight)| weight).sum();
    if weight_sum <= 0.0 {
        return 0.0;
    }

    let normalized: Vec<(f64, f64)> = active
        .iter()
        .map(|&(score, weight)| (score, weight / weight_sum))
        .collect();

    combine_weighted_scores(&normalized).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use kalimantanbio_shared::core::Taxonomy;

    fn taxonomy(kingdom_id: u64, class_id: u64) -> Taxonomy {
        Taxonomy {
            kingdom: "Animalia".into(),
            kingdom_id,
            phylum_division: "Chordata".into(),
            phylum_division_id: 2,
            class: "Mammalia".into(),
            class_id,
            order: "Primates".into(),
            order_id: 4,
            family: "Hominidae".into(),
            family_id: 5,
            genus: "Pongo".into(),
            genus_id: 6,
        }
    }

    fn orangutan() -> Species {
        Species {
            id: 1,
            scientific_name: "Pongo pygmaeus".into(),
            common_name: "Orangutan Kalimantan".into(),
            description: "Great ape endemic to Borneo, critically endangered.".into(),
            species_type: "Endemic".into(),
            iucn: "CR".into(),
            cites: "Appendix I".into(),
            p106: "Dilindungi".into(),
            is_verified: true,
            image_url: None,
            taxonomy: taxonomy(1, 3),
            habitats: Default::default(),
            characteristics: Default::default(),
            observation_count: 2,
            recorded_individuals_total: 7,
            latest_observation_year: Some(2025),
            created_at: "2026-01-01T00:00:00Z".into(),
            updated_at: "2026-01-01T00:00:00Z".into(),
        }
    }

    fn with_text(words: &[&str]) -> QueryFilters {
        QueryFilters {
            free_text: words.iter().map(|w| w.to_string()).collect(),
            ..QueryFilters::default()
        }
    }

    #[test]
    fn text_whole_word_in_name_scores_highest() {
        assert_eq!(score_text_match(&orangutan(), &with_text(&["orangutan"])), 1.0);
    }

    #[test]
    fn text_partial_name_beats_description_only() {
        let partial = score_text_match(&orangutan(), &with_text(&["orang"]));
        let description = score_text_match(&orangutan(), &with_text(&["borneo"]));
        assert_eq!(partial, 0.8);
        assert_eq!(description, 0.4);
        assert!(partial > description);
    }

    #[test]
    fn text_is_mean_over_keywords() {
        // "orangutan" -> 1.0, "borneo" -> 0.4, "xyz" -> 0.0
        let score = score_text_match(&orangutan(), &with_text(&["orangutan", "borneo", "xyz"]));
        assert!((score - (1.0 + 0.4 + 0.0) / 3.0).abs() < 1e-12);
    }

    #[test]
    fn text_is_case_insensitive_and_zero_without_keywords() {
        assert_eq!(score_text_match(&orangutan(), &with_text(&["PONGO"])), 1.0);
        assert_eq!(score_text_match(&orangutan(), &QueryFilters::default()), 0.0);
        assert_eq!(score_text_match(&orangutan(), &with_text(&["xyz123"])), 0.0);
    }

    #[test]
    fn taxonomy_is_fraction_of_satisfied_constraints() {
        let sp = orangutan(); // kingdom 1, class 3
        let both = QueryFilters { kingdom_id: Some(1), class_id: Some(3), ..Default::default() };
        let half = QueryFilters { kingdom_id: Some(1), class_id: Some(99), ..Default::default() };
        let none = QueryFilters { kingdom_id: Some(2), class_id: Some(99), ..Default::default() };
        assert_eq!(score_taxonomy_match(&sp, &both), 1.0);
        assert_eq!(score_taxonomy_match(&sp, &half), 0.5);
        assert_eq!(score_taxonomy_match(&sp, &none), 0.0);
        assert_eq!(score_taxonomy_match(&sp, &QueryFilters::default()), 0.0);
    }

    #[test]
    fn conservation_is_fraction_and_case_insensitive() {
        let sp = orangutan();
        let all = QueryFilters {
            iucn: Some("cr".into()),
            cites: Some("Appendix I".into()),
            p106: Some("Dilindungi".into()),
            ..Default::default()
        };
        let mixed = QueryFilters {
            iucn: Some("CR".into()),
            p106: Some("Tidak Dilindungi".into()),
            ..Default::default()
        };
        assert_eq!(score_conservation_match(&sp, &all), 1.0);
        assert_eq!(score_conservation_match(&sp, &mixed), 0.5);
        assert_eq!(score_conservation_match(&sp, &QueryFilters::default()), 0.0);
    }

    // The tests below call the shared `combine_weighted_scores`, which is still a
    // `todo!()` stub in `kalimantanbio-shared`. Remove `#[ignore]` once it is implemented.

    #[test]
    fn species_perfect_match_scores_one_even_with_few_criteria() {
        // Only text is used -> weights are re-normalized, so a perfect match is 1.0, not 0.5.
        let score = score_species(&orangutan(), &with_text(&["orangutan"]), &ScoringWeights::default());
        assert!((score - 1.0).abs() < 1e-12);
    }

    #[test]
    fn species_combines_components_with_renormalized_weights() {
        // text (0.5) + conservation (0.2) active -> 5/7 and 2/7.
        // text = 0.4 (description only), conservation = 1.0.
        let filters = QueryFilters {
            free_text: vec!["borneo".into()],
            iucn: Some("CR".into()),
            ..Default::default()
        };
        let expected = 0.4 * (0.5 / 0.7) + 1.0 * (0.2 / 0.7);
        let score = score_species(&orangutan(), &filters, &ScoringWeights::default());
        assert!((score - expected).abs() < 1e-12);
    }

    #[test]
    fn species_empty_query_scores_zero_and_stays_in_range() {
        assert_eq!(score_species(&orangutan(), &QueryFilters::default(), &ScoringWeights::default()), 0.0);
        let full = QueryFilters {
            free_text: vec!["pongo".into()],
            kingdom_id: Some(1),
            class_id: Some(3),
            iucn: Some("CR".into()),
            ..Default::default()
        };
        let score = score_species(&orangutan(), &full, &ScoringWeights::default());
        assert!((0.0..=1.0).contains(&score));
    }
}