# api-server

Unified Axum API server for KalimantanBio. Exposes the planned routes of all five modules and composes their `pub fn` interfaces at the application layer.

> **Status: interface scaffold — implementation pending.** Handlers call module stub functions and will panic at runtime until the modules are implemented.

## Routes

| Route | Module |
| --- | --- |
| `GET /api/v1/search` | 1 — Intelligent Species Search |
| `GET /api/v1/search/recommendations` | 1 — Intelligent Species Search |
| `GET /api/v1/species/{id}/relationships` | 2 — Species Relationship Explorer |
| `GET /api/v1/taxonomy/tree` | 3 — Taxonomy & Classification Explorer |
| `GET /api/v1/taxonomy/diversity` | 3 — Taxonomy & Classification Explorer |
| `GET /api/v1/taxonomy/gaps` | 3 — Taxonomy & Classification Explorer |
| `GET /api/v1/taxonomy/endemic` | 3 — Taxonomy & Classification Explorer |
| `GET /api/v1/compare` | 4 — Comparative Species Explorer |
| `GET /api/v1/publications` | 5 — Biodiversity Knowledge & Citation Explorer |
| `GET /api/v1/citations` | 5 — Biodiversity Knowledge & Citation Explorer |