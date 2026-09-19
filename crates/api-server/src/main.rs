//! Unified Axum API server.
//!
//! Exposes the planned routes of all five modules (MASTERPLAN §8.2). Route
//! handlers compose the module `pub fn` interfaces at the application layer.
//! No module crate depends on another module crate.
//!
//! > **Status: interface scaffold.** Handlers call module `pub fn` stubs and
//! > will panic at runtime until the modules are implemented.

#![warn(missing_docs)]

mod routes;

use axum::Router;
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;

/// Build the unified application router with all module routes.
///
/// # Returns
///
/// The composed [`Router`] for the whole API.
///
/// # Example
///
/// ```rust,ignore
/// pub fn build_routes() -> Router {
///     build_router()
/// }
/// ```
pub fn build_router() -> Router {
    Router::new()
        // Module 1 — Intelligent Species Search
        .route("/api/v1/search", axum::routing::get(routes::search::search_species))
        .route(
            "/api/v1/search/recommendations",
            axum::routing::get(routes::search::recommend_queries),
        )
        // Module 2 — Species Relationship Explorer
        .route(
            "/api/v1/species/{id}/relationships",
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
            axum::routing::get(routes::comparison::compare_species),
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
        .layer(CorsLayer::permissive())
}

/// Application entry point.
#[tokio::main]
pub async fn main() {
    let app = build_router();
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind address");
    println!("Axum API server listening on {addr}");
    axum::serve(listener, app).await.expect("server error");
}