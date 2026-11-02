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

## Rustdoc & Verification Standards

- `compare_species`, `calculate_pair_similarity` — complete rustdoc comments.
- All public types in signatures must be documented: `ComparisonQuery`, `ComparisonResult`, `SimilarityScore`, `SpeciesAttributeRow`, `ScoringWeights`, `ModuleError`.
- Doc examples must be tagged `rust,ignore` while bodies are stubs.

Canonical verification commands for this crate:
```bash
cargo fmt -p species-comparison -- --check
cargo check -p species-comparison
cargo test -p species-comparison
cargo clippy -p species-comparison
cargo doc -p species-comparison --no-deps
```

---

## Implementation Stages & Submodule Layout

Module 4 follows the domain/feature-oriented module layout across five incremental stages:

| Stage | Focus | Primary Location | Key Functions |
|:---|:---|:---|:---|
| **Stage 1** | Query validation & data retrieval | `src/lib.rs` | `validate_query`, `fetch_species_by_ids`, `build_kabupaten_set`, `build_observations_map` |
| **Stage 2** | Matrix assembly & attribute extraction | `src/matrix.rs` | `build_attribute_row`, `build_attribute_matrix`, `extract_attribute_values` |
| **Stage 3** | Shared & unique attribute analysis | `src/attributes.rs` | `is_attribute_shared`, `find_shared_attributes`, `find_unique_attributes`, `find_distinguishing_characteristics` |
| **Stage 4** | Similarity scoring & pairwise metrics | `src/scoring.rs` | `score_conservation`, `score_species_type`, `score_distribution`, `calculate_similarity_score`, `calculate_all_pairs`, `find_most_similar`, `calculate_pair_similarity` |
| **Stage 5** | Summary generation & pipeline integration | `src/lib.rs` | `generate_summary`, `compare_species` |

---

## Agent Skills Integration

Invoke these project skills from `.agents/skills/` during development:

| Skill | Trigger | Expected Outcome |
|:---|:---|:---|
| `rust-best-practices` | Writing/refactoring functions, collections, data modeling. | Zero-copy references, minimal allocations (e.g. `BTreeSet<&str>`), idiomatic `Option`/`Result` usage. |
| `tdd` / `rust-testing` | Implementing a stage or writing tests. | Red-Green-Refactor flow; edge-case coverage (empty inputs, missing observations) using shared fixtures. |
| `systematic-debugging` | Test failure or compiler diagnostic. | Root-cause analysis before editing; zero speculative patches. |
| `thermo-nuclear-code-quality-review` | **Mandatory before staging commits or creating PRs**. | Strict audit: search for code-judo simplifications, eliminate redundant branches/tables, keep files $< 1000$ LOC. |

---

## Pull Request Protocol & Review Standards

### 1. Merge Gate & Branch Protection ("Protect Main")
- Target branch is `main`.
- Active GitHub ruleset `Protect Main` requires 1 approving review and `require_code_owner_review: true`.
- In the absence of a `.github/CODEOWNERS` file, GitHub requires approval from an organization **`MEMBER`** (e.g. `@sleepymor`). Outside collaborator (`COLLABORATOR`) approvals do not unblock the merge gate.

### 2. PR Lifecycle
1. **Tracked Issue:** Create an issue via `gh issue create` before branching.
2. **Branching:** Format feature branches as `module4/<type>/<short-description>`.
3. **Atomic Commits:** Conventional Commits (`feat(species-comparison): ...`, `test(species-comparison): ...`).
4. **Thermo-Nuclear Review:** Run `thermo-nuclear-code-quality-review` before proposing or staging commits.
5. **No Autonomous Git Mutations:** All `git add`, `git commit`, and `git push` commands require explicit user consent.
6. **Draft PR Creation:** Open a Draft PR targeting `main` with `gh pr create --draft`.

### 3. High-Signal Pull Request Template
Every PR description should follow this structured template:

```markdown
## Summary
- <1-2 sentence overview of what this PR implements and why>
- <Stage addressed: e.g. Stage 2 (Matrix Assembly) per Planning Doc §5>

## Code-Judo & Structural Simplifications
- <Highlight simplifications made: e.g. eliminated redundant lookup tables, unified column parsing, zero-copy borrowing>
- <Files kept lean and modular under 1000 LOC>

## Invariants & Design Adherence
- [x] Exact planned public function signatures preserved per §17.2
- [x] Zero cross-module crate dependencies (shared crate only)
- [x] Internal helpers strictly encapsulated as `pub(crate)` or `private`
- [x] Decoupled purity (no UI coordinates, HTML/CSS, or Django presentation)

## Verification Proof
- [x] Audited with `thermo-nuclear-code-quality-review`
- [x] `cargo fmt -p species-comparison -- --check`
- [x] `cargo check -p species-comparison`
- [x] `cargo test -p species-comparison` (X passed, 0 failed)
- [x] `cargo clippy -p species-comparison`
- [x] `cargo doc -p species-comparison --no-deps`
- [x] `git diff --check`

## Related
- Closes #<issue-number>
- Planning Doc: `planning/Planning_Module_4_Comparative_Species_Explorer.md`
```