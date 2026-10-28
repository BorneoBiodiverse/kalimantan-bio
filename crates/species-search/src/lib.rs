#![warn(missing_docs)]
//! Module 1 — Intelligent Species Search.
//!
//! Provides intelligent ways to discover species using natural-language queries
//! and structured biodiversity attributes.
//!
//! Public API: `search` and `recommend_related_queries`. Internal helpers
//! (`extract_filters`, `matches_*`, `filter_species`, `score_*`, ...) stay
//! private.
//!
//! > **Status: interface scaffold.** Bodies are stubs pending implementation.

pub mod types;
pub(crate) mod parser;
pub(crate) mod filter;
pub(crate) mod scoring;
pub(crate) mod recommendation;

use kalimantanbio_shared::{
    core::Species,
    scoring::rank_by_score,
    text::{normalize_text, tokenize},
};

use crate::types::ScoringWeights;

/// Search species by a natural-language or structured query.
///
/// Pipeline: parse → filter → score → rank.
///
/// # Arguments
///
/// * `raw_query` - User query, e.g. `"orangutan kalimantan"` or `"iucn:CR"`.
/// * `all_species` - The full species dataset.
///
/// # Returns
///
/// Ranked matches as `(species, relevance_score)` pairs, highest first.
///
/// # Example
///
/// ```rust,ignore
/// let results = search("orangutan kalimantan", &all_species);
/// for (species, score) in results {
///     println!("{species} ({score})");
/// }
/// ```
pub fn search<'a>(
    raw_query: &str,
    all_species: &'a [Species],
) -> Vec<(&'a Species, f64)> {
    // 1. Normalisasi dan tokenisasi teks pencarian
    let normalized = normalize_text(raw_query);
    let tokens = tokenize(&normalized);

    //2. Ekstrak kriteria filter (Tahap 1)
    let filters = crate::parser::extract_filters(&tokens);

    //3. Saring species yang memenuhi kriteria (Tahap 2)
    let filtered = filter::filter_species(all_species, &filters);

    //4. Hitung skor relevansi untuk setiap species yang lolos filter (Tahap 3)
    let weights = ScoringWeights::default();
    let scored: Vec<(&'a Species, f64)> = filtered
        .into_iter()
        .map(|species| (species, scoring::score_species(species, &filters, &weights)))
        .collect();

    //5. Urutkan hasil berdasarkan skor relevansi (Tahap 4)
    rank_by_score(scored)
}

/// Recommend related follow-up queries.
///
/// Composes the private recommendation pipeline of the module.
///
/// # Arguments
///
/// * `raw_query` - The original user query.
/// * `all_species` - The full species dataset.
///
/// # Returns
///
/// Ranked query suggestions as `(query, score)` pairs.
///
/// # Example
///
/// ```rust,ignore
/// let suggestions = recommend_related_queries("mamalia", &all_species);
/// ```
pub fn recommend_related_queries(
    raw_query: &str,
    all_species: &[Species],
) -> Vec<(String, f64)> {
    //1. Ekstrak filter dari query awal
    let normalized = normalize_text(raw_query);
    let tokens = tokenize(&normalized);
    let filters = parser::extract_filters(&tokens);

    //2. Dapatkan species yang cocok dengan filter awal
    let filtered = filter::filter_species(all_species, &filters);

    //3. Ambil atribut yang dominan dari hasil pencarian (Tahap 4)
    let common_attr = recommendation::extract_common_attributes(&filtered);

    //4. Buat candidate queries lanjutan
    let candidates = recommendation::generate_candidate_queries(&filters, &common_attr);

    //5. Urutkan saran query berdasarkan relevansi
    recommendation::rank_related_queries(candidates, &filters)
}
