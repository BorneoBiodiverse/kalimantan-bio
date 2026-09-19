//! Module-specific types for the Taxonomy & Classification Explorer.

use std::collections::HashSet;

use kalimantanbio_shared::core::TaxonomicRank;
use serde::Serialize;

/// Identifier of a taxon in the taxonomy tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub struct TaxonId(pub u64);

/// A node in the taxonomy tree.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub struct Taxon {
    /// Taxon identifier.
    pub id: TaxonId,
    /// Taxon name.
    pub name: String,
    /// Taxon rank.
    pub rank: TaxonomicRank,
    /// Parent taxon identifier, when present.
    pub parent_id: Option<TaxonId>,
}

/// Raw taxon record used as pipeline input.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub struct RawTaxonRecord {
    /// Taxon identifier.
    pub id: TaxonId,
    /// Taxon name.
    pub name: String,
    /// Taxon rank.
    pub rank: TaxonomicRank,
    /// Parent taxon identifier, when present.
    pub parent_id: Option<TaxonId>,
}

/// Raw species record used as pipeline input.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RawSpeciesRecord {
    /// Species identifier.
    pub id: u64,
    /// Scientific name.
    pub scientific_name: String,
    /// Genus identifier the species belongs to.
    pub genus_id: TaxonId,
}

/// Diversity statistics of a dataset.
#[derive(Debug, Clone, Default, Serialize)]
pub struct DiversityStats {
    /// Number of distinct families.
    pub total_families: usize,
    /// Number of distinct genera.
    pub total_genera: usize,
    /// Number of species.
    pub total_species: usize,
    /// Number of endemic species.
    pub endemic_species_count: usize,
}

/// Input for the taxonomy exploration pipeline.
#[derive(Debug, Clone, Default, Serialize)]
pub struct BioDataInput {
    /// Raw taxon records.
    pub raw_taxons: Vec<RawTaxonRecord>,
    /// Raw species records.
    pub raw_species: Vec<RawSpeciesRecord>,
    /// Reference genera used for gap analysis.
    pub reference_genera: HashSet<String>,
}

/// Result of a taxonomic gap analysis.
#[derive(Debug, Clone, Default, Serialize)]
pub struct GapReport {
    /// Taxa missing from the dataset.
    pub missing_taxa: Vec<String>,
    /// Coverage score between 0.0 and 1.0.
    pub coverage_score: f64,
}

/// Full taxonomy exploration report.
#[derive(Debug, Clone, Default, Serialize)]
pub struct ExplorationReport {
    /// Endemic genera found.
    pub endemic_genera: Vec<Taxon>,
    /// Genera ranked by species richness.
    pub richness_rank: Vec<(Taxon, usize)>,
    /// Taxa missing from the dataset.
    pub missing_taxa: Vec<String>,
    /// Coverage score between 0.0 and 1.0.
    pub coverage_score: f64,
}

/// Errors reported by the taxonomy pipeline.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum PipelineError {
    /// Failed to parse input data.
    Parse(String),
    /// Failed to validate input data.
    Validation(String),
    /// The dataset produced no results.
    Empty(String),
}