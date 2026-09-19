//! Input validation helpers.

use std::hash::Hash;

use crate::error::ValidationError;

/// Validate an ID list: min/max count and no duplicates.
///
/// # Arguments
///
/// * `ids` - The list of identifiers.
/// * `min_count` - Minimum number of identifiers required.
/// * `max_count` - Maximum number of identifiers allowed.
///
/// # Returns
///
/// `Ok(())` when valid, otherwise a [`ValidationError`].
///
/// # Example
///
/// ```rust,ignore
/// validate_id_list(&[1, 2, 3], 2, 5)?;
/// ```
pub fn validate_id_list(
    ids: &[u64],
    min_count: usize,
    max_count: usize,
) -> Result<(), ValidationError> {
    todo!("shared validation phase 3: validate_id_list")
}

/// Validate that a score is finite and within `[min, max]`.
///
/// # Arguments
///
/// * `score` - The score to validate.
/// * `min` - Lower bound (inclusive).
/// * `max` - Upper bound (inclusive).
///
/// # Returns
///
/// `Ok(())` when valid, otherwise a [`ValidationError`].
///
/// # Example
///
/// ```rust,ignore
/// validate_score_range(0.5, 0.0, 1.0)?;
/// ```
pub fn validate_score_range(score: f64, min: f64, max: f64) -> Result<(), ValidationError> {
    todo!("shared validation phase 3: validate_score_range")
}

/// Validate weights: all finite, non-negative, sum to 1.0 within tolerance.
///
/// # Arguments
///
/// * `weights` - The weight values.
/// * `tolerance` - Allowed deviation from an exact sum of 1.0.
///
/// # Returns
///
/// `Ok(())` when valid, otherwise a [`ValidationError`].
///
/// # Example
///
/// ```rust,ignore
/// validate_weights_sum_to_one(&[0.6, 0.4], 1e-9)?;
/// ```
pub fn validate_weights_sum_to_one(
    weights: &[f64],
    tolerance: f64,
) -> Result<(), ValidationError> {
    todo!("shared validation phase 3: validate_weights_sum_to_one")
}

/// Check whether a slice contains duplicate elements.
///
/// # Arguments
///
/// * `items` - The values to inspect.
///
/// # Returns
///
/// `true` when at least one duplicate is present.
///
/// # Example
///
/// ```rust,ignore
/// assert!(has_duplicates(&[1, 2, 2]));
/// ```
pub fn has_duplicates<T: Hash + Eq>(items: &[T]) -> bool {
    todo!("shared validation phase 3: has_duplicates")
}