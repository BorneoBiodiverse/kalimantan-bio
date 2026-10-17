//! Module 1 domain types and configuration.
//!
//! Provides the structured representation of search filters ([`QueryFilters`])
//! and relevance scoring weights ([`ScoringWeights`]).

/// Parsed criteria for filtering and scoring species records.
///
/// Created by the query parser (Stage 1) and consumed by both
/// the multi-attribute filter (Stage 2) and the relevance scorer (Stage 3).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct QueryFilters {
    /// Free-text keywords matched against scientific names, common names, and descriptions.
    pub free_text: Vec<String>,
    /// Optional filter for taxonomic kingdom ID.
    pub kingdom_id: Option<u64>,
    /// Optional filter for taxonomic class ID.
    pub class_id: Option<u64>,
    /// Optional IUCN Red List conservation status code (e.g. "CR", "EN", "VU").
    pub iucn: Option<String>,
    /// Optional CITES appendix designation (e.g. "Appendix I", "Appendix II").
    pub cites: Option<String>,
    /// Optional Indonesian national protection status (P.106, e.g. "Dilindungi").
    pub p106: Option<String>,
    /// Optional species origin category (e.g. "Endemic", "Native", "Introduced").
    pub species_type: Option<String>,
    /// Optional filter for verified species records only.
    pub is_verified: Option<bool>,
}

/// Relative weights applied when calculating relevance scores.
///
/// Weights are normalized so that their sum equals 1.0, matching the contract
/// required by [`kalimantanbio_shared::scoring::combine_weighted_scores`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScoringWeights {
    /// Relative weight assigned to free-text keyword match score (0.0 to 1.0).
    pub text_weight: f64,
    /// Relative weight assigned to taxonomic similarity score (0.0 to 1.0).
    pub taxonomy_weight: f64,
    /// Relative weight assigned to conservation status match score (0.0 to 1.0).
    pub conservation_weight: f64,
}

impl Default for ScoringWeights {
    /// Creates a default weight configuration (50% text, 30% taxonomy, 20% conservation).
    fn default() -> Self {
        Self {
            text_weight: 0.50,
            taxonomy_weight: 0.30,
            conservation_weight: 0.20,
        }
    }
}
