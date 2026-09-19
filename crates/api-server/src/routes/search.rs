//! Module 1 routes — Intelligent Species Search.

use axum::extract::Query;
use axum::response::Json;
use kalimantanbio_shared::core::Species;
use serde::Deserialize;
use serde_json::{json, Value};

/// Query parameters for the species search endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct SearchParams {
    /// The search query text.
    pub q: String,
    /// Optional maximum number of results.
    pub limit: Option<usize>,
}

/// `GET /api/v1/search`
///
/// Composes `species_search::search`.
///
/// # Arguments
///
/// * `params` - Search query parameters.
///
/// # Returns
///
/// A JSON search response.
///
/// # Example
///
/// ```rust,ignore
/// let response = search_species(Query(SearchParams { q: "orangutan".into(), limit: None })).await;
/// ```
pub async fn search_species(Query(params): Query<SearchParams>) -> Json<Value> {
    let all_species: Vec<Species> = Vec::new();
    let results = species_search::search(&params.q, &all_species);
    Json(json!({
        "status": "success",
        "data": { "results": results, "limit": params.limit },
    }))
}

/// `GET /api/v1/search/recommendations`
///
/// Composes `species_search::recommend_related_queries`.
///
/// # Arguments
///
/// * `params` - Search query parameters.
///
/// # Returns
///
/// A JSON response with related-query recommendations.
pub async fn recommend_queries(Query(params): Query<SearchParams>) -> Json<Value> {
    let all_species: Vec<Species> = Vec::new();
    let suggestions = species_search::recommend_related_queries(&params.q, &all_species);
    Json(json!({
        "status": "success",
        "data": { "recommendations": suggestions },
    }))
}