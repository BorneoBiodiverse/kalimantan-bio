//! Module 2 routes — Species Relationship Explorer.

use axum::extract::{Path, Query};
use axum::response::Json;
use kalimantanbio_shared::core::Species;
use serde::Deserialize;
use serde_json::{json, Value};
use species_relationships::types::{RelationshipQuery, ScoreWeights};

/// Query parameters for the relationships endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct RelationshipParams {
    /// Minimum relationship score.
    pub min_score: Option<f64>,
    /// Maximum number of related species.
    pub limit: Option<usize>,
}

/// `GET /api/v1/species/{id}/relationships`
///
/// Composes `species_relationships::explore_relationships`.
///
/// # Arguments
///
/// * `id` - Center species identifier.
/// * `params` - Relationship query parameters.
///
/// # Returns
///
/// A JSON response with related species and network data.
///
/// # Example
///
/// ```rust,ignore
/// let response = get_relationships(Path(1), Query(RelationshipParams::default())).await;
/// ```
pub async fn get_relationships(
    Path(id): Path<u64>,
    Query(_params): Query<RelationshipParams>,
) -> Json<Value> {
    let input: Vec<Species> = Vec::new();
    let query = RelationshipQuery {
        species_id: id,
        min_score: _params.min_score.unwrap_or(0.0),
        limit: _params.limit.unwrap_or(50),
        required_basis: None,
    };
    let weights = ScoreWeights {
        taxonomy: 0.5,
        habitat: 0.3,
        characteristic: 0.2,
    };
    let results = species_relationships::explore_relationships(&input, &query, &weights);
    Json(json!({
        "status": "success",
        "data": { "results": results },
    }))
}