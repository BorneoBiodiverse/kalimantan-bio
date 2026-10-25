//! Stage 2 — Multi-Attribute Filtering.
//!
//! Evaluates candidate species against parsed [`QueryFilters`] using composable
//! boolean predicates.

use kalimantanbio_shared::core::Species;
use kalimantanbio_shared::text::normalize_text;

use crate::types::QueryFilters;

/// Case-insensitive string equality (helper murni).
fn eq_ci(a: &str, b: &str) -> bool {
    a.trim().eq_ignore_ascii_case(b.trim())
}

/// Checks whether a species matches the free-text search keywords.
///
/// Returns `true` if all keywords appear in the scientific name, common name,
/// or description of the species (or if no free-text keywords were specified).
pub(crate) fn matches_free_text(species: &Species, filters: &QueryFilters) -> bool {
    let haystack = normalize_text(&format!(
        "{} {} {}",
        species.scientific_name, species.common_name, species.description
    ));
    filters
        .free_text
        .iter()
        .all(|kw| haystack.contains(&normalize_text(kw)))
}

/// Checks whether a species matches taxonomic classification filters.
///
/// Evaluates kingdom ID and class ID constraints when specified.
pub(crate) fn matches_taxonomy(species: &Species, filters: &QueryFilters) -> bool {
    filters
        .kingdom_id
        .map_or(true, |id| species.taxonomy.kingdom_id == id)
        && filters
            .class_id
            .map_or(true, |id| species.taxonomy.class_id == id)
}

/// Checks whether a species matches conservation status criteria.
///
/// Evaluates IUCN Red List category, CITES appendix, and P.106 national
/// protection status.
pub(crate) fn matches_conservation(species: &Species, filters: &QueryFilters) -> bool {
    filters.iucn.as_ref().map_or(true, |v| eq_ci(&species.iucn, v))
        && filters.cites.as_ref().map_or(true, |v| eq_ci(&species.cites, v))
        && filters.p106.as_ref().map_or(true, |v| eq_ci(&species.p106, v))
}

/// Checks whether a species matches origin/type requirements (e.g. "Endemic").
pub(crate) fn matches_species_type(species: &Species, filters: &QueryFilters) -> bool {
    filters
        .species_type
        .as_ref()
        .map_or(true, |t| eq_ci(&species.species_type, t))
}

/// Filters a slice of species records, retaining only those matching all query criteria.
///
/// Applies all individual predicates sequentially via functional iterator composition.
pub(crate) fn filter_species<'a>(
    species_list: &'a [Species],
    filters: &QueryFilters,
) -> Vec<&'a Species> {
    species_list
        .iter()
        .filter(|s| matches_free_text(s, filters))
        .filter(|s| matches_taxonomy(s, filters))
        .filter(|s| matches_conservation(s, filters))
        .filter(|s| matches_species_type(s, filters))
        .filter(|s| filters.is_verified.map_or(true, |v| s.is_verified == v))
        .collect()
}
