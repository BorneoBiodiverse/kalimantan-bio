//! Module 2 routes — Species Relationship Explorer.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};
use species_relationships::types::{AttributeKind, ExplorerError, RelationshipQuery, ScoreWeights};

use crate::AppState;

/// Query parameters for the relationships endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct RelationshipParams {
    /// Minimum relationship score (`0.0`–`1.0`, default `0.0`).
    pub min_score: Option<f64>,
    /// Maximum number of related species (default `50`).
    pub limit: Option<usize>,
    /// Optional required evidence basis: `taxonomy`, `habitat`, or
    /// `characteristic`; any other value fails with `400 invalid_basis`.
    pub required_basis: Option<String>,
}

/// Build a structured JSON error body (`status`, `code`, `message`).
fn error(status: StatusCode, code: &str, message: impl Into<String>) -> (StatusCode, Json<Value>) {
    (
        status,
        Json(json!({
            "status": "error",
            "code": code,
            "message": message.into(),
        })),
    )
}

/// Map pipeline errors to HTTP status and error codes (planning Modul 2 §8).
fn map_error(err: ExplorerError) -> (StatusCode, Json<Value>) {
    match err {
        ExplorerError::InvalidQuery(message) => {
            error(StatusCode::BAD_REQUEST, "invalid_query", message)
        }
        ExplorerError::SpeciesNotFound(id) => error(
            StatusCode::NOT_FOUND,
            "species_not_found",
            format!("species {id} not found"),
        ),
        // 500: generic message to the caller, diagnosis stays in the log.
        ExplorerError::InvalidDataset(diagnostic) => {
            eprintln!("[api-server] dataset error: {diagnostic}");
            error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "invalid_dataset",
                "species data is invalid",
            )
        }
        ExplorerError::InvalidWeights => {
            eprintln!("[api-server] scoring configuration error: weights are invalid");
            error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "invalid_weights",
                "scoring configuration is invalid",
            )
        }
    }
}

/// Parse the optional `required_basis` parameter into [`AttributeKind`].
fn parse_basis(raw: Option<&str>) -> Result<Option<AttributeKind>, (StatusCode, Json<Value>)> {
    match raw {
        None => Ok(None),
        Some("taxonomy") => Ok(Some(AttributeKind::Taxonomy)),
        Some("habitat") => Ok(Some(AttributeKind::Habitat)),
        Some("characteristic") => Ok(Some(AttributeKind::Characteristic)),
        Some(other) => Err(error(
            StatusCode::BAD_REQUEST,
            "invalid_basis",
            format!("required_basis must be taxonomy, habitat, or characteristic (got {other})"),
        )),
    }
}

/// `GET /api/v1/species/{id}/relationships`
///
/// Composes `species_relationships::explore_relationships` over the snapshot
/// in [`AppState`].
///
/// # Arguments
///
/// * `state` - Shared species snapshot; `Err` yields `503 service_unavailable`.
/// * `id` - Center species identifier.
/// * `params` - `min_score`, `limit`, and optional `required_basis`.
///
/// # Returns
///
/// `200` with `{"status":"success","data":<ExplorerResult>}` (an empty result
/// still contains the center node), or an error body with `code` and
/// `message` mapped from planning Modul 2 §8: `400 invalid_query` /
/// `invalid_basis`, `404 species_not_found`, `500 invalid_dataset` /
/// `invalid_weights`, `503 data_unavailable`.
///
/// # Example
///
/// ```rust,ignore
/// let response = get_relationships(Path(1), Query(params)).await;
/// ```
pub async fn get_relationships(
    State(state): State<AppState>,
    Path(id): Path<u64>,
    Query(params): Query<RelationshipParams>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let required_basis = parse_basis(params.required_basis.as_deref())?;

    let species = state.species.as_ref().as_ref().map_err(|reason| {
        eprintln!("[api-server] data source unavailable: {reason}");
        error(
            StatusCode::SERVICE_UNAVAILABLE,
            "data_unavailable",
            "species data source is temporarily unavailable",
        )
    })?;

    let query = RelationshipQuery {
        species_id: id,
        min_score: params.min_score.unwrap_or(0.0),
        limit: params.limit.unwrap_or(50),
        required_basis,
    };
    let weights = ScoreWeights::default();

    match species_relationships::explore_relationships(species, &query, &weights) {
        Ok(result) => Ok(Json(json!({ "status": "success", "data": result }))),
        Err(err) => Err(map_error(err)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build_router;
    use axum::body::{to_bytes, Body};
    use axum::http::{Request, StatusCode};
    use kalimantanbio_shared::core::Species;
    use std::sync::Arc;
    use tower::ServiceExt;

    /// Snapshot dari fixture bersama (5 spesies, id 1–5).
    fn fixture_state() -> AppState {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../shared/fixtures/species.json"
        );
        let raw = std::fs::read_to_string(path).expect("fixture must exist");
        let species: Vec<Species> = serde_json::from_str(&raw).expect("fixture must parse");
        AppState {
            species: Arc::new(Ok(species)),
        }
    }

    fn unavailable_state() -> AppState {
        AppState {
            species: Arc::new(Err("database unreachable".to_string())),
        }
    }

    async fn get(app: axum::Router, uri: &str) -> (StatusCode, Value) {
        let response = app
            .oneshot(Request::get(uri).body(Body::empty()).unwrap())
            .await
            .unwrap();
        let status = response.status();
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body: Value = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
        (status, body)
    }

    #[tokio::test]
    async fn unknown_basis_is_rejected_with_400() {
        let (status, body) = get(
            build_router(fixture_state()),
            "/api/v1/species/1/relationships?required_basis=banana",
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(body["code"], "invalid_basis");
        assert_eq!(body["status"], "error");
    }

    #[tokio::test]
    async fn known_basis_is_accepted_parameter() {
        let (status, body) = get(
            build_router(fixture_state()),
            "/api/v1/species/999/relationships?required_basis=habitat",
        )
        .await;
        // basis valid → lanjut ke pipeline; id 999 tidak ada → 404
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(body["code"], "species_not_found");
    }

    #[tokio::test]
    async fn missing_center_returns_404() {
        let (status, body) = get(
            build_router(fixture_state()),
            "/api/v1/species/999/relationships?min_score=0.2&limit=10",
        )
        .await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(body["code"], "species_not_found");
        assert!(body["message"].as_str().unwrap().contains("999"));
    }

    #[tokio::test]
    async fn invalid_limit_returns_400() {
        let (status, body) = get(
            build_router(fixture_state()),
            "/api/v1/species/1/relationships?limit=99",
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(body["code"], "invalid_query");
    }

    #[tokio::test]
    async fn data_source_outage_returns_503() {
        let (status, body) = get(
            build_router(unavailable_state()),
            "/api/v1/species/1/relationships",
        )
        .await;
        assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(body["code"], "data_unavailable");
    }

    #[tokio::test]
    async fn valid_request_returns_200_with_network() {
        let (status, body) = get(
            build_router(fixture_state()),
            "/api/v1/species/1/relationships?min_score=0.0&limit=10",
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["status"], "success");

        let data = &body["data"];
        assert_eq!(data["center_id"], 1);
        assert_eq!(data["scoring_version"], "relationship-v1");

        let related = data["related"].as_array().expect("related must be a list");
        let nodes = data["network"]["nodes"]
            .as_array()
            .expect("nodes must be a list");
        let edges = data["network"]["edges"]
            .as_array()
            .expect("edges must be a list");
        assert_eq!(nodes.len(), related.len() + 1);
        assert_eq!(edges.len(), related.len());
        assert!(nodes
            .iter()
            .any(|node| node["id"] == 1 && node["is_center"] == true));
    }

    #[tokio::test]
    async fn second_center_request_reuses_network_contract() {
        // permintaan ulang dengan pusat baru (eksplorasi tetangga, §7)
        let (status, body) = get(
            build_router(fixture_state()),
            "/api/v1/species/3/relationships?min_score=0.2&limit=10&required_basis=taxonomy",
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["status"], "success");
        assert_eq!(body["data"]["center_id"], 3);
        assert!(body["data"]["network"]["nodes"]
            .as_array()
            .unwrap()
            .iter()
            .any(|node| node["id"] == 3 && node["is_center"] == true));
    }
}
