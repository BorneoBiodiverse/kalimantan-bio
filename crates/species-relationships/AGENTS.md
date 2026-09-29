# AGENTS.md — species-relationships (Module 2: Species Relationship Explorer)

## Module Overview

**Crate:** `species-relationships`  
**Package:** `species-relationships`  
**Module:** 2 — Species Relationship Explorer  
**Location:** `crates/species-relationships/`  
**Planning Source:** `planning/planning-modul-2-species-relationship-explorer.md` §17

---

## Public API (§17.2)

| `pub fn` | Signature | Purpose |
|----------|-----------|---------|
| `explore_relationships` | `fn explore_relationships(input: &[Species], query: &RelationshipQuery, weights: &ScoreWeights) -> Result<ExplorerResult, ExplorerError>` | Main pipeline: validate → evidence → score → rank → network |
| `calculate_relationship` | `fn calculate_relationship(a: &Species, b: &Species, habitats_a: &HashSet<String>, habitats_b: &HashSet<String>, weights: &ScoreWeights) -> Result<RelationshipScore, ValidationError>` | Optional: pair-wise score for reuse/testing |

---

## Public Types in Signatures (§2.3, §3.2)

| Type | Kind | Description |
|------|------|-------------|
| `AttributeKind` | `enum` | `Taxonomy`, `Habitat`, `Characteristic` |
| `ScoreWeights` | `struct` | `taxonomy: f64`, `habitat: f64`, `characteristic: f64` |
| `RelationshipQuery` | `struct` | `species_id: u64`, `min_score: f64`, `limit: usize`, `required_basis: Option<AttributeKind>` |
| `AttributeEvidence` | `struct` | `attribute: AttributeKind`, `similarity: Option<f64>`, `shared_values: Vec<String>` |
| `RelationshipScore` | `struct` | `total: f64`, `coverage: f64`, `evidence: Vec<AttributeEvidence>` |
| `RelatedSpecies` | `struct` | `species: Species`, `score: RelationshipScore`, `explanations: Vec<String>` |
| `NetworkNode` | `struct` | `id: u64`, `label: String`, `is_center: bool` |
| `NetworkEdge` | `struct` | `source: u64`, `target: u64`, `score: RelationshipScore`, `explanations: Vec<String>` |
| `SpeciesNetwork` | `struct` | `nodes: Vec<NetworkNode>`, `edges: Vec<NetworkEdge>` |
| `ExplorerResult` | `struct` | `center_id: u64`, `related: Vec<RelatedSpecies>`, `network: SpeciesNetwork`, `weights: ScoreWeights`, `scoring_version: String` |
| `ExplorerError` | `enum` | `InvalidQuery`, `InvalidWeights`, `InvalidDataset`, `SpeciesNotFound` |

All public types and fields **must have rustdoc comments**.

---

## Shared Library Dependencies

```rust
use kalimantanbio_shared::{
    core::{Species, Taxonomy},
    taxonomy::calculate_taxonomy_similarity,
    collections::jaccard_similarity,
    scoring::combine_weighted_scores,
    validation::{validate_score_range, validate_weights_sum_to_one},
    db::{create_pool, fetch_all_species, fetch_species_by_id},
};
```

---

## Internal (Private) Functions — Not Exported (§17.3)

- `normalize_species`, `prepare_species`, `validate_query`, `validate_weights`, `find_species`
- `taxonomy_evidence`, `set_evidence`, `collect_evidence`
- `candidate_species`, `explain_relationship`, `score_candidates`, `rank_related`
- `make_node`, `make_edge`, `build_network`, `assemble_result`

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

## Rustdoc Requirements (§17.5)

- `explore_relationships` — complete with `# Arguments`, `# Returns`, `# Errors`, `# Example`
- `calculate_relationship` — formula and weight prerequisites
- All public types in signatures: `ExplorerResult`, `RelationshipQuery`, `ScoreWeights`, `RelatedSpecies`, `SpeciesNetwork`, `ExplorerError`

```bash
cargo doc -p species-relationships --no-deps --open
cargo check -p species-relationships
cargo test -p species-relationships
```

---

## Pipeline Composition (§9)

```rust
fn explore_relationships(
    input: &[Species],
    query: &RelationshipQuery,
    weights: &ScoreWeights,
) -> Result<ExplorerResult, ExplorerError> {
    validate_query(query)?;
    validate_weights(weights)?;

    let species = prepare_species(input)?;
    let center = find_species(&species, &query.species_id)?;
    let candidates = candidate_species(&species, &center.id);
    let scored = score_candidates(center, &candidates, weights);
    let related = rank_related(scored, query);

    Ok(assemble_result(center, related, weights))
}
```

---

## Verification

```bash
cargo check -p species-relationships
cargo doc -p species-relationships --no-deps
cargo test -p species-relationships
cargo clippy -p species-relationships
```

---

## Stub Convention

```rust
todo!("species-relationships phase 5: explore_relationships")
todo!("species-relationships phase 2: calculate_relationship")
```

Signatures must match planning §17.2 exactly.