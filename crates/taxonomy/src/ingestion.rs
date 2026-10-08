//! Module 3 Tahap 1 — Data Ingestion, Parsing, and Hierarchy Validation.
//!
//! Responsible for ingesting raw biological taxonomy and species records,
//! validating structural invariants (uniqueness, completeness, sanitization),
//! and verifying hierarchical integrity (detecting cycles and orphan nodes)
//! using pure, functional programming principles.

use std::collections::HashSet;

use crate::types::{PipelineError, RawSpeciesRecord, RawTaxonRecord, Taxon, TaxonId};

/// Parses and validates a slice of raw taxon records into domain `Taxon` models.
///
/// Ensures that:
/// - The input is not empty.
/// - All taxon names are non-empty after trimming.
/// - All taxon IDs are unique across the dataset.
///
/// # Arguments
///
/// * `raw` - Slice of [`RawTaxonRecord`] to ingest.
///
/// # Returns
///
/// `Ok(Vec<Taxon>)` containing the mapped domain records, or a [`PipelineError`]
/// if validation fails.
///
/// # Example
///
/// ```rust,ignore
/// use taxonomy::types::{RawTaxonRecord, TaxonId};
/// use taxonomy::ingestion::parse_taxons;
/// use kalimantanbio_shared::core::TaxonomicRank;
///
/// let raw = vec![RawTaxonRecord {
///     id: TaxonId(1),
///     name: "Plantae".to_string(),
///     rank: TaxonomicRank::Kingdom,
///     parent_id: None,
/// }];
/// let taxons = parse_taxons(&raw).unwrap();
/// assert_eq!(taxons.len(), 1);
/// ```
pub fn parse_taxons(raw: &[RawTaxonRecord]) -> Result<Vec<Taxon>, PipelineError> {
    if raw.is_empty() {
        return Err(PipelineError::Empty(
            "Raw taxon input dataset cannot be empty".to_string(),
        ));
    }

    // Functional validation for empty names
    let has_empty_name = raw.iter().any(|r| r.name.trim().is_empty());
    if has_empty_name {
        return Err(PipelineError::Validation(
            "Taxon record contains an empty or whitespace-only name".to_string(),
        ));
    }

    // Functional duplicate detection using fold and HashSet
    let (has_duplicate_id, _) = raw
        .iter()
        .fold((false, HashSet::new()), |(duplicate_found, mut seen), record| {
            if duplicate_found || !seen.insert(record.id) {
                (true, seen)
            } else {
                (false, seen)
            }
        });

    if has_duplicate_id {
        return Err(PipelineError::Validation(
            "Duplicate taxon ID detected in input records".to_string(),
        ));
    }

    // Pure mapping from raw records to domain Taxon
    let taxons = raw
        .iter()
        .map(|r| Taxon {
            id: r.id,
            name: r.name.trim().to_string(),
            rank: r.rank,
            parent_id: r.parent_id,
        })
        .collect();

    Ok(taxons)
}

/// Parses and validates a slice of raw species records.
///
/// Ensures that:
/// - The input is not empty.
/// - All scientific names are non-empty after trimming.
/// - All species IDs are unique across the dataset.
///
/// # Arguments
///
/// * `raw` - Slice of [`RawSpeciesRecord`] to ingest.
///
/// # Returns
///
/// `Ok(Vec<RawSpeciesRecord>)` containing the sanitized records, or a [`PipelineError`]
/// if validation fails.
///
/// # Example
///
/// ```rust,ignore
/// use taxonomy::types::{RawSpeciesRecord, TaxonId};
/// use taxonomy::ingestion::parse_species;
///
/// let raw = vec![RawSpeciesRecord {
///     id: 101,
///     scientific_name: "Pongo pygmaeus".to_string(),
///     genus_id: TaxonId(5),
/// }];
/// let species = parse_species(&raw).unwrap();
/// assert_eq!(species.len(), 1);
/// ```
pub fn parse_species(raw: &[RawSpeciesRecord]) -> Result<Vec<RawSpeciesRecord>, PipelineError> {
    if raw.is_empty() {
        return Err(PipelineError::Empty(
            "Raw species input dataset cannot be empty".to_string(),
        ));
    }

    // Check for empty scientific names
    let has_empty_name = raw.iter().any(|r| r.scientific_name.trim().is_empty());
    if has_empty_name {
        return Err(PipelineError::Validation(
            "Species record contains an empty or whitespace-only scientific name".to_string(),
        ));
    }

    // Check for duplicate species IDs
    let (has_duplicate_id, _) = raw
        .iter()
        .fold((false, HashSet::new()), |(duplicate_found, mut seen), record| {
            if duplicate_found || !seen.insert(record.id) {
                (true, seen)
            } else {
                (false, seen)
            }
        });

    if has_duplicate_id {
        return Err(PipelineError::Validation(
            "Duplicate species ID detected in input records".to_string(),
        ));
    }

    // Pure mapping/sanitization of species records
    let species = raw
        .iter()
        .map(|r| RawSpeciesRecord {
            id: r.id,
            scientific_name: r.scientific_name.trim().to_string(),
            genus_id: r.genus_id,
        })
        .collect();

    Ok(species)
}

/// Validates hierarchical integrity of a collection of `Taxon` nodes.
///
/// Pure function that checks:
/// 1. **No Orphan Nodes:** Every taxon with a `parent_id` must reference an ID present in the dataset.
/// 2. **No Self-References:** A taxon cannot be its own parent (`parent_id != Some(id)`).
/// 3. **No Cycles:** Traversing upward from any taxon along the `parent_id` chain must terminate
///    at a root node (`parent_id: None`) without revisiting any node.
///
/// # Arguments
///
/// * `taxons` - Slice of [`Taxon`] nodes to validate.
///
/// # Returns
///
/// `true` if the entire hierarchy forms a valid forest/tree (acyclic and without orphan nodes),
/// `false` otherwise.
///
/// # Example
///
/// ```rust,ignore
/// use taxonomy::types::{Taxon, TaxonId};
/// use taxonomy::ingestion::validate_hierarchy;
/// use kalimantanbio_shared::core::TaxonomicRank;
///
/// let root = Taxon {
///     id: TaxonId(1),
///     name: "Animalia".to_string(),
///     rank: TaxonomicRank::Kingdom,
///     parent_id: None,
/// };
/// let child = Taxon {
///     id: TaxonId(2),
///     name: "Chordata".to_string(),
///     rank: TaxonomicRank::Phylum,
///     parent_id: Some(TaxonId(1)),
/// };
/// assert!(validate_hierarchy(&[root, child]));
/// ```
pub fn validate_hierarchy(taxons: &[Taxon]) -> bool {
    if taxons.is_empty() {
        return true;
    }

    let all_ids: HashSet<TaxonId> = taxons.iter().map(|t| t.id).collect();

    // Check for orphan nodes: parent_id exists but is not in all_ids
    let has_orphan = taxons.iter().any(|t| match t.parent_id {
        Some(pid) => !all_ids.contains(&pid),
        None => false,
    });

    if has_orphan {
        return false;
    }

    // Check for self-reference
    let has_self_reference = taxons.iter().any(|t| t.parent_id == Some(t.id));
    if has_self_reference {
        return false;
    }

    // Map each taxon ID to its parent ID for fast lookup during cycle detection
    let parent_map: std::collections::HashMap<TaxonId, Option<TaxonId>> =
        taxons.iter().map(|t| (t.id, t.parent_id)).collect();

    // Pure cycle detection for each taxon node
    let has_cycle = taxons.iter().any(|start_taxon| {
        let mut visited_in_path = HashSet::new();
        let mut current_id = Some(start_taxon.id);

        while let Some(id) = current_id {
            if !visited_in_path.insert(id) {
                // Cycle detected
                return true;
            }
            current_id = parent_map.get(&id).copied().flatten();
        }

        false
    });

    !has_cycle
}

#[cfg(test)]
mod tests {
    use super::*;
    use kalimantanbio_shared::core::TaxonomicRank;

    #[test]
    fn test_parse_taxons_valid() {
        let raw = vec![
            RawTaxonRecord {
                id: TaxonId(1),
                name: " Plantae ".to_string(),
                rank: TaxonomicRank::Kingdom,
                parent_id: None,
            },
            RawTaxonRecord {
                id: TaxonId(2),
                name: "Dipterocarpaceae".to_string(),
                rank: TaxonomicRank::Family,
                parent_id: Some(TaxonId(1)),
            },
        ];

        let result = parse_taxons(&raw);
        assert!(result.is_ok());
        let taxons = result.unwrap();
        assert_eq!(taxons.len(), 2);
        assert_eq!(taxons[0].name, "Plantae");
        assert_eq!(taxons[1].name, "Dipterocarpaceae");
    }

    #[test]
    fn test_parse_taxons_empty_input() {
        let raw = vec![];
        let result = parse_taxons(&raw);
        assert!(matches!(result, Err(PipelineError::Empty(_))));
    }

    #[test]
    fn test_parse_taxons_empty_name() {
        let raw = vec![RawTaxonRecord {
            id: TaxonId(1),
            name: "   ".to_string(),
            rank: TaxonomicRank::Kingdom,
            parent_id: None,
        }];
        let result = parse_taxons(&raw);
        assert!(matches!(result, Err(PipelineError::Validation(_))));
    }

    #[test]
    fn test_parse_taxons_duplicate_id() {
        let raw = vec![
            RawTaxonRecord {
                id: TaxonId(1),
                name: "Plantae".to_string(),
                rank: TaxonomicRank::Kingdom,
                parent_id: None,
            },
            RawTaxonRecord {
                id: TaxonId(1),
                name: "Animalia".to_string(),
                rank: TaxonomicRank::Kingdom,
                parent_id: None,
            },
        ];
        let result = parse_taxons(&raw);
        assert!(matches!(result, Err(PipelineError::Validation(_))));
    }

    #[test]
    fn test_parse_species_valid() {
        let raw = vec![
            RawSpeciesRecord {
                id: 1,
                scientific_name: " Shorea leprosula ".to_string(),
                genus_id: TaxonId(10),
            },
            RawSpeciesRecord {
                id: 2,
                scientific_name: "Pongo pygmaeus".to_string(),
                genus_id: TaxonId(20),
            },
        ];

        let result = parse_species(&raw);
        assert!(result.is_ok());
        let species = result.unwrap();
        assert_eq!(species.len(), 2);
        assert_eq!(species[0].scientific_name, "Shorea leprosula");
    }

    #[test]
    fn test_parse_species_empty_input() {
        let raw = vec![];
        let result = parse_species(&raw);
        assert!(matches!(result, Err(PipelineError::Empty(_))));
    }

    #[test]
    fn test_parse_species_empty_name() {
        let raw = vec![RawSpeciesRecord {
            id: 1,
            scientific_name: "  ".to_string(),
            genus_id: TaxonId(10),
        }];
        let result = parse_species(&raw);
        assert!(matches!(result, Err(PipelineError::Validation(_))));
    }

    #[test]
    fn test_parse_species_duplicate_id() {
        let raw = vec![
            RawSpeciesRecord {
                id: 10,
                scientific_name: "Species A".to_string(),
                genus_id: TaxonId(1),
            },
            RawSpeciesRecord {
                id: 10,
                scientific_name: "Species B".to_string(),
                genus_id: TaxonId(2),
            },
        ];
        let result = parse_species(&raw);
        assert!(matches!(result, Err(PipelineError::Validation(_))));
    }

    #[test]
    fn test_validate_hierarchy_valid_tree() {
        let taxons = vec![
            Taxon {
                id: TaxonId(1),
                name: "Animalia".to_string(),
                rank: TaxonomicRank::Kingdom,
                parent_id: None,
            },
            Taxon {
                id: TaxonId(2),
                name: "Chordata".to_string(),
                rank: TaxonomicRank::Phylum,
                parent_id: Some(TaxonId(1)),
            },
            Taxon {
                id: TaxonId(3),
                name: "Mammalia".to_string(),
                rank: TaxonomicRank::Class,
                parent_id: Some(TaxonId(2)),
            },
        ];

        assert!(validate_hierarchy(&taxons));
    }

    #[test]
    fn test_validate_hierarchy_orphan_node() {
        let taxons = vec![Taxon {
            id: TaxonId(2),
            name: "Chordata".to_string(),
            rank: TaxonomicRank::Phylum,
            parent_id: Some(TaxonId(999)), // Non-existent parent
        }];

        assert!(!validate_hierarchy(&taxons));
    }

    #[test]
    fn test_validate_hierarchy_self_reference() {
        let taxons = vec![Taxon {
            id: TaxonId(1),
            name: "LoopNode".to_string(),
            rank: TaxonomicRank::Kingdom,
            parent_id: Some(TaxonId(1)), // Self-parent
        }];

        assert!(!validate_hierarchy(&taxons));
    }

    #[test]
    fn test_validate_hierarchy_cycle_detected() {
        let taxons = vec![
            Taxon {
                id: TaxonId(1),
                name: "A".to_string(),
                rank: TaxonomicRank::Kingdom,
                parent_id: Some(TaxonId(3)), // 1 -> 3 -> 2 -> 1
            },
            Taxon {
                id: TaxonId(2),
                name: "B".to_string(),
                rank: TaxonomicRank::Phylum,
                parent_id: Some(TaxonId(1)),
            },
            Taxon {
                id: TaxonId(3),
                name: "C".to_string(),
                rank: TaxonomicRank::Class,
                parent_id: Some(TaxonId(2)),
            },
        ];

        assert!(!validate_hierarchy(&taxons));
    }

    #[test]
    fn test_validate_hierarchy_empty() {
        let taxons = vec![];
        assert!(validate_hierarchy(&taxons));
    }
}
