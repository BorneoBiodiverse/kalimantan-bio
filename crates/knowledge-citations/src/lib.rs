#![warn(missing_docs)]
//! Module 5 — Biodiversity Knowledge & Citation Explorer.
//!
//! Connects biodiversity knowledge with scientific publications, research
//! topics, locations, and citations: publication exploration, research
//! timelines, coverage analysis, and citation recommendation/export.
//!
//! Public API: `explore_knowledge`, `recommend_citations`,
//! `export_citations`. Internal helpers stay private.
//!
//! > **Status: interface scaffold.** Bodies are stubs pending implementation.

pub mod types;

use crate::types::{CitationFormat, KnowledgeQuery, KnowledgeReport, ModuleError,
    Publication, PublicationQuery};

/// Explore biodiversity knowledge across publications.
///
/// Pipeline: parse → normalize → deduplicate → filter → rank → timeline →
/// coverage → understudied species → knowledge graph → assemble report.
///
/// # Arguments
///
/// * `query` - Exploration criteria (species, topic, location, year).
/// * `publications` - The publication dataset.
///
/// # Returns
///
/// Ranked publications, timeline, coverage, understudied species, and graph.
///
/// # Example
///
/// ```rust,ignore
/// let report = explore_knowledge(&query, &publications)?;
/// ```
pub fn explore_knowledge(
    query: &KnowledgeQuery,
    publications: &[Publication],
) -> Result<KnowledgeReport, ModuleError> {
    todo!("knowledge-citations phase 5: explore_knowledge")
}

/// Recommend the most relevant publications for citation.
///
/// # Arguments
///
/// * `items` - The publication dataset.
/// * `query` - The recommendation criteria.
/// * `limit` - Maximum number of recommendations.
///
/// # Returns
///
/// Publications ordered by relevance, limited to `limit`.
///
/// # Example
///
/// ```rust,ignore
/// let recommended = recommend_citations(&publications, &query, 5);
/// ```
pub fn recommend_citations<'a>(
    items: &'a [Publication],
    query: &PublicationQuery,
    limit: usize,
) -> Vec<&'a Publication> {
    todo!("knowledge-citations phase 5: recommend_citations")
}

/// Export publications as citations in a chosen format.
///
/// # Arguments
///
/// * `items` - The publications to export.
/// * `format` - Target citation format (APA or BibTeX).
///
/// # Returns
///
/// A citation string for all provided publications.
///
/// # Example
///
/// ```rust,ignore
/// let bibtex = export_citations(&publications, CitationFormat::Bibtex);
/// ```
pub fn export_citations(
    items: &[Publication],
    format: CitationFormat,
) -> String {
    todo!("knowledge-citations phase 5: export_citations")
}