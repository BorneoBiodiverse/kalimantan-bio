//! Statistical helpers: counting, frequency, coverage and timelines.

use std::collections::{BTreeMap, HashMap};
use std::hash::Hash;

/// Count occurrences of each key.
///
/// # Arguments
///
/// * `items` - The collection to aggregate.
/// * `key_fn` - Extracts the grouping key from each item.
///
/// # Returns
///
/// A map from key to occurrence count.
///
/// # Example
///
/// ```rust,ignore
/// let counts = count_by_key(&species, |s| s.taxonomy.family.clone());
/// ```
pub fn count_by_key<T, K: Hash + Eq>(
    items: &[T],
    key_fn: impl Fn(&T) -> K,
) -> HashMap<K, usize> {
    todo!("shared stats phase 3: count_by_key")
}

/// Get a frequency distribution sorted by count (descending).
///
/// # Arguments
///
/// * `items` - The collection to aggregate.
/// * `key_fn` - Extracts the grouping key from each item.
///
/// # Returns
///
/// A list of `(key, count)` sorted by count descending.
///
/// # Example
///
/// ```rust,ignore
/// let distribution = frequency_distribution(&species, |s| s.taxonomy.genus.clone());
/// ```
pub fn frequency_distribution<T, K: Hash + Eq + Clone>(
    items: &[T],
    key_fn: impl Fn(&T) -> K,
) -> Vec<(K, usize)> {
    todo!("shared stats phase 3: frequency_distribution")
}

/// Calculate coverage percentage (0.0 - 1.0).
///
/// # Arguments
///
/// * `available` - Number of covered items.
/// * `total` - Total expected items.
///
/// # Returns
///
/// A coverage ratio between 0.0 and 1.0.
///
/// # Example
///
/// ```rust,ignore
/// let coverage = calculate_coverage_percentage(80, 100);
/// ```
pub fn calculate_coverage_percentage(available: usize, total: usize) -> f64 {
    todo!("shared stats phase 3: calculate_coverage_percentage")
}

/// Build a timeline: count items per year.
///
/// # Arguments
///
/// * `items` - The collection to aggregate.
/// * `year_fn` - Extracts the year from each item.
///
/// # Returns
///
/// An ordered map from year to count.
///
/// # Example
///
/// ```rust,ignore
/// let timeline = build_timeline(&publications, |p| p.publication_year);
/// ```
pub fn build_timeline<T>(items: &[T], year_fn: impl Fn(&T) -> u16) -> BTreeMap<u16, usize> {
    todo!("shared stats phase 3: build_timeline")
}