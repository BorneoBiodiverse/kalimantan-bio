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