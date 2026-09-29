# AGENTS.md — kalimantanbio-shared (Shared Library)

## Module Overview

**Crate:** `shared`  
**Package:** `kalimantanbio-shared`  
**Role:** Core types, shared functions, error types, and test fixtures for all modules.  
**Location:** `crates/shared/`  
**Planning Source:** `MASTERPLAN.md` Section 5

---

## Public Types (MASTERPLAN §5.2)

| Type | Kind | Description |
|------|------|-------------|
| `Species` | `struct` | Complete species data from production database |
| `Taxonomy` | `struct` | Taxonomic hierarchy (kingdom → genus) |
| `TaxonomicRank` | `enum` | Taxonomic level (Kingdom, Phylum, Class, Order, Family, Genus, Species) |
| `Observation` | `struct` | Species observation record per kabupaten |

All public types and their fields **must have rustdoc comments**.

---

## Public Function Modules (MASTERPLAN §5.3)

Each submodule is `pub` and re-exported from crate root.

### `taxonomy`
```rust
pub fn calculate_taxonomy_similarity(a: &Taxonomy, b: &Taxonomy) -> f64;
pub fn build_taxonomy_path(taxonomy: &Taxonomy) -> String;
pub fn get_common_rank(a: &Taxonomy, b: &Taxonomy) -> Option<TaxonomicRank>;
pub fn taxonomy_distance(a: &Taxonomy, b: &Taxonomy) -> u32;
```

### `collections`
```rust
pub fn jaccard_similarity<T>(a: &HashSet<T>, b: &HashSet<T>) -> f64
where T: Eq + Hash;
pub fn set_intersection<T>(a: &HashSet<T>, b: &HashSet<T>) -> Vec<T>
where T: Eq + Hash + Clone;
pub fn set_difference<T>(a: &HashSet<T>, b: &HashSet<T>) -> Vec<T>
where T: Eq + Hash + Clone;
pub fn set_union<T>(a: &HashSet<T>, b: &HashSet<T>) -> Vec<T>
where T: Eq + Hash + Clone;
```

### `scoring`
```rust
pub fn combine_weighted_scores(scores: &[(f64, f64)]) -> f64;
pub fn normalize_score(score: f64, min: f64, max: f64) -> f64;
pub fn rank_by_score<T>(items: Vec<(T, f64)>) -> Vec<(T, f64)>;
pub fn top_n<T>(items: Vec<(T, f64)>, n: usize) -> Vec<(T, f64)>;
pub fn filter_by_threshold<T>(items: Vec<(T, f64)>, threshold: f64) -> Vec<(T, f64)>;
```

### `text`
```rust
pub fn normalize_text(input: &str) -> String;
pub fn sanitize_label(label: &str) -> String;
pub fn tokenize(text: &str) -> Vec<String>;
```

### `validation`
```rust
pub fn validate_id_list(ids: &[u64], min: usize, max: usize) -> Result<(), ValidationError>;
pub fn validate_score_range(score: f64, min: f64, max: f64) -> Result<(), ValidationError>;
pub fn validate_weights_sum_to_one(weights: &[f64], tolerance: f64) -> Result<(), ValidationError>;
pub fn has_duplicates<T>(items: &[T]) -> bool
where T: Eq + Hash;
```

### `stats`
```rust
pub fn count_by_key<T, K>(items: &[T], key_fn: impl Fn(&T) -> K) -> HashMap<K, usize>
where K: Eq + Hash;
pub fn frequency_distribution<T, K>(items: &[T], key_fn: impl Fn(&T) -> K) -> Vec<(K, usize)>
where K: Eq + Hash + Ord;
pub fn calculate_coverage_percentage(available: usize, total: usize) -> f64;
pub fn build_timeline<T>(items: &[T], year_fn: impl Fn(&T) -> u16) -> BTreeMap<u16, usize>;
```

### `db`
```rust
pub fn create_pool(database_url: &str) -> Result<Pool<Postgres>, DatabaseError>;
pub fn fetch_all_species(pool: &Pool<Postgres>) -> Result<Vec<Species>, DatabaseError>;
pub fn fetch_species_by_id(pool: &Pool<Postgres>, id: u64) -> Result<Option<Species>, DatabaseError>;
pub fn fetch_observations_for_species(pool: &Pool<Postgres>, species_id: u64) -> Result<Vec<Observation>, DatabaseError>;
```

---

## Error Types (MASTERPLAN §5.3.8)

```rust
pub enum ValidationError {
    InvalidIdList(String),
    ScoreOutOfRange { score: f64, min: f64, max: f64 },
    WeightsSumMismatch { sum: f64, expected: f64, tolerance: f64 },
    DuplicateItems(String),
}

pub enum DatabaseError {
    ConnectionFailed(String),
    QueryFailed(String),
    NotFound(String),
    MigrationFailed(String),
}
```

Both enums and all variants **must have rustdoc comments**.

---

## Fixtures (MASTERPLAN §5.5)

| File | Path | Purpose |
|------|------|---------|
| `species.json` | `crates/shared/fixtures/species.json` | Species test data |
| `taxonomy.json` | `crates/shared/fixtures/taxonomy.json` | Taxonomy test data |
| `observations.json` | `crates/shared/fixtures/observations.json` | Observation test data |

---

## Rustdoc Requirements

- Crate root: `#![warn(missing_docs)]`
- Every `pub fn`, `pub struct`, `pub enum`, and public field documented
- Doc comment format: summary, `# Arguments`, `# Returns`, `# Example` (as `rust,ignore` code blocks)
- Examples must be `rust,ignore` while bodies are stubs

---

## Verification

```bash
cargo check -p kalimantanbio-shared
cargo doc -p kalimantanbio-shared --no-deps
cargo test -p kalimantanbio-shared
cargo clippy -p kalimantanbio-shared
```

---

## Dependencies

- **Workspace deps only:** `serde`, `tokio`, `sqlx`, `thiserror` (from workspace `Cargo.toml`)
- **No external crate dependencies** beyond workspace
- **No module crate dependencies** (shared is the foundation)

---

## Stub Convention

All function bodies are stubs:
```rust
todo!("shared <submodule>: <function_name>")
```

Do not implement domain logic. Signatures must match MASTERPLAN §5.3 exactly.