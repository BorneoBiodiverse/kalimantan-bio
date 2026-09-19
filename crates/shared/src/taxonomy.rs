//! Taxonomy scoring and hierarchy utilities.

use crate::core::{TaxonomicRank, Taxonomy};

/// Calculate taxonomy similarity score (0.0 - 1.0).
///
/// Scoring weights by rank: genus = 1.0, family = 0.8, order = 0.6,
/// class = 0.4, phylum = 0.2, kingdom = 0.1.
///
/// # Arguments
///
/// * `a` - First taxonomy.
/// * `b` - Second taxonomy.
///
/// # Returns
///
/// A similarity score between 0.0 (no shared rank) and 1.0 (identical genus).
///
/// # Example
///
/// ```rust,ignore
/// let score = calculate_taxonomy_similarity(&tax_a, &tax_b);
/// assert_eq!(score, 1.0);
/// ```
pub fn calculate_taxonomy_similarity(a: &Taxonomy, b: &Taxonomy) -> f64 {
    todo!("shared taxonomy phase 3: calculate_taxonomy_similarity")
}

/// Build taxonomy path string: `Kingdom > Phylum > Class > Order > Family > Genus`.
///
/// # Arguments
///
/// * `taxonomy` - The taxonomy to render.
///
/// # Returns
///
/// A human-readable hierarchy path.
///
/// # Example
///
/// ```rust,ignore
/// let path = build_taxonomy_path(&taxonomy);
/// println!("{path}");
/// ```
pub fn build_taxonomy_path(taxonomy: &Taxonomy) -> String {
    todo!("shared taxonomy phase 3: build_taxonomy_path")
}

/// Get the highest taxonomic rank where two taxonomies match.
///
/// # Arguments
///
/// * `a` - First taxonomy.
/// * `b` - Second taxonomy.
///
/// # Returns
///
/// The deepest shared rank, or `None` when no rank matches.
///
/// # Example
///
/// ```rust,ignore
/// let rank = get_common_rank(&tax_a, &tax_b);
/// assert!(rank.is_some());
/// ```
pub fn get_common_rank(a: &Taxonomy, b: &Taxonomy) -> Option<TaxonomicRank> {
    todo!("shared taxonomy phase 3: get_common_rank")
}

/// Calculate taxonomic distance between two species.
///
/// Distance is 0 for the same genus, 6 for different kingdoms.
///
/// # Arguments
///
/// * `a` - First taxonomy.
/// * `b` - Second taxonomy.
///
/// # Returns
///
/// A distance between 0 and 6.
///
/// # Example
///
/// ```rust,ignore
/// let distance = taxonomy_distance(&tax_a, &tax_b);
/// ```
pub fn taxonomy_distance(a: &Taxonomy, b: &Taxonomy) -> usize {
    todo!("shared taxonomy phase 3: taxonomy_distance")
}