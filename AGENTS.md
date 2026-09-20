# AGENTS.md — KalimantanBio Rust Workspace Scaffold

## Role

You are the agent responsible for creating and maintaining the **KalimantanBio Rust workspace scaffold** inside the `kalimantan-bio` monorepo (`github.com/BorneoBiodiverse/kalimantan-bio`).

**Goal:** a compiling Cargo workspace where every module's `pub fn` interface is **declared** with the exact signatures from the planning documentation and fully documented with **rustdoc doc comments**. Function **bodies are stubs** — no working logic until each team implements its module.

**Definition of done:**

- `cargo check --workspace` passes
- `cargo doc --workspace --no-deps` generates
- `cargo test --workspace` passes

## Source of truth

The workspace API surface comes from these documents (do not invent signatures):

- `MASTERPLAN.md` — Section 5 (shared library), Section 4 (workspace, Cargo.toml), Section 16 (inter-module `pub fn` model)
- Module planning docs — Section 17 of each (`planning-modul1-…`, `planning-modul-2-…`, `Planning_Modul_3_…`, `Planning_Module_4_…`, `Planning_Modul_5_…`)

## Workspace layout

7 crates under `crates/`:

| Crate | Cargo package | Responsibilities |
| --- | --- | --- |
| `shared` | `kalimantanbio-shared` | Core types, shared functions (taxonomy, collections, scoring, text, validation, stats, db), error types, fixtures |
| `species-search` | `species-search` | Module 1 — Intelligent Species Search |
| `species-relationships` | `species-relationships` | Module 2 — Species Relationship Explorer |
| `taxonomy` | `taxonomy` | Module 3 — Taxonomy & Classification Explorer |
| `species-comparison` | `species-comparison` | Module 4 — Comparative Species Explorer |
| `knowledge-citations` | `knowledge-citations` | Module 5 — Biodiversity Knowledge & Citation Explorer |
| `api-server` | `api-server` | Unified Axum API server composing module `pub fn` |

Root workspace `Cargo.toml`: `resolver = "2"`, edition 2021, workspace deps:

- `tokio` (1, `full`)
- `serde` (1, `derive`)
- `serde_json` (1)
- `axum` (0.7)
- `sqlx` (0.7, `runtime-tokio-rustls`, `postgres`)
- `tower-http` (0.5, `cors`)
- `thiserror` (1)

## Stub body convention

Declared functions keep their **exact planned signature**; the body is a single stub:

```rust
todo!("<crate> <phase>: <function_name>")
```

Do not implement domain logic. Do not replace the signature with a placeholder.

## Rustdoc requirements

- Every `pub fn` and every public type (struct, enum, and public fields) is documented.
- Each crate root sets `#![warn(missing_docs)]`.
- Doc comment format (per MASTERPLAN §17.1): summary, `# Arguments`, `# Returns`, `# Example`.
- **Doc examples are `rust,ignore`-tagged code blocks** presenting intended usage. They must not execute while bodies are stubs (`cargo test` stays green). When a team implements the real logic, it converts the example to a runnable doctest.

```rust
/// Searches species by natural-language query.
///
/// # Arguments
///
/// * `raw_query` - The user-provided search text.
/// * `all_species` - The full species dataset.
///
/// # Returns
///
/// Ranked matches as `(species, relevance_score)` pairs.
///
/// # Example
///
/// ```rust,ignore
/// let results = search("orangutan kalimantan", &all_species);
/// ```
pub fn search(raw_query: &str, all_species: &[Species]) -> Vec<(&Species, f64)> {
    todo!("species-search phase 5: search")
}
```

## Diagrams in documentation

- All architecture, request-response flows, sequence diagrams, and pipeline compositions in planning and documentation files (`planning/*.md`) must be formatted as **Mermaid.js** diagrams (`mermaid` code blocks).
- Do not use plain text ASCII art diagrams.
- Use supported Mermaid types: `flowchart TD`, `flowchart LR`, or `sequenceDiagram`. Quote node labels containing special characters and avoid raw HTML tags in labels.

## Shared library (`kalimantanbio-shared`)

Public types (MASTERPLAN §5.2):

- struct `Species`
- struct `Taxonomy`
- enum `TaxonomicRank`
- struct `Observation`

Public folders (MASTERPLAN §5.3):

- `taxonomy` — `calculate_taxonomy_similarity`, `build_taxonomy_path`, `get_common_rank`, `taxonomy_distance`
- `collections` — `jaccard_similarity`, `set_intersection`, `set_difference`, `set_union`
- `scoring` — `combine_weighted_scores`, `normalize_score`, `rank_by_score`, `top_n`, `filter_by_threshold`
- `text` — `normalize_text`, `sanitize_label`, `tokenize`
- `validation` — `validate_id_list`, `validate_score_range`, `validate_weights_sum_to_one`, `has_duplicates`
- `stats` — `count_by_key`, `frequency_distribution`, `calculate_coverage_percentage`, `build_timeline`
- `db` — `create_pool`, `fetch_all_species`, `fetch_species_by_id`, `fetch_observations_for_species`

Error types (MASTERPLAN §5.3.8):

- enum `ValidationError`
- enum `DatabaseError`

Fixtures (MASTERPLAN §5.5): `shared/fixtures/species.json`, `shared/fixtures/taxonomy.json`, `shared/fixtures/observations.json`.

## Module crates — public API (stub declarations only)

| Crate | `pub fn` |
| --- | --- |
| `species-search` | `search`, `recommend_related_queries` |
| `species-relationships` | `explore_relationships`, `calculate_relationship` |
| `taxonomy` | `generate_taxonomy_report`, `calculate_diversity`, `analyze_taxonomic_gap` |
| `species-comparison` | `compare_species`, `calculate_pair_similarity` |
| `knowledge-citations` | `explore_knowledge`, `recommend_citations`, `export_citations` |

Module crates depend only on `kalimantanbio-shared` and workspace crates they genuinely own — **never on another module crate** (MASTERPLAN §16).

## api-server rule

- A unified Axum `Router` exposing the planned routes (MASTERPLAN §8.2).
- Route handlers **compose module `pub fn` declarations** to demonstrate the inter-module contract (the 40% rubrik item). Bodies remain stubs — no working logic.
- No module→module crate dependencies; the server composes via `pub fn` only.

## Client integration (Django)

- **Django (Port 8000)** is the confirmed frontend host and API gateway consuming the Axum API server (Port 3000) over HTTP/JSON (MASTERPLAN §9).
- **Decoupled purity:** Axum route handlers return pure domain data and graph networks without 2D/3D GUI layout coordinates or HTML/CSS formatting. Layout computation and rendering are delegated to Django templates and client-side graph visualizers (e.g., Cytoscape.js/D3.js).
- Axum configures CORS to permit the Django origin (`http://localhost:8000`).

## Git operations & Pull Request workflow

- **No autonomous git mutations:** Do not run `git add`, `git commit`, `git push`, `git restore --staged`, `git reset`, or other git state commands without explicit user instruction. All file modifications remain unstaged for user review by default.
- **Pre-execution confirmation:** When instructed to perform git/GitHub operations, always confirm the exact plan, commands, and naming with the user before executing.
- **Branch naming convention:** Always format feature branches as:
  `module/<module-crate-name>/<issue-number>-<short-description>`
  (e.g., `module/species-relationships/1-django-client-mermaid`). Never use numeric module prefixes like `module-2/...`.
- **Standard Issue-to-Draft-PR Protocol:**
  1. **Create Issue first:** Use `gh issue create` to get the assigned issue number (`#<issue-number>`).
  2. **Branch:** Check out the branch adhering to `module/<module-crate-name>/<issue-number>-<short-description>`.
  3. **Atomic commits:** Organize changes into focused atomic commits with Conventional Commit prefixes (e.g., `docs(species-relationships): ...`).
  4. **Push:** Push the feature branch to `origin` with upstream tracking (`git push -u origin <branch-name>`).
  5. **Local `main` cleanliness:** Never leave unpushed commits on local `main`. Keep local `main` synchronized with `origin/main`.
  6. **Draft PR:** Create a Draft Pull Request targeting the default branch `main` via `gh pr create --draft`, linking back to the issue (`Resolves #<issue-number>`).

## Verification

```bash
cargo check --workspace
cargo doc --workspace --no-deps
cargo test --workspace
cargo clippy --workspace
```

## Accuracy rules

- Use exact signatures from the planning documents. Never invent functions, types, dependencies, technologies, statistics, partners, or deployment facts.
- Status is **"interface scaffold — implementation pending."** Do not claim features are implemented, tested, or deployed.
- Keep signatures stable: changing a shared function signature requires updating MASTERPLAN §5.3 and all module planning §17 tables.