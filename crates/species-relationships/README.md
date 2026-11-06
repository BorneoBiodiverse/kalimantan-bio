# species-relationships

Module 2 — Species Relationship Explorer. Discovers and explains relationships between species: scoring, explanations, and network data.

> **Status: Implemented (Stages 1–5 complete, Axum API integrated).**

## Features

- **Validation & Normalization** (`validation`): sanitizes species attributes, validates dataset snapshots, query parameters, and scoring weights.
- **Evidence Extraction** (`evidence`): extracts multi-attribute similarity evidence for taxonomy, habitat, and characteristics.
- **Relationship Scoring** (`scoring`): computes weighted similarity scores and coverage across attributes (`calculate_relationship`).
- **Discovery & Ranking** (`discovery`): filters peer candidates, generates deterministic explanations, and ranks related species.
- **Species Network** (`network`): builds domain network models (`SpeciesNetwork`) connecting the center species to related neighbors.

Public API: `explore_relationships`, `calculate_relationship`. See the generated rustdoc (`cargo doc -p species-relationships`) for full signature details and examples.
