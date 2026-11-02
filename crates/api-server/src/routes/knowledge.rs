//! Module 5 routes — Biodiversity Knowledge & Citation Explorer.

use axum::extract::Query;
use axum::http::StatusCode;
use axum::response::Json;
use knowledge_citations::types::{
    CitationFormat, KnowledgeQuery, ModuleError, Publication, ResearchTopic,
};
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

/// Map knowledge-module errors to the HTTP response used by these routes.
fn map_error(error: ModuleError) -> (StatusCode, Json<Value>) {
    let (status, message) = match error {
        ModuleError::Parse(message) => (StatusCode::UNPROCESSABLE_ENTITY, message),
        ModuleError::Validation(message) => (StatusCode::BAD_REQUEST, message),
        ModuleError::Empty(message) => (StatusCode::SERVICE_UNAVAILABLE, message),
        ModuleError::NotFound(message) => (StatusCode::NOT_FOUND, message),
    };
    (status, Json(json!({ "status": "error", "error": message })))
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
pub async fn get_publications(
    Query(params): Query<PublicationParams>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let query = KnowledgeQuery {
        species_ids: params.species_id.map(|id| vec![id]).unwrap_or_default(),
        topic: params
            .topic
            .map(|topic| match topic.to_lowercase().as_str() {
                "taxonomy" => ResearchTopic::Taxonomy,
                "ecology" => ResearchTopic::Ecology,
                "conservation" => ResearchTopic::Conservation,
                "ethnobotany" => ResearchTopic::Ethnobotany,
                "habitat" => ResearchTopic::Habitat,
                "speciesidentification" | "species identification" => {
                    ResearchTopic::SpeciesIdentification
                }
                _ => ResearchTopic::Other(topic),
            }),
        ..Default::default()
    };
    let raw = include_str!("../../../knowledge-citations/fixtures/publications.json");
    let publications: Vec<Publication> =
        knowledge_citations::parse_publications(raw).map_err(map_error)?;
    match knowledge_citations::explore_knowledge(&query, &publications) {
        Ok(report) => Ok(Json(json!({ "status": "success", "data": report }))),
        Err(error) => Err(map_error(error)),
    }
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
/// A JSON response with exported citations, or a mapped error response if the
/// publication fixture cannot be parsed.
pub async fn export_citations(
    Query(params): Query<CitationParams>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let raw = include_str!("../../../knowledge-citations/fixtures/publications.json");
    let all_publications = knowledge_citations::parse_publications(raw).map_err(map_error)?;
    let publications: Vec<Publication> = if let Some(ref ids_str) = params.publication_ids {
        let requested_ids: std::collections::HashSet<&str> =
            ids_str.split(',').map(str::trim).collect();
        all_publications
            .into_iter()
            .filter(|p| requested_ids.contains(p.id.as_str()))
            .collect()
    } else {
        all_publications
    };
    let format = match params.format.as_deref() {
        Some("bibtex") => CitationFormat::Bibtex,
        _ => CitationFormat::Apa,
    };
    let citations = knowledge_citations::export_citations(&publications, format);
    Ok(Json(json!({
        "status": "success",
        "data": {
            "citations": citations,
            "requested": params.publication_ids,
        },
    })))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_error_maps_to_unprocessable_entity() {
        let (status, body) = map_error(ModuleError::Parse(String::from("invalid JSON")));

        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(body.0["status"], "error");
        assert_eq!(body.0["error"], "invalid JSON");
    }
}
