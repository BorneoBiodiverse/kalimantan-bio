#![warn(missing_docs)]
//! Module 1 — Intelligent Species Search.
//!
//! Provides intelligent ways to discover species using natural-language queries
//! and structured biodiversity attributes.
//!
//! Public API: `search` and `recommend_related_queries`. Internal helpers
//! (`extract_filters`, `matches_*`, `filter_species`, `score_*`, ...) stay
//! private.
//!
//! > **Status: interface scaffold.** Bodies are stubs pending implementation.

use kalimantanbio_shared::core::Species;

/// Search species by a natural-language or structured query.
///
/// Pipeline: parse → filter → score → rank.
///
/// # Arguments
///
/// * `raw_query` - User query, e.g. `"orangutan kalimantan"` or `"iucn:CR"`.
/// * `all_species` - The full species dataset.
///
/// # Returns
///
/// Ranked matches as `(species, relevance_score)` pairs, highest first.
///
/// # Example
///
/// ```rust,ignore
/// let results = search("orangutan kalimantan", &all_species);
/// for (species, score) in results {
///     println!("{species} ({score})");
/// }
/// ```
pub fn search<'a>(
    raw_query: &str,
    all_species: &'a [Species],
) -> Vec<(&'a Species, f64)> {
    todo!("species-search phase 5: search")
}

/// Recommend related follow-up queries.
///
/// Composes the private recommendation pipeline of the module.
///
/// # Arguments
///
/// * `raw_query` - The original user query.
/// * `all_species` - The full species dataset.
///
/// # Returns
///
/// Ranked query suggestions as `(query, score)` pairs.
///
/// # Example
///
/// ```rust,ignore
/// let suggestions = recommend_related_queries("mamalia", &all_species);
/// ```
pub fn recommend_related_queries(
    raw_query: &str,
    all_species: &[Species],
) -> Vec<(String, f64)> {
    todo!("species-search phase 5: recommend_related_queries")
}