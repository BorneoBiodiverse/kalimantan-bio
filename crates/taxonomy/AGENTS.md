# AGENTS.md — taxonomy (Module 3: Taxonomy & Classification Explorer)

## Module Overview

**Crate:** `taxonomy`  
**Package:** `taxonomy`  
**Module:** 3 — Taxonomy & Classification Explorer  
**Location:** `crates/taxonomy/`  
**Planning Source:** `planning/Planning_Modul_3_Taxonomy.md` §17

---

## Public API (§17.2)

| `pub fn` | Signature | Purpose |
|----------|-----------|---------|
| `generate_taxonomy_report` | `fn generate_taxonomy_report(input: &BioDataInput) -> Result<ExplorationReport, PipelineError>` | Main entry point: full pipeline Tahap 1–4 |
| `calculate_diversity` | `fn calculate_diversity(species_subset: &[Species]) -> DiversityStats` | Diversity statistics for a species subset |
| `analyze_taxonomic_gap` | `fn analyze_taxonomic_gap(our_genera: &HashSet<String>, reference_genera: &HashSet<String>) -> GapReport` | Gap analysis: local vs reference genera |

---

## Public Types in Signatures (§2.3, §3.2)

| Type | Kind | Description |
|------|------|-------------|
| `TaxonId` | `struct` | `pub u64` wrapper |
| `Taxon` | `struct` | `id: TaxonId`, `name: String`, `rank: TaxonomicRank`, `parent_id: Option<TaxonId>` |
| `DiversityStats` | `struct` | `total_families`, `total_genera`, `total_species`, `endemic_species_count: usize` |
| `BioDataInput` | `struct` | `raw_taxons: Vec<RawTaxonRecord>`, `raw_species: Vec<RawSpeciesRecord>`, `reference_genera: HashSet<String>` |
| `GapReport` | `struct` | `missing_taxa: Vec<String>`, `coverage_score: f64` |
| `ExplorationReport` | `struct` | `endemic_genera: Vec<Taxon>`, `richness_rank: Vec<(Taxon, usize)>`, `missing_taxa: Vec<String>`, `coverage_score: f64` |
| `PipelineError` | `enum` | Error type for pipeline failures |
| `ParseError` | `enum` | Error type for parsing failures |
| `RawTaxonRecord` | `struct` | Raw taxon data from source |
| `RawSpeciesRecord` | `struct` | Raw species data from source |

All public types and fields **must have rustdoc comments**.

---

## Shared Library Dependencies

```rust
use kalimantanbio_shared::{
    core::{Species, Taxonomy, TaxonomicRank},
    collections::set_difference,
    stats::{count_by_key, frequency_distribution, calculate_coverage_percentage},
    db::{create_pool, fetch_all_species},
};
```

---

## Internal (Private) Functions — Not Exported (§17.3)

- `parse_taxons`, `parse_species`, `validate_hierarchy`
- `build_taxonomy_map`, `get_lineage`, `get_subtree_species`, `extract_our_genera`
- `filter_endemic_taxa`, `rank_taxon_richness`
- `generate_gap_report`

---

## Inter-Module Communication (§17.4)

### Provider (this module exposes):

| `pub fn` | Consumer | Purpose | Data Sent | Priority |
|----------|----------|---------|-----------|----------|
| `generate_taxonomy_report` | Axum handler `/api/v1/taxonomy/*` | Full taxonomy report | `ExplorationReport` | **Required** |
| `calculate_diversity` | Module 2, Module 4 | Diversity stats as context | `DiversityStats` | Optional |
| `analyze_taxonomic_gap` | Module 5 | Understudied species for research priority | `GapReport` | Optional |

### Receiver (this module consumes):

- **None** — Module 3 does not depend on other module crates

---

## Rustdoc Requirements (§17.5)

- `generate_taxonomy_report`, `calculate_diversity`, `analyze_taxonomic_gap` — complete docs
- All public types in signatures: `BioDataInput`, `ExplorationReport`, `DiversityStats`, `GapReport`, `Taxon`, `RawTaxonRecord`, `RawSpeciesRecord`

```bash
cargo doc -p taxonomy --no-deps --open
cargo check -p taxonomy
cargo test -p taxonomy
```

---

## Pipeline Composition (§9)

```rust
fn generate_taxonomy_report(input: &BioDataInput) -> Result<ExplorationReport, PipelineError> {
    let step_1_taxons = parse_taxons(&input.raw_taxons)?;
    let step_1_species = parse_species(&input.raw_species)?;

    let step_2_map = build_taxonomy_map(&step_1_taxons);
    let our_genera = extract_our_genera(&step_1_taxons);

    let step_3_endemic = filter_endemic_taxa(&step_1_taxons, &step_1_species);
    let step_3_richness = rank_taxon_richness(&step_2_map, &step_1_species);

    let step_4_missing = set_difference(&input.reference_genera, &our_genera);
    let step_4_score = calculate_coverage_percentage(
        our_genera.len(),
        input.reference_genera.len()
    );

    Ok(ExplorationReport {
        endemic_genera: step_3_endemic,
        richness_rank: step_3_richness,
        missing_taxa: step_4_missing,
        coverage_score: step_4_score,
    })
}
```

---

## Verification

```bash
cargo check -p taxonomy
cargo doc -p taxonomy --no-deps
cargo test -p taxonomy
cargo clippy -p taxonomy
```

---

## Stub Convention

```rust
todo!("taxonomy phase 5: generate_taxonomy_report")
todo!("taxonomy phase 3: calculate_diversity")
todo!("taxonomy phase 4: analyze_taxonomic_gap")
```

Signatures must match planning §17.2 exactly.