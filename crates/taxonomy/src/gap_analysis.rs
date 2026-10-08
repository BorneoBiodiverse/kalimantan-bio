//! Module 3 Tahap 4 — Coverage & Gap Analysis.
//!
//! Responsible for comparing local taxonomic data against global reference checklists
//! to discover taxonomic gaps (missing taxa) and calculate dataset coverage score
//! using pure functional programming and set operations.
//!
//! # Planning Reference
//! - Document: `Planning_Modul_3_Taxonomy.md` §7 (Tahap 4 — Coverage & Gap Analysis)
//! - PIC: Anggota 4 — Analisis coverage dan gap (wisnu - 11241063)

use std::collections::HashSet;

use kalimantanbio_shared::collections::set_difference;
use kalimantanbio_shared::core::TaxonomicRank;
use kalimantanbio_shared::stats::calculate_coverage_percentage;

use crate::types::{GapReport, Taxon};

/// Extracts all unique genus names from a collection of [`Taxon`] nodes.
///
/// Filters taxon records where `rank == TaxonomicRank::Genus`, trims whitespace,
/// discards empty names, and collects the result into a [`HashSet<String>`].
///
/// # Arguments
///
/// * `taxons` - Slice of [`Taxon`] records from the local dataset.
///
/// # Returns
///
/// A [`HashSet<String>`] containing unique, sanitized genus names present in the dataset.
///
/// # Example
///
/// ```rust,ignore
/// use taxonomy::types::{Taxon, TaxonId};
/// use taxonomy::gap_analysis::extract_our_genera;
/// use kalimantanbio_shared::core::TaxonomicRank;
///
/// let taxons = vec![
///     Taxon {
///         id: TaxonId(1),
///         name: "Shorea".to_string(),
///         rank: TaxonomicRank::Genus,
///         parent_id: None,
///     },
///     Taxon {
///         id: TaxonId(2),
///         name: "Dipterocarpaceae".to_string(),
///         rank: TaxonomicRank::Family,
///         parent_id: None,
///     },
/// ];
/// let genera = extract_our_genera(&taxons);
/// assert!(genera.contains("Shorea"));
/// assert_eq!(genera.len(), 1);
/// ```
pub fn extract_our_genera(taxons: &[Taxon]) -> HashSet<String> {
    taxons
        .iter()
        .filter(|t| t.rank == TaxonomicRank::Genus)
        .map(|t| t.name.trim().to_string())
        .filter(|name| !name.is_empty())
        .collect()
}

/// Finds taxa present in the reference checklist that are missing from our local dataset.
///
/// Uses set difference (`reference_genera \ our_genera`) via the shared library
/// [`set_difference`] function, returning an alphabetically sorted list of missing taxa.
///
/// # Arguments
///
/// * `reference_genera` - Set of expected genus names from reference literature/checklists.
/// * `our_genera` - Set of genus names available in our local dataset.
///
/// # Returns
///
/// A sorted [`Vec<String>`] of genus names that exist in `reference_genera` but not in `our_genera`.
///
/// # Example
///
/// ```rust,ignore
/// use std::collections::HashSet;
/// use taxonomy::gap_analysis::find_missing_taxa;
///
/// let reference: HashSet<String> = ["Dipterocarpus", "Shorea", "Vatica"].into_iter().map(String::from).collect();
/// let our_db: HashSet<String> = ["Shorea"].into_iter().map(String::from).collect();
///
/// let missing = find_missing_taxa(&reference, &our_db);
/// assert_eq!(missing, vec!["Dipterocarpus".to_string(), "Vatica".to_string()]);
/// ```
pub fn find_missing_taxa(
    reference_genera: &HashSet<String>,
    our_genera: &HashSet<String>,
) -> Vec<String> {
    let mut missing = set_difference(reference_genera, our_genera);
    missing.sort();
    missing
}

/// Calculates the coverage score (ratio between 0.0 and 1.0) of available items against total expected.
///
/// Uses [`calculate_coverage_percentage`] from the shared library.
/// Handles edge cases gracefully (e.g., returns `0.0` if total expected items is `0`).
///
/// # Arguments
///
/// * `available` - Number of covered/matching taxa.
/// * `total` - Total expected taxa from reference checklist.
///
/// # Returns
///
/// A coverage ratio `f64` in the range `[0.0, 1.0]`.
///
/// # Example
///
/// ```rust,ignore
/// use taxonomy::gap_analysis::calculate_coverage_score;
///
/// let score = calculate_coverage_score(80, 100);
/// assert!((score - 0.80).abs() < f64::EPSILON);
/// ```
pub fn calculate_coverage_score(available: usize, total: usize) -> f64 {
    calculate_coverage_percentage(available, total)
}

/// Constructs a [`GapReport`] from missing taxa list and coverage score.
///
/// Ensures that `missing_taxa` is consistently sorted and `coverage_score` is clamped.
///
/// # Arguments
///
/// * `missing` - Slice of missing taxon names.
/// * `score` - Calculated coverage score.
///
/// # Returns
///
/// A populated [`GapReport`] structure.
///
/// # Example
///
/// ```rust,ignore
/// use taxonomy::gap_analysis::generate_gap_report;
///
/// let report = generate_gap_report(&["Vatica".to_string()], 0.85);
/// assert_eq!(report.missing_taxa, vec!["Vatica".to_string()]);
/// assert_eq!(report.coverage_score, 0.85);
/// ```
pub fn generate_gap_report(missing: &[String], score: f64) -> GapReport {
    let mut missing_taxa = missing.to_vec();
    missing_taxa.sort();
    GapReport {
        missing_taxa,
        coverage_score: score.clamp(0.0, 1.0),
    }
}

/// Analyzes taxonomic gaps by comparing platform dataset genera against an external reference checklist.
///
/// Composes set difference operations and coverage calculations:
/// 1. Finds missing taxa via [`find_missing_taxa`].
/// 2. Calculates the number of reference taxa covered: `reference_count - missing_count`.
/// 3. Computes the coverage score via [`calculate_coverage_score`].
/// 4. Assembles the final [`GapReport`] via [`generate_gap_report`].
///
/// # Arguments
///
/// * `our_genera` - Set of genera present in the platform dataset.
/// * `reference_genera` - Set of genera expected from the reference taxonomy.
///
/// # Returns
///
/// A [`GapReport`] containing the missing taxa and the coverage score.
///
/// # Example
///
/// ```rust,ignore
/// use std::collections::HashSet;
/// use taxonomy::gap_analysis::analyze_taxonomic_gap;
///
/// let our_genera: HashSet<String> = ["Shorea", "Hopea"].into_iter().map(String::from).collect();
/// let ref_genera: HashSet<String> = ["Shorea", "Hopea", "Vatica"].into_iter().map(String::from).collect();
///
/// let report = analyze_taxonomic_gap(&our_genera, &ref_genera);
/// assert_eq!(report.missing_taxa, vec!["Vatica".to_string()]);
/// assert!((report.coverage_score - (2.0 / 3.0)).abs() < 1e-6);
/// ```
pub fn analyze_taxonomic_gap(
    our_genera: &HashSet<String>,
    reference_genera: &HashSet<String>,
) -> GapReport {
    let missing = find_missing_taxa(reference_genera, our_genera);
    let covered_count = reference_genera.len().saturating_sub(missing.len());
    let score = calculate_coverage_score(covered_count, reference_genera.len());
    generate_gap_report(&missing, score)
}

/// Convenience function to run gap analysis directly from a slice of [`Taxon`] nodes.
///
/// First extracts local genus names using [`extract_our_genera`], then delegates
/// to [`analyze_taxonomic_gap`].
///
/// # Arguments
///
/// * `taxons` - Slice of local [`Taxon`] records.
/// * `reference_genera` - Set of expected genus names from reference taxonomy.
///
/// # Returns
///
/// A [`GapReport`] containing missing taxa and coverage score.
///
/// # Example
///
/// ```rust,ignore
/// use std::collections::HashSet;
/// use taxonomy::types::{Taxon, TaxonId};
/// use taxonomy::gap_analysis::analyze_taxonomic_gap_from_taxons;
/// use kalimantanbio_shared::core::TaxonomicRank;
///
/// let taxons = vec![Taxon {
///     id: TaxonId(1),
///     name: "Shorea".to_string(),
///     rank: TaxonomicRank::Genus,
///     parent_id: None,
/// }];
/// let reference: HashSet<String> = ["Shorea", "Vatica"].into_iter().map(String::from).collect();
///
/// let report = analyze_taxonomic_gap_from_taxons(&taxons, &reference);
/// assert_eq!(report.missing_taxa, vec!["Vatica".to_string()]);
/// assert!((report.coverage_score - 0.5).abs() < f64::EPSILON);
/// ```
pub fn analyze_taxonomic_gap_from_taxons(
    taxons: &[Taxon],
    reference_genera: &HashSet<String>,
) -> GapReport {
    let our_genera = extract_our_genera(taxons);
    analyze_taxonomic_gap(&our_genera, reference_genera)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::TaxonId;

    #[test]
    fn test_extract_our_genera_filters_only_genera() {
        let taxons = vec![
            Taxon {
                id: TaxonId(1),
                name: "Plantae".to_string(),
                rank: TaxonomicRank::Kingdom,
                parent_id: None,
            },
            Taxon {
                id: TaxonId(2),
                name: "Dipterocarpaceae".to_string(),
                rank: TaxonomicRank::Family,
                parent_id: Some(TaxonId(1)),
            },
            Taxon {
                id: TaxonId(3),
                name: "  Shorea  ".to_string(),
                rank: TaxonomicRank::Genus,
                parent_id: Some(TaxonId(2)),
            },
            Taxon {
                id: TaxonId(4),
                name: "Hopea".to_string(),
                rank: TaxonomicRank::Genus,
                parent_id: Some(TaxonId(2)),
            },
            Taxon {
                id: TaxonId(5),
                name: "Shorea leprosula".to_string(),
                rank: TaxonomicRank::Species,
                parent_id: Some(TaxonId(3)),
            },
        ];

        let genera = extract_our_genera(&taxons);
        assert_eq!(genera.len(), 2);
        assert!(genera.contains("Shorea"));
        assert!(genera.contains("Hopea"));
        assert!(!genera.contains("Plantae"));
        assert!(!genera.contains("Dipterocarpaceae"));
        assert!(!genera.contains("Shorea leprosula"));
    }

    #[test]
    fn test_extract_our_genera_empty_input() {
        let taxons = vec![];
        let genera = extract_our_genera(&taxons);
        assert!(genera.is_empty());
    }

    #[test]
    fn test_extract_our_genera_ignores_empty_and_whitespace_names() {
        let taxons = vec![
            Taxon {
                id: TaxonId(1),
                name: "   ".to_string(),
                rank: TaxonomicRank::Genus,
                parent_id: None,
            },
            Taxon {
                id: TaxonId(2),
                name: "".to_string(),
                rank: TaxonomicRank::Genus,
                parent_id: None,
            },
            Taxon {
                id: TaxonId(3),
                name: "Durio".to_string(),
                rank: TaxonomicRank::Genus,
                parent_id: None,
            },
        ];

        let genera = extract_our_genera(&taxons);
        assert_eq!(genera.len(), 1);
        assert!(genera.contains("Durio"));
    }

    #[test]
    fn test_find_missing_taxa_scenario_2_planning() {
        // From Planning_Modul_3_Taxonomy §15 Skenario 2:
        // our_db = ["Genus A", "Genus B"], reference = ["Genus A", "Genus B", "Genus C"]
        // Expected missing: ["Genus C"]
        let our_db: HashSet<String> = vec!["Genus A".to_string(), "Genus B".to_string()]
            .into_iter()
            .collect();
        let reference: HashSet<String> = vec![
            "Genus A".to_string(),
            "Genus B".to_string(),
            "Genus C".to_string(),
        ]
        .into_iter()
        .collect();

        let missing = find_missing_taxa(&reference, &our_db);
        assert_eq!(missing, vec!["Genus C".to_string()]);
    }

    #[test]
    fn test_find_missing_taxa_all_covered() {
        let our_db: HashSet<String> = vec!["Shorea".to_string(), "Hopea".to_string()]
            .into_iter()
            .collect();
        let reference: HashSet<String> = vec!["Shorea".to_string(), "Hopea".to_string()]
            .into_iter()
            .collect();

        let missing = find_missing_taxa(&reference, &our_db);
        assert!(missing.is_empty());
    }

    #[test]
    fn test_find_missing_taxa_none_covered() {
        let our_db: HashSet<String> = vec!["Pongo".to_string(), "Nasalis".to_string()]
            .into_iter()
            .collect();
        let reference: HashSet<String> = vec!["Shorea".to_string(), "Dipterocarpus".to_string()]
            .into_iter()
            .collect();

        let missing = find_missing_taxa(&reference, &our_db);
        assert_eq!(
            missing,
            vec!["Dipterocarpus".to_string(), "Shorea".to_string()]
        );
    }

    #[test]
    fn test_calculate_coverage_score() {
        assert_eq!(calculate_coverage_score(80, 100), 0.80);
        assert_eq!(calculate_coverage_score(100, 100), 1.0);
        assert_eq!(calculate_coverage_score(0, 100), 0.0);
        assert_eq!(calculate_coverage_score(0, 0), 0.0);
        assert_eq!(calculate_coverage_score(50, 0), 0.0);
    }

    #[test]
    fn test_generate_gap_report() {
        let missing = vec!["Vatica".to_string(), "Anisoptera".to_string()];
        let report = generate_gap_report(&missing, 0.75);

        assert_eq!(
            report.missing_taxa,
            vec!["Anisoptera".to_string(), "Vatica".to_string()]
        );
        assert_eq!(report.coverage_score, 0.75);
    }

    #[test]
    fn test_analyze_taxonomic_gap_full_pipeline() {
        let our_genera: HashSet<String> =
            vec!["Shorea".to_string(), "Hopea".to_string(), "Durio".to_string()]
                .into_iter()
                .collect();
        let ref_genera: HashSet<String> = vec![
            "Shorea".to_string(),
            "Hopea".to_string(),
            "Vatica".to_string(),
            "Dipterocarpus".to_string(),
        ]
        .into_iter()
        .collect();

        let report = analyze_taxonomic_gap(&our_genera, &ref_genera);

        // Reference has 4 genera: Shorea, Hopea, Vatica, Dipterocarpus
        // our_genera covers 2 of them (Shorea, Hopea)
        // missing: Dipterocarpus, Vatica
        // score: 2 / 4 = 0.50
        assert_eq!(
            report.missing_taxa,
            vec!["Dipterocarpus".to_string(), "Vatica".to_string()]
        );
        assert!((report.coverage_score - 0.50).abs() < f64::EPSILON);
    }

    #[test]
    fn test_analyze_taxonomic_gap_empty_reference() {
        let our_genera: HashSet<String> = vec!["Shorea".to_string()].into_iter().collect();
        let ref_genera: HashSet<String> = HashSet::new();

        let report = analyze_taxonomic_gap(&our_genera, &ref_genera);
        assert!(report.missing_taxa.is_empty());
        assert_eq!(report.coverage_score, 0.0);
    }

    #[test]
    fn test_analyze_taxonomic_gap_from_taxons() {
        let taxons = vec![
            Taxon {
                id: TaxonId(1),
                name: "Plantae".to_string(),
                rank: TaxonomicRank::Kingdom,
                parent_id: None,
            },
            Taxon {
                id: TaxonId(2),
                name: "Shorea".to_string(),
                rank: TaxonomicRank::Genus,
                parent_id: Some(TaxonId(1)),
            },
        ];

        let reference: HashSet<String> =
            vec!["Shorea".to_string(), "Vatica".to_string(), "Hopea".to_string()]
                .into_iter()
                .collect();

        let report = analyze_taxonomic_gap_from_taxons(&taxons, &reference);
        assert_eq!(
            report.missing_taxa,
            vec!["Hopea".to_string(), "Vatica".to_string()]
        );
        assert!((report.coverage_score - (1.0 / 3.0)).abs() < 1e-6);
    }
}
