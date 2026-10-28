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

impl Default for ScoringWeights {
    /// Agreed baseline: taxonomy `0.40`, conservation `0.30`, species type
    /// `0.15`, distribution `0.15` — the four values sum to `1.0`.
    ///
    /// `compare_species` receives no weights from its caller (three-parameter
    /// contract), so every pair it scores uses this baseline.
    fn default() -> Self {
        Self {
            taxonomy: 0.40,
            conservation: 0.30,
            species_type: 0.15,
            distribution: 0.15,
        }
    }
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

/// Query validation failures, raised before any species data is fetched.
///
/// Internal to the crate: the public API only ever exposes [`ModuleError`].
/// Kept separate from the shared `ValidationError` so the three
/// comparison-specific failure modes stay directly assertable in unit tests.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum QueryError {
    /// Fewer than the minimum number of species.
    TooFewSpecies,
    /// The same species id was requested twice.
    DuplicateId,
    /// More species than the agreed maximum.
    TooManySpecies,
}

impl From<QueryError> for ModuleError {
    fn from(error: QueryError) -> Self {
        let message = match error {
            QueryError::TooFewSpecies => {
                format!("too few species (minimum {})", crate::MIN_SPECIES)
            }
            QueryError::DuplicateId => String::from("duplicate species id"),
            QueryError::TooManySpecies => {
                format!("too many species (maximum {})", crate::MAX_SPECIES)
            }
        };

        ModuleError::InvalidQuery(message)
    }
}

/// Data retrieval failures raised while resolving a comparison query.
///
/// Internal to the crate, like [`QueryError`]: callers of the public API only
/// ever see [`ModuleError`]. Kept precise so a test can assert the exact
/// missing id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum DataError {
    /// A requested species id is absent from the provided dataset.
    SpeciesNotFound(u64),
}

impl From<DataError> for ModuleError {
    /// Convert a retrieval failure into the module's public error, so the `?`
    /// operator can propagate it out of `compare_species`.
    fn from(error: DataError) -> Self {
        match error {
            DataError::SpeciesNotFound(id) => ModuleError::SpeciesNotFound(id),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_weights_match_the_agreed_scale() {
        let weights = ScoringWeights::default();
        assert_eq!(weights.taxonomy, 0.40);
        assert_eq!(weights.conservation, 0.30);
        assert_eq!(weights.species_type, 0.15);
        assert_eq!(weights.distribution, 0.15);
    }

    #[test]
    fn default_weights_sum_to_one() {
        let w = ScoringWeights::default();
        let sum = w.taxonomy + w.conservation + w.species_type + w.distribution;
        // Binary floats make 0.40 + 0.30 + 0.15 + 0.15 land slightly above 1.0,
        // so the check uses the same 1e-9 tolerance as the shared
        // `validate_weights_sum_to_one` contract.
        assert!(
            (sum - 1.0).abs() < 1e-9,
            "weights sum to {sum}, expected 1.0"
        );
    }
}