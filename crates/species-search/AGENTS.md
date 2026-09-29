# AGENTS.md — species-search (Module 1: Intelligent Species Search)

## Module Overview

**Crate:** `species-search`  
**Package:** `species-search`  
**Module:** 1 — Intelligent Species Search  
**Location:** `crates/species-search/`  
**Planning Source:** `planning/planning-modul1-intelligent-species-search.md` §17

---

## Public API (§17.2)

| `pub fn` | Signature | Purpose |
|----------|-----------|---------|
| `search` | `fn search(raw_query: &str, all_species: &[Species]) -> Vec<(&Species, f64)>` | Main entry point: parse → filter → score → rank |
| `recommend_related_queries` | `fn recommend_related_queries(raw_query: &str, all_species: &[Species]) -> Vec<(String, f64)>` | Optional: recommend follow-up queries |

---

## Public Types in Signatures

- `Species` — from `kalimantanbio_shared::core` (re-exported or used directly)

---

## Shared Library Dependencies

```rust
use kalimantanbio_shared::{
    core::{Species, Taxonomy},
    text::{normalize_text, tokenize},
    taxonomy::calculate_taxonomy_similarity,
    scoring::{combine_weighted_scores, rank_by_score},
    db::{create_pool, fetch_all_species},
};
```

---

## Internal (Private) Functions — Not Exported

Per §17.3, these remain `private` (module-internal helpers):

- `extract_filters`
- `matches_free_text`, `matches_taxonomy`, `matches_conservation`, `matches_species_type`, `filter_species`
- `score_text_match`, `score_taxonomy_match`, `score_conservation_match`, `score_species`
- `extract_common_attributes`, `generate_candidate_queries`, `score_query_candidate`, `rank_related_queries`

---

## Inter-Module Communication (§17.4)

### Provider (this module exposes):

| `pub fn` | Consumer | Purpose | Data Sent | Priority |
|----------|----------|---------|-----------|----------|
| `search` | Axum handler `/api/v1/search` | Primary search API | `Vec<(&Species, f64)>` | **Required** |
| `search` | Module 4 | Candidate species_ids for comparison | `species_ids` | Optional |
| `search` | Module 2 | Center species for relationship exploration | `Species` (one) | Optional |
| `search` | Module 5 | Species scope for publication search | `species_ids` | Optional |

### Receiver (this module consumes):

- **None** — Module 1 does not depend on other module crates

---

## Rustdoc Requirements (§17.6)

- `search` — complete with `# Arguments`, `# Returns`, `# Example`
- `recommend_related_queries` — if exposed, same doc requirements
- Any public type appearing in signatures

```bash
cargo doc -p species-search --no-deps --open
cargo check -p species-search
cargo test -p species-search
```

---

## Module-Specific Types (from planning §2.3)

```rust
#[derive(Debug, Clone, Default)]
struct QueryFilters {
    free_text: Vec<String>,
    kingdom_id: Option<u64>,
    class_id: Option<u64>,
    iucn: Option<String>,
    cites: Option<String>,
    species_type: Option<String>,
    is_verified: Option<bool>,
    sort: Option<String>,
}
```

(Internal only — not exported unless needed by other modules)

---

## Pipeline Composition (§9)

```rust
pub fn search(raw_query: &str, all_species: &[Species]) -> Vec<(&Species, f64)> {
    let normalized = normalize_text(raw_query);
    let tokens = tokenize(&normalized);
    let filters = extract_filters(&tokens);
    let filtered = filter_species(all_species, &filters);
    let scored: Vec<(&Species, f64)> = filtered.into_iter()
        .map(|s| (s, score_species(s, &filters)))
        .collect();
    rank_by_score(scored)
}
```

---

## Verification

```bash
cargo check -p species-search
cargo doc -p species-search --no-deps
cargo test -p species-search
cargo clippy -p species-search
```

---

## Stub Convention

```rust
todo!("species-search phase 5: search")
todo!("species-search phase 4: recommend_related_queries")
```

Signatures must match planning §17.2 exactly.