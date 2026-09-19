# kalimantanbio-shared

Shared library for the KalimantanBio workspace. Provides core domain types, shared utility functions, a database access layer, error types, and test fixtures. All module crates depend only on this crate.

> **Status: interface scaffold — implementation pending.**

## Modules

- `core` — `Species`, `Taxonomy`, `TaxonomicRank`, `Observation`
- `taxonomy` — taxonomy similarity, path, common rank, distance
- `collections` — Jaccard similarity and set operations
- `scoring` — weighted scoring, normalization, ranking
- `text` — normalization, sanitization, tokenization
- `validation` — ID list, score, and weight validation
- `stats` — counting, frequency, coverage, timeline
- `db` — connection pooling and species queries
- `error` — `ValidationError`, `DatabaseError`

See the generated rustdoc (`cargo doc --no-deps`) for the full API.