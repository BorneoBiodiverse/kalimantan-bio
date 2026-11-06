//! Unified Axum API server.
//!
//! Exposes the planned routes of all five modules (MASTERPLAN A8.2). Route
//! handlers compose the module `pub fn` interfaces at the application layer.
//! No module crate depends on another module crate.
//!
//! > **Status: partial implementation.** The Module 2 relationships handler
//! > loads a species snapshot (database via `DATABASE_URL`, or the shared
//! > fixtures fallback) and composes `explore_relationships`; the other
//! > handlers remain `todo!()` stubs.

#![warn(missing_docs)]

mod routes;

use std::sync::Arc;

use axum::http::{HeaderValue, Method};
use axum::Router;
use kalimantanbio_shared::core::Species;
use kalimantanbio_shared::db::{create_pool, fetch_all_species};
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;

/// Shared API state: the species snapshot used by route handlers.
///
/// `Err(reason)` means the data source is unavailable; handlers answer
/// `503 service_unavailable` with the reason logged (planning Modul 2 §8).
#[derive(Clone)]
pub struct AppState {
    /// `Ok(snapshot)` when the source is ready, `Err(reason)` when it is not.
    pub species: Arc<Result<Vec<Species>, String>>,
}

/// Load the species snapshot for the API.
///
/// Reads `DATABASE_URL` first (production source through the shared
/// `create_pool` / `fetch_all_species` adapter); otherwise falls back to the
/// shared fixture file (`SPECIES_FIXTURE`, default
/// `crates/shared/fixtures/species.json`).
///
/// # Returns
///
/// `Ok(snapshot)` or `Err(reason)` describing why the source is unavailable.
async fn load_species() -> Result<Vec<Species>, String> {
    // ponytail: no timeout/retry config yet; also `create_pool` /
    // `fetch_all_species` in shared are still `todo!()` — the DATABASE_URL
    // path panics until shared lands the db fns. Fixture path (no env var)
    // is the working default. Upgrade: pool options + deployment config crate.
    if let Ok(url) = std::env::var("DATABASE_URL") {
        let pool = create_pool(&url)
            .await
            .map_err(|e| format!("database unreachable: {e}"))?;
        return fetch_all_species(&pool)
            .await
            .map_err(|e| format!("species query failed: {e}"));
    }

    // ponytail: fixture path is relative to the repo root when
    // SPECIES_FIXTURE is unset; override the env var from other cwds.
    let path = std::env::var("SPECIES_FIXTURE")
        .unwrap_or_else(|_| "crates/shared/fixtures/species.json".to_string());
    let raw =
        std::fs::read_to_string(&path).map_err(|e| format!("fixture unreadable ({path}): {e}"))?;
    serde_json::from_str(&raw).map_err(|e| format!("fixture invalid ({path}): {e}"))
}

/// Build the CORS layer allowed for the Django client origin.
///
/// The origin comes from `CORS_ORIGIN` and defaults to
/// `http://localhost:8000` (planning Modul 2 §8, MASTERPLAN §9). An
/// unparsable value falls back to permissive CORS.
fn build_cors() -> CorsLayer {
    let origin = std::env::var("CORS_ORIGIN").unwrap_or_else(|_| "http://localhost:8000".into());
    match origin.parse::<HeaderValue>() {
        Ok(value) => CorsLayer::new()
            .allow_origin(value)
            .allow_methods([Method::GET, Method::POST]),
        Err(_) => CorsLayer::permissive(),
    }
}

/// Build the unified application router with all module routes.
///
/// # Arguments
///
/// * `state` - Shared [`AppState`] holding the species snapshot.
///
/// # Returns
///
/// The composed [`Router`] for the whole API.
///
/// # Example
///
/// ```rust,ignore
/// let state = AppState { species: Arc::new(Ok(species)) };
/// let app = build_router(state);
/// ```
pub fn build_router(state: AppState) -> Router {
    Router::new()
        // Module 1 — Intelligent Species Search
        .route(
            "/api/v1/search",
            axum::routing::get(routes::search::search_species),
        )
        .route(
            "/api/v1/search/recommendations",
            axum::routing::get(routes::search::recommend_queries),
        )
        // Module 2 — Species Relationship Explorer
        .route(
            // axum 0.7 memakai sintaks `:param`, bukan `{param}` (axum 0.8)
            "/api/v1/species/:id/relationships",
            axum::routing::get(routes::relationship::get_relationships),
        )
        // Module 3 — Taxonomy & Classification Explorer
        .route(
            "/api/v1/taxonomy/tree",
            axum::routing::get(routes::taxonomy::get_tree),
        )
        .route(
            "/api/v1/taxonomy/diversity",
            axum::routing::get(routes::taxonomy::get_diversity),
        )
        .route(
            "/api/v1/taxonomy/gaps",
            axum::routing::get(routes::taxonomy::get_gaps),
        )
        .route(
            "/api/v1/taxonomy/endemic",
            axum::routing::get(routes::taxonomy::get_endemic),
        )
        // Module 4 — Comparative Species Explorer
        .route(
            "/api/v1/compare",
            axum::routing::post(routes::comparison::compare_species),
        )
        // Module 5 — Biodiversity Knowledge & Citation Explorer
        .route(
            "/api/v1/publications",
            axum::routing::get(routes::knowledge::get_publications),
        )
        .route(
            "/api/v1/citations",
            axum::routing::get(routes::knowledge::export_citations),
        )
        .layer(build_cors())
        .with_state(state)
}

/// Application entry point.
#[tokio::main]
pub async fn main() {
    let species = match load_species().await {
        Ok(species) => {
            println!("species snapshot loaded: {} records", species.len());
            species
        }
        Err(reason) => {
            eprintln!("[api-server] data source unavailable: {reason}");
            eprintln!(
                "[api-server] relationship requests will answer 503 until the source recovers"
            );
            Vec::new()
        }
    };
    let state = AppState {
        species: Arc::new(Ok(species)),
    };

    let app = build_router(state);
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind address");
    println!("Axum API server listening on {addr}");
    axum::serve(listener, app).await.expect("server error");
}
