#![warn(missing_docs)]
//! Module 2 — Species Relationship Explorer.
//!
//! Discovers and explains relationships between species: related-species
//! discovery, relationship scoring, explanations, and network data generation.
//!
//! Public API: `explore_relationships` and `calculate_relationship`. Internal
//! helpers (evidence extraction, scoring, explanation, network building) stay
//! private within domain-specific modules.

pub mod error;
pub mod types;

pub(crate) mod discovery;
pub(crate) mod evidence;
pub(crate) mod network;
pub(crate) mod scoring;
pub(crate) mod validation;

#[cfg(test)]
pub(crate) mod test_fixtures;

use kalimantanbio_shared::core::Species;

pub use crate::error::ExplorerError;
pub use crate::scoring::calculate_relationship;
pub use crate::types::{ExplorerResult, RelationshipQuery, ScoreWeights, SpeciesId};

/// Explore all relationships around a center species.
///
/// Pipeline (planning §9): validate query and weights → prepare (normalize)
/// the dataset → find the center → discover candidates → score and explain →
/// rank and filter → build the network → assemble the result.
///
/// # Arguments
///
/// * `input` - The full species dataset (snapshot from the repository).
/// * `query` - Center species, minimum score, limit, and optional required
///   evidence basis.
/// * `weights` - Scoring weights; must be finite, non-negative, and sum to
///   `1.0` within tolerance `1e-9`.
///
/// # Returns
///
/// [`ExplorerResult`] with related species (scores and explanations), the
/// centered network, the weights used, and `scoring_version`. An empty result
/// still contains the center node.
///
/// # Errors
///
/// * [`ExplorerError::InvalidQuery`] - `species_id` is `0`, `min_score` is
///   outside `0.0..=1.0` or non-finite, or `limit` is outside `1..=50`.
/// * [`ExplorerError::InvalidWeights`] - weights fail validation.
/// * [`ExplorerError::InvalidDataset`] - duplicate IDs, ID `0`, or empty
///   scientific name in the dataset.
/// * [`ExplorerError::SpeciesNotFound`] - the center species is absent from
///   the dataset.
///
/// # Example
///
/// ```
/// use kalimantanbio_shared::core::Species;
/// use species_relationships::explore_relationships;
/// use species_relationships::types::{RelationshipQuery, ScoreWeights};
///
/// # fn species(id: u64, genus: &str, habitat: &str) -> Species {
/// #     serde_json::from_value(serde_json::json!({
/// #         "id": id,
/// #         "scientific_name": "Species synthetica",
/// #         "common_name": "Synthetic species",
/// #         "description": "",
/// #         "species_type": "native",
/// #         "iucn": "-",
/// #         "cites": "-",
/// #         "p106": "-",
/// #         "is_verified": true,
/// #         "image_url": null,
/// #         "taxonomy": {
/// #             "kingdom": "animalia",
/// #             "kingdom_id": 1,
/// #             "phylum_division": "chordata",
/// #             "phylum_division_id": 2,
/// #             "class": "mammalia",
/// #             "class_id": 3,
/// #             "order": "primates",
/// #             "order_id": 4,
/// #             "family": "hominidae",
/// #             "family_id": 5,
/// #             "genus": genus,
/// #             "genus_id": 6
/// #         },
/// #         "habitats": [habitat],
/// #         "characteristics": ["woody"],
/// #         "observation_count": 0,
/// #         "recorded_individuals_total": 0,
/// #         "latest_observation_year": null,
/// #         "created_at": "2026-01-01T00:00:00Z",
/// #         "updated_at": "2026-01-01T00:00:00Z"
/// #     }))
/// #     .unwrap()
/// # }
/// let input = vec![species(1, "pongo", "wetland"), species(2, "pongo", "wetland")];
/// let query = RelationshipQuery {
///     species_id: 1,
///     min_score: 0.0,
///     limit: 10,
///     required_basis: None,
/// };
/// let weights = ScoreWeights::default();
///
/// let result = explore_relationships(&input, &query, &weights).expect("valid query");
/// assert_eq!(result.center_id, 1);
/// assert_eq!(result.scoring_version, "relationship-v1");
/// // star graph: k rekomendasi -> k + 1 node dan k edge, selalu satu pusat
/// assert_eq!(result.network.nodes.len(), result.related.len() + 1);
/// assert_eq!(result.network.edges.len(), result.related.len());
/// assert!(result.network.nodes.iter().any(|node| node.is_center));
/// ```
pub fn explore_relationships(
    input: &[Species],
    query: &RelationshipQuery,
    weights: &ScoreWeights,
) -> Result<ExplorerResult, ExplorerError> {
    validation::validate_query(query)?;
    validation::validate_weights(weights)?;

    let species = validation::prepare_species(input)?;
    let center = validation::find_species(&species, query.species_id)?;
    let candidates = discovery::candidate_species(&species, center.id);
    let scored = discovery::score_candidates(center, &candidates, weights);
    let related = discovery::rank_related(scored, query);

    Ok(network::assemble_result(center, related, weights))
}
