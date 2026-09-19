# KalimantanBio

**Biodiversity Knowledge Platform** — Rust workspace.

This workspace implements the interface scaffold for the five Functional Programming exploration modules of KalimantanBio. All module `pub fn` interfaces are declared and fully documented with rustdoc. Function bodies are stubs pending implementation by each module team.

> **Status: interface scaffold — implementation pending.**

## Workspace

| Crate | Module |
| --- | --- |
| `kalimantanbio-shared` | Shared library (core types + common functions) |
| `species-search` | Module 1 — Intelligent Species Search |
| `species-relationships` | Module 2 — Species Relationship Explorer |
| `taxonomy` | Module 3 — Taxonomy & Classification Explorer |
| `species-comparison` | Module 4 — Comparative Species Explorer |
| `knowledge-citations` | Module 5 — Biodiversity Knowledge & Citation Explorer |
| `api-server` | Unified Axum API server |

## Commands

```bash
cargo check --workspace
cargo doc --workspace --no-deps
cargo test --workspace
cargo run --bin api-server
```

## Documentation

- [AGENTS.md](.github/AGENTS.md) — scaffold rules and per-crate interface declarations
- [GETTING_STARTED.md](.github/GETTING_STARTED.md) — GitHub workflow guide (Indonesian/English)

See the [project specification](https://gusti-alfarisy.github.io/blog/2026/pbl-fp-2026/#kalimantanbio-biodiversity-knowledge-platform) and the [existing platform](https://kalimantanbio.com/repository/).