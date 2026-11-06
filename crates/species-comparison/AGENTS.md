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
| **4** | Similarity scoring & pair calculation (`score_conservation`, `score_species_type`, `score_distribution`, `calculate_similarity_score`, `calculate_all_pairs`, `find_most_similar`, `calculate_pair_similarity`) | `src/scoring.rs` | Completed |
| **5** | Pipeline composition & summary generation (`generate_summary`, full `compare_species`) | `src/lib.rs` | Completed |

---

## Agent Skills & Quality Directives

Apply these installed project skills on every task in this crate:

- **`thermo-nuclear-code-quality-review`**:
  - Ruthlessly eliminate accidental complexity, spaghetti conditionals, and one-off booleans.
  - Seek "code judo" refactorings: make modules dramatically simpler, direct, and cohesive.
  - Hard limit: Keep all source files well under 500 lines (strictly block any file approaching 1,000 lines).
  - Enforce canonical layers: zero leakage of external or presentation concerns into domain logic.
- **`rust-best-practices`**:
  - Prefer borrowing over cloning (`&str`, `&[T]`, `&HashSet<u64>`).
  - Explicit error boundaries: internal error enums (`QueryError`, `DataError`) converted via `From` into public `ModuleError`.
  - Zero panics: no `.unwrap()` or `.expect()` in non-test paths.
- **`rust-testing` & `tdd`**:
  - Write test-first or test-alongside code covering empty sets, singletons, boundary limits (2 to 10 species), and realistic data.
  - Ground all integration and unit tests in shared fixtures (`species.json`, `observations.json`).
- **`systematic-debugging`**:
  - Isolate root causes through targeted unit tests before proposing fixes.

---

## Offline Maintainer Protocol (Never Get Blocked)

When maintainers or reviewers are offline/sleeping, GitHub branch protection (`REVIEW_REQUIRED`) will pause merging to `main`. **Do not stop or idle**:

1. **Stack the next stage:** Branch the next phase directly off your current phase:
   ```text
   main ───> PR #43 (Stage 2) ───> PR #45 (Stage 3) ───> module4/feat/similarity-scoring (Stage 4) ───> Stage 5
   ```
2. **Implement & verify locally:** Build, test, and lint the new stage completely:
   ```bash
   cargo check --workspace
   cargo test -p species-comparison
   cargo clippy -p species-comparison
   cargo fmt -p species-comparison -- --check
   ```
3. **Open Draft PRs with upstream tracking:** Create the issue and push the stacked branch as a Draft PR.
4. **When maintainer returns:** The reviewer can approve and merge down the stack seamlessly.

---

## Pull Request Standard & Review Template

Every PR must be opened using this structured description:

```markdown
## Summary
Concise statement of what capability this PR adds and the architectural approach.

## Code Quality & Architecture (Thermo-Nuclear Standards)
- [x] Zero spaghetti: no ad-hoc branching, clean state/policy boundaries
- [x] Modular sizing: all crate files kept well under 500 LOC
- [x] Type cleanliness: borrowing over cloning, zero unwrap in domain logic

## Changes
- `<module/file>`: Added `<functions>`
- Error handling & type conversions

## Verification
- [x] `cargo check --workspace` passed
- [x] `cargo test -p species-comparison` passed (XX tests, 0 failures)
- [x] `cargo clippy -p species-comparison` clean (0 warnings)
- [x] `cargo fmt -p species-comparison -- --check` clean
- [x] `cargo doc -p species-comparison --no-deps` clean

Resolves #<issue-number>
```
