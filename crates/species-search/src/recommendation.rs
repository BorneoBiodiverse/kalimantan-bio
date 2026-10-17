//! Stage 4 — Related-Query Recommendation.
//!
//! Generates intelligent follow-up query suggestions based on common patterns
//! and dominant attributes found within the current search results.

use kalimantanbio_shared::core::Species;

use crate::types::QueryFilters;

/// Extracts frequently occurring attributes from current search results.
///
/// Identifies recurring patterns such as dominant taxonomic families,
/// shared conservation statuses, or common habitats.
///
/// # Arguments
///
/// * `results` - The matching species from the primary search.
///
/// # Returns
///
/// A list of representative attribute labels.
pub(crate) fn extract_common_attributes(results: &[&Species]) -> Vec<String> {
    todo!("species-search phase 4: extract_common_attributes")
}

/// Generates candidate follow-up queries by combining original filters with common attributes.
///
/// # Arguments
///
/// * `filters` - The original query filters.
/// * `attrs` - Prominent attributes discovered in the search results.
///
/// # Returns
///
/// A collection of suggested query strings.
pub(crate) fn generate_candidate_queries(
    filters: &QueryFilters,
    attrs: &[String],
) -> Vec<String> {
    todo!("species-search phase 4: generate_candidate_queries")
}

/// Evaluates how relevant a candidate query suggestion is to the user's intent.
///
/// # Arguments
///
/// * `query` - The candidate suggestion string.
/// * `original_filters` - The initial user query criteria.
///
/// # Returns
///
/// A relevance score between 0.0 and 1.0.
pub(crate) fn score_query_candidate(query: &str, original_filters: &QueryFilters) -> f64 {
    todo!("species-search phase 4: score_query_candidate")
}

/// Ranks candidate query suggestions descending by relevance score.
///
/// # Arguments
///
/// * `candidates` - Candidate query strings to rank.
/// * `filters` - The original query criteria.
///
/// # Returns
///
/// Ordered query candidates paired with their relevance scores.
pub(crate) fn rank_related_queries(
    candidates: Vec<String>,
    filters: &QueryFilters,
) -> Vec<(String, f64)> {
    todo!("species-search phase 4: rank_related_queries")
}
