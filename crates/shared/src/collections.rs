//! Set and collection similarity operations.

use std::collections::HashSet;
use std::hash::Hash;

/// Calculate Jaccard similarity coefficient: `|A ∩ B| / |A ∪ B|`.
///
/// # Arguments
///
/// * `a` - First set.
/// * `b` - Second set.
///
/// # Returns
///
/// A coefficient between 0.0 and 1.0. Empties are treated as 0.0.
///
/// # Example
///
/// ```rust,ignore
/// let score = jaccard_similarity(&set_a, &set_b);
/// ```
pub fn jaccard_similarity<T: Hash + Eq>(a: &HashSet<T>, b: &HashSet<T>) -> f64 {
    todo!("shared collections phase 3: jaccard_similarity")
}

/// Set intersection: elements present in both sets.
///
/// # Arguments
///
/// * `a` - First set.
/// * `b` - Second set.
///
/// # Returns
///
/// A new set with the common elements.
///
/// # Example
///
/// ```rust,ignore
/// let common = set_intersection(&set_a, &set_b);
/// ```
pub fn set_intersection<T: Hash + Eq + Clone>(a: &HashSet<T>, b: &HashSet<T>) -> HashSet<T> {
    todo!("shared collections phase 3: set_intersection")
}

/// Set difference: elements in `A` but not in `B`.
///
/// # Arguments
///
/// * `a` - First set (source).
/// * `b` - Second set (comparison).
///
/// # Returns
///
/// A vector of elements only present in `a`.
///
/// # Example
///
/// ```rust,ignore
/// let missing = set_difference(&ours, &reference);
/// ```
pub fn set_difference<T: Hash + Eq + Clone>(a: &HashSet<T>, b: &HashSet<T>) -> Vec<T> {
    todo!("shared collections phase 3: set_difference")
}

/// Set union: all elements from both sets.
///
/// # Arguments
///
/// * `a` - First set.
/// * `b` - Second set.
///
/// # Returns
///
/// A new set with every element of `a` and `b`.
///
/// # Example
///
/// ```rust,ignore
/// let combined = set_union(&set_a, &set_b);
/// ```
pub fn set_union<T: Hash + Eq + Clone>(a: &HashSet<T>, b: &HashSet<T>) -> HashSet<T> {
    todo!("shared collections phase 3: set_union")
}