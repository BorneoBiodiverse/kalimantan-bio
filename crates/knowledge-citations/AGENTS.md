# AGENTS.md — knowledge-citations (Module 5: Biodiversity Knowledge & Citation Explorer)

## Module Overview

**Crate:** `knowledge-citations`  
**Package:** `knowledge-citations`  
**Module:** 5 — Biodiversity Knowledge & Citation Explorer  
**Location:** `crates/knowledge-citations/`  
**Planning Source:** `planning/Planning_Modul_5_Biodiversity_Knowledge_Citation_Explorer.md` §17

---

## Public API (§17.2)

| `pub fn` | Signature | Purpose |
|----------|-----------|---------|
| `explore_knowledge` | `fn explore_knowledge(query: &KnowledgeQuery, publications: &[Publication]) -> Result<KnowledgeReport, ModuleError>` | Main pipeline: parse → normalize → query → analyze → rank → export |
| `recommend_citations` | `fn recommend_citations(items: &[Publication], query: &PublicationQuery, limit: usize) -> Vec<&Publication>` | Optional: top-N citation recommendations |
| `export_citations` | `fn export_citations(items: &[Publication], format: CitationFormat) -> String` | Optional: APA/BibTeX export |

---

## Public Types in Signatures (§2.3, §3)

| Type | Kind | Description |
|------|------|-------------|
| `Publication` | `struct` | `id`, `title`, `abstract_text`, `publication_year`, `authors`, `venue`, `doi`, `url`, `source_type`, `species_ids`, `taxon_names`, `topics`, `locations`, `citation_count` |
| `Author` | `struct` | `id`, `name`, `affiliation` |
| `SourceType` | `enum` | `JournalArticle`, `ConferencePaper`, `Thesis`, `Report`, `BookChapter`, `Dataset` |
| `ResearchTopic` | `enum` | `Taxonomy`, `Ecology`, `Conservation`, `Ethnobotany`, `Habitat`, `SpeciesIdentification`, `Other(String)` |
| `ResearchLocation` | `struct` | `name`, `province`, `regency`, `location_type` |
| `LocationType` | `enum` | `Province`, `Regency`, `Forest`, `ConservationArea`, `Other(String)` |
| `PublicationQuery` | `struct` | `text`, `species_id`, `taxon_name`, `topic`, `location`, `year_from`, `year_to`, `source_type` |
| `KnowledgeQuery` | `struct` | Query parameters for `explore_knowledge` |
| `KnowledgeReport` | `struct` | `ranked_publications`, `timeline`, `coverage`, `understudied`, `graph` |
| `CoverageRecord` | `struct` | `entity_name`, `publication_count`, `species_count`, `first_year`, `latest_year` |
| `KnowledgeGraph` | `struct` | Node/edge structure for knowledge network |
| `CitationFormat` | `enum` | `APA`, `BibTeX` |
| `ModuleError` | `enum` | Pipeline error variants |
| `ParseError` | `enum` | Parsing error variants |
| `PipelineError` | `enum` | Pipeline execution errors |

All public types and fields **must have rustdoc comments**.

---

## Shared Library Dependencies

```rust
use kalimantanbio_shared::{
    core::Species,
    text::normalize_text,
    stats::{build_timeline, count_by_key, calculate_coverage_percentage},
    scoring::{rank_by_score, filter_by_threshold, top_n},
};
```

---

## Internal (Private) Functions — Not Exported (§17.3)

- `parse_publications`, `normalize_publication`, `validate_publication`, `deduplicate_publications`, `build_species_index`
- `publications_for_species`, `publications_for_taxon`, `publications_for_topic`, `publications_for_location`, `query_publications`
- `build_research_timeline`, `calculate_topic_distribution`, `calculate_location_distribution`, `calculate_entity_coverage`, `find_understudied_species`, `calculate_coverage_score`
- `score_publication_relevance`, `rank_publications`
- `format_apa`, `format_bibtex`
- `build_knowledge_graph`, `related_publications`

---

## Inter-Module Communication (§17.4)

### Provider (this module exposes):

| `pub fn` | Consumer | Purpose | Data Sent | Priority |
|----------|----------|---------|-----------|----------|
| `explore_knowledge` | Axum handler `/api/v1/knowledge/*` | Primary knowledge API | `KnowledgeReport` | **Required** |
| `recommend_citations` | Axum handler `/api/v1/knowledge/citations` | Citation recommendations | `Vec<Publication>` | Optional |
| `export_citations` | Axum handler `/api/v1/knowledge/export` | Citation export | `String` (APA/BibTeX) | Optional |

### Receiver (this module consumes):

| Provider | `pub fn` | Purpose | Data Received | Priority |
|----------|----------|---------|---------------|----------|
| Module 1 | `search` | Species scope for publication search | `species_ids` | Optional |
| Module 2 | `explore_relationships` | Species references from relationships | `species_ids` | Optional |
| Module 3 | `analyze_taxonomic_gap` | Understudied species for research priority | `GapReport` | Optional |

---

## Rustdoc Requirements (§17.5)

- `explore_knowledge`, `recommend_citations`, `export_citations` — complete docs
- All public types in signatures: `KnowledgeQuery`, `KnowledgeReport`, `Publication`, `ResearchTopic`, `ResearchLocation`, `KnowledgeGraph`, `CitationFormat`

```bash
cargo doc -p knowledge-citations --no-deps --open
cargo check -p knowledge-citations
cargo test -p knowledge-citations
```

---

## Pipeline Composition (§9)

```rust
fn explore_knowledge(input: &KnowledgeInput) -> Result<KnowledgeReport, PipelineError> {
    let parsed = parse_publications(&input.raw_publications)?;
    let normalized = parsed
        .into_iter()
        .map(normalize_publication)
        .collect::<Vec<_>>();
    let publications = deduplicate_publications(&normalized);

    let matching = query_publications(&publications, &input.query);
    let ranked = rank_publications(&matching, &input.query);
    let timeline = build_research_timeline(&publications);
    let coverage = calculate_entity_coverage(&publications, &input.species_ids);
    let understudied = find_understudied_species(&coverage, input.understudied_threshold);
    let graph = build_knowledge_graph(&publications);

    Ok(KnowledgeReport {
        ranked_publications: ranked,
        timeline,
        coverage,
        understudied,
        graph,
    })
}
```

---

## Verification

```bash
cargo check -p knowledge-citations
cargo doc -p knowledge-citations --no-deps
cargo test -p knowledge-citations
cargo clippy -p knowledge-citations
```

---

## Stub Convention

```rust
todo!("knowledge-citations phase 5: explore_knowledge")
todo!("knowledge-citations phase 4: recommend_citations")
todo!("knowledge-citations phase 4: export_citations")
```

Signatures must match planning §17.2 exactly.