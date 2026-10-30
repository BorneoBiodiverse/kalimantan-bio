//! Input validation helpers.

use crate::error::ValidationError;
use std::{collections::HashSet, hash::Hash};

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
/// # Used By
///
/// - **Module 2 (species-relationships)**: Validate center species ID exists (min=1, max=1)
/// - **Module 4 (species-comparison)**: Validate species ID list for comparison (min=2, max=10)
/// - **Module 5 (knowledge-citations)**: Validate publication IDs for export (min=1, max=100)
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
    if ids.len() < min_count {
        return Err(ValidationError::TooFewItems {
            found: ids.len(),
            minimum: min_count,
        });
    }

    if ids.len() > max_count {
        return Err(ValidationError::TooManyItems {
            found: ids.len(),
            maximum: max_count,
        });
    }

    if has_duplicates(ids) {
        return Err(ValidationError::DuplicateIds);
    }

    Ok(())
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
/// # Used By
///
/// - **Module 2 (species-relationships)**: Validate min_score parameter (0.0-1.0)
/// - **Module 4 (species-comparison)**: Validate individual weight values (0.0-1.0)
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
/// # Used By
///
/// - **Module 2 (species-relationships)**: Validate ScoreWeights before scoring (taxonomy + habitat + characteristic must sum to 1.0, tolerance 1e-9)
/// - **Module 4 (species-comparison)**: Validate ScoringWeights before comparison (multi-dimensional weights must sum to 1.0, tolerance 1e-9)
///
/// # Example
///
/// ```rust
/// use kalimantanbio_shared::validation::validate_weights_sum_to_one;
/// use kalimantanbio_shared::ValidationError;
///
/// validate_weights_sum_to_one(&[0.5, 0.3, 0.2], 1e-9).unwrap();
///
/// let error = validate_weights_sum_to_one(&[0.5, 0.4], 1e-9).unwrap_err();
/// assert!(matches!(error, ValidationError::WeightsSumMismatch { .. }));
/// ```
pub fn validate_weights_sum_to_one(weights: &[f64], tolerance: f64) -> Result<(), ValidationError> {
    for weight in weights {
        if !weight.is_finite() {
            return Err(ValidationError::NonFiniteWeight);
        }
        if *weight < 0.0 {
            return Err(ValidationError::NegativeWeight);
        }
    }

    let sum: f64 = weights.iter().sum();
    if (sum - 1.0).abs() > tolerance {
        return Err(ValidationError::WeightsSumMismatch { sum, expected: 1.0 });
    }

    Ok(())
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
/// # Used By
///
/// - **Module 2 (species-relationships)**: Ensure no duplicate species IDs in dataset
/// - **Module 4 (species-comparison)**: Ensure user didn't provide duplicate species IDs
///
/// # Example
///
/// ```rust,ignore
/// assert!(has_duplicates(&[1, 2, 2]));
/// ```
pub fn has_duplicates<T: Hash + Eq>(items: &[T]) -> bool {
    let mut set = HashSet::with_capacity(items.len());
    for item in items {
        if !set.insert(item) {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_weights_pass() {
        assert!(validate_weights_sum_to_one(&[0.5, 0.3, 0.2], 1e-9).is_ok());
    }

    #[test]
    fn negative_weight_fails() {
        assert!(matches!(
            validate_weights_sum_to_one(&[1.1, -0.1], 1e-9),
            Err(ValidationError::NegativeWeight)
        ));
    }

    #[test]
    fn non_finite_weight_fails() {
        assert!(matches!(
            validate_weights_sum_to_one(&[f64::NAN, 0.5, 0.5], 1e-9),
            Err(ValidationError::NonFiniteWeight)
        ));
        assert!(matches!(
            validate_weights_sum_to_one(&[f64::INFINITY, 0.0], 1e-9),
            Err(ValidationError::NonFiniteWeight)
        ));
    }

    #[test]
    fn sum_mismatch_fails() {
        let result = validate_weights_sum_to_one(&[0.5, 0.4, 0.2], 1e-9);
        assert!(matches!(
            result,
            Err(ValidationError::WeightsSumMismatch { .. })
        ));
    }

    #[test]
    fn tolerance_boundary_is_inclusive() {
        assert!(validate_weights_sum_to_one(&[0.5, 0.5], 0.0).is_ok());
        assert!(validate_weights_sum_to_one(&[0.5, 0.5000001], 1e-9).is_err());
    }

    #[test]
    fn empty_weights_fail_sum_check() {
        assert!(matches!(
            validate_weights_sum_to_one(&[], 1e-9),
            Err(ValidationError::WeightsSumMismatch { .. })
        ));
    }
}
