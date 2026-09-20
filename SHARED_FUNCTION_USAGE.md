# Shared Library Function Usage Map

**Purpose:** Cross-reference showing where each shared function is used across all modules  
**Date:** 2026-09-20  
**Status:** Interface Scaffold Phase

---

## 📊 Overview

This document maps every function in `kalimantanbio-shared` to its consumers across the 5 modules.

---

## 🔧 Core Types Usage

### `Species`
**Definition:** `kalimantanbio-shared::core::Species`  
**Used by:**
- ✅ Module 1 (species-search) - Primary data type for search results
- ✅ Module 2 (species-relationships) - Input/output for relationship analysis
- ✅ Module 3 (taxonomy) - Input for taxonomy report generation
- ✅ Module 4 (species-comparison) - Input for multi-species comparison
- ✅ Module 5 (knowledge-citations) - Context for publication exploration

### `Taxonomy`
**Definition:** `kalimantanbio-shared::core::Taxonomy`  
**Used by:**
- ✅ Module 1 (species-search) - Filter by taxonomic rank
- ✅ Module 2 (species-relationships) - Taxonomy similarity scoring
- ✅ Module 3 (taxonomy) - Primary data type for taxonomic analysis
- ✅ Module 4 (species-comparison) - Taxonomy comparison dimension
- ✅ Module 5 (knowledge-citations) - Taxonomic context for publications

### `TaxonomicRank`
**Definition:** `kalimantanbio-shared::core::TaxonomicRank`  
**Used by:**
- ✅ Module 2 (species-relationships) - Rank-level similarity calculation
- ✅ Module 3 (taxonomy) - Tree hierarchy and diversity analysis
- ✅ Module 4 (species-comparison) - Taxonomy comparison

### `Observation`
**Definition:** `kalimantanbio-shared::core::Observation`  
**Used by:**
- ✅ Module 4 (species-comparison) - Geographic distribution comparison

---

## 📐 Taxonomy Module Functions

### `calculate_taxonomy_similarity`
**Signature:** `fn calculate_taxonomy_similarity(a: &Taxonomy, b: &Taxonomy) -> f64`  
**Purpose:** Calculate taxonomy similarity score (0.0-1.0)  
**Scoring:** genus=1.0, family=0.8, order=0.6, class=0.4, phylum=0.2, kingdom=0.1

**Used by:**
- ✅ **Module 1 (species-search)** 
  - File: `crates/species-search/src/lib.rs`
  - Function: `search()` (internal scoring helper)
  - Purpose: Boost relevance when query matches taxonomic terms

- ✅ **Module 2 (species-relationships)** 
  - File: `crates/species-relationships/src/lib.rs`
  - Function: `calculate_relationship()` - taxonomy evidence
  - Purpose: Taxonomy dimension of relationship scoring
  - Weight: 0.50 (default)

- ✅ **Module 4 (species-comparison)** 
  - File: `crates/species-comparison/src/lib.rs`
  - Function: `calculate_pair_similarity()` - taxonomy dimension
  - Purpose: Taxonomy similarity in multi-species comparison
  - Weight: Configurable

**NOT used by:**
- ❌ Module 3 - Has its own internal taxonomy operations
- ❌ Module 5 - Focuses on publications, not taxonomy scoring

---

### `build_taxonomy_path`
**Signature:** `fn build_taxonomy_path(taxonomy: &Taxonomy) -> String`  
**Purpose:** Build hierarchical path: "Kingdom > Phylum > Class > Order > Family > Genus"

**Used by:**
- ✅ **Module 2 (species-relationships)**
  - File: Internal helper
  - Function: `explain_relationship()` (internal)
  - Purpose: Generate human-readable taxonomy explanations

- ✅ **Module 3 (taxonomy)**
  - File: `crates/taxonomy/src/lib.rs`
  - Function: `generate_taxonomy_report()` (internal tree building)
  - Purpose: Display full taxonomy paths in reports

- ✅ **Module 4 (species-comparison)**
  - File: `crates/species-comparison/src/lib.rs`
  - Function: `compare_species()` (internal display)
  - Purpose: Show taxonomy paths in comparison table

**NOT used by:**
- ❌ Module 1 - Uses taxonomy for filtering, not path display
- ❌ Module 5 - Publications don't need full taxonomy paths

---

### `get_common_rank`
**Signature:** `fn get_common_rank(a: &Taxonomy, b: &Taxonomy) -> Option<TaxonomicRank>`  
**Purpose:** Get highest taxonomic rank where two taxonomies match

**Used by:**
- ✅ **Module 2 (species-relationships)**
  - File: Internal helper
  - Function: `explain_relationship()` (internal)
  - Purpose: Generate explanations like "Share the same family"

- ✅ **Module 4 (species-comparison)**
  - File: Internal helper
  - Function: Distinguishing features analysis
  - Purpose: Identify taxonomic divergence point

**NOT used by:**
- ❌ Module 1 - Binary filtering only
- ❌ Module 3 - Has internal rank operations
- ❌ Module 5 - Not relevant for publications

---

### `taxonomy_distance`
**Signature:** `fn taxonomy_distance(a: &Taxonomy, b: &Taxonomy) -> usize`  
**Purpose:** Calculate distance (0=same genus, 6=different kingdom)

**Used by:**
- ✅ **Module 2 (species-relationships)**
  - File: Internal helper
  - Function: Relationship scoring (internal)
  - Purpose: Alternative to similarity scoring (0-6 scale vs 0-1)

- ✅ **Module 3 (taxonomy)**
  - File: Internal helper
  - Function: Diversity and gap analysis
  - Purpose: Measure taxonomic diversity spread

**NOT used by:**
- ❌ Module 1 - Uses similarity, not distance
- ❌ Module 4 - Uses similarity scoring
- ❌ Module 5 - Not relevant

---

## 🔢 Collections Module Functions

### `jaccard_similarity`
**Signature:** `fn jaccard_similarity<T: Hash + Eq>(a: &HashSet<T>, b: &HashSet<T>) -> f64`  
**Purpose:** Calculate Jaccard coefficient: |A ∩ B| / |A ∪ B|

**Used by:**
- ✅ **Module 2 (species-relationships)**
  - File: `crates/species-relationships/src/lib.rs`
  - Function: `calculate_relationship()` - habitat & characteristic evidence
  - Purpose: Habitat similarity (weight 0.30) + characteristic similarity (weight 0.20)
  - Input: Habitat sets, characteristic sets

- ✅ **Module 3 (taxonomy)**
  - File: Internal helper
  - Function: Gap analysis
  - Purpose: Compare our genus sets vs reference genus sets
  - Input: Sets of genus names

- ✅ **Module 4 (species-comparison)**
  - File: `crates/species-comparison/src/lib.rs`
  - Function: `calculate_pair_similarity()` - habitat & distribution dimensions
  - Purpose: Multi-dimensional similarity scoring
  - Input: Habitat sets, kabupaten (regency) sets

**NOT used by:**
- ❌ Module 1 - Uses text matching, not set similarity
- ❌ Module 5 - Uses different similarity metrics for publications

---

### `set_intersection`
**Signature:** `fn set_intersection<T: Hash + Eq + Clone>(a: &HashSet<T>, b: &HashSet<T>) -> HashSet<T>`  
**Purpose:** Return elements present in both sets

**Used by:**
- ✅ **Module 2 (species-relationships)**
  - File: Internal helper
  - Function: Evidence collection
  - Purpose: Find shared habitat/characteristic values for explanations

- ✅ **Module 4 (species-comparison)**
  - File: `crates/species-comparison/src/lib.rs`
  - Function: Shared attributes analysis
  - Purpose: List attributes common to all compared species

**NOT used by:**
- ❌ Module 1 - No set operations needed
- ❌ Module 3 - Uses custom set logic
- ❌ Module 5 - No set operations

---

### `set_difference`
**Signature:** `fn set_difference<T: Hash + Eq + Clone>(a: &HashSet<T>, b: &HashSet<T>) -> Vec<T>`  
**Purpose:** Return elements in A but not in B

**Used by:**
- ✅ **Module 3 (taxonomy)**
  - File: `crates/taxonomy/src/lib.rs`
  - Function: `analyze_taxonomic_gap()`
  - Purpose: Find missing genera (in reference but not in our dataset)

- ✅ **Module 4 (species-comparison)**
  - File: Internal helper
  - Function: Unique attributes analysis
  - Purpose: Find characteristics unique to each species

**NOT used by:**
- ❌ Module 1 - No set difference needed
- ❌ Module 2 - Uses intersection for evidence
- ❌ Module 5 - No set operations

---

### `set_union`
**Signature:** `fn set_union<T: Hash + Eq + Clone>(a: &HashSet<T>, b: &HashSet<T>) -> HashSet<T>`  
**Purpose:** Return all elements from both sets

**Used by:**
- ✅ **Module 4 (species-comparison)**
  - File: Internal helper
  - Function: Attribute matrix construction
  - Purpose: Get all possible attributes across compared species

**NOT used by:**
- ❌ Module 1 - No set union needed
- ❌ Module 2 - Jaccard handles union internally
- ❌ Module 3 - Uses difference, not union
- ❌ Module 5 - No set operations

---

## 📈 Scoring Module Functions

### `combine_weighted_scores`
**Signature:** `fn combine_weighted_scores(scores: &[(f64, f64)]) -> f64`  
**Purpose:** Calculate sum(score_i × weight_i)

**Used by:**
- ✅ **Module 1 (species-search)**
  - File: Internal helper
  - Function: Final relevance scoring
  - Purpose: Combine name match + taxonomy match + attribute match scores
  - Example: `[(name_score, 0.5), (taxonomy_score, 0.3), (attr_score, 0.2)]`

- ✅ **Module 2 (species-relationships)**
  - File: `crates/species-relationships/src/lib.rs`
  - Function: `calculate_relationship()`
  - Purpose: Combine taxonomy + habitat + characteristic scores
  - Example: `[(0.8, 0.5), (0.6, 0.3), (0.4, 0.2)]` = 0.66

- ✅ **Module 4 (species-comparison)**
  - File: `crates/species-comparison/src/lib.rs`
  - Function: `calculate_pair_similarity()`
  - Purpose: Combine multi-dimensional similarity scores
  - Input: Taxonomy, habitat, distribution, morphology dimensions

- ✅ **Module 5 (knowledge-citations)**
  - File: Internal helper
  - Function: Publication relevance ranking
  - Purpose: Combine topic match + recency + citation count scores

**NOT used by:**
- ❌ Module 3 - Uses counts/statistics, not weighted scoring

---

### `normalize_score`
**Signature:** `fn normalize_score(score: f64, min: f64, max: f64) -> f64`  
**Purpose:** Normalize score to 0.0-1.0 range

**Used by:**
- ✅ **Module 1 (species-search)**
  - File: Internal helper
  - Function: Score normalization before ranking
  - Purpose: Normalize different scoring dimensions to comparable range

- ✅ **Module 5 (knowledge-citations)**
  - File: Internal helper
  - Function: Citation count normalization
  - Purpose: Convert citation counts to 0-1 scores for weighting

**NOT used by:**
- ❌ Module 2 - Scores already 0-1 (Jaccard, taxonomy similarity)
- ❌ Module 3 - Uses raw counts
- ❌ Module 4 - Scores already normalized

---

### `rank_by_score`
**Signature:** `fn rank_by_score<T>(items: Vec<(T, f64)>) -> Vec<(T, f64)>`  
**Purpose:** Sort items by score descending (highest first)

**Used by:**
- ✅ **Module 1 (species-search)**
  - File: `crates/species-search/src/lib.rs`
  - Function: `search()` - final ranking
  - Purpose: Return top matches by relevance

- ✅ **Module 2 (species-relationships)**
  - File: Internal helper (`rank_related`)
  - Function: Discovery ranking
  - Purpose: Order related species by relationship strength

- ✅ **Module 5 (knowledge-citations)**
  - File: Internal helper
  - Function: Publication ranking
  - Purpose: Order publications by relevance

**NOT used by:**
- ❌ Module 3 - Uses frequency sorting, not score ranking
- ❌ Module 4 - Uses custom comparison ordering

---

### `top_n`
**Signature:** `fn top_n<T>(items: Vec<(T, f64)>, n: usize) -> Vec<(T, f64)>`  
**Purpose:** Get top N items by score

**Used by:**
- ✅ **Module 1 (species-search)**
  - File: Internal helper
  - Function: Limit search results
  - Purpose: Return only top N matches (default 10)

- ✅ **Module 2 (species-relationships)**
  - File: Internal helper
  - Function: Limit related species
  - Purpose: Return top N related species (max 50)

- ✅ **Module 5 (knowledge-citations)**
  - File: `crates/knowledge-citations/src/lib.rs`
  - Function: `recommend_citations()` with limit parameter
  - Purpose: Limit recommendation count

**NOT used by:**
- ❌ Module 3 - Returns full reports, no top-N needed
- ❌ Module 4 - Compares specific species list (no limiting)

---

### `filter_by_threshold`
**Signature:** `fn filter_by_threshold<T>(items: Vec<(T, f64)>, min_score: f64) -> Vec<(T, f64)>`  
**Purpose:** Keep items with score >= threshold

**Used by:**
- ✅ **Module 1 (species-search)**
  - File: Internal helper
  - Function: Filter weak matches
  - Purpose: Remove results below relevance threshold

- ✅ **Module 2 (species-relationships)**
  - File: Internal helper (`rank_related`)
  - Function: Filter by min_score parameter
  - Purpose: Only show relationships above minimum strength

**NOT used by:**
- ❌ Module 3 - No threshold filtering
- ❌ Module 4 - Shows all comparison results
- ❌ Module 5 - No score threshold filtering

---

## 📝 Text Module Functions

### `normalize_text`
**Signature:** `fn normalize_text(input: &str) -> String`  
**Purpose:** Lowercase, trim, remove extra whitespace

**Used by:**
- ✅ **Module 1 (species-search)**
  - File: `crates/species-search/src/lib.rs`
  - Function: `search()` - query preprocessing
  - Purpose: Normalize user query before matching
  - Example: "  Orangutan   Kalimantan " → "orangutan kalimantan"

- ✅ **Module 2 (species-relationships)**
  - File: Internal helper
  - Function: Data preparation (`normalize_species`)
  - Purpose: Normalize habitat/characteristic labels for comparison

- ✅ **Module 5 (knowledge-citations)**
  - File: Internal helper
  - Function: Publication text normalization
  - Purpose: Normalize titles, topics for matching

**NOT used by:**
- ❌ Module 3 - Uses taxonomy labels as-is
- ❌ Module 4 - Uses normalized data from shared

---

### `sanitize_label`
**Signature:** `fn sanitize_label(input: &str) -> String`  
**Purpose:** Trim and remove extra spaces

**Used by:**
- ✅ **Module 2 (species-relationships)**
  - File: Internal helper
  - Function: Label preparation
  - Purpose: Clean habitat/characteristic labels

- ✅ **Module 3 (taxonomy)**
  - File: Internal helper
  - Function: Taxonomy label sanitization
  - Purpose: Clean genus/family names for comparison

**NOT used by:**
- ❌ Module 1 - Uses full normalize_text
- ❌ Module 4 - Uses full normalize_text
- ❌ Module 5 - Uses full normalize_text

---

### `tokenize`
**Signature:** `fn tokenize(text: &str) -> Vec<String>`  
**Purpose:** Split text by whitespace

**Used by:**
- ✅ **Module 1 (species-search)**
  - File: Internal helper
  - Function: Query parsing
  - Purpose: Extract search terms from query
  - Example: "orangutan kalimantan" → ["orangutan", "kalimantan"]

- ✅ **Module 5 (knowledge-citations)**
  - File: Internal helper
  - Function: Topic/keyword extraction
  - Purpose: Parse publication topics for matching

**NOT used by:**
- ❌ Module 2 - Works with structured attributes
- ❌ Module 3 - Works with structured taxonomy
- ❌ Module 4 - Works with structured data

---

## ✅ Validation Module Functions

### `validate_id_list`
**Signature:** `fn validate_id_list(ids: &[u64], min_count: usize, max_count: usize) -> Result<(), ValidationError>`  
**Purpose:** Check ID count and duplicates

**Used by:**
- ✅ **Module 2 (species-relationships)**
  - File: Internal helper (`validate_query`)
  - Function: Query validation
  - Purpose: Validate center species ID exists
  - Parameters: `min_count=1, max_count=1`

- ✅ **Module 4 (species-comparison)**
  - File: `crates/species-comparison/src/lib.rs`
  - Function: Query validation
  - Purpose: Validate species ID list for comparison
  - Parameters: `min_count=2, max_count=10` (must compare at least 2)

- ✅ **Module 5 (knowledge-citations)**
  - File: Internal helper
  - Function: Publication ID validation
  - Purpose: Validate publication IDs for export
  - Parameters: `min_count=1, max_count=100`

**NOT used by:**
- ❌ Module 1 - Search doesn't use ID lists
- ❌ Module 3 - Doesn't use ID lists

---

### `validate_score_range`
**Signature:** `fn validate_score_range(score: f64, min: f64, max: f64) -> Result<(), ValidationError>`  
**Purpose:** Check score is finite and in range

**Used by:**
- ✅ **Module 2 (species-relationships)**
  - File: Internal helper (`validate_query`)
  - Function: Validate min_score parameter
  - Purpose: Ensure threshold is 0.0-1.0
  - Parameters: `min=0.0, max=1.0`

- ✅ **Module 4 (species-comparison)**
  - File: Internal helper
  - Function: Weight validation
  - Purpose: Validate individual weight values
  - Parameters: `min=0.0, max=1.0`

**NOT used by:**
- ❌ Module 1 - No explicit score thresholds
- ❌ Module 3 - No score parameters
- ❌ Module 5 - No score thresholds

---

### `validate_weights_sum_to_one`
**Signature:** `fn validate_weights_sum_to_one(weights: &[f64], tolerance: f64) -> Result<(), ValidationError>`  
**Purpose:** Check weights are finite, non-negative, sum to 1.0

**Used by:**
- ✅ **Module 2 (species-relationships)**
  - File: `crates/species-relationships/src/lib.rs`
  - Function: `calculate_relationship()` - weight validation
  - Purpose: Validate ScoreWeights before scoring
  - Input: `[taxonomy, habitat, characteristic]` must sum to 1.0
  - Tolerance: `1e-9`

- ✅ **Module 4 (species-comparison)**
  - File: `crates/species-comparison/src/lib.rs`
  - Function: `calculate_pair_similarity()` - weight validation
  - Purpose: Validate ScoringWeights before comparison
  - Input: Multi-dimensional weights must sum to 1.0
  - Tolerance: `1e-9`

**NOT used by:**
- ❌ Module 1 - Uses fixed internal weights
- ❌ Module 3 - No weighted scoring
- ❌ Module 5 - Uses different validation

---

### `has_duplicates`
**Signature:** `fn has_duplicates<T: Hash + Eq>(items: &[T]) -> bool`  
**Purpose:** Check if slice contains duplicate elements

**Used by:**
- ✅ **Module 2 (species-relationships)**
  - File: Internal helper (`prepare_species`)
  - Function: Dataset validation
  - Purpose: Ensure no duplicate species IDs
  - Returns error if duplicates found

- ✅ **Module 4 (species-comparison)**
  - File: Internal helper
  - Function: Query validation
  - Purpose: Ensure user didn't provide duplicate species IDs
  - Returns error if duplicates found

**NOT used by:**
- ❌ Module 1 - No duplicate checking needed
- ❌ Module 3 - No duplicate checking needed
- ❌ Module 5 - Uses validate_id_list which checks internally

---

## 📊 Stats Module Functions

### `count_by_key`
**Signature:** `fn count_by_key<T, K: Hash + Eq>(items: &[T], key_fn: impl Fn(&T) -> K) -> HashMap<K, usize>`  
**Purpose:** Count occurrences of each key

**Used by:**
- ✅ **Module 3 (taxonomy)**
  - File: `crates/taxonomy/src/lib.rs`
  - Function: `calculate_diversity()` and `generate_taxonomy_report()`
  - Purpose: Count species per family, genus, etc.
  - Example: `count_by_key(&species, |s| s.taxonomy.family.clone())`

- ✅ **Module 5 (knowledge-citations)**
  - File: Internal helper
  - Function: Research coverage analysis
  - Purpose: Count publications per topic, location, species
  - Example: `count_by_key(&pubs, |p| p.topic.clone())`

**NOT used by:**
- ❌ Module 1 - No counting needed
- ❌ Module 2 - No counting needed
- ❌ Module 4 - No counting needed

---

### `frequency_distribution`
**Signature:** `fn frequency_distribution<T, K: Hash + Eq + Clone>(items: &[T], key_fn: impl Fn(&T) -> K) -> Vec<(K, usize)>`  
**Purpose:** Get counts sorted descending

**Used by:**
- ✅ **Module 3 (taxonomy)**
  - File: Internal helper
  - Function: Diversity reporting
  - Purpose: Show most common families/genera
  - Example: Top 10 most diverse families

- ✅ **Module 5 (knowledge-citations)**
  - File: Internal helper
  - Function: Research topic ranking
  - Purpose: Show most studied topics/locations
  - Example: Top research topics by publication count

**NOT used by:**
- ❌ Module 1 - No frequency analysis
- ❌ Module 2 - No frequency analysis
- ❌ Module 4 - No frequency analysis

---

### `calculate_coverage_percentage`
**Signature:** `fn calculate_coverage_percentage(available: usize, total: usize) -> f64`  
**Purpose:** Calculate ratio (0.0-1.0)

**Used by:**
- ✅ **Module 3 (taxonomy)**
  - File: `crates/taxonomy/src/lib.rs`
  - Function: `analyze_taxonomic_gap()`
  - Purpose: Calculate percentage of reference genera covered
  - Example: 80/100 genera = 0.80 coverage

- ✅ **Module 5 (knowledge-citations)**
  - File: Internal helper
  - Function: Research coverage analysis
  - Purpose: Calculate percentage of species with publications
  - Example: 50/200 species documented = 0.25 coverage

**NOT used by:**
- ❌ Module 1 - No coverage calculation
- ❌ Module 2 - Uses custom coverage calculation for evidence
- ❌ Module 4 - No coverage calculation

---

### `build_timeline`
**Signature:** `fn build_timeline<T>(items: &[T], year_fn: impl Fn(&T) -> u16) -> BTreeMap<u16, usize>`  
**Purpose:** Count items per year, ordered

**Used by:**
- ✅ **Module 5 (knowledge-citations)**
  - File: `crates/knowledge-citations/src/lib.rs`
  - Function: `explore_knowledge()` - research timeline
  - Purpose: Show publication count by year
  - Example: `{2020: 5, 2021: 8, 2022: 12, ...}`

**NOT used by:**
- ❌ Module 1 - No temporal analysis
- ❌ Module 2 - No temporal analysis
- ❌ Module 3 - No temporal analysis
- ❌ Module 4 - No temporal analysis

---

## 💾 Database Module Functions

### `create_pool`
**Signature:** `async fn create_pool(database_url: &str) -> Result<Pool<Postgres>, sqlx::Error>`  
**Purpose:** Create PostgreSQL connection pool

**Used by:**
- ✅ **API Server**
  - File: `crates/api-server/src/main.rs`
  - Function: `main()` initialization
  - Purpose: Single shared connection pool for all routes
  - Called once at startup

**NOT used by modules directly** - modules receive data through function parameters, not direct DB access

---

### `fetch_all_species`
**Signature:** `async fn fetch_all_species(pool: &Pool<Postgres>) -> Result<Vec<Species>, sqlx::Error>`  
**Purpose:** Fetch entire species dataset

**Used by:**
- ✅ **API Server**
  - File: Route handlers
  - All endpoints: Fetch data to pass to module functions
  - Example: Search route fetches all species, passes to `search()`

**NOT used by modules directly** - modules work with data passed as parameters

---

### `fetch_species_by_id`
**Signature:** `async fn fetch_species_by_id(pool: &Pool<Postgres>, id: u64) -> Result<Option<Species>, sqlx::Error>`  
**Purpose:** Fetch single species

**Used by:**
- ✅ **API Server**
  - File: Route handlers
  - Module 2 routes: Fetch center species for relationship exploration
  - Module 4 routes: Fetch specific species for comparison
  - Detail routes: Individual species pages

**NOT used by modules directly**

---

### `fetch_observations_for_species`
**Signature:** `async fn fetch_observations_for_species(pool: &Pool<Postgres>, species_id: u64) -> Result<Vec<Observation>, sqlx::Error>`  
**Purpose:** Fetch observations for a species

**Used by:**
- ✅ **API Server**
  - File: Route handlers
  - Module 4 routes: Get observations for distribution comparison
  - Observation detail routes

**NOT used by modules directly**

---

## 📊 Usage Summary by Module

### Module 1: Intelligent Species Search
**Shared functions used: 8**
- Core: `Species`, `Taxonomy`
- Taxonomy: `calculate_taxonomy_similarity` (optional scoring boost)
- Scoring: `combine_weighted_scores`, `normalize_score`, `rank_by_score`, `top_n`, `filter_by_threshold`
- Text: `normalize_text`, `tokenize`

### Module 2: Species Relationship Explorer
**Shared functions used: 13**
- Core: `Species`, `Taxonomy`, `TaxonomicRank`
- Taxonomy: `calculate_taxonomy_similarity`, `build_taxonomy_path`, `get_common_rank`, `taxonomy_distance`
- Collections: `jaccard_similarity`, `set_intersection`
- Scoring: `combine_weighted_scores`, `rank_by_score`, `filter_by_threshold`
- Text: `normalize_text`, `sanitize_label`
- Validation: `validate_score_range`, `validate_weights_sum_to_one`, `has_duplicates`

### Module 3: Taxonomy & Classification Explorer
**Shared functions used: 9**
- Core: `Species`, `Taxonomy`, `TaxonomicRank`
- Taxonomy: `build_taxonomy_path`, `taxonomy_distance`
- Collections: `jaccard_similarity`, `set_difference`
- Text: `sanitize_label`
- Stats: `count_by_key`, `frequency_distribution`, `calculate_coverage_percentage`

### Module 4: Comparative Species Explorer
**Shared functions used: 12**
- Core: `Species`, `Taxonomy`, `TaxonomicRank`, `Observation`
- Taxonomy: `calculate_taxonomy_similarity`, `build_taxonomy_path`, `get_common_rank`
- Collections: `jaccard_similarity`, `set_intersection`, `set_difference`, `set_union`
- Scoring: `combine_weighted_scores`
- Validation: `validate_id_list`, `validate_weights_sum_to_one`, `has_duplicates`

### Module 5: Biodiversity Knowledge & Citation Explorer
**Shared functions used: 10**
- Core: `Species`, `Taxonomy`
- Scoring: `combine_weighted_scores`, `normalize_score`, `rank_by_score`, `top_n`
- Text: `normalize_text`, `tokenize`
- Validation: `validate_id_list`
- Stats: `count_by_key`, `frequency_distribution`, `calculate_coverage_percentage`, `build_timeline`

### API Server
**Shared functions used: 4 (DB only)**
- Database: `create_pool`, `fetch_all_species`, `fetch_species_by_id`, `fetch_observations_for_species`
- Purpose: Data retrieval to pass to modules

---

## 📈 Function Usage Statistics

| Function | M1 | M2 | M3 | M4 | M5 | API | Total |
|----------|----|----|----|----|----|----|-------|
| `Species` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | 6 |
| `Taxonomy` | ✅ | ✅ | ✅ | ✅ | ✅ | - | 5 |
| `calculate_taxonomy_similarity` | ✅ | ✅ | - | ✅ | - | - | 3 |
| `jaccard_similarity` | - | ✅ | ✅ | ✅ | - | - | 3 |
| `combine_weighted_scores` | ✅ | ✅ | - | ✅ | ✅ | - | 4 |
| `normalize_text` | ✅ | ✅ | - | - | ✅ | - | 3 |
| `rank_by_score` | ✅ | ✅ | - | - | ✅ | - | 3 |
| `build_taxonomy_path` | - | ✅ | ✅ | ✅ | - | - | 3 |
| `validate_weights_sum_to_one` | - | ✅ | - | ✅ | - | - | 2 |
| `tokenize` | ✅ | - | - | - | ✅ | - | 2 |
| `set_intersection` | - | ✅ | - | ✅ | - | - | 2 |
| `top_n` | ✅ | ✅ | - | - | ✅ | - | 3 |
| `count_by_key` | - | - | ✅ | - | ✅ | - | 2 |
| `validate_id_list` | - | ✅ | - | ✅ | ✅ | - | 3 |
| `has_duplicates` | - | ✅ | - | ✅ | - | - | 2 |

**Most used functions:**
1. `Species` type - Used by all 6 crates
2. `Taxonomy` type - Used by 5 crates
3. `combine_weighted_scores` - Used by 4 modules
4. `calculate_taxonomy_similarity`, `jaccard_similarity`, `rank_by_score`, `validate_id_list`, `top_n` - Used by 3 modules each

---

## 🎯 Verification Checklist

For each module implementation, verify:

- [ ] All specified shared functions are imported
- [ ] No shared function is duplicated/reimplemented
- [ ] Function usage matches this document
- [ ] Proper error handling for Result types
- [ ] No direct database access (use API server layer)

---

**Document Version:** 1.0  
**Last Updated:** 2026-09-20  
**Status:** Complete for Progress 1
