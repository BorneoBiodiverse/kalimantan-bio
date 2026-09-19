//! Module-specific types for the Comparative Species Explorer.

use serde::Serialize;
/// Query for a multi-species comparison.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct ComparisonQuery {
    /// Species identifiers to compare.
    pub species_ids: Vec<u64>,
}

/// Scoring weights per comparison dimension.
#[derive(Debug, Clone, Serialize)]
pub struct ScoringWeights {
    /// Taxonomy weight.
    pub taxonomy: f64,
    /// Conservation status weight.
    pub conservation: f64,
    /// Species-type weight.
    pub species_type: f64,
    /// Geographic distribution weight.
    pub distribution: f64,
}

/// Similarity score of a species pair.
#[derive(Debug, Clone, Serialize)]
pub struct SimilarityScore {
    /// First species identifier.
    pub species_a_id: u64,
    /// Second species identifier.
    pub species_b_id: u64,
    /// Taxonomy similarity.
    pub taxonomy_score: f64,
    /// Conservation similarity.
    pub conservation_score: f64,
    /// Species-type similarity.
    pub type_score: f64,
    /// Distribution similarity.
    pub distribution_score: f64,
    /// Combined total score.
    pub total_score: f64,
}

/// One row of the attribute comparison matrix.
#[derive(Debug, Clone, Serialize)]
pub struct SpeciesAttributeRow {
    /// Species identifier.
    pub species_id: u64,
    /// Scientific name.
    pub scientific_name: String,
    /// Common name.
    pub common_name: String,
    /// Full taxonomy path.
    pub taxonomy_path: String,
    /// Species origin type.
    pub species_type: String,
    /// IUCN status.
    pub iucn: String,
    /// CITES appendix.
    pub cites: String,
    /// National protection status.
    pub p106: String,
    /// Observation count.
    pub observation_count: u32,
    /// Regent regions where the species is recorded.
    pub kabupaten_list: Vec<String>,
    /// Verification flag.
    pub is_verified: bool,
}

/// Result of a multi-species comparison.
#[derive(Debug, Clone, Default, Serialize)]
pub struct ComparisonResult {
    /// The query that produced this result.
    pub query: ComparisonQuery,
    /// Attribute matrix for the compared species.
    pub attribute_matrix: Vec<SpeciesAttributeRow>,
    /// Attributes shared by all compared species.
    pub shared_attributes: Vec<String>,
    /// Attributes unique to individual species.
    pub unique_attributes: Vec<String>,
    /// Distinguishing characteristics.
    pub distinguishing_characteristics: Vec<String>,
    /// Pairwise similarity scores.
    pub similarity_scores: Vec<SimilarityScore>,
    /// Human-readable comparison summary.
    pub summary: String,
}

/// Errors reported by the comparison pipeline.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum ModuleError {
    /// The query is invalid (too few/target ids, duplicates).
    InvalidQuery(String),
    /// A requested species was not found.
    SpeciesNotFound(u64),
    /// The datasets produced no comparable results.
    Empty(String),
    /// A validation rule was violated.
    Validation(String),
}