//! Taxonomy scoring and hierarchy utilities.

use crate::core::{TaxonomicRank, Taxonomy};

/// Calculate taxonomy similarity score (0.0 - 1.0).
///
/// Only the order, family, and genus ranks are compared, and the deepest
/// matching rank determines the score:
///
/// | Matching rank | Similarity |
/// | --- | --- |
/// | Genus | `1.0` |
/// | Family, no genus match | `0.6` |
/// | Order, no family/genus match | `0.3` |
/// | No matching rank | `0.0` |
///
/// A rank is skipped when either label is empty, so missing data never
/// produces a match. Higher ranks (class, phylum, kingdom) are ignored.
/// The function returns `0.0` both when the comparable ranks differ and when
/// no rank can be compared; callers that need to distinguish those cases must
/// check label availability themselves.
///
/// # Arguments
///
/// * `a` - First taxonomy (labels normalized, e.g. lowercase).
/// * `b` - Second taxonomy (labels normalized, e.g. lowercase).
///
/// # Returns
///
/// A similarity score between 0.0 (no shared rank) and 1.0 (identical genus).
///
/// # Used By
///
/// - **Module 1 (species-search)**: Boost search relevance for taxonomic matches
/// - **Module 2 (species-relationships)**: Taxonomy dimension scoring (weight 0.50)
/// - **Module 4 (species-comparison)**: Taxonomy similarity in comparisons
///
/// # Example
///
/// ```rust
/// use kalimantanbio_shared::core::Taxonomy;
/// use kalimantanbio_shared::taxonomy::calculate_taxonomy_similarity;
///
/// # fn taxonomy(genus: &str, family: &str, order: &str) -> Taxonomy {
/// #     Taxonomy {
/// #         kingdom: String::new(),
/// #         kingdom_id: 0,
/// #         phylum_division: String::new(),
/// #         phylum_division_id: 0,
/// #         class: String::new(),
/// #         class_id: 0,
/// #         order: order.to_string(),
/// #         order_id: 0,
/// #         family: family.to_string(),
/// #         family_id: 0,
/// #         genus: genus.to_string(),
/// #         genus_id: 0,
/// #     }
/// # }
/// let a = taxonomy("genus-a", "family-a", "order-a");
/// let b = taxonomy("genus-b", "family-a", "order-a");
///
/// assert!((calculate_taxonomy_similarity(&a, &b) - 0.6).abs() < 1e-9);
/// ```
pub fn calculate_taxonomy_similarity(a: &Taxonomy, b: &Taxonomy) -> f64 {
    fn matches(left: &str, right: &str) -> bool {
        !left.is_empty() && left == right
    }

    if matches(&a.genus, &b.genus) {
        1.0
    } else if matches(&a.family, &b.family) {
        0.6
    } else if matches(&a.order, &b.order) {
        0.3
    } else {
        0.0
    }
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
/// # Used By
///
/// - **Module 2 (species-relationships)**: Generate taxonomy explanations
/// - **Module 3 (taxonomy)**: Display full taxonomy paths in reports
/// - **Module 4 (species-comparison)**: Show taxonomy in comparison tables
///
/// # Example
///
/// ```rust,ignore
/// let path = build_taxonomy_path(&taxonomy);
/// println!("{path}");
/// ```
pub fn build_taxonomy_path(taxonomy: &Taxonomy) -> String {
    [
        taxonomy.kingdom.as_str(),
        taxonomy.phylum_division.as_str(),
        taxonomy.class.as_str(),
        taxonomy.order.as_str(),
        taxonomy.family.as_str(),
        taxonomy.genus.as_str(),
    ]
    .iter()
    .map(|rank| rank.trim())
    .filter(|rank| !rank.is_empty())
    .collect::<Vec<_>>()
    .join(" > ")
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
/// # Used By
///
/// - **Module 2 (species-relationships)**: Generate explanations like "Share the same family"
/// - **Module 4 (species-comparison)**: Identify taxonomic divergence point
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
/// # Used By
///
/// - **Module 2 (species-relationships)**: Alternative distance metric for relationships
/// - **Module 3 (taxonomy)**: Measure taxonomic diversity spread
///
/// # Example
///
/// ```rust,ignore
/// let distance = taxonomy_distance(&tax_a, &tax_b);
/// ```
pub fn taxonomy_distance(a: &Taxonomy, b: &Taxonomy) -> usize {
    todo!("shared taxonomy phase 3: taxonomy_distance")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn taxonomy(order: &str, family: &str, genus: &str) -> Taxonomy {
        Taxonomy {
            kingdom: String::new(),
            kingdom_id: 0,
            phylum_division: String::new(),
            phylum_division_id: 0,
            class: String::new(),
            class_id: 0,
            order: order.to_string(),
            order_id: 0,
            family: family.to_string(),
            family_id: 0,
            genus: genus.to_string(),
            genus_id: 0,
        }
    }

    #[test]
    fn same_genus_scores_one() {
        let a = taxonomy("order-a", "family-a", "genus-a");
        let b = taxonomy("order-b", "family-b", "genus-a");
        assert_eq!(calculate_taxonomy_similarity(&a, &b), 1.0);
    }

    #[test]
    fn same_family_without_genus_scores_point_six() {
        let a = taxonomy("order-a", "family-a", "genus-a");
        let b = taxonomy("order-b", "family-a", "genus-b");
        assert_eq!(calculate_taxonomy_similarity(&a, &b), 0.6);
    }

    #[test]
    fn same_order_only_scores_point_three() {
        let a = taxonomy("order-a", "family-a", "genus-a");
        let b = taxonomy("order-a", "family-b", "genus-b");
        assert_eq!(calculate_taxonomy_similarity(&a, &b), 0.3);
    }

    #[test]
    fn all_comparable_ranks_different_scores_zero() {
        let a = taxonomy("order-a", "family-a", "genus-a");
        let b = taxonomy("order-b", "family-b", "genus-b");
        assert_eq!(calculate_taxonomy_similarity(&a, &b), 0.0);
    }

    #[test]
    fn empty_labels_are_skipped() {
        let a = taxonomy("order-a", "family-a", "");
        let b = taxonomy("order-a", "family-a", "genus-b");
        assert_eq!(calculate_taxonomy_similarity(&a, &b), 0.6);
    }

    #[test]
    fn empty_genus_on_both_sides_is_not_a_match() {
        let a = taxonomy("order-a", "family-a", "");
        let b = taxonomy("order-a", "family-b", "");
        assert_eq!(calculate_taxonomy_similarity(&a, &b), 0.3);
    }

    #[test]
    fn higher_ranks_are_ignored() {
        let mut a = taxonomy("order-a", "family-a", "genus-a");
        let mut b = taxonomy("order-b", "family-b", "genus-b");
        a.kingdom = "kingdom-a".to_string();
        b.kingdom = "kingdom-a".to_string();
        a.class = "class-a".to_string();
        b.class = "class-a".to_string();
        assert_eq!(calculate_taxonomy_similarity(&a, &b), 0.0);
    }

    #[test]
    fn all_labels_empty_scores_zero() {
        let a = taxonomy("", "", "");
        let b = taxonomy("", "", "");
        assert_eq!(calculate_taxonomy_similarity(&a, &b), 0.0);
    }

    #[test]
    fn similarity_is_symmetric() {
        let a = taxonomy("order-a", "family-a", "genus-a");
        let b = taxonomy("order-a", "family-b", "genus-b");
        assert_eq!(
            calculate_taxonomy_similarity(&a, &b),
            calculate_taxonomy_similarity(&b, &a)
        );
    }

    #[test]
    fn build_taxonomy_path_renders_full_pongo_breadcrumb() {
        let taxonomy = Taxonomy {
            kingdom: "Animalia".to_string(),
            kingdom_id: 1,
            phylum_division: "Chordata".to_string(),
            phylum_division_id: 2,
            class: "Mammalia".to_string(),
            class_id: 3,
            order: "Primates".to_string(),
            order_id: 4,
            family: "Hominidae".to_string(),
            family_id: 5,
            genus: "Pongo".to_string(),
            genus_id: 6,
        };

        let path = build_taxonomy_path(&taxonomy);

        assert_eq!(
            path,
            "Animalia > Chordata > Mammalia > Primates > Hominidae > Pongo"
        );
    }
}
