//! Stage 4 — Related-Query Recommendation.
//!
//! Generates intelligent follow-up query suggestions based on common patterns
//! and dominant attributes found within the current search results.
//!
//! The stage is a composition of four pure steps:
//!
//! 1. [`extract_common_attributes`] — find the attributes shared by the results.
//! 2. [`generate_candidate_queries`] — turn those attributes into query strings
//!    that the Stage 1 parser understands.
//! 3. [`score_query_candidate`] — measure how well a candidate preserves the
//!    user's original intent.
//! 4. [`rank_related_queries`] — order the candidates by that score.
//!
//! Score combination and ranking reuse the shared library
//! ([`combine_weighted_scores`] and [`rank_by_score`]); every function here is
//! otherwise pure: it depends only on its arguments, mutates nothing, and
//! performs no I/O.

use kalimantanbio_shared::core::Species;
use kalimantanbio_shared::scoring::{combine_weighted_scores, rank_by_score};

use crate::types::QueryFilters;

/// Relative weight of free-text preservation when scoring a candidate query.
const CANDIDATE_TEXT_WEIGHT: f64 = 0.6;

/// Relative weight of attribute alignment when scoring a candidate query.
const CANDIDATE_ATTRIBUTE_WEIGHT: f64 = 0.4;

/// Normalizes a value for comparison: lowercase, trimmed, with `-` and `_`
/// treated as spaces so `"Appendix I"`, `"appendix-i"` and `"APPENDIX_I"` match.
fn normalize_value(value: &str) -> String {
    value
        .trim()
        .to_lowercase()
        .replace(['-', '_'], " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Converts a normalized value into a single whitespace-free query token.
///
/// The tokenizer splits queries on whitespace, so multi-word values (e.g.
/// `"Tidak Dilindungi"`) become hyphenated (`"tidak-dilindungi"`), which the
/// Stage 1 parser maps back to its canonical form.
fn query_token(value: &str) -> String {
    normalize_value(value).replace(' ', "-")
}

/// Returns `true` when a raw attribute value carries information.
///
/// Empty strings and the `"-"` placeholder used by the production database are
/// treated as "no attribute".
fn is_present(value: &str) -> bool {
    let trimmed = value.trim();
    !trimmed.is_empty() && trimmed != "-"
}

/// Builds a canonical `field:value` label, or `None` for placeholder values.
fn attribute_label(field: &str, value: &str) -> Option<String> {
    if is_present(value) {
        Some(format!("{field}:{}", query_token(value)))
    } else {
        None
    }
}

/// Collects the canonical `field:value` labels of a single species.
///
/// Only attributes that the module understands are emitted: IUCN category,
/// CITES appendix, P.106 protection status, origin type, family and genus.
fn species_attributes(species: &Species) -> Vec<String> {
    [
        attribute_label("iucn", &species.iucn),
        attribute_label("cites", &species.cites),
        attribute_label("p106", &species.p106),
        attribute_label("type", &species.species_type),
        attribute_label("family", &species.taxonomy.family),
        attribute_label("genus", &species.taxonomy.genus),
    ]
    .into_iter()
    .flatten()
    .collect()
}

/// Extracts frequently occurring attributes from current search results.
///
/// Identifies recurring patterns such as dominant taxonomic families, shared
/// conservation statuses, or common habitats.
///
/// An attribute counts as *common* when at least half of the results carry it
/// (rounded up), so a single-result search still yields suggestions while a
/// mixed result set does not promote one-off attributes. Labels are canonical
/// `field:value` strings (e.g. `"iucn:cr"`, `"family:hominidae"`), sorted by
/// frequency descending and then alphabetically for deterministic output.
///
/// # Arguments
///
/// * `results` - The matching species from the primary search.
///
/// # Returns
///
/// A list of representative attribute labels; empty when there are no results.
///
/// # Example
///
/// ```rust,ignore
/// let attrs = extract_common_attributes(&results);
/// assert!(attrs.contains(&"iucn:cr".to_string()));
/// ```
pub(crate) fn extract_common_attributes(results: &[&Species]) -> Vec<String> {
    if results.is_empty() {
        return Vec::new();
    }

    // (attribute label, number of results carrying it)
    let counts = results
        .iter()
        .fold(Vec::<(String, usize)>::new(), |counts, species| {
            species_attributes(species)
                .into_iter()
                .fold(counts, |mut counts, label| {
                    match counts.iter().position(|(existing, _)| *existing == label) {
                        Some(index) => counts[index].1 += 1,
                        None => counts.push((label, 1)),
                    }
                    counts
                })
        });

    let minimum = results.len().div_ceil(2);

    let mut dominant: Vec<(String, usize)> = counts
        .into_iter()
        .filter(|(_, count)| *count >= minimum)
        .collect();

    dominant.sort_by(|(label_a, count_a), (label_b, count_b)| {
        count_b.cmp(count_a).then_with(|| label_a.cmp(label_b))
    });

    dominant.into_iter().map(|(label, _)| label).collect()
}

/// Renders the active query filters as whitespace-separated query fragments.
///
/// Used to keep the user's current context when building narrower candidates.
fn filter_fragments(filters: &QueryFilters) -> Vec<String> {
    let mut fragments: Vec<String> = filters
        .free_text
        .iter()
        .map(|keyword| keyword.trim().to_lowercase())
        .filter(|keyword| !keyword.is_empty())
        .collect();

    if let Some(code) = &filters.iucn {
        fragments.push(format!("iucn:{}", query_token(code)));
    }
    if let Some(cites) = &filters.cites {
        fragments.push(format!("cites:{}", query_token(cites)));
    }
    if let Some(status) = &filters.p106 {
        fragments.push(format!("p106:{}", query_token(status)));
    }
    if let Some(kind) = &filters.species_type {
        fragments.push(format!("type:{}", query_token(kind)));
    }
    if let Some(id) = filters.kingdom_id {
        fragments.push(format!("kingdom_id:{id}"));
    }
    if let Some(id) = filters.class_id {
        fragments.push(format!("class_id:{id}"));
    }
    if let Some(verified) = filters.is_verified {
        fragments.push(format!("verified:{verified}"));
    }

    fragments
}

/// Returns the value the query already constrains a canonical field to.
///
/// `None` means the query leaves that field open, so a candidate may introduce
/// it without contradicting the user.
fn filter_value(filters: &QueryFilters, field: &str) -> Option<String> {
    match field {
        "iucn" => filters.iucn.as_deref().map(normalize_value),
        "cites" => filters.cites.as_deref().map(normalize_value),
        "p106" => filters.p106.as_deref().map(normalize_value),
        "type" => filters.species_type.as_deref().map(normalize_value),
        "kingdom_id" => filters.kingdom_id.map(|id| id.to_string()),
        "class_id" => filters.class_id.map(|id| id.to_string()),
        "is_verified" => filters.is_verified.map(|flag| flag.to_string()),
        _ => None,
    }
}

/// Converts a dominant attribute label into a query fragment the parser accepts.
///
/// `iucn`, `cites`, `p106` and `type` labels are already written as parser
/// tokens. `genus` becomes a free-text keyword because genus names appear in
/// scientific names. `family` is intentionally not convertible: the parser only
/// supports numeric `kingdom_id`/`class_id` filters, so a family suggestion
/// could never match a species and would mislead the user.
fn attribute_fragment(label: &str) -> Option<String> {
    let (field, value) = label.split_once(':')?;

    match field {
        "iucn" | "cites" | "p106" | "type" => Some(label.to_string()),
        "genus" => Some(value.replace('-', " ")),
        _ => None,
    }
}

/// Appends `candidate` unless an equivalent suggestion is already listed.
fn push_candidate(candidates: &mut Vec<String>, candidate: String) {
    if !candidates
        .iter()
        .any(|existing| existing.eq_ignore_ascii_case(&candidate))
    {
        candidates.push(candidate);
    }
}

/// Generates candidate follow-up queries by combining original filters with common attributes.
///
/// Two kinds of suggestion are produced for every usable attribute: a
/// *narrower* query that keeps the current context
/// (`"orangutan iucn:cr"`) and a *broader* one that drops it (`"iucn:cr"`).
/// Attributes that the query already constrains are skipped, as are
/// attributes that cannot be expressed as a supported query (see
/// [`attribute_fragment`]). Suggestions are deduplicated case-insensitively.
///
/// # Arguments
///
/// * `filters` - The original query filters.
/// * `attrs` - Prominent attributes discovered in the search results.
///
/// # Returns
///
/// A collection of suggested query strings, narrowing suggestions first.
///
/// # Example
///
/// ```rust,ignore
/// let filters = QueryFilters { free_text: vec!["orangutan".into()], ..Default::default() };
/// let candidates = generate_candidate_queries(&filters, &["iucn:cr".to_string()]);
/// // ["orangutan iucn:cr", "iucn:cr"]
/// ```
pub(crate) fn generate_candidate_queries(filters: &QueryFilters, attrs: &[String]) -> Vec<String> {
    let context = filter_fragments(filters);
    let context_query = context.join(" ");

    let mut candidates: Vec<String> = Vec::new();

    for attr in attrs {
        let Some((field, _)) = attr.split_once(':') else {
            continue;
        };

        // The query already constrains this facet, so a suggestion would either
        // repeat it or contradict it.
        if filter_value(filters, field).is_some() {
            continue;
        }

        let Some(fragment) = attribute_fragment(attr) else {
            continue;
        };

        // The fragment is already part of the context (e.g. genus keyword that
        // the user typed as free text).
        if context
            .iter()
            .any(|existing| existing.eq_ignore_ascii_case(&fragment))
        {
            continue;
        }

        if !context_query.is_empty() {
            push_candidate(&mut candidates, format!("{context_query} {fragment}"));
        }
        push_candidate(&mut candidates, fragment);
    }

    candidates
}

/// Scores how much of the original free-text intent a candidate query keeps.
///
/// Returns the fraction of the original keywords contained in the candidate,
/// or `1.0` when the original query had no keywords.
fn text_preservation(query: &str, filters: &QueryFilters) -> f64 {
    let keywords: Vec<String> = filters
        .free_text
        .iter()
        .map(|keyword| keyword.trim().to_lowercase())
        .filter(|keyword| !keyword.is_empty())
        .collect();

    if keywords.is_empty() {
        return 1.0;
    }

    let haystack = query.to_lowercase();
    let kept = keywords
        .iter()
        .filter(|keyword| haystack.contains(keyword.as_str()))
        .count();

    kept as f64 / keywords.len() as f64
}

/// Parses a `field:value` word from a candidate query into canonical form.
///
/// Mirrors the field aliases accepted by the Stage 1 parser so scoring agrees
/// with parsing. Words without a known prefix return `None`.
fn structured_token(token: &str) -> Option<(&'static str, String)> {
    let (prefix, value) = token.split_once(':')?;

    let field = match prefix.to_lowercase().as_str() {
        "iucn" => "iucn",
        "cites" => "cites",
        "p106" | "protection" => "p106",
        "type" | "species_type" | "origin" => "type",
        "kingdom_id" | "kingdom" => "kingdom_id",
        "class_id" | "class" => "class_id",
        "verified" | "is_verified" => "is_verified",
        _ => return None,
    };

    Some((field, normalize_value(value)))
}

/// Scores how well a candidate query's attributes agree with the original query.
///
/// A candidate attribute that repeats the original value or introduces a facet
/// the user left open is aligned; one that contradicts an active filter is not.
/// Returns `1.0` when the candidate carries no structured attributes.
fn attribute_alignment(query: &str, filters: &QueryFilters) -> f64 {
    let tokens: Vec<(&'static str, String)> = query
        .split_whitespace()
        .filter_map(structured_token)
        .collect();

    if tokens.is_empty() {
        return 1.0;
    }

    let aligned = tokens
        .iter()
        .map(|(field, value)| match filter_value(filters, field) {
            Some(active) => active == *value,
            None => true,
        })
        .filter(|is_aligned| *is_aligned)
        .count();

    aligned as f64 / tokens.len() as f64
}

/// Evaluates how relevant a candidate query suggestion is to the user's intent.
///
/// Two signals are combined with the shared `combine_weighted_scores`: how much
/// of the original free-text intent is preserved (60%) and how well the
/// candidate's structured attributes agree with the active filters (40%).
/// Candidates that contradict an active filter lose the attribute component.
///
/// # Arguments
///
/// * `query` - The candidate suggestion string.
/// * `original_filters` - The initial user query criteria.
///
/// # Returns
///
/// A relevance score between 0.0 and 1.0.
///
/// # Example
///
/// ```rust,ignore
/// let filters = QueryFilters { free_text: vec!["orangutan".into()], ..Default::default() };
/// assert_eq!(score_query_candidate("orangutan iucn:cr", &filters), 1.0);
/// ```
pub(crate) fn score_query_candidate(query: &str, original_filters: &QueryFilters) -> f64 {
    let text_score = text_preservation(query, original_filters);
    let attribute_score = attribute_alignment(query, original_filters);

    combine_weighted_scores(&[
        (text_score, CANDIDATE_TEXT_WEIGHT),
        (attribute_score, CANDIDATE_ATTRIBUTE_WEIGHT),
    ])
    .clamp(0.0, 1.0)
}

/// Ranks candidate query suggestions descending by relevance score.
///
/// Every candidate is scored with [`score_query_candidate`] against the
/// original filters, then ordered with the shared `rank_by_score`.
///
/// # Arguments
///
/// * `candidates` - Candidate query strings to rank.
/// * `filters` - The original query criteria.
///
/// # Returns
///
/// Ordered query candidates paired with their relevance scores, highest first.
///
/// # Example
///
/// ```rust,ignore
/// let ranked = rank_related_queries(candidates, &filters);
/// let (top_query, top_score) = &ranked[0];
/// ```
pub(crate) fn rank_related_queries(
    candidates: Vec<String>,
    filters: &QueryFilters,
) -> Vec<(String, f64)> {
    let scored = candidates
        .into_iter()
        .map(|candidate| {
            let score = score_query_candidate(&candidate, filters);
            (candidate, score)
        })
        .collect();

    rank_by_score(scored)
}

#[cfg(test)]
mod tests {
    use super::*;
    use kalimantanbio_shared::core::Taxonomy;

    fn species(
        scientific_name: &str,
        species_type: &str,
        iucn: &str,
        cites: &str,
        p106: &str,
        family: &str,
        genus: &str,
    ) -> Species {
        Species {
            id: 1,
            scientific_name: scientific_name.into(),
            common_name: scientific_name.into(),
            description: String::new(),
            species_type: species_type.into(),
            iucn: iucn.into(),
            cites: cites.into(),
            p106: p106.into(),
            is_verified: true,
            image_url: None,
            taxonomy: Taxonomy {
                kingdom: "Animalia".into(),
                kingdom_id: 1,
                phylum_division: "Chordata".into(),
                phylum_division_id: 2,
                class: "Mammalia".into(),
                class_id: 3,
                order: "Primates".into(),
                order_id: 4,
                family: family.into(),
                family_id: 5,
                genus: genus.into(),
                genus_id: 6,
            },
            observation_count: 1,
            recorded_individuals_total: 1,
            latest_observation_year: None,
            created_at: "2026-01-01T00:00:00Z".into(),
            updated_at: "2026-01-01T00:00:00Z".into(),
        }
    }

    fn orangutan() -> Species {
        species(
            "Pongo pygmaeus",
            "Endemic",
            "CR",
            "Appendix I",
            "Dilindungi",
            "Hominidae",
            "Pongo",
        )
    }

    fn orangutan_sumatra() -> Species {
        species(
            "Pongo abelii",
            "Native",
            "CR",
            "Appendix I",
            "Dilindungi",
            "Hominidae",
            "Pongo",
        )
    }

    fn tiger() -> Species {
        species(
            "Panthera tigris",
            "Native",
            "EN",
            "Appendix I",
            "Dilindungi",
            "Felidae",
            "Panthera",
        )
    }

    #[test]
    fn common_attributes_require_at_least_half_of_the_results() {
        let results = [&orangutan(), &orangutan_sumatra(), &tiger()];

        let attributes = extract_common_attributes(&results);

        // Shared by 3/3 or 2/3 results.
        assert!(attributes.contains(&"p106:dilindungi".to_string()));
        assert!(attributes.contains(&"cites:appendix-i".to_string()));
        assert!(attributes.contains(&"iucn:cr".to_string()));
        assert!(attributes.contains(&"family:hominidae".to_string()));

        // Carried by a single result only.
        assert!(!attributes.contains(&"iucn:en".to_string()));
        assert!(!attributes.contains(&"family:felidae".to_string()));
        assert!(!attributes.contains(&"type:endemic".to_string()));
    }

    #[test]
    fn common_attributes_are_sorted_by_frequency_then_label() {
        let results = [&orangutan(), &orangutan_sumatra(), &tiger()];

        let attributes = extract_common_attributes(&results);

        // Present in 3/3 results; `cites` sorts before `p106` on the tie.
        assert_eq!(attributes.first(), Some(&"cites:appendix-i".to_string()));
        assert_eq!(attributes.get(1), Some(&"p106:dilindungi".to_string()));
        // The remaining labels are all shared by 2/3 results, so they are ordered alphabetically.
        let rest = &attributes[2..];
        assert!(rest.windows(2).all(|pair| pair[0] <= pair[1]));
    }

    #[test]
    fn common_attributes_ignore_placeholders() {
        let rafflesia = species(
            "Rafflesia arnoldii",
            "Endemic",
            "-",
            "-",
            "-",
            "Rafflesiaceae",
            "Rafflesia",
        );

        let attributes = extract_common_attributes(&[&rafflesia]);

        assert!(!attributes.iter().any(|label| label.starts_with("iucn:")));
        assert!(!attributes.iter().any(|label| label.starts_with("cites:")));
        assert!(!attributes.iter().any(|label| label.starts_with("p106:")));
        assert!(attributes.contains(&"family:rafflesiaceae".to_string()));
    }

    #[test]
    fn common_attributes_of_empty_results_are_empty() {
        assert!(extract_common_attributes(&[]).is_empty());
    }

    #[test]
    fn candidates_narrow_and_broaden_from_the_original_query() {
        let filters = QueryFilters {
            free_text: vec!["orangutan".into()],
            ..Default::default()
        };

        let candidates = generate_candidate_queries(&filters, &["iucn:cr".to_string()]);

        assert_eq!(
            candidates,
            vec!["orangutan iucn:cr".to_string(), "iucn:cr".to_string()]
        );
    }

    #[test]
    fn candidates_keep_existing_filters_as_context() {
        let filters = QueryFilters {
            free_text: vec!["orangutan".into()],
            p106: Some("Dilindungi".into()),
            ..Default::default()
        };

        let candidates = generate_candidate_queries(&filters, &["iucn:cr".to_string()]);

        assert_eq!(
            candidates,
            vec![
                "orangutan p106:dilindungi iucn:cr".to_string(),
                "iucn:cr".to_string()
            ]
        );
    }

    #[test]
    fn candidates_skip_attributes_the_query_already_constrains() {
        let filters = QueryFilters {
            iucn: Some("CR".into()),
            ..Default::default()
        };

        assert!(generate_candidate_queries(&filters, &["iucn:cr".to_string()]).is_empty());
        assert!(generate_candidate_queries(&filters, &["iucn:en".to_string()]).is_empty());
    }

    #[test]
    fn candidates_skip_attributes_the_parser_cannot_express() {
        let candidates =
            generate_candidate_queries(&QueryFilters::default(), &["family:hominidae".to_string()]);

        assert!(candidates.is_empty());
    }

    #[test]
    fn candidates_emit_genus_as_free_text_and_deduplicate() {
        let filters = QueryFilters {
            free_text: vec!["orangutan".into()],
            ..Default::default()
        };

        let candidates = generate_candidate_queries(
            &filters,
            &["genus:pongo".to_string(), "genus:pongo".to_string()],
        );

        assert_eq!(
            candidates,
            vec!["orangutan pongo".to_string(), "pongo".to_string()]
        );
    }

    #[test]
    fn candidates_are_empty_without_attributes() {
        assert!(generate_candidate_queries(&QueryFilters::default(), &[]).is_empty());
    }

    // The tests below call the shared `combine_weighted_scores` and
    // `rank_by_score`, which are still `todo!()` stubs in `kalimantanbio-shared`.
    // Remove `#[ignore]` once they are implemented.

    #[test]
    fn identical_candidate_scores_one() {
        let filters = QueryFilters {
            free_text: vec!["orangutan".into()],
            iucn: Some("CR".into()),
            ..Default::default()
        };

        assert_eq!(score_query_candidate("orangutan iucn:cr", &filters), 1.0);
    }

    #[test]
    fn candidate_that_keeps_the_intent_scores_higher() {
        let filters = QueryFilters {
            free_text: vec!["orangutan".into()],
            ..Default::default()
        };

        let refined = score_query_candidate("orangutan iucn:cr", &filters);
        let unrelated = score_query_candidate("iucn:en", &filters);

        assert!(refined > unrelated);
    }

    #[test]
    fn candidate_contradicting_an_active_filter_loses_attribute_weight() {
        let filters = QueryFilters {
            iucn: Some("CR".into()),
            ..Default::default()
        };

        // No free-text intent (1.0 * 0.6) and a contradictory attribute (0.0 * 0.4).
        let score = score_query_candidate("iucn:en", &filters);

        assert!((score - 0.6).abs() < 1e-12);
    }

    #[test]
    fn ranking_orders_candidates_by_relevance() {
        let filters = QueryFilters {
            free_text: vec!["orangutan".into()],
            ..Default::default()
        };

        let ranked = rank_related_queries(
            vec!["iucn:en".to_string(), "orangutan iucn:cr".to_string()],
            &filters,
        );

        assert_eq!(ranked[0].0, "orangutan iucn:cr");
        assert!(ranked.windows(2).all(|pair| pair[0].1 >= pair[1].1));
    }
}
