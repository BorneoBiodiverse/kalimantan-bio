//! Stage 1 — Query Parsing and Filter Extraction.
//!
//! Converts raw search tokens into a structured [`QueryFilters`] instance.

use crate::types::QueryFilters;

/// Extracts structured filter criteria from a collection of query tokens.
///
/// Recognizes known biodiversity attributes (such as IUCN status codes, CITES
/// categories, and protection labels) and treats unrecognized tokens as
/// free-text keywords.
///
/// # Arguments
///
/// * `tokens` - Normalized words/tokens extracted from the user's raw query.
///
/// # Returns
///
/// A populated [`QueryFilters`] struct.
///
/// # Example
///
/// ```rust,ignore
/// let tokens = vec!["orangutan".to_string(), "cr".to_string()];
/// let filters = extract_filters(&tokens);
/// assert_eq!(filters.iucn, Some("CR".to_string()));
/// ```
pub(crate) fn extract_filters(tokens: &[String]) -> QueryFilters {
    todo!("species-search phase 1: extract_filters")
}
