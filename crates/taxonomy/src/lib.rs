#![warn(missing_docs)]
//! Module 3 — Taxonomy & Classification Explorer.
//!
//! Provides interactive exploration of biological taxonomy and classification:
//! taxonomic tree construction, diversity analysis, gap analysis, and endemic
//! taxa exploration.
//!
//! Public API: `generate_taxonomy_report`, `calculate_diversity`,
//! `analyze_taxonomic_gap`. Internal helpers stay private.
//!
//! > **Status: interface scaffold.** Bodies are stubs pending implementation.

pub mod types;

use std::collections::HashSet;

use kalimantanbio_shared::core::Species;

use crate::types::{BioDataInput, DiversityStats, ExplorationReport, GapReport, PipelineError};

/// Analyze which genera are under-represented versus a reference set.
///
/// # Arguments
///
/// * `our_genera` - Genera present in the platform dataset.
/// * `reference_genera` - Genera expected from the reference taxonomy.
///
/// # Returns
///
/// Missing taxa and the coverage score.
///
/// # Example
///
/// ```rust,ignore
/// let report = analyze_taxonomic_gap(&our_genera, &reference_genera);
/// ```
pub fn analyze_taxonomic_gap(
    our_genera: &HashSet<String>,
    reference_genera: &HashSet<String>,
) -> GapReport {
    todo!("taxonomy phase 5: analyze_taxonomic_gap")
}

/// Calculate diversity statistics for a species list.
///
/// # Arguments
///
/// * `species_list` - The species dataset.
///
/// # Returns
///
/// Family/genus/species counts and endemic species count.
///
/// # Example
///
/// ```rust,ignore
/// let stats = calculate_diversity(&species);
/// ```
pub fn calculate_diversity(species_list: &[Species]) -> DiversityStats {
    todo!("taxonomy phase 5: calculate_diversity")
}

/// Generate the full taxonomy exploration report.
///
/// Pipeline: parse → validate → build tree → diversity → endemic → gap
/// analysis → assemble the report.
///
/// # Arguments
///
/// * `input` - Raw taxon data, raw species data, and reference genera.
///
/// # Returns
///
/// The exploration report, or a pipeline error.
///
/// # Example
///
/// ```rust,ignore
/// let report = generate_taxonomy_report(&input)?;
/// ```
pub fn generate_taxonomy_report(
    input: &BioDataInput,
) -> Result<ExplorationReport, PipelineError> {
    todo!("taxonomy phase 5: generate_taxonomy_report")
}