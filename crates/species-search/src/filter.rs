//! Stage 2 — Multi-Attribute Filtering.
//!
//! Evaluates candidate species against parsed [`QueryFilters`] using composable
//! boolean predicates.

use kalimantanbio_shared::core::Species;

use crate::types::QueryFilters;

/// Checks whether a species matches the free-text search keywords.
///
/// Returns `true` if all keywords appear in the scientific name, common name,
/// or description of the species (or if no free-text keywords were specified).
///
/// # Arguments
///
/// * `species` - The species record to evaluate.
/// * `filters` - Active query filters containing candidate keywords.
pub(crate) fn matches_free_text(species: &Species, filters: &QueryFilters) -> bool {
    todo!("species-search phase 2: matches_free_text")
}

/// Checks whether a species matches taxonomic classification filters.
///
/// Evaluates kingdom ID and class ID constraints when specified.
///
/// # Arguments
///
/// * `species` - The species record to evaluate.
/// * `filters` - Active query filters containing taxonomic constraints.
pub(crate) fn matches_taxonomy(species: &Species, filters: &QueryFilters) -> bool {
    todo!("species-search phase 2: matches_taxonomy")
}

/// Checks whether a species matches conservation status criteria.
///
/// Evaluates IUCN Red List category, CITES appendix, and P.106 national
/// protection status.
///
/// # Arguments
///
/// * `species` - The species record to evaluate.
/// * `filters` - Active query filters containing conservation constraints.
pub(crate) fn matches_conservation(species: &Species, filters: &QueryFilters) -> bool {
    todo!("species-search phase 2: matches_conservation")
}

/// Checks whether a species matches origin/type requirements (e.g. "Endemic").
///
/// # Arguments
///
/// * `species` - The species record to evaluate.
/// * `filters` - Active query filters containing species type constraints.
pub(crate) fn matches_species_type(species: &Species, filters: &QueryFilters) -> bool {
    todo!("species-search phase 2: matches_species_type")
}

/// Filters a slice of species records, retaining only those matching all query criteria.
///
/// Applies all individual predicates sequentially via functional iterator composition.
///
/// # Arguments
///
/// * `species_list` - Complete species slice to filter from.
/// * `filters` - Parsed query filters to evaluate against.
///
/// # Returns
///
/// A vector of references to matching species records.
///
/// # Example
///
/// ```rust,ignore
/// let matched = filter_species(&all_species, &filters);
/// ```
pub(crate) fn filter_species<'a>(
    species_list: &'a [Species],
    filters: &QueryFilters,
) -> Vec<&'a Species> {
    todo!("species-search phase 2: filter_species")
}
