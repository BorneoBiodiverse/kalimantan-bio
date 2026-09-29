# AGENTS.md — api-server (Unified Axum API Server)

## Module Overview

**Crate:** `api-server`  
**Package:** `api-server`  
**Role:** Unified Axum HTTP server composing all module `pub fn` interfaces  
**Location:** `crates/api-server/`  
**Planning Source:** `MASTERPLAN.md` §8, §16

---

## Architecture

- **No domain logic** — only HTTP routing, request parsing, response serialization
- **Composes module `pub fn`** — demonstrates inter-module contract (40% rubrik)
- **CORS configured** for Django origin (`http://localhost:8000`)
- **Bodies remain stubs** — `todo!("api-server: <route>")`

---

## Routes (MASTERPLAN §8.2)

| Method | Path | Module | Handler Stub |
|--------|------|--------|--------------|
| `GET` | `/api/v1/search` | species-search | `search::search_handler` |
| `GET` | `/api/v1/search/recommendations` | species-search | `search::recommend_handler` |
| `GET` | `/api/v1/species/{id}/relationships` | species-relationships | `relationship::relationships_handler` |
| `GET` | `/api/v1/taxonomy/report` | taxonomy | `taxonomy::report_handler` |
| `GET` | `/api/v1/taxonomy/diversity` | taxonomy | `taxonomy::diversity_handler` |
| `GET` | `/api/v1/taxonomy/gaps` | taxonomy | `taxonomy::gaps_handler` |
| `POST` | `/api/v1/compare` | species-comparison | `comparison::compare_handler` |
| `GET` | `/api/v1/knowledge/explore` | knowledge-citations | `knowledge::explore_handler` |
| `GET` | `/api/v1/knowledge/citations` | knowledge-citations | `knowledge::citations_handler` |
| `GET` | `/api/v1/knowledge/export` | knowledge-citations | `knowledge::export_handler` |
| `GET` | `/health` | — | Health check |

---

## Module Dependencies

```toml
[dependencies]
kalimantanbio-shared = { path = "../shared" }
species-search = { path = "../species-search" }
species-relationships = { path = "../species-relationships" }
taxonomy = { path = "../taxonomy" }
species-comparison = { path = "../species-comparison" }
knowledge-citations = { path = "../knowledge-citations" }
axum = { workspace = true }
tokio = { workspace = true }
tower-http = { workspace = true, features = ["cors"] }
serde = { workspace = true }
serde_json = { workspace = true }
```

**Rule:** api-server depends on ALL module crates. Module crates NEVER depend on each other or api-server.

---

## Handler Pattern (Stub)

```rust
// crates/api-server/src/routes/search.rs
use axum::{extract::Query, Json};
use species_search::search;
use kalimantanbio_shared::core::Species;

pub async fn search_handler(Query(params): Query<SearchParams>) -> Json<SearchResponse> {
    // TODO: fetch all_species from shared db or fixtures
    let all_species: Vec<Species> = vec![]; // stub
    let results = search(&params.q, &all_species);
    todo!("api-server: search_handler")
}
```

---

## CORS Configuration

```rust
// In main.rs router setup
let cors = tower_http::cors::CorsLayer::new()
    .allow_origin("http://localhost:8000".parse().unwrap())
    .allow_methods([Method::GET, Method::POST])
    .allow_headers([http::header::CONTENT_TYPE]);
```

---

## Decoupled Purity Rule (MASTERPLAN §9)

- Handlers return **pure domain data** (structs/enums from modules)
- **No 2D/3D layout coordinates** (x, y, z, force-directed positions)
- **No HTML/CSS formatting**
- Layout computation → Django templates + client-side visualizers (Cytoscape.js/D3.js)

---

## Rustdoc Requirements

- Every public route handler documented
- Request/response types documented
- Module re-exports documented

```bash
cargo doc -p api-server --no-deps --open
cargo check -p api-server
cargo test -p api-server
```

---

## Verification

```bash
cargo check -p api-server
cargo doc -p api-server --no-deps
cargo test -p api-server
cargo clippy -p api-server
```

---

## Stub Convention

```rust
todo!("api-server: search_handler")
todo!("api-server: relationships_handler")
# etc.
```