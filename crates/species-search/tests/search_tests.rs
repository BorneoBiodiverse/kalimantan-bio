//! Integration tests for Module 1 (Intelligent Species Search).
//!
//! Uses realistic Borneo biodiversity fixtures from `crates/shared/fixtures/species.json`.

use kalimantanbio_shared::core::Species;

/// Helper function to load test fixtures from shared library.
fn load_species_fixtures() -> Vec<Species> {
    let fixture_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../shared/fixtures/species.json"
    );
    let json_data = std::fs::read_to_string(fixture_path)
        .unwrap_or_else(|e| panic!("Gagal membaca fixture data species.json: {e}"));
    
    serde_json::from_str(&json_data)
        .unwrap_or_else(|e| panic!("Gagal parsing data JSON: {e}"))
}

#[test]
#[ignore = "Menunggu implementasi logika dari tim"]
fn test_search_by_common_name_orangutan() {
    let all_species = load_species_fixtures();
    let results = species_search::search("orangutan", &all_species);

    assert!(!results.is_empty(), "Pencarian 'orangutan' harus mengembalikan hasil");
    
    let (top_species, score) = &results[0];
    assert_eq!(top_species.common_name, "Orangutan Kalimantan");
    assert!(*score > 0.0, "Skor relevansi harus lebih besar dari 0");
}

#[test]
#[ignore = "Menunggu implementasi logika dari tim"]
fn test_search_by_iucn_conservation_status() {
    let all_species = load_species_fixtures();
    let results = species_search::search("iucn:CR", &all_species);

    assert!(!results.is_empty(), "Harus ada spesies dengan status CR");
    for (species, _) in results {
        assert_eq!(species.iucn, "CR", "Semua hasil harus berstatus CR");
    }
}

#[test]
#[ignore = "Menunggu implementasi logika dari tim"]
fn test_search_unknown_keyword_returns_empty() {
    let all_species = load_species_fixtures();
    let results = species_search::search("hewan_tidak_ada_xyz_123", &all_species);

    assert!(results.is_empty(), "Kata kunci tidak dikenal harus mengembalikan hasil kosong");
}

#[test]
#[ignore = "Menunggu implementasi logika dari tim"]
fn test_recommend_related_queries() {
    let all_species = load_species_fixtures();
    let suggestions = species_search::recommend_related_queries("orangutan", &all_species);

    assert!(!suggestions.is_empty(), "Harus menghasilkan minimal satu saran query");
}
