# AGENTS.md — species-comparison (Module 4: Comparative Species Explorer)

## Module Overview

**Crate:** `species-comparison`  
**Package:** `species-comparison`  
**Module:** 4 — Comparative Species Explorer  
**Location:** `crates/species-comparison/`  
**Planning Source:** `planning/Planning_Module_4_Comparative_Species_Explorer.md` §17

---

## Frozen contract (decided — do not re-litigate)

Resolved against the code and the `api-server` call site. Where the planning
document disagreed with itself (§8/§9 draft vs §17/§19 final), the code won.

| Decision | Value |
|----------|-------|
| Weights type name | `ScoringWeights` — no rename to `SimilarityWeights` |
| Query validator name | `validate_query` — no rename to `validate_comparison_query` |
| `compare_species` arity | 3 parameters; weights come from `ScoringWeights::default()` |
| Public error | `ModuleError` only; `QueryError` / `DataError` are private |
| Retired names | `ComparisonError`, `best_score` — they do not exist in code |
| Max species per query | 10 (matches shared `validate_id_list`) |
| `jaccard` / `normalize` / taxonomy scale | owned by `kalimantanbio-shared`; this module consumes them |

Evidence: `crates/api-server/src/routes/comparison.rs:43` calls
`compare_species(&query, &species, &observations)` with three arguments, and
`cargo check --workspace` passes.

---

## Public API (§17.2)

| `pub fn` | Signature | Purpose |
|----------|-----------|---------|
| `compare_species` | `fn compare_species(query: &ComparisonQuery, species: &[Species], observations: &[Observation]) -> Result<ComparisonResult, ModuleError>` | Main pipeline: validate → matrix → shared/unique → similarity → summary. Pair scoring uses `ScoringWeights::default()` (0.40 / 0.30 / 0.15 / 0.15). |
| `calculate_pair_similarity` | `fn calculate_pair_similarity(a: &Species, b: &Species, kabupaten_a: &HashSet<u64>, kabupaten_b: &HashSet<u64>, weights: &ScoringWeights) -> SimilarityScore` | Optional: pair-wise similarity for reuse/testing, and the only entry point that accepts custom weights |

---

## Public Types in Signatures (§2.3, §3.2)

| Type | Kind | Description |
|------|------|-------------|
| `ComparisonQuery` | `struct` | `species_ids: Vec<u64>` |
| `ScoringWeights` | `struct` | `taxonomy: f64`, `conservation: f64`, `species_type: f64`, `distribution: f64`; `Default` = 0.40 / 0.30 / 0.15 / 0.15 |
| `SimilarityScore` | `struct` | `species_a_id`, `species_b_id: u64`, `taxonomy_score`, `conservation_score`, `type_score`, `distribution_score`, `total_score: f64` |
| `SpeciesAttributeRow` | `struct` | `species_id`, `scientific_name`, `common_name`, `taxonomy_path`, `species_type`, `iucn`, `cites`, `p106`, `observation_count`, `kabupaten_list`, `is_verified` |
| `ComparisonResult` | `struct` | `query`, `attribute_matrix`, `shared_attributes`, `unique_attributes`, `distinguishing_characteristics`, `similarity_scores`, `summary` |
| `ModuleError` | `enum` | The only error type this module exports: `InvalidQuery(String)`, `SpeciesNotFound(u64)`, `Empty(String)`, `Validation(String)` |

All public types and fields **must have rustdoc comments**.

---

## Shared Library Dependencies

```rust
use kalimantanbio_shared::{
    core::{Species, Taxonomy, Observation},
    taxonomy::{build_taxonomy_path, calculate_taxonomy_similarity},
    collections::jaccard_similarity,
    scoring::{combine_weighted_scores, rank_by_score, top_n},
    validation::validate_id_list,
    db::{create_pool, fetch_all_species, fetch_observations_for_species},
};
```

This module owns none of these helpers. `jaccard`, token normalization, and the
taxonomy similarity scale live in `kalimantanbio-shared` and are consumed
read-only (planning §19.1).

---

## Internal (Private) Functions — Not Exported (§17.3)

- `validate_query` (uses shared `validate_id_list`; min 2, max 10) → `Result<(), QueryError>`
- `fetch_species_by_ids`, `build_kabupaten_set`
- `build_attribute_row`, `build_attribute_matrix`, `extract_attribute_values`
- `is_attribute_shared`, `find_shared_attributes`, `find_unique_attributes`, `find_distinguishing_characteristics`
- `score_conservation`, `score_species_type`, `score_distribution`
- `calculate_similarity_score`, `calculate_all_pairs`, `find_most_similar`
- `generate_summary`
- `QueryError` — private enum `{ TooFewSpecies, DuplicateId, TooManySpecies }`
- `DataError` — private enum `{ SpeciesNotFound(u64) }`

**Error contract.** Only `ModuleError` leaves the module. Internals use the
precise private enums so edge cases stay assertable in unit tests
(`assert_eq!(validate_query(&q), Err(QueryError::DuplicateId))` instead of
matching free-text messages). `impl From<QueryError> for ModuleError` and
`impl From<DataError> for ModuleError` let the `?` operator convert at the
boundary, so `compare_species` still returns a single public error type.

---

## Inter-Module Communication (§17.4)

### Provider (this module exposes):

| `pub fn` | Consumer | Purpose | Data Sent | Priority |
|----------|----------|---------|-----------|----------|
| `compare_species` | Axum handler `/api/v1/compare` | Primary comparison API | `ComparisonResult` | **Required** |

### Receiver (this module consumes):

| Provider | `pub fn` | Purpose | Data Received | Priority |
|----------|----------|---------|---------------|----------|
| Module 1 | `search` | Candidate species_ids from search | `species_ids` | Optional |
| Module 2 | `explore_relationships` | Species IDs for comparison | `species_ids` | Optional |
| Module 3 | `calculate_diversity` | Diversity context | `DiversityStats` | Optional |

---

## Rustdoc Requirements (§17.5)

- `compare_species`, `calculate_pair_similarity` — complete docs
- All public types in signatures: `ComparisonQuery`, `ComparisonResult`, `SimilarityScore`, `SpeciesAttributeRow`, `ScoringWeights`, `ModuleError`

```bash
cargo doc -p species-comparison --no-deps --open
cargo check -p species-comparison
cargo test -p species-comparison
```

---

## Pipeline Composition (§9)

```rust
fn compare_species(
    query: &ComparisonQuery,
    all_species: &[Species],
    all_observations: &[Observation],
) -> Result<ComparisonResult, ModuleError> {
    validate_query(query)?;
    let weights = ScoringWeights::default(); // 0.40 / 0.30 / 0.15 / 0.15
    let species_list = fetch_species_by_ids(&query.species_ids, all_species)?;

    let observations_map: HashMap<u64, HashSet<u64>> = species_list
        .iter()
        .map(|s| {
            let obs = fetch_observations_for_species(s.id, all_observations);
            let kabupaten_set = build_kabupaten_set(&obs);
            (s.id, kabupaten_set)
        })
        .collect();

    let attribute_matrix = build_attribute_matrix(&species_list, all_observations);

    let shared_attributes = find_shared_attributes(&attribute_matrix);
    let unique_attributes = find_unique_attributes(&attribute_matrix);
    let distinguishing_characteristics = find_distinguishing_characteristics(&attribute_matrix);

    let similarity_scores = calculate_all_pairs(&species_list, &observations_map, &weights);

    let mut result = ComparisonResult {
        query: query.clone(),
        attribute_matrix,
        shared_attributes,
        unique_attributes,
        distinguishing_characteristics,
        similarity_scores,
        summary: String::new(),
    };
    result.summary = generate_summary(&result);

    Ok(result)
}
```

---

## Verification

```bash
cargo check -p species-comparison
cargo doc -p species-comparison --no-deps
cargo test -p species-comparison
cargo clippy -p species-comparison
```

---

## Stub Convention

```rust
todo!("species-comparison phase 5: compare_species")
todo!("species-comparison phase 4: calculate_pair_similarity")
```

Signatures must match planning §17.2 exactly.