//! Module 4 routes — Comparative Species Explorer.
//!
//! Accepts a versioned JSON payload from Django containing the selected
/// species records (with their observations) and returns the comparison
/// report computed by `species_comparison::compare_species`.
use axum::http::StatusCode;
use axum::response::Json as AxumJson;
use kalimantanbio_shared::core::{Observation, Species};
use serde::Deserialize;
use serde_json::{json, Value};
use species_comparison::types::ComparisonQuery;

/// A species record together with its observation records, as posted by Django.
///
/// Mirrors the versioned Django→Rust contract (X-API-Version: 1):
///
/// ```json
/// {
///   "id": 1,
///   "scientific_name": "Pongo pygmaeus",
///   "common_name": "Orangutan Kalimantan",
///   "taxonomy": { ... },
///   "iucn": "CR",
///   "observations": [ ... ]
/// }
///
#[derive(Debug, Clone, Deserialize)]
pub struct SpeciesWithObservations {
    /// The species record itself (all `kalimantanbio_shared::core::Species` fields).
    #[serde(flatten)]
    pub species: Species,
    /// Observation records for this species, including regency (kabupaten) data.
    pub observations: Vec<Observation>,
}

/// Request body for the comparison endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct ComparePayload {
    /// Species records to compare (2 to 10, distinct IDs, in selection order).
    pub species: Vec<SpeciesWithObservations>,
}

/// `POST /api/v1/compare`
///
/// Composes `species_comparison::compare_species` from the posted species
/// records and their observations. Django owns data access and
/// authorization; Rust owns the comparison calculation.
///
/// # Errors
///
/// * `400` — the ID list is invalid (too few, too many, or duplicates).
/// * `404` — a requested species ID is not present in the dataset.
/// * `422` — a validation rule was violated.
///
pub async fn compare_species(
    AxumJson(payload): AxumJson<ComparePayload>,
) -> Result<AxumJson<Value>, (StatusCode, AxumJson<Value>)> {
    let species: Vec<Species> = payload
        .species
        .iter()
        .map(|swo| swo.species.clone())
        .collect();
    let observations: Vec<Observation> = payload
        .species
        .iter()
        .flat_map(|swo| swo.observations.clone())
        .collect();
    let ids: Vec<u64> = species.iter().map(|s| s.id).collect();
    let query = ComparisonQuery { species_ids: ids };

    match species_comparison::compare_species(&query, &species, &observations) {
        Ok(report) => Ok(AxumJson(json!({ "status": "success", "data": report }))),
        Err(species_comparison::types::ModuleError::InvalidQuery(msg)) => Err((
            StatusCode::BAD_REQUEST,
            AxumJson(json!({ "status": "error", "error": msg })),
        )),
        Err(species_comparison::types::ModuleError::SpeciesNotFound(id)) => Err((
            StatusCode::NOT_FOUND,
            AxumJson(json!({ "status": "error", "error": format!("species {id} not found") })),
        )),
        Err(species_comparison::types::ModuleError::Empty(msg))
        | Err(species_comparison::types::ModuleError::Validation(msg)) => Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            AxumJson(json!({ "status": "error", "error": msg })),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{build_router, AppState};
    use axum::body::{to_bytes, Body};
    use axum::http::{Request, StatusCode};
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

    async fn post(app: axum::Router, uri: &str, body: &str) -> (StatusCode, Value) {
        let response = app
            .oneshot(
                Request::post(uri)
                    .header("content-type", "application/json")
                    .body(Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        let status = response.status();
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body: Value = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
        (status, body)
    }

    fn species_payload(ids: &[u64]) -> String {
        let species: Vec<Value> = ids
            .iter()
            .map(|&id| {
                json!({
                    "id": id,
                    "scientific_name": format!("Species {id}"),
                    "common_name": format!("Common {id}"),
                    "description": "",
                    "species_type": "Native",
                    "iucn": "LC",
                    "cites": "Not Listed",
                    "p106": "Bebas",
                    "is_verified": false,
                    "image_url": null,
                    "taxonomy": {
                        "kingdom": "Animalia", "kingdom_id": 1,
                        "phylum_division": "Chordata", "phylum_division_id": 2,
                        "class": "Mammalia", "class_id": 3,
                        "order": "Primates", "order_id": 4,
                        "family": "Hominidae", "family_id": 5,
                        "genus": "Pongo", "genus_id": 6
                    },
                    "observation_count": 0,
                    "recorded_individuals_total": 0,
                    "latest_observation_year": null,
                    "created_at": "2026-01-01T00:00:00Z",
                    "updated_at": "2026-01-01T00:00:00Z",
                    "observations": []
                })
            })
            .collect();
        json!({ "species": species }).to_string()
    }

    #[tokio::test]
    async fn valid_comparison_returns_200_with_report() {
        let (status, body) = post(
            build_router(fixture_state()),
            "/api/v1/compare",
            &species_payload(&[1, 3]),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["status"], "success");
        assert_eq!(body["data"]["query"]["species_ids"], json!([1, 3]));
        assert_eq!(
            body["data"]["attribute_matrix"].as_array().unwrap().len(),
            2
        );
        assert!(body["data"]["similarity_scores"].as_array().unwrap().len() >= 1);
    }

    #[tokio::test]
    async fn too_few_species_returns_400() {
        let (status, body) = post(
            build_router(fixture_state()),
            "/api/v1/compare",
            &species_payload(&[1]),
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(body["status"], "error");
    }

    #[tokio::test]
    async fn too_many_species_returns_400() {
        let ids: Vec<u64> = (1..=11).collect();
        let (status, _) = post(
            build_router(fixture_state()),
            "/api/v1/compare",
            &species_payload(&ids),
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn duplicate_ids_return_400() {
        let (status, _) = post(
            build_router(fixture_state()),
            "/api/v1/compare",
            &species_payload(&[1, 1]),
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn species_without_observations_still_returns_200() {
        // Species 5 (Rafflesia arnoldii) has zero observations in the fixture.
        let (status, body) = post(
            build_router(fixture_state()),
            "/api/v1/compare",
            &species_payload(&[1, 5]),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["status"], "success");
    }
}
