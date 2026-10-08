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
/// # Used By
///
/// - **Module 2 (species-relationships)**: Habitat similarity (weight 0.30) and characteristic similarity (weight 0.20)
/// - **Module 3 (taxonomy)**: Compare genus sets for gap analysis
/// - **Module 4 (species-comparison)**: Habitat and distribution similarity scoring
///
/// # Example
///
/// ```rust
/// use kalimantanbio_shared::collections::jaccard_similarity;
/// use std::collections::HashSet;
///
/// let a: HashSet<&str> = ["wetland", "lowland"].into_iter().collect();
/// let b: HashSet<&str> = ["wetland"].into_iter().collect();
///
/// assert!((jaccard_similarity(&a, &b) - 0.5).abs() < 1e-9);
/// ```
pub fn jaccard_similarity<T: Hash + Eq>(a: &HashSet<T>, b: &HashSet<T>) -> f64 {
    if a.is_empty() && b.is_empty() {
        return 0.0;
    }

    let intersection_count = a.intersection(b).count();
    let union_count = a.union(b).count();

    if union_count == 0 {
        0.0
    } else {
        intersection_count as f64 / union_count as f64
    }
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
/// # Used By
///
/// - **Module 2 (species-relationships)**: Find shared habitat/characteristic values for explanations
/// - **Module 4 (species-comparison)**: List attributes common to all compared species
///
/// # Example
///
/// ```rust
/// use kalimantanbio_shared::collections::set_intersection;
/// use std::collections::HashSet;
///
/// let a: HashSet<&str> = ["wetland", "lowland"].into_iter().collect();
/// let b: HashSet<&str> = ["lowland", "montane"].into_iter().collect();
///
/// assert_eq!(set_intersection(&a, &b), ["lowland"].into_iter().collect());
/// ```
pub fn set_intersection<T: Hash + Eq + Clone>(a: &HashSet<T>, b: &HashSet<T>) -> HashSet<T> {
    a.intersection(b).cloned().collect()
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
/// # Used By
///
/// - **Module 3 (taxonomy)**: Find missing genera (in reference but not in our dataset)
/// - **Module 4 (species-comparison)**: Find characteristics unique to each species
///
/// # Example
///
/// ```rust,ignore
/// let missing = set_difference(&ours, &reference);
/// ```
pub fn set_difference<T: Hash + Eq + Clone>(a: &HashSet<T>, b: &HashSet<T>) -> Vec<T> {
    a.difference(b).cloned().collect()
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
/// # Used By
///
/// - **Module 4 (species-comparison)**: Get all possible attributes across compared species
///
/// # Example
///
/// ```rust,ignore
/// let combined = set_union(&set_a, &set_b);
/// ```
pub fn set_union<T: Hash + Eq + Clone>(a: &HashSet<T>, b: &HashSet<T>) -> HashSet<T> {
    a.union(b).cloned().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn set(values: &[&str]) -> HashSet<String> {
        values.iter().map(|value| (*value).to_string()).collect()
    }

    #[test]
    fn jaccard_identical_sets_score_one() {
        let a = set(&["wetland", "lowland"]);
        assert_eq!(jaccard_similarity(&a, &a), 1.0);
    }

    #[test]
    fn jaccard_disjoint_sets_score_zero() {
        let a = set(&["wetland"]);
        let b = set(&["montane"]);
        assert_eq!(jaccard_similarity(&a, &b), 0.0);
    }

    #[test]
    fn jaccard_partial_overlap() {
        let a = set(&["wetland", "lowland"]);
        let b = set(&["wetland"]);
        assert!((jaccard_similarity(&a, &b) - 0.5).abs() < 1e-9);
    }

    #[test]
    fn jaccard_both_empty_scores_zero() {
        let empty: HashSet<String> = HashSet::new();
        assert_eq!(jaccard_similarity(&empty, &empty), 0.0);
    }

    #[test]
    fn jaccard_one_empty_scores_zero() {
        let empty: HashSet<String> = HashSet::new();
        let filled = set(&["wetland"]);
        assert_eq!(jaccard_similarity(&empty, &filled), 0.0);
    }

    #[test]
    fn intersection_returns_shared_values() {
        let a = set(&["wetland", "lowland"]);
        let b = set(&["lowland", "montane"]);
        assert_eq!(set_intersection(&a, &b), set(&["lowland"]));
    }

    #[test]
    fn intersection_with_empty_set_is_empty() {
        let empty: HashSet<String> = HashSet::new();
        let filled = set(&["wetland"]);
        assert!(set_intersection(&empty, &filled).is_empty());
    }
}