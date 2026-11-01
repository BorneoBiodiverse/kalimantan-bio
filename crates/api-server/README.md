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

## Module 2: relationships endpoint

`GET /api/v1/species/{id}/relationships` composes
`species_relationships::explore_relationships` over the species snapshot held
in `AppState` (database via `DATABASE_URL`, or the shared fixtures fallback via
`SPECIES_FIXTURE`) — planning Modul 2 §8. Note: shared `create_pool` /
`fetch_all_species` are still `todo!()` stubs, so the `DATABASE_URL` path panics
until the shared crate lands them; without the env var the fixtures path is the
working default.

### Query parameters

| Parameter | Type | Default | Notes |
| --- | --- | --- | --- |
| `min_score` | float | `0.0` | Must be finite and in `0.0..=1.0`. |
| `limit` | int | `50` | Must be in `1..=50`. |
| `required_basis` | string | – | `taxonomy`, `habitat`, or `characteristic`; other values → `400 invalid_basis`. |

### Example

```bash
curl "http://localhost:3000/api/v1/species/1/relationships?min_score=0.2&limit=10"
```

Success (`200`; empty results still contain the center node):

```json
{
  "status": "success",
  "data": {
    "center_id": 1,
    "related": [],
    "network": { "nodes": [{ "id": 1, "label": "Pongo pygmaeus", "is_center": true }], "edges": [] },
    "weights": { "taxonomy": 0.5, "habitat": 0.3, "characteristic": 0.2 },
    "scoring_version": "relationship-v1"
  }
}
```

Errors carry `status`, `code`, and an actionable `message`:

| Condition | HTTP | `code` |
| --- | --- | --- |
| Invalid query or unknown `required_basis` | `400` | `invalid_query` / `invalid_basis` |
| Center species not found | `404` | `species_not_found` |
| Invalid dataset or weights (detail logged server-side) | `500` | `invalid_dataset` / `invalid_weights` |
| Data source unavailable | `503` | `data_unavailable` |

CORS allows the Django client origin (`http://localhost:8000`, override with
`CORS_ORIGIN`).

### Running the server

```bash
cargo run -p api-server          # listens on 0.0.0.0:3000
```

Environment variables (all optional):

| Variable | Default | Purpose |
| --- | --- | --- |
| `DATABASE_URL` | unset | Production source (Postgres) through shared `create_pool` / `fetch_all_species`. Note: those shared fns are still `todo!()` — the path panics until shared lands them. |
| `SPECIES_FIXTURE` | `crates/shared/fixtures/species.json` | Snapshot fallback used when `DATABASE_URL` is unset (run from the repo root). |
| `CORS_ORIGIN` | `http://localhost:8000` | Allowed Django client origin. |

With no configuration the server serves the shared fixture snapshot (5
species, ids 1–5); an unloadable source yields `503 data_unavailable`.

M2 module sources: `crates/species-relationships` (pipeline
`explore_relationships`, `cargo test -p species-relationships`), scoring
version `relationship-v1`, default weights `T=0.50, H=0.30, C=0.20`.

### Backend smoke test — local run log (2026-11-02)

Fixture: `crates/shared/fixtures/species.json` (5 species, ids 1–5),
scoring `relationship-v1`, no env vars set.

```text
$ cargo run -p api-server
species snapshot loaded: 5 records
Axum API server listening on 0.0.0.0:3000

$ curl "localhost:3000/api/v1/species/1/relationships?min_score=0.2&limit=10"            -> 200
    center 1 "Pongo pygmaeus" | related: [3 "Pongo abelii" total 0.5 coverage 0.5]
    network: 2 nodes, 1 edge, explanations generated (shared genus pongo 0.50;
    habitat & characteristic reported unavailable -> no score added)

$ curl "localhost:3000/api/v1/species/3/relationships?min_score=0.2&limit=10&required_basis=taxonomy" -> 200
    center switched to 3 (neighbor re-explore, §7: new request, no neighbor-neighbor edges)

$ curl "localhost:3000/api/v1/species/999/relationships"                                -> 404
    {"status":"error","code":"species_not_found","message":"species 999 not found"}

$ curl "localhost:3000/api/v1/species/1/relationships?required_basis=banana"            -> 400
    {"status":"error","code":"invalid_basis","message":"required_basis must be taxonomy, habitat, or characteristic (got banana)"}

$ curl "localhost:3000/api/v1/species/1/relationships?limit=99"                         -> 400
    {"status":"error","code":"invalid_query","message":"limit must be between 1 and 50"}
```

Notes: the `DATABASE_URL` path panics until the shared db fns are
implemented; `503 data_unavailable` is covered by tests (fault injection)
rather than a live outage here. Latency measurement on a large volume and a
target-environment smoke test are still pending per plan §14.