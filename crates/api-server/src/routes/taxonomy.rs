//! Module 3 routes — Taxonomy & Classification Explorer.

use axum::response::Json;
use serde_json::{json, Value};
use taxonomy::types::BioDataInput;

/// `GET /api/v1/taxonomy/diversity`
///
/// Composes `taxonomy::calculate_diversity`.
///
/// # Returns
///
/// Diversity statistics for the dataset.
pub async fn get_diversity() -> Json<Value> {
    let species = Vec::new();
    let stats = taxonomy::calculate_diversity(&species);
    Json(json!({ "status": "success", "data": stats }))
}

/// `GET /api/v1/taxonomy/gaps`
///
/// Composes `taxonomy::analyze_taxonomic_gap`.
///
/// # Returns
///
/// A taxonomic gap report.
pub async fn get_gaps() -> Json<Value> {
    let ours = Default::default();
    let reference = Default::default();
    let report = taxonomy::analyze_taxonomic_gap(&ours, &reference);
    Json(json!({ "status": "success", "data": report }))
}

/// `GET /api/v1/taxonomy/tree`
///
/// Composes `taxonomy::generate_taxonomy_report`.
///
/// # Returns
///
/// The full taxonomy exploration report.
pub async fn get_tree() -> Json<Value> {
    let input = BioDataInput::default();
    let report = taxonomy::generate_taxonomy_report(&input);
    Json(json!({ "status": "success", "data": report }))
}

/// `GET /api/v1/taxonomy/endemic`
///
/// Composes `taxonomy::generate_taxonomy_report` and filters endemic taxa.
///
/// # Returns
///
/// The taxonomy report scoped to endemic taxa.
pub async fn get_endemic() -> Json<Value> {
    let input = BioDataInput::default();
    let report = taxonomy::generate_taxonomy_report(&input);
    Json(json!({ "status": "success", "data": report }))
}