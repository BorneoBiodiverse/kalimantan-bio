# knowledge-citations

Module 5 — Biodiversity Knowledge & Citation Explorer. Connects biodiversity
knowledge with scientific publications, research topics, locations, and
citations.

The crate takes publication records as explicit input, so its analysis and
citation formatting do not make database or network requests.

## Public API

- `explore_knowledge(&KnowledgeQuery, &[Publication])` validates and
  normalizes the records, removes duplicates, applies all query filters, then
  returns ranked publications, timeline, topic/location distributions,
  species coverage, understudied species, and a knowledge network.
- `recommend_citations(&[Publication], &PublicationQuery, limit)` returns
  matching publication references in deterministic relevance order.
- `export_citations(&[Publication], CitationFormat)` exports APA or BibTeX.
- `parse_publications(&str)` deserializes JSON publication datasets.
- `publications_for_species_entity(&[Publication], &Species)` connects shared species to scientific publications.
- `related_publications(&KnowledgeGraph, &str, &[Publication])` finds related publications in the knowledge graph.

Relevance uses a normalized weighted mean: free-text match (weight 5), species
match (4), taxon/topic/location (3 each), and year/source type (1 each).
Ties are broken by newest year, highest citation count, then ascending
publication ID.

An empty dataset is an error for `explore_knowledge`; a query with no matching
records returns a report with an empty ranked list. Species with publication
counts less than or equal to the supplied `understudied_threshold` are
understudied; the default threshold is `1`. Topic and location distributions
use normalized, case-insensitive keys and deterministic map ordering.

Run focused checks from the workspace root:

```text
cargo test -p knowledge-citations
cargo clippy -p knowledge-citations
cargo doc -p knowledge-citations --no-deps
```