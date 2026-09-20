//! Database access layer: connection pooling and species queries.

use sqlx::{Pool, Postgres};

use crate::core::{Observation, Species};

/// Create a database connection pool.
///
/// # Arguments
///
/// * `database_url` - PostgreSQL connection string (e.g. from `DATABASE_URL`).
///
/// # Returns
///
/// A ready [`Pool<Postgres>`], or a [`sqlx::Error`].
///
/// # Used By
///
/// - **API Server**: Single shared connection pool for all routes (called once at startup)
///
/// # Example
///
/// ```rust,ignore
/// let pool = create_pool("postgresql://user:pass@host/db").await?;
/// ```
pub async fn create_pool(database_url: &str) -> Result<Pool<Postgres>, sqlx::Error> {
    todo!("shared db phase 3: create_pool")
}

/// Fetch all species from the database.
///
/// # Arguments
///
/// * `pool` - The shared connection pool.
///
/// # Returns
///
/// The full list of species, or a [`sqlx::Error`].
///
/// # Used By
///
/// - **API Server**: All route handlers fetch data to pass to module functions
///   - Search route: Fetches all species, passes to `search()`
///   - Relationship route: Fetches dataset for relationship exploration
///   - Taxonomy route: Fetches species for diversity analysis
///   - Comparison route: Fetches species for comparison
///
/// # Example
///
/// ```rust,ignore
/// let all = fetch_all_species(&pool).await?;
/// ```
pub async fn fetch_all_species(pool: &Pool<Postgres>) -> Result<Vec<Species>, sqlx::Error> {
    todo!("shared db phase 3: fetch_all_species")
}

/// Fetch a single species by identifier.
///
/// # Arguments
///
/// * `pool` - The shared connection pool.
/// * `id` - The species identifier.
///
/// # Returns
///
/// The species when found, `None` otherwise, or a [`sqlx::Error`].
///
/// # Used By
///
/// - **API Server**: Route handlers for specific species operations
///   - Module 2 routes: Fetch center species for relationship exploration
///   - Module 4 routes: Fetch specific species for comparison
///   - Detail routes: Individual species pages
///
/// # Example
///
/// ```rust,ignore
/// let species = fetch_species_by_id(&pool, 42).await?;
/// ```
pub async fn fetch_species_by_id(
    pool: &Pool<Postgres>,
    id: u64,
) -> Result<Option<Species>, sqlx::Error> {
    todo!("shared db phase 3: fetch_species_by_id")
}

/// Fetch observations for a species.
///
/// # Arguments
///
/// * `pool` - The shared connection pool.
/// * `species_id` - The species identifier.
///
/// # Returns
///
/// The list of observations, or a [`sqlx::Error`].
///
/// # Used By
///
/// - **API Server**: Route handlers needing observation data
///   - Module 4 routes: Get observations for distribution comparison
///   - Observation detail routes: Show species observation history
///
/// # Example
///
/// ```rust,ignore
/// let observations = fetch_observations_for_species(&pool, 42).await?;
/// ```
pub async fn fetch_observations_for_species(
    pool: &Pool<Postgres>,
    species_id: u64,
) -> Result<Vec<Observation>, sqlx::Error> {
    todo!("shared db phase 3: fetch_observations_for_species")
}