# MASTERPLAN - KalimantanBio Biodiversity Knowledge Platform

**Project**: KalimantanBio Biodiversity Knowledge Platform  
**Timeline**: 1-2 Months (September - November 2026)  
**Architecture**: Rust Workspace with Shared Library  
**Technology**: Rust (Axum) + Django  
**Database**: PostgreSQL (Production)  

---

## Table of Contents

1. [Project Overview](#1-project-overview)
2. [System Architecture](#2-system-architecture)
3. [Technology Stack](#3-technology-stack)
4. [Workspace Structure](#4-workspace-structure)
5. [Shared Library Specification](#5-shared-library-specification)
6. [Module Architecture](#6-module-architecture)
7. [Database Design](#7-database-design)
8. [API Architecture (Unified Axum Server)](#8-api-architecture-unified-axum-server)
9. [Integration with Django Frontend](#9-integration-with-django-frontend)
10. [Development Timeline](#10-development-timeline)
11. [Team Structure & Responsibilities](#11-team-structure--responsibilities)
12. [Functional Programming Principles](#12-functional-programming-principles)
13. [Testing Strategy](#13-testing-strategy)
14. [Deployment Strategy](#14-deployment-strategy)
15. [Risk Management](#15-risk-management)
16. [Komunikasi Antar Modul (mod & pub fn)](#16-komunikasi-antar-modul-mod--pub-fn)
17. [Rustdoc](#17-rustdoc)
18. [Rubrik Penilaian & Verifikasi Dosen](#18-rubrik-penilaian--verifikasi-dosen)

---

## 1. Project Overview

### 1.1 Project Identity

**KalimantanBio: Biodiversity Knowledge Platform** is a comprehensive biodiversity information system focused on Kalimantan, consisting of five specialized modules that extend the existing KalimantanBio ecosystem with intelligent and analytical capabilities.

### 1.2 Five Modules Overview

The platform consists of five independent but complementary modules:

1. **Module 1: Intelligent Species Search** - Natural language search, multi-attribute filtering, relevance ranking
2. **Module 2: Species Relationship Explorer** - Relationship discovery, scoring, network visualization
3. **Module 3: Taxonomy & Classification Explorer** - Taxonomic tree, diversity analysis, gap analysis
4. **Module 4: Comparative Species Explorer** - Multi-species comparison, similarity scoring
5. **Module 5: Biodiversity Knowledge & Citation Explorer** - Publication exploration, citation management

### 1.3 Core Principles

- **One Platform, Five Independent Modules**: Unified identity, independent development
- **Shared Foundation**: Common types and utilities via shared library
- **Parallel Development**: All modules developed simultaneously
- **Production-Ready**: Direct integration with production KalimantanBio API

### 1.4 Timeline

- **Duration**: 1-2 months (September - November 2026)
- **Approach**: Parallel development with shared library foundation
- **Target**: Production deployment integrated with existing KalimantanBio platform

---

## 2. System Architecture

### 2.1 High-Level Architecture

```text
┌─────────────────────────────────────────────────────────────────┐
│                       Django Frontend                            │
│                   (Python - Client Side)                         │
│            User Interface & Interaction Layer                    │
└────────────────────────┬────────────────────────────────────────┘
                         │ HTTP/JSON
                         ▼
┌─────────────────────────────────────────────────────────────────┐
│                     Django API Layer                             │
│                  (Python - API Gateway)                          │
│          Request Routing & Response Formatting                   │
└────────────────────────┬────────────────────────────────────────┘
                         │ HTTP/JSON
                         ▼
┌─────────────────────────────────────────────────────────────────┐
│              Unified Axum API Server (Rust)                      │
│                     Port: 3000                                   │
│  ┌──────────┬────────────┬──────────┬──────────┬──────────┐    │
│  │ Module 1 │  Module 2  │ Module 3 │ Module 4 │ Module 5 │    │
│  │  Search  │Relationship│ Taxonomy │Comparison│Knowledge │    │
│  │  Routes  │   Routes   │  Routes  │  Routes  │  Routes  │    │
│  └────┬─────┴──────┬─────┴────┬─────┴────┬─────┴────┬─────┘    │
│       └────────────┴──────────┴──────────┴──────────┘          │
│                                │                                 │
│           ┌────────────────────┴────────────────────┐           │
│           │     Shared Library (Rust Crate)         │           │
│           │  - Core Types (Species, Taxonomy)       │           │
│           │  - Common Functions (Scoring, Text)     │           │
│           │  - Database Access (SQLx Pool)          │           │
│           │  - Validation & Error Handling          │           │
│           └────────────────────┬────────────────────┘           │
└────────────────────────────────┼────────────────────────────────┘
                                 │ SQLx Connection Pool
                                 ▼
┌─────────────────────────────────────────────────────────────────┐
│           Production Database (PostgreSQL)                       │
│                                                                  │
│  Tables: species, taxonomy, observations, publications, etc.     │
└─────────────────────────────────────────────────────────────────┘
```

### 2.2 Architecture Principles

1. **Unified Axum Server**: Single Rust service handling all module routes
2. **Shared Library Foundation**: Common types and functions for all modules
3. **Independent Modules**: Each module as separate crate, no inter-module dependencies
4. **Django Integration**: Django acts as API gateway and frontend host
5. **Database Pooling**: Shared connection pool managed by shared library

### 2.3 Module Independence

```text
                    kalimantanbio-shared
                            │
        ┌───────────────────┼───────────────────┐
        │                   │                   │
        ▼                   ▼                   ▼
    Module 1            Module 2            Module 3
   (Search)          (Relationship)        (Taxonomy)
        │                   │                   │
        └───────────────────┼───────────────────┘
                            │
                    ┌───────┴───────┐
                    ▼               ▼
                Module 4        Module 5
              (Comparison)     (Knowledge)
```

**Key Point**: Modules depend ONLY on `shared` library, never on each other.

**Cross-Module Communication (Hybrid)**:

- Modules remain **independent crates** with no crate-to-crate dependency (no `Cargo.toml` dependency between modules).
- Each module exposes a small, deliberate set of **`pub fn` entry points** (its module API). All internal helpers stay private.
- Optional cross-module communication happens through these agreed `pub fn` interfaces, composed at the **application layer** (the unified Axum API server), not through module-level imports.
- During **parallel development**, a consumer module may rely on **mocks** of a provider's `pub fn` output; the mock is replaced by the real implementation once available.
- During the project review, unify module definitions through a standard interface at the beginning of the project. In the eyes of the evaluator, each module may act as a **Provider** (exposing `pub fn` output) or a **Receiver** (consuming another module's `pub fn` output) through the API layer. See [Section 16](#16-komunikasi-antar-modul-mod--pub-fn).

---

## 3. Technology Stack

### 3.1 Backend (Rust)

| Component | Technology | Version | Purpose |
|-----------|-----------|---------|---------|
| Language | Rust | 2021 Edition | Core programming language |
| Web Framework | Axum | 0.7 | HTTP server and routing |
| Database Client | SQLx | 0.7 | PostgreSQL async driver |
| Async Runtime | Tokio | 1.x | Async execution |
| Serialization | Serde | 1.x | JSON serialization/deserialization |
| Error Handling | thiserror | 1.x | Error type definitions |
| CORS | tower-http | 0.5 | Cross-origin resource sharing |

### 3.2 Frontend (Python)

| Component | Technology | Purpose |
|-----------|-----------|---------|
| Framework | Django | Web framework and API gateway |
| Communication | REST API | JSON over HTTP |
| Client | HTTP Client | Requests to Axum API |

### 3.3 Database

| Component | Technology | Purpose |
|-----------|-----------|---------|
| DBMS | PostgreSQL | Primary database |
| Schema | Production | KalimantanBio production schema |
| Connection | SQLx Pool | Async connection pooling |

### 3.4 Development Tools

- **Version Control**: Git + GitHub
- **CI/CD**: GitHub Actions
- **Testing**: Cargo test
- **Documentation**: rustdoc
- **Workspace**: Cargo workspace

---

## 4. Workspace Structure

### 4.1 Repository Layout

```
kalimantanbio/
├── Cargo.toml                              # Workspace root configuration
├── README.md                               # Project overview
├── .gitignore
├── .github/
│   ├── workflows/
│   │   └── ci.yml                         # GitHub Actions CI/CD
│   ├── AGENTS.md                          # AI agent instructions
│   ├── GETTING_STARTED.md
│   └── important/
│       ├── MASTERPLAN.md                  # This document
│       ├── planning/
│       │   ├── planning-modul1-intelligent-species-search.md
│       │   ├── planning-modul-2-species-relationship-explorer.md
│       │   ├── Planning_Modul_3_Taxonomy.md
│       │   ├── Planning_Module_4_Comparative_Species_Explorer.md
│       │   └── Planning_Modul_5_Biodiversity_Knowledge_Citation_Explorer.md
│       └── template/
│           ├── planning-template.md
│           └── readme-template.md
│
└── crates/
    ├── shared/                            # Shared library (Foundation)
    │   ├── Cargo.toml
    │   ├── README.md
    │   ├── src/
    │   │   ├── lib.rs                     # Public API exports
    │   │   ├── core/                      # Domain types
    │   │   │   ├── mod.rs
    │   │   │   ├── species.rs
    │   │   │   ├── taxonomy.rs
    │   │   │   └── observation.rs
    │   │   ├── db/                        # Database utilities
    │   │   │   ├── mod.rs
    │   │   │   ├── pool.rs
    │   │   │   └── queries.rs
    │   │   ├── taxonomy/                  # Taxonomy operations
    │   │   │   ├── mod.rs
    │   │   │   ├── similarity.rs
    │   │   │   ├── path.rs
    │   │   │   └── distance.rs
    │   │   ├── scoring/                   # Scoring utilities
    │   │   │   ├── mod.rs
    │   │   │   ├── combine.rs
    │   │   │   └── ranking.rs
    │   │   ├── collections/               # Set operations
    │   │   │   ├── mod.rs
    │   │   │   ├── jaccard.rs
    │   │   │   └── set_ops.rs
    │   │   ├── text/                      # Text processing
    │   │   │   ├── mod.rs
    │   │   │   ├── normalize.rs
    │   │   │   └── tokenize.rs
    │   │   ├── validation/                # Input validation
    │   │   │   ├── mod.rs
    │   │   │   ├── id.rs
    │   │   │   ├── score.rs
    │   │   │   └── weights.rs
    │   │   ├── stats/                     # Statistical functions
    │   │   │   ├── mod.rs
    │   │   │   ├── frequency.rs
    │   │   │   ├── coverage.rs
    │   │   │   └── timeline.rs
    │   │   └── error.rs                   # Error types
    │   ├── tests/
    │   │   ├── taxonomy_tests.rs
    │   │   ├── scoring_tests.rs
    │   │   ├── collections_tests.rs
    │   │   └── db_tests.rs
    │   └── fixtures/                      # Test data
    │       ├── species.json
    │       ├── taxonomy.json
    │       └── observations.json
    │
    ├── species-search/                    # Module 1: Intelligent Species Search
    │   ├── Cargo.toml
    │   ├── README.md
    │   └── src/
    │       ├── lib.rs
    │       ├── parsing.rs
    │       ├── filtering.rs
    │       ├── scoring.rs
    │       └── recommendation.rs
    │
    ├── species-relationships/             # Module 2: Species Relationship Explorer
    │   ├── Cargo.toml
    │   ├── README.md
    │   └── src/
    │       ├── lib.rs
    │       ├── validation.rs
    │       ├── evidence.rs
    │       ├── scoring.rs
    │       └── network.rs
    │
    ├── taxonomy/                          # Module 3: Taxonomy & Classification Explorer
    │   ├── Cargo.toml
    │   ├── README.md
    │   └── src/
    │       ├── lib.rs
    │       ├── parsing.rs
    │       ├── tree.rs
    │       ├── diversity.rs
    │       └── gap_analysis.rs
    │
    ├── species-comparison/                # Module 4: Comparative Species Explorer
    │   ├── Cargo.toml
    │   ├── README.md
    │   └── src/
    │       ├── lib.rs
    │       ├── retrieval.rs
    │       ├── matrix.rs
    │       ├── analysis.rs
    │       └── similarity.rs
    │
    ├── knowledge-citations/               # Module 5: Biodiversity Knowledge & Citation Explorer
    │   ├── Cargo.toml
    │   ├── README.md
    │   └── src/
    │       ├── lib.rs
    │       ├── ingestion.rs
    │       ├── explorer.rs
    │       ├── timeline.rs
    │       └── citation.rs
    │
    └── api-server/                        # Unified Axum API Server
        ├── Cargo.toml
        ├── README.md
        └── src/
            ├── main.rs                    # Server entry point
            ├── config.rs                  # Configuration
            ├── middleware/
            │   ├── mod.rs
            │   └── cors.rs                # CORS configuration
            └── routes/
                ├── mod.rs                 # Route aggregation
                ├── search.rs              # Module 1 routes
                ├── relationship.rs        # Module 2 routes
                ├── taxonomy.rs            # Module 3 routes
                ├── comparison.rs          # Module 4 routes
                └── knowledge.rs           # Module 5 routes
```

### 4.2 Workspace Configuration

**Root `Cargo.toml`:**
```toml
[workspace]
members = [
    "crates/shared",
    "crates/species-search",
    "crates/species-relationships",
    "crates/taxonomy",
    "crates/species-comparison",
    "crates/knowledge-citations",
    "crates/api-server",
]
resolver = "2"

[workspace.package]
version = "0.1.0"
edition = "2021"
authors = ["KalimantanBio Team"]

[workspace.dependencies]
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
axum = "0.7"
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "postgres"] }
tower-http = { version = "0.5", features = ["cors"] }
thiserror = "1"
```

---

## 5. Shared Library Specification

### 5.1 Overview

The shared library (`kalimantanbio-shared`) provides:
- **Standardized domain types** used by all modules
- **Common utility functions** to avoid code duplication
- **Database access layer** with connection pooling
- **Validation and error handling** primitives

**Ownership**: Shared library is owned and maintained by the project lead.

### 5.2 Core Domain Types

#### 5.2.1 Species

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Species {
    pub id: u64,
    pub scientific_name: String,
    pub common_name: String,
    pub description: String,
    pub species_type: String,           // "Endemic", "Native", "Introduced"
    pub iucn: String,                    // "CR", "EN", "VU", "NT", "LC", "DD", "-"
    pub cites: String,                   // "Appendix I", "II", "III", "-"
    pub p106: String,                    // "Dilindungi", "Tidak Dilindungi", "-"
    pub is_verified: bool,
    pub image_url: Option<String>,
    pub taxonomy: Taxonomy,
    pub observation_count: u32,
    pub recorded_individuals_total: u32,
    pub latest_observation_year: Option<u32>,
    pub created_at: String,              // ISO 8601 timestamp
    pub updated_at: String,              // ISO 8601 timestamp
}
```

**Usage**: All modules

#### 5.2.2 Taxonomy

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Taxonomy {
    pub kingdom: String,
    pub kingdom_id: u64,
    pub phylum_division: String,
    pub phylum_division_id: u64,
    pub class: String,
    pub class_id: u64,
    pub order: String,
    pub order_id: u64,
    pub family: String,
    pub family_id: u64,
    pub genus: String,
    pub genus_id: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum TaxonomicRank {
    Kingdom = 1,
    Phylum = 2,
    Class = 3,
    Order = 4,
    Family = 5,
    Genus = 6,
}
```

**Usage**: All modules

#### 5.2.3 Observation

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Observation {
    pub id: u64,
    pub species_id: u64,
    pub latitude: f64,
    pub longitude: f64,
    pub date: String,                    // ISO 8601 date
    pub count: i32,
    pub is_verified: bool,
    pub source_method: String,
    pub kabupaten_id: u64,
    pub kabupaten_name: String,
    pub province_name: String,
}
```

**Usage**: Module 4 (primarily)

### 5.3 Shared Modules

#### 5.3.1 Taxonomy Module (`taxonomy/`)

**Path**: `kalimantanbio_shared::taxonomy`  
**Used by**: Module 1, 2, 3, 4

**Functions**:

```rust
/// Calculate taxonomy similarity score (0.0 - 1.0)
/// Scoring: genus=1.0, family=0.8, order=0.6, class=0.4, phylum=0.2, kingdom=0.1
pub fn calculate_taxonomy_similarity(a: &Taxonomy, b: &Taxonomy) -> f64

/// Build taxonomy path string: "Kingdom > Phylum > Class > Order > Family > Genus"
pub fn build_taxonomy_path(taxonomy: &Taxonomy) -> String

/// Get the highest taxonomic rank where two taxonomies match
pub fn get_common_rank(a: &Taxonomy, b: &Taxonomy) -> Option<TaxonomicRank>

/// Calculate taxonomic distance between two species (0 = same genus, 6 = different kingdom)
pub fn taxonomy_distance(a: &Taxonomy, b: &Taxonomy) -> usize
```

#### 5.3.2 Collections Module (`collections/`)

**Path**: `kalimantanbio_shared::collections`  
**Used by**: Module 2, 3, 4

**Functions**:

```rust
/// Calculate Jaccard similarity coefficient: |A ∩ B| / |A ∪ B|
pub fn jaccard_similarity<T: Hash + Eq>(a: &HashSet<T>, b: &HashSet<T>) -> f64

/// Set intersection
pub fn set_intersection<T: Hash + Eq + Clone>(a: &HashSet<T>, b: &HashSet<T>) -> HashSet<T>

/// Set difference: elements in A but not in B
pub fn set_difference<T: Hash + Eq + Clone>(a: &HashSet<T>, b: &HashSet<T>) -> Vec<T>

/// Set union
pub fn set_union<T: Hash + Eq + Clone>(a: &HashSet<T>, b: &HashSet<T>) -> HashSet<T>
```

#### 5.3.3 Scoring Module (`scoring/`)

**Path**: `kalimantanbio_shared::scoring`  
**Used by**: Module 1, 2, 4, 5

**Functions**:

```rust
/// Combine weighted scores: sum(score_i * weight_i)
/// Expects weights to be validated (sum = 1.0)
pub fn combine_weighted_scores(scores: &[(f64, f64)]) -> f64

/// Normalize score to 0.0-1.0 range
pub fn normalize_score(score: f64, min: f64, max: f64) -> f64

/// Rank items by score in descending order (highest first)
pub fn rank_by_score<T>(items: Vec<(T, f64)>) -> Vec<(T, f64)>

/// Get top N items by score
pub fn top_n<T>(items: Vec<(T, f64)>, n: usize) -> Vec<(T, f64)>

/// Filter items by minimum score threshold (inclusive)
pub fn filter_by_threshold<T>(items: Vec<(T, f64)>, min_score: f64) -> Vec<(T, f64)>
```

#### 5.3.4 Text Module (`text/`)

**Path**: `kalimantanbio_shared::text`  
**Used by**: Module 1, 2, 5

**Functions**:

```rust
/// Normalize text: lowercase, trim, remove extra whitespace
pub fn normalize_text(input: &str) -> String

/// Sanitize label: trim and remove extra spaces
pub fn sanitize_label(input: &str) -> String

/// Tokenize text by whitespace
pub fn tokenize(text: &str) -> Vec<String>
```

#### 5.3.5 Validation Module (`validation/`)

**Path**: `kalimantanbio_shared::validation`  
**Used by**: Module 2, 4, 5

**Functions**:

```rust
/// Validate ID list: min/max count, no duplicates
pub fn validate_id_list(
    ids: &[u64], 
    min_count: usize, 
    max_count: usize
) -> Result<(), ValidationError>

/// Validate score is finite and in range [min, max]
pub fn validate_score_range(
    score: f64, 
    min: f64, 
    max: f64
) -> Result<(), ValidationError>

/// Validate weights: all finite, non-negative, sum to 1.0 within tolerance
pub fn validate_weights_sum_to_one(
    weights: &[f64], 
    tolerance: f64
) -> Result<(), ValidationError>

/// Check if slice contains duplicate elements
pub fn has_duplicates<T: Hash + Eq>(items: &[T]) -> bool
```

#### 5.3.6 Stats Module (`stats/`)

**Path**: `kalimantanbio_shared::stats`  
**Used by**: Module 3, 5

**Functions**:

```rust
/// Count occurrences of each key
pub fn count_by_key<T, K: Hash + Eq>(
    items: &[T],
    key_fn: impl Fn(&T) -> K
) -> HashMap<K, usize>

/// Get frequency distribution sorted by count (descending)
pub fn frequency_distribution<T, K: Hash + Eq + Clone>(
    items: &[T],
    key_fn: impl Fn(&T) -> K
) -> Vec<(K, usize)>

/// Calculate coverage percentage (0.0 - 1.0)
pub fn calculate_coverage_percentage(available: usize, total: usize) -> f64

/// Build timeline: count items per year
pub fn build_timeline<T>(
    items: &[T],
    year_fn: impl Fn(&T) -> u16
) -> BTreeMap<u16, usize>
```

#### 5.3.7 Database Module (`db/`)

**Path**: `kalimantanbio_shared::db`  
**Used by**: All modules

**Functions**:

```rust
/// Create database connection pool
pub async fn create_pool(database_url: &str) -> Result<Pool<Postgres>, sqlx::Error>

/// Fetch all species from database
pub async fn fetch_all_species(pool: &Pool<Postgres>) -> Result<Vec<Species>, sqlx::Error>

/// Fetch species by ID
pub async fn fetch_species_by_id(
    pool: &Pool<Postgres>, 
    id: u64
) -> Result<Option<Species>, sqlx::Error>

/// Fetch observations for species
pub async fn fetch_observations_for_species(
    pool: &Pool<Postgres>,
    species_id: u64
) -> Result<Vec<Observation>, sqlx::Error>
```

#### 5.3.8 Error Module (`error.rs`)

**Path**: `kalimantanbio_shared::error`  
**Used by**: All modules

**Types**:

```rust
#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("Too few items: found {found}, minimum {minimum}")]
    TooFewItems { found: usize, minimum: usize },
    
    #[error("Too many items: found {found}, maximum {maximum}")]
    TooManyItems { found: usize, maximum: usize },
    
    #[error("Duplicate IDs found")]
    DuplicateIds,
    
    #[error("Non-finite score")]
    NonFiniteScore,
    
    #[error("Score {score} out of range [{min}, {max}]")]
    ScoreOutOfRange { score: f64, min: f64, max: f64 },
    
    #[error("Non-finite weight")]
    NonFiniteWeight,
    
    #[error("Negative weight")]
    NegativeWeight,
    
    #[error("Weights sum to {sum}, expected {expected}")]
    WeightsSumMismatch { sum: f64, expected: f64 },
}

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Database error: {0}")]
    SqlxError(#[from] sqlx::Error),
    
    #[error("Species not found: {0}")]
    SpeciesNotFound(u64),
}
```

### 5.4 Shared Library Dependencies

**`crates/shared/Cargo.toml`**:
```toml
[package]
name = "kalimantanbio-shared"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "postgres"] }
thiserror = "1"

[dev-dependencies]
tokio = { version = "1", features = ["full"] }
```

### 5.5 Test Fixtures

**`crates/shared/fixtures/species.json`** - Sample species data for testing
**`crates/shared/fixtures/taxonomy.json`** - Sample taxonomy data
**`crates/shared/fixtures/observations.json`** - Sample observation data

All modules can use these fixtures for unit testing without database dependency.

---

## 6. Module Architecture

### 6.1 Module 1: Intelligent Species Search

**Crate**: `species-search`  
**Routes**: `/api/v1/search`  
**Team Size**: 5 members  

**Purpose**: Enable users to discover species using natural language queries and structured filters.

**Key Responsibilities**:
- Natural language query parsing
- Multi-attribute filtering (taxonomy, conservation, species type)
- Relevance scoring and ranking
- Related query recommendations

**Shared Library Dependencies**:
- Core types: `Species`, `Taxonomy`
- Functions: `normalize_text`, `tokenize`, `combine_weighted_scores`, `rank_by_score`

**Module-Specific Logic**:
- Query parsing and filter extraction
- Free-text matching against species names/descriptions
- Custom scoring algorithms for search relevance
- Query recommendation generation

### 6.2 Module 2: Species Relationship Explorer

**Crate**: `species-relationships`  
**Routes**: `/api/v1/species/:id/relationships`  
**Team Size**: 3 members  

**Purpose**: Help users discover and understand relationships between species.

**Key Responsibilities**:
- Related species discovery
- Relationship scoring (taxonomy, habitat, characteristics)
- Relationship explanations
- Species network data generation

**Shared Library Dependencies**:
- Core types: `Species`, `Taxonomy`
- Functions: `calculate_taxonomy_similarity`, `jaccard_similarity`, `combine_weighted_scores`, `validate_weights_sum_to_one`

**Module-Specific Logic**:
- Evidence extraction and comparison
- Relationship scoring formula
- Explanation generation
- Network graph construction

### 6.3 Module 3: Taxonomy & Classification Explorer

**Crate**: `taxonomy`  
**Routes**: `/api/v1/taxonomy/*`  
**Team Size**: 5 members  

**Purpose**: Provide interactive exploration of biological taxonomy and classification.

**Key Responsibilities**:
- Taxonomic tree construction
- Diversity statistics (family, genus, species counts)
- Taxonomic gap analysis
- Endemic taxa filtering

**Shared Library Dependencies**:
- Core types: `Species`, `Taxonomy`, `TaxonomicRank`
- Functions: `set_difference`, `calculate_coverage_percentage`, `count_by_key`, `frequency_distribution`

**Module-Specific Logic**:
- Tree building and traversal
- Lineage extraction
- Gap analysis against reference datasets
- Endemic species filtering

### 6.4 Module 4: Comparative Species Explorer

**Crate**: `species-comparison`  
**Routes**: `/api/v1/compare`  
**Team Size**: 4 members  

**Purpose**: Enable multi-species comparison to identify similarities, differences, and distinguishing characteristics.

**Key Responsibilities**:
- Multi-species comparison
- Shared and unique attribute analysis
- Similarity scoring
- Distinguishing characteristic identification

**Shared Library Dependencies**:
- Core types: `Species`, `Taxonomy`, `Observation`
- Functions: `build_taxonomy_path`, `calculate_taxonomy_similarity`, `jaccard_similarity`, `combine_weighted_scores`, `validate_id_list`

**Module-Specific Logic**:
- Attribute matrix construction
- Similarity calculation across multiple dimensions
- Distribution comparison via observations
- Summary generation

### 6.5 Module 5: Biodiversity Knowledge & Citation Explorer

**Crate**: `knowledge-citations`  
**Routes**: `/api/v1/publications`, `/api/v1/citations`  
**Team Size**: 5 members  

**Purpose**: Connect biodiversity knowledge with scientific publications and citations.

**Key Responsibilities**:
- Publication exploration by species/topic/location
- Research timeline analysis
- Coverage analysis and understudied species identification
- Citation ranking and export (APA, BibTeX)

**Shared Library Dependencies**:
- Functions: `normalize_text`, `build_timeline`, `count_by_key`, `rank_by_score`, `calculate_coverage_percentage`

**Module-Specific Logic**:
- Publication parsing and normalization
- Knowledge graph construction
- Citation formatting
- Research gap identification

---

## 7. Database Design

### 7.1 Database Technology

- **DBMS**: PostgreSQL (Production)
- **Schema**: Matches existing KalimantanBio production schema
- **Connection**: Async via SQLx with connection pooling
- **Pool Management**: Handled by shared library

### 7.2 Core Tables

Based on production KalimantanBio API structure:

#### `species` Table
- `id` (u64) - Primary key
- `scientific_name` (String)
- `common_name` (String)
- `description` (Text)
- `species_type` (String) - Endemic/Native/Introduced
- `iucn` (String) - Conservation status
- `cites` (String) - CITES status
- `p106` (String) - National protection status
- `is_verified` (Boolean)
- `image_url` (String, nullable)
- Taxonomy fields (kingdom, phylum, class, order, family, genus with IDs)
- `observation_count` (u32)
- `recorded_individuals_total` (u32)
- `latest_observation_year` (u32, nullable)
- `created_at`, `updated_at` (Timestamps)

#### `observations` Table
- `id` (u64) - Primary key
- `species_id` (u64) - Foreign key to species
- `latitude`, `longitude` (f64)
- `date` (String/Date)
- `count` (i32)
- `is_verified` (Boolean)
- `source_method` (String)
- `kabupaten_id`, `kabupaten_name`, `province_name`

#### `publications` Table (Module 5)
- Publication metadata
- Author information
- DOI, URL
- Topics, locations
- Citation count

### 7.3 Connection Pooling

Managed by shared library (`db::create_pool`):
- Pool size: 5 connections (configurable)
- Connection URL from environment variable: `DATABASE_URL`
- Shared across all module route handlers

---

## 8. API Architecture (Unified Axum Server)

### 8.1 Server Overview

**Single Axum service** running on port 3000, serving all 5 modules through dedicated route groups.

**Entry Point**: `crates/api-server/src/main.rs`

### 8.2 Route Structure

```
Base URL: http://localhost:3000

Module 1 - Species Search:
  GET  /api/v1/search?q={query}&limit={n}

Module 2 - Relationships:
  GET  /api/v1/species/:id/relationships?min_score={score}&limit={n}

Module 3 - Taxonomy:
  GET  /api/v1/taxonomy/tree
  GET  /api/v1/taxonomy/diversity?taxon={name}
  GET  /api/v1/taxonomy/gaps
  GET  /api/v1/taxonomy/endemic

Module 4 - Comparison:
  GET  /api/v1/compare?species_ids={id1,id2,id3}

Module 5 - Knowledge:
  GET  /api/v1/publications?species_id={id}&topic={topic}
  GET  /api/v1/citations?publication_ids={ids}&format={apa|bibtex}
```

### 8.3 Server Configuration

**`crates/api-server/src/main.rs`** (Conceptual):
```rust
use axum::{Router, routing::get};
use tower_http::cors::CorsLayer;
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    // Create database pool from shared library
    let db_pool = kalimantanbio_shared::db::create_pool(
        &std::env::var("DATABASE_URL").expect("DATABASE_URL required")
    )
    .await
    .expect("Failed to create database pool");
    
    // Build unified router
    let app = Router::new()
        // Module 1 routes
        .route("/api/v1/search", get(routes::search::search_species))
        
        // Module 2 routes
        .route("/api/v1/species/:id/relationships", 
               get(routes::relationship::get_relationships))
        
        // Module 3 routes
        .route("/api/v1/taxonomy/tree", get(routes::taxonomy::get_tree))
        .route("/api/v1/taxonomy/diversity", get(routes::taxonomy::get_diversity))
        .route("/api/v1/taxonomy/gaps", get(routes::taxonomy::get_gaps))
        .route("/api/v1/taxonomy/endemic", get(routes::taxonomy::get_endemic))
        
        // Module 4 routes
        .route("/api/v1/compare", get(routes::comparison::compare_species))
        
        // Module 5 routes
        .route("/api/v1/publications", get(routes::knowledge::get_publications))
        .route("/api/v1/citations", get(routes::knowledge::export_citations))
        
        // CORS for Django frontend
        .layer(CorsLayer::permissive())
        
        // Shared database pool state
        .with_state(db_pool);
    
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("Axum server listening on {}", addr);
    
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
```

### 8.4 Response Format

All endpoints return JSON:

**Success Response**:
```json
{
  "status": "success",
  "data": { ... },
  "metadata": {
    "count": 10,
    "timestamp": "2026-09-17T11:46:22Z"
  }
}
```

**Error Response**:
```json
{
  "status": "error",
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Invalid species ID",
    "details": { ... }
  }
}
```

### 8.5 HTTP Status Codes

- `200 OK` - Successful request
- `400 Bad Request` - Invalid input/query parameters
- `404 Not Found` - Resource not found (e.g., species ID)
- `500 Internal Server Error` - Server/database error
- `503 Service Unavailable` - Database connection issues

---

## 9. Integration with Django Frontend

### 9.1 Architecture Flow

```
User Browser
    ↓ (HTTP)
Django Frontend (Templates/Views)
    ↓ (Python HTTP Client)
Django API Layer (Views/Serializers)
    ↓ (HTTP/JSON)
Axum API Server
    ↓ (SQLx)
PostgreSQL Database
```

### 9.2 Django API Layer Responsibilities

- **Request Routing**: Forward user requests to appropriate Axum endpoints
- **Authentication**: Handle user authentication before calling Axum
- **Response Formatting**: Transform Axum JSON responses for Django templates
- **Error Handling**: Translate Axum errors into user-friendly messages
- **Caching**: Optional caching layer for frequently accessed data

### 9.3 Communication Protocol

- **Protocol**: HTTP/1.1 or HTTP/2
- **Format**: JSON (Content-Type: application/json)
- **Methods**: Primarily GET (read-only operations)
- **Encoding**: UTF-8

### 9.4 CORS Configuration

Axum server configured to accept requests from Django origin:

```rust
use tower_http::cors::{CorsLayer, Any};

let cors = CorsLayer::new()
    .allow_origin("http://localhost:8000".parse::<HeaderValue>().unwrap())
    .allow_methods([Method::GET, Method::OPTIONS])
    .allow_headers(Any);

let app = Router::new()
    // ... routes
    .layer(cors);
```

---

## 10. Development Timeline

### 10.1 Overview

**Total Duration**: 1-2 months (8 weeks maximum)  
**Start Date**: September 2026  
**Target Completion**: November 2026  

### 10.2 Phase-by-Phase Breakdown

#### **Week 1: Shared Library Foundation** (CRITICAL PATH)

**Owner**: Project Lead (You)  
**Objective**: Complete shared library for all modules to use

**Tasks**:
- Days 1-2: Workspace setup, core types (`Species`, `Taxonomy`, `Observation`)
- Days 3-4: Tier 1 functions (taxonomy, jaccard, scoring, text)
- Day 5: Database module (connection pool, queries)
- Days 6-7: Validation, stats, error types, tests, fixtures

**Deliverable**: `kalimantanbio-shared` v0.1.0 published to workspace

**Success Criteria**:
- [ ] All core types defined and documented
- [ ] All Tier 1 shared functions implemented
- [ ] Unit tests passing (>80% coverage)
- [ ] Test fixtures available
- [ ] All module teams can start using shared library

---

#### **Weeks 2-3: Parallel Module Implementation**

**Objective**: All 5 modules implement domain logic using shared library

**Module 1 Team (5 members)**:
- Implement query parsing and filter extraction
- Implement multi-attribute filtering
- Implement relevance scoring
- Implement related query recommendations
- Write unit tests

**Module 2 Team (3 members)**:
- Implement validation and normalization
- Implement evidence extraction and scoring
- Implement ranking and explanations
- Implement network data generation
- Write unit tests

**Module 3 Team (5 members)**:
- Implement data parsing and tree construction
- Implement diversity analysis
- Implement gap analysis
- Write unit tests

**Module 4 Team (4 members)**:
- Implement data retrieval and validation
- Implement attribute matrix construction
- Implement shared/unique analysis
- Implement similarity scoring
- Write unit tests

**Module 5 Team (5 members)**:
- Implement publication parsing
- Implement explorers (species, topic, location)
- Implement timeline and coverage analysis
- Implement citation ranking and export
- Write unit tests

**Deliverable**: All 5 module crates functional with tests passing

**Success Criteria**:
- [ ] Each module's domain logic implemented
- [ ] Unit tests passing for each module
- [ ] Integration with shared library verified
- [ ] Module-specific documentation complete

---

#### **Weeks 3-4: Unified Axum Server**

**Owner**: API Server Team  
**Objective**: Create unified Axum service integrating all 5 modules

**Tasks**:
- Create `api-server` crate
- Implement route handlers for all modules
- Configure CORS for Django
- Implement error handling and response formatting
- Write integration tests
- Database connection testing

**Deliverable**: Unified Axum API server running with all routes functional

**Success Criteria**:
- [ ] All module routes accessible
- [ ] Database connection pool working
- [ ] CORS configured correctly
- [ ] Integration tests passing
- [ ] API documentation complete

---

#### **Weeks 4-5: Database Integration & Testing**

**Objective**: Connect to production database and ensure data integrity

**Tasks**:
- Configure production database connection
- Test all database queries
- Verify data serialization/deserialization
- Performance testing and optimization
- Index optimization if needed
- End-to-end testing

**Deliverable**: Backend fully operational with production database

**Success Criteria**:
- [ ] Production database connected
- [ ] All queries working correctly
- [ ] Response times acceptable (<500ms for most queries)
- [ ] No data corruption or serialization issues
- [ ] Load testing passed

---

#### **Weeks 5-6: Django Integration**

**Owner**: Frontend Team  
**Objective**: Connect Django frontend to Axum API

**Tasks**:
- Implement Django views calling Axum endpoints
- Create API client in Django
- Error handling and user feedback
- Template integration
- User testing
- Bug fixes

**Deliverable**: Full system functional from frontend to database

**Success Criteria**:
- [ ] Django can call all Axum endpoints
- [ ] Data displays correctly in frontend
- [ ] Error messages user-friendly
- [ ] User acceptance testing passed

---

#### **Weeks 6-8: Testing, Documentation & Deployment**

**Objective**: Production-ready system

**Tasks**:
- Comprehensive integration testing
- Load testing and performance optimization
- Security review
- Documentation completion (API docs, deployment guides)
- Deployment preparation
- Staging environment testing
- Production deployment

**Deliverable**: System deployed to production

**Success Criteria**:
- [ ] All tests passing
- [ ] Performance benchmarks met
- [ ] Security review completed
- [ ] Documentation complete
- [ ] Successfully deployed to production
- [ ] Monitoring and logging configured

---

### 10.3 Critical Path

```
Week 1: Shared Library (MUST COMPLETE)
    ↓
Weeks 2-3: Module Development (PARALLEL)
    ↓
Weeks 3-4: Unified Server
    ↓
Weeks 4-5: Database Integration
    ↓
Weeks 5-6: Django Integration
    ↓
Weeks 6-8: Testing & Deployment
```

**Critical Dependency**: Week 1 shared library completion is CRITICAL PATH. All other work depends on this.

---

## 11. Team Structure & Responsibilities

### 11.1 Shared Library Owner

**Role**: Project Lead (You)  
**Responsibilities**:
- Own and maintain `kalimantanbio-shared` crate
- Define and update core types
- Implement and test shared functions
- Review module integration with shared library
- Ensure API stability
- Resolve shared library issues
- Coordinate cross-module concerns

**Critical Task**: Complete shared library Week 1

### 11.2 Module Teams

#### Module 1 Team (5 Members)
**Crate**: `species-search`  
**Responsibilities**:
- Natural language query parsing
- Multi-attribute filtering logic
- Relevance scoring algorithms
- Related query recommendations
- Unit tests and documentation

#### Module 2 Team (3 Members)
**Crate**: `species-relationships`  
**Responsibilities**:
- Evidence extraction logic
- Relationship scoring
- Explanation generation
- Network data structures
- Unit tests and documentation

#### Module 3 Team (5 Members)
**Crate**: `taxonomy`  
**Responsibilities**:
- Data ingestion and hierarchy validation
- Taxonomic tree construction and traversal
- Diversity and endemic analytics
- Coverage and gap analysis
- Pipeline integration, unit tests, and documentation

#### Module 4 Team (4 Members)
**Crate**: `species-comparison`  
**Responsibilities**:
- Data retrieval and query validation
- Attribute matrix construction
- Shared/unique attribute and distinguishing analysis
- Similarity scoring, summary generation, unit tests, and documentation

#### Module 5 Team (5 Members)
**Crate**: `knowledge-citations`  
**Responsibilities**:
- Publication parsing
- Timeline and coverage analysis
- Citation formatting
- Unit tests and documentation

### 11.3 API Server Team

**Crate**: `api-server`  
**Responsibilities**:
- Route handler implementation
- Request/response formatting
- Error handling
- CORS configuration
- Integration testing
- Server deployment

### 11.4 Frontend Team

**Technology**: Django  
**Responsibilities**:
- Django views and API client
- Frontend-backend integration
- User interface
- User testing

### 11.5 Communication Structure

- **Daily Standups**: Each team reports progress and blockers
- **Weekly Sync**: All teams + project lead
- **Shared Library Issues**: Direct communication with project lead
- **Inter-Module Questions**: Resolved via shared library owner
- **Documentation**: Each team maintains own module docs

---

## 12. Functional Programming Principles

### 12.1 Core Principles

All modules MUST adhere to functional programming principles:

#### 12.1.1 Pure Functions

Functions should not modify external state and should produce the same output for the same input.

**Good**:
```rust
fn calculate_score(a: &Species, b: &Species) -> f64 {
    calculate_taxonomy_similarity(&a.taxonomy, &b.taxonomy)
}
```

**Bad**:
```rust
static mut GLOBAL_SCORE: f64 = 0.0;
fn calculate_score(a: &Species, b: &Species) -> f64 {
    unsafe { GLOBAL_SCORE += 1.0; GLOBAL_SCORE }  // Modifies global state!
}
```

#### 12.1.2 Immutability

Avoid modifying input data. Use references for reading, return new data for transformations.

**Good**:
```rust
fn normalize_species(species: &Species) -> Species {
    let mut normalized = species.clone();
    normalized.scientific_name = normalize_text(&species.scientific_name);
    normalized
}
```

**Bad**:
```rust
fn normalize_species(species: &mut Species) {
    species.scientific_name = normalize_text(&species.scientific_name);  // Modifies input!
}
```

#### 12.1.3 Higher-Order Functions

Leverage `.map()`, `.filter()`, `.fold()`, etc. instead of manual loops.

**Good**:
```rust
let filtered: Vec<&Species> = species_list
    .iter()
    .filter(|s| s.iucn == "CR")
    .collect();
```

**Bad**:
```rust
let mut filtered = Vec::new();
for species in &species_list {
    if species.iucn == "CR" {
        filtered.push(species);
    }
}
```

#### 12.1.4 Function Composition

Break complex logic into small, composable functions.

**Good**:
```rust
fn search(query: &str, species: &[Species]) -> Vec<(&Species, f64)> {
    let normalized = normalize_text(query);
    let tokens = tokenize(&normalized);
    let filters = extract_filters(&tokens);
    let filtered = filter_species(species, &filters);
    rank_species(&filtered, &filters)
}
```

### 12.2 I/O Separation

Separate pure functions from I/O operations:

**Pure Domain Logic** (testable without I/O):
```rust
fn calculate_similarity(a: &Species, b: &Species) -> f64 {
    // Pure calculation
}
```

**I/O Adapter** (database/network):
```rust
async fn fetch_species(pool: &Pool<Postgres>) -> Result<Vec<Species>, Error> {
    // Database I/O
}
```

### 12.3 Error Handling

Use `Result` types, avoid panics in production code:

**Good**:
```rust
fn validate_query(query: &Query) -> Result<(), ValidationError> {
    if query.ids.is_empty() {
        return Err(ValidationError::TooFewItems { found: 0, minimum: 1 });
    }
    Ok(())
}
```

**Bad**:
```rust
fn validate_query(query: &Query) {
    assert!(!query.ids.is_empty());  // Panic in production!
}
```

---

## 13. Testing Strategy

### 13.1 Shared Library Testing

**Coverage Target**: Minimum 80%

**Test Types**:
- Unit tests for every public function
- Integration tests for database operations
- Property-based tests for mathematical functions (jaccard, scoring)

**Test Fixtures**:
- `shared/fixtures/species.json` - Sample species data
- `shared/fixtures/taxonomy.json` - Sample taxonomy data
- `shared/fixtures/observations.json` - Sample observation data

**Example**:
```rust
#[test]
fn test_taxonomy_similarity_same_genus() {
    let tax_a = Taxonomy { genus_id: 6, family_id: 5, /* ... */ };
    let tax_b = Taxonomy { genus_id: 6, family_id: 5, /* ... */ };
    assert_eq!(calculate_taxonomy_similarity(&tax_a, &tax_b), 1.0);
}
```

### 13.2 Module Testing

**Each module must have**:
- Unit tests for all domain logic functions
- Integration tests with shared library
- End-to-end pipeline tests
- Edge case tests (empty input, invalid data, etc.)

**Test Data**:
- Use shared fixtures from `shared/fixtures/`
- Create module-specific test data as needed
- No dependency on production database for unit tests

### 13.3 API Server Testing

**Test Types**:
- Route handler tests
- HTTP integration tests
- Database connection tests
- CORS functionality tests
- Error response tests

**Tools**:
- `axum-test` or `tower::ServiceExt::oneshot` for route testing
- Mock database for isolated testing

### 13.4 Integration Testing

**Full Stack Tests**:
- Django → Axum → Database → Response flow
- Test all major user scenarios
- Performance benchmarks
- Load testing

### 13.5 CI/CD Testing

**GitHub Actions Workflow**:
```yaml
name: CI
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
      - run: cargo test --workspace
      - run: cargo clippy --workspace
      - run: cargo fmt --check
```

---

## 14. Deployment Strategy

### 14.1 Environment Configuration

**Environment Variables**:
- `DATABASE_URL` - PostgreSQL connection string
- `AXUM_PORT` - Server port (default: 3000)
- `RUST_LOG` - Logging level
- `CORS_ORIGIN` - Django frontend URL

### 14.2 Deployment Architecture

**Option A: Single Server**
```
Server
├── Axum Process (Port 3000)
├── Django Process (Port 8000)
└── PostgreSQL Database
```

**Option B: Separate Services**
```
Axum Server (Port 3000)
    ↓
Django Server (Port 8000)
    ↓
Database Server (PostgreSQL)
```

### 14.3 Build Process

```bash
# Build all crates in release mode
cargo build --release --workspace

# Binary location
./target/release/api-server
```

### 14.4 Docker Deployment (Optional)

**Dockerfile for Axum**:
```dockerfile
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release --bin api-server

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/api-server /usr/local/bin/
ENV DATABASE_URL=postgresql://...
EXPOSE 3000
CMD ["api-server"]
```

### 14.5 Monitoring & Logging

- **Logging**: `tracing` crate for structured logging
- **Metrics**: Response times, error rates
- **Health Check**: `/health` endpoint
- **Database Monitoring**: Connection pool statistics

---

## 15. Risk Management

### 15.1 Identified Risks

| Risk | Impact | Probability | Mitigation Strategy |
|------|--------|-------------|---------------------|
| Shared library delayed | **Critical** | Medium | Project lead focuses exclusively on Week 1; no distractions |
| Type changes mid-project | **High** | Low | Finalize types Day 2, get all teams' approval before proceeding |
| Database schema mismatch | **High** | Medium | Validate with production DB schema Week 1; create test queries |
| Module integration conflicts | **Medium** | Medium | Clear API contracts; comprehensive shared library documentation |
| Performance issues | **Medium** | Low | Benchmark critical paths early (Jaccard, taxonomy scoring) |
| Team coordination overhead | **Medium** | Medium | Daily standups; clear ownership boundaries |
| Parallel development blocking | **High** | Low | Ensure shared library complete before module work begins |
| Django-Axum integration issues | **Medium** | Medium | Early proof-of-concept; document API contract clearly |

### 15.2 Contingency Plans

**If Shared Library Delayed**:
- Extend Week 1 by 2-3 days maximum
- Provide partial shared library (core types only) for teams to start
- Complete remaining shared functions Week 2

**If Module Team Blocked**:
- Identify blocker immediately in daily standup
- Project lead provides immediate support
- Reassign tasks within team if needed

**If Database Issues**:
- Use test fixtures to continue development
- Mock database layer temporarily
- Resolve production DB issues in parallel

**If Performance Problems**:
- Profile with `cargo flamegraph`
- Optimize critical paths (likely: database queries, Jaccard calculations)
- Consider caching layer if needed

### 15.3 Success Metrics

**Week 1**:
- [ ] Shared library v0.1.0 complete
- [ ] All core types defined
- [ ] All Tier 1 functions implemented and tested

**Week 3**:
- [ ] All 5 modules functional
- [ ] Unit tests passing

**Week 4**:
- [ ] Unified Axum server running
- [ ] All routes accessible

**Week 6**:
- [ ] Django integration complete
- [ ] End-to-end testing passed

**Week 8**:
- [ ] Production deployment successful
- [ ] All acceptance criteria met

---

## 16. Komunikasi Antar Modul (mod & pub fn)

Bagian ini merupakan spesifikasi utama untuk aspek **Komunikasi Antar Module** pada rubrik penilaian (40%). Prinsip yang dipakai adalah **Hybrid**: modul tetap independen, komunikasi dilakukan melalui `pub fn` yang disepakati, dikomposisikan di lapisan aplikasi (Axum), dengan dukungan mock selama pengembangan paralel.

### 16.1 Prinsip

1. Setiap modul adalah crate independen yang **hanya bergantung pada `kalimantanbio-shared`**.
2. Setiap modul mengekspos **sedikit `pub fn`** (module API) sebagai *contract* yang bisa dipakai aplikasi dan modul lain. Semua helper internal tetap **private**.
3. Tidak ada *mod-to-mod dependency* (`Cargo.toml`). Komunikasi antar modul nyata terjadi melalui komposisi `pub fn` di **Axum api-server** (lapisan aplikasi).
4. Selama pengembangan paralel, receiver boleh memakai **mock** output provider; setelah integrasi, mock tinggal diganti implementasi nyata. Ini menjaga kelima tim dapat berjalan bersamaan.
5. Setiap `pub fn` wajib didokumentasikan di rustdoc dengan contoh penggunaan. Lihat [Section 17](#17-rustdoc).

### 16.2 Module Public Interfaces (Ringkasan)

| Module | Crate | `pub fn` Utama (Module API) | Dipanggil oleh |
| --- | --- | --- | --- |
| Modul 1 (Search) | `species-search` | `search`, `recommend_related_queries` | Axum `/api/v1/search`; opsional M2/M4/M5 |
| Modul 2 (Relationships) | `species-relationships` | `explore_relationships`, `calculate_relationship` | Axum `/api/v1/species/:id/relationships`; opsional M4/M5 |
| Modul 3 (Taxonomy) | `taxonomy` | `generate_taxonomy_report`, `calculate_diversity`, `analyze_taxonomic_gap` | Axum `/api/v1/taxonomy/*`; opsional M2/M4/M5 |
| Modul 4 (Comparison) | `species-comparison` | `compare_species`, `calculate_pair_similarity` | Axum `/api/v1/compare` |
| Modul 5 (Knowledge) | `knowledge-citations` | `explore_knowledge`, `recommend_citations`, `export_citations` | Axum `/api/v1/publications`, `/api/v1/citations` |

### 16.3 Communication Matrix (Provider → Receiver)

Semua kolom **Optional** berarti komunikasi opsional yang dikomposisikan di lapisan aplikasi; tidak membentuk dependency antarcrate.

| Provider | `pub fn` | Receiver | Purpose | Data yang Dikirim | Priority |
| --- | --- | --- | --- | --- | --- |
| Modul 1 | `search` | Axum handler | Respons pencarian | query + semua spesies | **Required** |
| Modul 1 | `search` | Modul 4 (aplikasi) | Kandidat species_ids untuk perbandingan | `Vec<(&Species, f64)>` → species_ids | Optional |
| Modul 1 | `search` | Modul 2 (aplikasi) | Spesies pusat untuk eksplorasi relasi | satu species | Optional |
| Modul 1 | `search` | Modul 5 (aplikasi) | Cakupan spesies untuk eksplorasi publikasi | species_ids | Optional |
| Modul 2 | `explore_relationships` | Axum handler | Respons relasi spesies | snapshot + query + weights | **Required** |
| Modul 2 | `explore_relationships` | Modul 4 (aplikasi) | Penerusan ID spesies untuk perbandingan | species_ids | Optional |
| Modul 2 | `explore_relationships` | Modul 5 (aplikasi) | Membuka referensi spesies pada relasi | species_ids | Optional |
| Modul 3 | `generate_taxonomy_report` | Axum handler | Respons taksonomi | `BioDataInput` | **Required** |
| Modul 3 | `calculate_diversity` | Modul 2/4 (aplikasi) | Statistik keanekaragaman sebagai konteks | `DiversityStats` | Optional |
| Modul 3 | `analyze_taxonomic_gap` | Modul 5 (aplikasi) | Spesies understudied untuk prioritas riset | `GapReport` | Optional |
| Modul 4 | `compare_species` | Axum handler | Respons perbandingan | `ComparisonQuery` | **Required** |
| Modul 5 | `explore_knowledge` | Axum handler | Respons eksplorasi pengetahuan | `KnowledgeQuery` | **Required** |
| Modul 5 | `recommend_citations` / `export_citations` | Axum handler | Respons rekomendasi/ekspor sitasi | query / publication_ids | **Required** |

> Semua receiver mengonsumsi output provider **melalui `pub fn` di lapisan aplikasi**, kecuali yang ditandai "internal" di dokumen masing-masing modul. Tidak ada modul yang mengimpor crate modul lain.

### 16.4 Model Provider → Receiver

```text
                      kalimantanbio-shared
                              │ (satu-satunya dependency crate)
        ┌──────────┬──────────┼──────────┬──────────┐
        ▼          ▼          ▼          ▼          ▼
   Modul 1      Modul 2    Modul 3    Modul 4    Modul 5
   (Search)   (Relasi)   (Taksonomi) (Banding) (Pengetahuan)
      │            │          │          │          │
      └──── pub fn ┴─ contract ┴ composable ┴─────────┘
                              │
                             │  (dikomposisikan di sini)
                             ▼
                   Axum api-server (aplikasi)
                              │
                             ▼
                    Django API → Frontend
```

Tim boleh menyepakati kontrak antar modul sejak awal (mis. format `species_ids`), selama kontrak itu tidak menimbulkan dependency crate.

### 16.5 Detail Per Modul

Detail batas `mod`/`pub fn`, tabel visibilitas, dan matriks komunikasi per modul tercantum pada bagian **17. Interface Publik & Komunikasi Antar Modul** dan **18. Rubrik — Evidence** di dokumen planning masing-masing modul:

- [planning-modul1-intelligent-species-search.md](planning/planning-modul1-intelligent-species-search.md)
- [planning-modul-2-species-relationship-explorer.md](planning/planning-modul-2-species-relationship-explorer.md)
- [Planning_Modul_3_Taxonomy.md](planning/Planning_Modul_3_Taxonomy.md)
- [Planning_Module_4_Comparative_Species_Explorer.md](planning/Planning_Module_4_Comparative_Species_Explorer.md)
- [Planning_Modul_5_Biodiversity_Knowledge_Citation_Explorer.md](planning/Planning_Modul_5_Biodiversity_Knowledge_Citation_Explorer.md)

---

## 17. Rustdoc

### 17.1 Requirement

- **Seluruh `pub fn`** (module API) wajib memiliki doc comment yang mencakup deskripsi, `# Arguments`, `# Returns`, dan contoh penggunaan (`# Example`) yang dapat diverifikasi `cargo test`.
- **Seluruh tipe publik** yang muncul di signature `pub fn` wajib didokumentasikan.
- Doc comment menggunakan bahasa konsisten (nama/istilah teknis boleh Bahasa Indonesia).

### 17.2 Command Verifikasi

```bash
# Generate rustdoc untuk seluruh workspace
cargo doc --workspace --no-deps --open

# Pastikan code berkompilasi
cargo check --workspace

# Pastikan contoh di doc-comment benar
cargo test --workspace
```

### 17.3 Status

- Saat ini di seluruh dokumen planning: **Rustdoc planned** — requirements telah ditetapkan per modul (Bagian 17.6/17.5 di masing-masing dokumen modul), dokumen belum diklaim *generated & verified* sampai diimplementasikan.
- Setelah implementasi, setiap tim menjalankan command di atas dan mencatat bukti (screenshot/URL hasil `cargo doc`).

---

## 18. Rubrik Penilaian & Verifikasi Dosen

Dokumen planning ini ditargetkan agar memenuhi rubrik berikut. Tabel melacak di mana bukti *planning* berada dan apa bukti *implementasi* yang masih perlu dihasilkan.

| Rubrik | Bobot | Bukti di Planning | Bukti Implementasi yang Masih Diperlukan |
| --- | --- | --- | --- |
| A. Repositori Github | 15% | Struktur workspace, crate per modul, command build/test/rustdoc (Section 4, 17) | Repositori dibuat, diakses dosen, README diisi |
| B. Prioritas Modul | 25% | Prioritas fitur & tahapan per modul (Section 6 + bagan timeline Section 10) | Demonstrasi fitur prioritas yang sudah jalan |
| C. Rustdoc | 20% | Requirement rustdoc per modul (Section 17 + Bagian 17 di tiap planning modul) | `cargo doc` dihasilkan & aksesibel |
| D. Komunikasi Antar Module (mod & pub fn) | 40% | Batas `mod`/`pub fn`, tabel visibilitas, matriks komunikasi (Section 16 + Bagian 17 di tiap planning modul) | Interface diimplementasikan, pengujian komposisi di api-server |

### 18.1 Checklist Verifikasi

- [ ] Repositori Github dibuat dan link dicantumkan di A.4.
- [ ] Browser/dosen dapat membuka halaman rustdoc (`cargo doc`).
- [ ] Setiap modul memiliki minimal satu `pub fn` yang dipanggil dari Axum handler.
- [ ] Tidak ada dependency crate antar modul (hanya bergantung shared).
- [ ] Komunikasi antar modul didemonstrasikan minimal satu jalur (mis. hasil `search` sebagai input `compare_species`) dengan mock/integrasi nyata di lapisan aplikasi.
- [ ] Tidak ada klaim fitur/statistik yang tidak terverifikasi.

---

## Appendix A: Quick Reference

### A.1 Key Commands

```bash
# Build entire workspace
cargo build --workspace

# Run all tests
cargo test --workspace

# Check code
cargo clippy --workspace

# Format code
cargo fmt --workspace

# Run API server
cargo run --bin api-server

# Generate rustdoc (lihat Section 17)
cargo doc --workspace --no-deps --open

# Build for production
cargo build --release --workspace
```

### A.2 Module Dependencies Summary

| Module | Shared Library Dependencies |
|--------|----------------------------|
| Module 1 | core, text, scoring, taxonomy |
| Module 2 | core, taxonomy, collections, scoring, validation |
| Module 3 | core, collections, stats, taxonomy |
| Module 4 | core, taxonomy, collections, scoring, validation |
| Module 5 | text, stats, scoring, validation |

### A.3 Contact & Ownership

- **Shared Library Owner**: Project Lead
- **Module 1**: 5-person team
- **Module 2**: 3-person team
- **Module 3**: 5-person team
- **Module 4**: 4-person team
- **Module 5**: 5-person team
- **API Server**: Dedicated API team
- **Frontend**: Django team

### A.4 Important Links

- **Project Specification**: https://gusti-alfarisy.github.io/blog/2026/pbl-fp-2026/
- **Repository**: `<GitHub repository URL>` (placeholder — repositori belum dibuat; isi setelah dibuat, jangan copy-paste URL yang tidak diverifikasi)
- **Documentation**: (To be generated with rustdoc — lihat Section 17)

Checklist akses dosen:
- [ ] Link repository dapat dibuka oleh dosen.
- [ ] Dosen dapat mengakses README dan struktur workspace.
- [ ] Dosen dapat menjalankan `cargo doc` / membuka rustdoc.

---

## Document History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 0.1.0 | 2026-09-17 | Project Lead | Initial MASTERPLAN creation |
| 0.2.0 | 2026-09-18 | Project Lead | Added Section 16 (Komunikasi Antar Modul), Section 17 (Rustdoc), Section 18 (Rubrik Verifikasi); added cross-module `pub fn` communication model |

---

**End of MASTERPLAN Document**
