# AGENTS.md — species-comparison (Module 4: Comparative Species Explorer)

## Module Overview

**Crate:** `species-comparison`  
**Package:** `species-comparison`  
**Module:** 4 — Comparative Species Explorer  
**Location:** `crates/species-comparison/`  
**Planning Source:** `planning/Planning_Module_4_Comparative_Species_Explorer.md` §17

---

## Frozen Contract (Decided)

Resolved against the code and the `api-server` call site. Where the planning
document disagreed with itself (§8/§9 draft vs §17/§19 final), the code won.

| Decision | Value |
|----------|-------|
| Weights type name | `ScoringWeights` — no rename to `SimilarityWeights` |
| Query validator name | `validate_query` — no rename to `validate_comparison_query` |
| `compare_species` arity | 3 parameters; weights come from `ScoringWeights::default()` |
| Public error | `ModuleError` only; `QueryError` / `DataError` are private |
| Retired names | `ComparisonError`, `best_score` — do not introduce |
| Max species per query | 10 (matches shared `validate_id_list`) |
| Shared logic | `jaccard_similarity`, `normalize_score`, and taxonomy scaling are owned by `kalimantanbio-shared` |

---

## Public API & Signatures (§17.2)

| `pub fn` | Signature | Purpose |
|----------|-----------|---------|
| `compare_species` | `fn compare_species(query: &ComparisonQuery, species: &[Species], observations: &[Observation]) -> Result<ComparisonResult, ModuleError>` | Main pipeline: validate → matrix → shared/unique → similarity → summary. Pair scoring uses `ScoringWeights::default()`. |
| `calculate_pair_similarity` | `fn calculate_pair_similarity(a: &Species, b: &Species, kabupaten_a: &HashSet<u64>, kabupaten_b: &HashSet<u64>, weights: &ScoringWeights) -> SimilarityScore` | Pairwise similarity score calculation with custom weights. |

### Public Domain Types (`src/types.rs`)

- `ComparisonQuery`: `species_ids: Vec<u64>`
- `ScoringWeights`: `taxonomy: f64`, `conservation: f64`, `species_type: f64`, `distribution: f64` (`Default` = 0.40 / 0.30 / 0.15 / 0.15)
- `SimilarityScore`: `species_a_id`, `species_b_id: u64`, `taxonomy_score`, `conservation_score`, `type_score`, `distribution_score`, `total_score: f64`
- `SpeciesAttributeRow`: `species_id`, `scientific_name`, `common_name`, `taxonomy_path`, `species_type`, `iucn`, `cites`, `p106`, `observation_count`, `kabupaten_list`, `is_verified`
- `ComparisonResult`: `query`, `attribute_matrix`, `shared_attributes`, `unique_attributes`, `distinguishing_characteristics`, `similarity_scores`, `summary`
- `ModuleError`: `InvalidQuery(String)`, `SpeciesNotFound(u64)`, `Empty(String)`, `Validation(String)`

---

## Implementation Stages

| Stage | Feature / Task | File | Status |
|-------|----------------|------|--------|
| **1** | Query validation & species retrieval (`validate_query`, `fetch_species_by_ids`) | `src/validation.rs` | Completed (PR #33) |
| **2** | Matrix assembly & attribute extraction (`build_attribute_matrix`, `extract_attribute_values`) | `src/matrix.rs` | Completed (PR #43, awaiting review) |
| **3** | Shared & unique attribute analysis (`is_attribute_shared`, `find_shared_attributes`, `find_unique_attributes`, `find_distinguishing_characteristics`) | `src/attributes.rs` | Implemented |
| **4** | Similarity scoring & pair calculation (`score_conservation`, `score_species_type`, `score_distribution`, `calculate_similarity_score`, `calculate_all_pairs`, `find_most_similar`, `calculate_pair_similarity`) | `src/scoring.rs` | Next |
| **5** | Pipeline composition & summary generation (`generate_summary`, full `compare_species`) | `src/lib.rs` | Next |

---

## Agent Skills & Quality Standards

When implementing, refactoring, or auditing code in this module, apply the installed skills:

1. **`thermo-nuclear-code-quality-review`**:
   - Zero tolerance for spaghetti code or ad-hoc conditionals.
   - Keep files well below 1,000 LOC through domain modularity (`validation.rs`, `matrix.rs`, `attributes.rs`, `scoring.rs`).
   - Eliminate redundant wrappers and favor direct, idiomatic Rust.
2. **`rust-best-practices`**:
   - Borrowing over cloning (`&str`, `&[T]` in argument types).
   - Clear type boundaries: private enums (`QueryError`, `DataError`) converted to public `ModuleError` via `From` implementations.
   - No silent unwrap/expect in production code paths.
3. **`rust-testing`**:
   - Comprehensive unit tests covering edge cases (empty matrices, singletons, missing values, boundary limits).
   - Ground tests in shared JSON fixtures (`shared/fixtures/species.json`, `shared/fixtures/observations.json`).

---

## Continuous Workflow: Stacked Feature Branches

To prevent development bottlenecks when repository maintainers or code owners are offline:

1. **Stack Next Stage Branches:** Do not block waiting for PR merge. Branch the next stage directly off the completed stage branch:
   ```text
   main ───> PR #43 (Stage 2) ───> feat/shared-and-unique-attributes (Stage 3) ───> feat/similarity-scoring (Stage 4)
   ```
2. **Branch Naming Standard:** `module4/<type>/<short-description>` (e.g., `module4/feat/shared-and-unique-attributes`).
3. **Pre-PR Verification Suite:** Run before opening any PR:
   ```bash
   cargo check --workspace
   cargo test -p species-comparison
   cargo clippy -p species-comparison
   cargo fmt -p species-comparison -- --check
   cargo doc -p species-comparison --no-deps
   ```
4. **Targeting PRs:**
   - If targeting `main`, ensure each branch is clean, atomic, and rebasable.
   - Once the owner approves and merges the parent PR, rebase or fast-forward the child branch onto `main`.

---

## Pull Request Guidelines & Template

When opening a Pull Request via `gh pr create`, use the following structured body:

```markdown
## Summary
Concise description of the stage/feature implemented.

## Changes
- Stage implementation details (functions added, files created)
- Error handling and boundary conversions

## Verification
- [x] `cargo check --workspace` passes
- [x] `cargo test -p species-comparison` (X tests passed, 0 failures)
- [x] `cargo clippy -p species-comparison` clean (0 warnings)
- [x] `cargo fmt -p species-comparison -- --check` clean
- [x] `cargo doc -p species-comparison --no-deps` clean

Resolves #<issue-number>
```
