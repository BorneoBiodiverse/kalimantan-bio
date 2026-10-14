use kalimantanbio_shared::validation::validate_id_list;
use kalimantanbio_shared::ValidationError;

#[test]
fn a_single_id_is_too_few_for_a_comparison() {
    let err = validate_id_list(&[1], 2, 10).unwrap_err();

    println!("Debug: {}", err);

    assert!(
        matches!(
            err,
            ValidationError::TooFewItems {
                found: 1,
                minimum: 2
            }
        ),
        "expected TooFewItems, got {err}"
    );
}

#[test]
fn more_ids_than_the_maximum_are_rejected() {
    let ids = [1, 2, 3, 4, 5, 6];
    let err = validate_id_list(&ids, 2, 5).unwrap_err();

    assert!(
        matches!(
            err,
            ValidationError::TooManyItems {
                found: 6,
                maximum: 5
            }
        ),
        "expected TooManyItems, got {err}"
    );
}

#[test]
fn duplicate_ids_are_rejected() {
    // Two ids satisfies the minimum count, so only the duplicate rule can fire.
    let err = validate_id_list(&[1, 1], 2, 10).unwrap_err();

    assert!(
        matches!(err, ValidationError::DuplicateIds),
        "expected DuplicateIds, got {err}"
    );
}

#[test]
fn a_valid_pair_passes() {
    assert!(validate_id_list(&[1, 2], 2, 10).is_ok());
}

#[test]
fn the_boundaries_are_inclusive() {
    assert!(
        validate_id_list(&[1, 2], 2, 10).is_ok(),
        "exactly the minimum must pass"
    );

    let max_ids: Vec<u64> = (1..=10).collect();
    assert!(
        validate_id_list(&max_ids, 2, 10).is_ok(),
        "exactly the maximum must pass"
    );
}
