# AGENTS.md — species-relationships (Module 2: Species Relationship Explorer)

## Module Overview

**Crate:** `species-relationships`  
**Package:** `species-relationships`  
**Module:** 2 — Species Relationship Explorer  
**Location:** `crates/species-relationships/`  
**Planning Source:** `planning/planning-modul-2-species-relationship-explorer.md` §17

---

## File and Module Structure

Complies with the domain/feature-oriented convention established in root `AGENTS.md`:

```text
crates/species-relationships/
├── Cargo.toml
├── README.md
├── AGENTS.md
├── src/
│   ├── lib.rs              # Crate root, pipeline composition, re-exports
│   ├── types.rs            # Public domain types
│   ├── error.rs            # Public error types (thiserror)
│   ├── validation.rs       # Data sanitization, preparation, and validation
│   ├── evidence.rs         # Multi-attribute similarity evidence extraction
│   ├── scoring.rs          # Weighted evidence scoring and pair calculation
│   ├── discovery.rs        # Candidate filtering, explanations, and ranking
│   └── network.rs          # Domain species network data structures
└── tests/
    ├── common/             # Integration test fixtures and helpers
    └── pipeline_integration.rs # End-to-end integration test suite
```

---

## Public API (§17.2)

| `pub fn` | Signature | Purpose |
|----------|-----------|---------|
| `explore_relationships` | `fn explore_relationships(input: &[Species], query: &RelationshipQuery, weights: &ScoreWeights) -> Result<ExplorerResult, ExplorerError>` | Main pipeline: validate → evidence → score → rank → network |
| `calculate_relationship` | `fn calculate_relationship(a: &Species, b: &Species, habitats_a: &HashSet<String>, habitats_b: &HashSet<String>, weights: &ScoreWeights) -> Result<RelationshipScore, ValidationError>` | Pair-wise score calculation across taxonomy, habitat, characteristics |

---

## Public Types in Signatures (§2.3, §3.2)

| Type | Kind | Location | Description |
|------|------|----------|-------------|
| `AttributeKind` | `enum` | `types.rs` | `Taxonomy`, `Habitat`, `Characteristic` |
| `ScoreWeights` | `struct` | `types.rs` | `taxonomy: f64`, `habitat: f64`, `characteristic: f64` |
| `RelationshipQuery` | `struct` | `types.rs` | `species_id: u64`, `min_score: f64`, `limit: usize`, `required_basis: Option<AttributeKind>` |
| `AttributeEvidence` | `struct` | `types.rs` | `attribute: AttributeKind`, `similarity: Option<f64>`, `shared_values: Vec<String>` |
| `RelationshipScore` | `struct` | `types.rs` | `total: f64`, `coverage: f64`, `evidence: Vec<AttributeEvidence>` |
| `RelatedSpecies` | `struct` | `types.rs` | `species: Species`, `score: RelationshipScore`, `explanations: Vec<String>` |
| `NetworkNode` | `struct` | `types.rs` | `id: u64`, `label: String`, `is_center: bool` |
| `NetworkEdge` | `struct` | `types.rs` | `source: u64`, `target: u64`, `score: RelationshipScore`, `explanations: Vec<String>` |
| `SpeciesNetwork` | `struct` | `types.rs` | `nodes: Vec<NetworkNode>`, `edges: Vec<NetworkEdge>` |
| `ExplorerResult` | `struct` | `types.rs` | `center_id: u64`, `related: Vec<RelatedSpecies>`, `network: SpeciesNetwork`, `weights: ScoreWeights`, `scoring_version: String` |
| `ExplorerError` | `enum` | `error.rs` | `InvalidQuery`, `InvalidWeights`, `InvalidDataset`, `SpeciesNotFound` |

All public types and fields **must have rustdoc comments**.

---

## Shared Library Dependencies

```rust
use kalimantanbio_shared::{
    core::{Species, Taxonomy},
    text::normalize_text,
    taxonomy::calculate_taxonomy_similarity,
    collections::{jaccard_similarity, set_intersection},
    scoring::combine_weighted_scores,
    validation::validate_weights_sum_to_one,
};
```

---

## Feature Modules & Internal Functions (§17.3)

- **`validation`**: `normalize_species`, `prepare_species`, `validate_query`, `validate_weights`, `find_species`
- **`evidence`**: `taxonomy_evidence`, `set_evidence`, `collect_evidence`
- **`scoring`**: `score_evidence`, `calculate_relationship`
- **`discovery`**: `candidate_species`, `explain_relationship`, `score_candidates`, `rank_related`
- **`network`**: `make_node`, `make_edge`, `build_network`, `assemble_result`

---

## Inter-Module Communication (§17.4)

### Provider (this module exposes):

| `pub fn` | Consumer | Purpose | Data Sent | Priority |
|----------|----------|---------|-----------|----------|
| `explore_relationships` | Axum handler `/api/v1/species/:id/relationships` | Primary relationship API | `ExplorerResult` | **Required** |
| `explore_relationships` | Module 4 | Species IDs for comparison | `species_ids` | Optional |
| `explore_relationships` | Module 5 | Species IDs for publication lookup | `species_ids` | Optional |

### Receiver (this module consumes):

| Provider | `pub fn` | Purpose | Data Received | Priority |
|----------|----------|---------|---------------|----------|
| Module 1 | `search` | Center species from search results | `Species` (one) | Optional |
| Module 3 | Taxonomy interface | Taxonomic info for relationships | `Taxonomy` info | Optional |

---

## Pipeline Composition (§9)

```rust
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
```

---

## Verification

```bash
cargo check -p species-relationships
cargo doc -p species-relationships --no-deps
cargo test -p species-relationships
cargo clippy -p species-relationships --no-deps -- -D warnings
```
