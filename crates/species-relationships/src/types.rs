//! Module-specific types for the Species Relationship Explorer.

use kalimantanbio_shared::core::Species;
use serde::Serialize;

/// Identifier of a species in the network.
pub type SpeciesId = u64;

/// Evidence basis for a relationship.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum AttributeKind {
    /// Shared taxonomy.
    Taxonomy,
    /// Shared habitat.
    Habitat,
    /// Shared characteristic.
    Characteristic,
}

/// Weighted contribution of each evidence basis.
#[derive(Debug, Clone, Serialize)]
pub struct ScoreWeights {
    /// Taxonomy weight.
    pub taxonomy: f64,
    /// Habitat weight.
    pub habitat: f64,
    /// Characteristic weight.
    pub characteristic: f64,
}

/// Query for relationship exploration.
#[derive(Debug, Clone, Serialize)]
pub struct RelationshipQuery {
    /// Center species identifier.
    pub species_id: SpeciesId,
    /// Minimum acceptable relationship score.
    pub min_score: f64,
    /// Maximum number of related species to return.
    pub limit: usize,
    /// Optional constraint on the evidence basis.
    pub required_basis: Option<AttributeKind>,
}

/// Evidence collected for one relationship dimension.
#[derive(Debug, Clone, Serialize)]
pub struct AttributeEvidence {
    /// Evidence basis.
    pub attribute: AttributeKind,
    /// Similarity for this dimension, when computed.
    pub similarity: Option<f64>,
    /// Shared values for this dimension.
    pub shared_values: Vec<String>,
}

/// Relationship score for a pair of species.
#[derive(Debug, Clone, Serialize)]
pub struct RelationshipScore {
    /// Combined score.
    pub total: f64,
    /// Coverage of the evidence available.
    pub coverage: f64,
    /// Per-basis evidence.
    pub evidence: Vec<AttributeEvidence>,
}

/// A related species with its score and explanations.
#[derive(Debug, Clone, Serialize)]
pub struct RelatedSpecies {
    /// The related species.
    pub species: Species,
    /// Relationship score.
    pub score: RelationshipScore,
    /// Human-readable explanations.
    pub explanations: Vec<String>,
}

/// Network node.
#[derive(Debug, Clone, Serialize)]
pub struct NetworkNode {
    /// Species identifier.
    pub id: SpeciesId,
    /// Display label.
    pub label: String,
    /// Whether this node is the center species.
    pub is_center: bool,
}

/// Network edge.
#[derive(Debug, Clone, Serialize)]
pub struct NetworkEdge {
    /// Source node.
    pub source: SpeciesId,
    /// Target node.
    pub target: SpeciesId,
    /// Relationship score of the edge.
    pub score: RelationshipScore,
    /// Human-readable explanations.
    pub explanations: Vec<String>,
}

/// The species relationship network.
#[derive(Debug, Clone, Serialize)]
pub struct SpeciesNetwork {
    /// Network nodes.
    pub nodes: Vec<NetworkNode>,
    /// Network edges.
    pub edges: Vec<NetworkEdge>,
}

/// Result of a relationship exploration.
#[derive(Debug, Clone, Serialize)]
pub struct ExplorerResult {
    /// Identifier of the center species.
    pub center_id: SpeciesId,
    /// Top related species.
    pub related: Vec<RelatedSpecies>,
    /// Network representation.
    pub network: SpeciesNetwork,
    /// Weights used for scoring.
    pub weights: ScoreWeights,
    /// Scoring formula version.
    pub scoring_version: String,
}

/// Errors reported by the exploration pipeline.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum ExplorerError {
    /// The query is invalid.
    InvalidQuery(String),
    /// The weights are invalid.
    InvalidWeights,
    /// The dataset is invalid.
    InvalidDataset(String),
    /// The center species was not found.
    SpeciesNotFound(SpeciesId),
}