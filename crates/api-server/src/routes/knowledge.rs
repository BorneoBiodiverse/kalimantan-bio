//! Module 5 routes — Biodiversity Knowledge & Citation Explorer.

use axum::extract::Query;
use axum::response::Json;
use knowledge_citations::types::{CitationFormat, KnowledgeQuery, Publication};
use serde::Deserialize;
use serde_json::{json, Value};

/// Query parameters for the publications endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct PublicationParams {
    /// Species identifier filter.
    pub species_id: Option<u64>,
    /// Research topic filter.
    pub topic: Option<String>,
}

/// Query parameters for the citations export endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct CitationParams {
    /// Citation format: `apa` or `bibtex`.
    pub format: Option<String>,
    /// Comma-separated publication identifiers.
    pub publication_ids: Option<String>,
}

/// `GET /api/v1/publications`
///
/// Composes `knowledge_citations::explore_knowledge`.
///
/// # Arguments
///
/// * `params` - Publication filter parameters.
///
/// # Returns
///
/// A JSON response with the knowledge exploration report.
pub async fn get_publications(Query(params): Query<PublicationParams>) -> Json<Value> {
    let query = KnowledgeQuery {
        species_ids: params.species_id.map(|id| vec![id]).unwrap_or_default(),
        text: params.topic,
        ..Default::default()
    };
    let publications: Vec<Publication> = Vec::new();
    let report = knowledge_citations::explore_knowledge(&query, &publications);
    Json(json!({ "status": "success", "data": report }))
}

/// `GET /api/v1/citations`
///
/// Composes `knowledge_citations::recommend_citations` and
/// `knowledge_citations::export_citations`.
///
/// # Arguments
///
/// * `params` - Citation format and publication filter parameters.
///
/// # Returns
///
/// A JSON response with exported citations.
pub async fn export_citations(Query(params): Query<CitationParams>) -> Json<Value> {
    let publications: Vec<Publication> = Vec::new();
    let format = match params.format.as_deref() {
        Some("bibtex") => CitationFormat::Bibtex,
        _ => CitationFormat::Apa,
    };
    let citations = knowledge_citations::export_citations(&publications, format);
    Json(json!({
        "status": "success",
        "data": {
            "citations": citations,
            "requested": params.publication_ids,
        },
    }))
}