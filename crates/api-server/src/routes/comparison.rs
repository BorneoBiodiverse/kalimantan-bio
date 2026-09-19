//! Module 4 routes — Comparative Species Explorer.

use axum::extract::Query;
use axum::response::Json;
use kalimantanbio_shared::core::{Observation, Species};
use serde::Deserialize;
use serde_json::{json, Value};
use species_comparison::types::ComparisonQuery;

/// Query parameters for the comparison endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct CompareParams {
    /// Comma-separated species identifiers.
    pub species_ids: String,
}

/// `GET /api/v1/compare`
///
/// Composes `species_comparison::compare_species` from a comma-separated id list.
///
/// # Arguments
///
/// * `params` - Comparison query parameters.
///
/// # Returns
///
/// A JSON response with the comparison report.
///
/// # Example
///
/// ```rust,ignore
/// let response = compare_species(Query(CompareParams { species_ids: "1,2,3".into() })).await;
/// ```
pub async fn compare_species(Query(params): Query<CompareParams>) -> Json<Value> {
    let ids: Vec<u64> = params
        .species_ids
        .split(',')
        .filter_map(|part| part.trim().parse().ok())
        .collect();
    let query = ComparisonQuery { species_ids: ids };
    let species: Vec<Species> = Vec::new();
    let observations: Vec<Observation> = Vec::new();
    let report = species_comparison::compare_species(&query, &species, &observations);
    Json(json!({ "status": "success", "data": report }))
}