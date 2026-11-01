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

/// Helper to lazily load species fixtures from the shared library.
fn load_species_fixtures() -> &'static [Species] {
    static FIXTURES: std::sync::OnceLock<Vec<Species>> = std::sync::OnceLock::new();
    FIXTURES.get_or_init(|| {
        let data = include_str!("../../../shared/fixtures/species.json");
        serde_json::from_str(data).expect("Valid species fixtures JSON")
    })
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
    let all_species = load_species_fixtures();
    let results = species_search::search(&params.q, all_species);
    let results: Vec<_> = match params.limit {
        Some(limit) => results.into_iter().take(limit).collect(),
        None => results,
    };
    Json(json!({
        "status": "success",
        "data": {
            "query": params.q,
            "count": results.len(),
            "results": results.into_iter().map(|(sp, score)| {
                json!({
                    "id": sp.id,
                    "scientific_name": sp.scientific_name,
                    "common_name": sp.common_name,
                    "species_type": sp.species_type,
                    "iucn": sp.iucn,
                    "description": sp.description,
                    "taxonomy": sp.taxonomy,
                    "score": score
                })
            }).collect::<Vec<_>>(),
            "limit": params.limit
        },
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
    let all_species = load_species_fixtures();
    let suggestions = species_search::recommend_related_queries(&params.q, all_species);
    Json(json!({
        "status": "success",
        "data": {
            "query": params.q,
            "recommendations": suggestions.into_iter().map(|(q, score)| {
                json!({
                    "query": q,
                    "score": score
                })
            }).collect::<Vec<_>>()
        },
    }))
}
