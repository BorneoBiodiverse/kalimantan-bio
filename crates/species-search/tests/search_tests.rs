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
fn test_structured_queries_return_positive_scores() {
    let all_species = load_species_fixtures();

    for query in [
        "mamalia",
        "tumbuhan",
        "endemik",
        "terverifikasi",
        "cites:appendix-i",
        "mamalia kritis dilindungi",
    ] {
        let results = species_search::search(query, &all_species);
        assert!(!results.is_empty(), "query '{query}' harus menghasilkan");
        for (species, score) in &results {
            assert!(
                *score > 0.0 && *score <= 1.0,
                "query '{query}': skor {} tidak valid untuk {}",
                score,
                species.scientific_name
            );
        }
    }
}

#[test]
fn test_scores_are_sorted_descending() {
    let all_species = load_species_fixtures();

    for query in ["orangutan", "ape", "borneo", "mamalia kritis"] {
        let results = species_search::search(query, &all_species);
        assert!(
            results.windows(2).all(|pair| pair[0].1 >= pair[1].1),
            "query '{query}' harus terurut menurun"
        );
    }
}

#[test]
fn test_punctuation_does_not_change_results() {
    let all_species = load_species_fixtures();

    let plain: Vec<_> = species_search::search("mamalia kritis", &all_species)
        .into_iter()
        .map(|(s, _)| s.scientific_name.clone())
        .collect();
    let punctuated: Vec<_> = species_search::search("mamalia, kritis!", &all_species)
        .into_iter()
        .map(|(s, _)| s.scientific_name.clone())
        .collect();

    assert_eq!(plain, punctuated);
    assert_eq!(plain.len(), 2, "hanya mamalia berstatus CR");
}

#[test]
fn test_filter_by_kingdom_and_species_type() {
    let all_species = load_species_fixtures();

    let plants = species_search::search("tumbuhan", &all_species);
    assert_eq!(plants.len(), 1);
    assert_eq!(plants[0].0.scientific_name, "Rafflesia arnoldii");

    let endemic = species_search::search("endemik", &all_species);
    assert_eq!(endemic.len(), 2);
    assert!(endemic.iter().all(|(s, _)| s.species_type == "Endemic"));
}

#[test]
fn test_filter_by_cites_and_protection() {
    let all_species = load_species_fixtures();

    let cites_i = species_search::search("cites:appendix-i", &all_species);
    assert_eq!(cites_i.len(), 4);
    assert!(cites_i.iter().all(|(s, _)| s.cites == "Appendix I"));

    let protected = species_search::search("p106:dilindungi", &all_species);
    assert_eq!(protected.len(), 5);

    let unprotected = species_search::search("tidak dilindungi", &all_species);
    assert!(unprotected.is_empty());
}

#[test]
fn test_filter_by_verification_flag() {
    let all_species = load_species_fixtures();

    assert_eq!(species_search::search("terverifikasi", &all_species).len(), 5);
    assert!(species_search::search("verified:false", &all_species).is_empty());
}

#[test]
fn test_search_by_common_name_orangutan() {
    let all_species = load_species_fixtures();
    let results = species_search::search("orangutan", &all_species);

    assert!(!results.is_empty(), "Pencarian 'orangutan' harus mengembalikan hasil");
    
    let (top_species, score) = &results[0];
    assert_eq!(top_species.common_name, "Orangutan Kalimantan");
    assert!(*score > 0.0, "Skor relevansi harus lebih besar dari 0");
}

#[test]
fn test_search_by_iucn_conservation_status() {
    let all_species = load_species_fixtures();
    let results = species_search::search("iucn:CR", &all_species);

    assert!(!results.is_empty(), "Harus ada spesies dengan status CR");
    for (species, _) in results {
        assert_eq!(species.iucn, "CR", "Semua hasil harus berstatus CR");
    }
}

#[test]
fn test_search_unknown_keyword_returns_empty() {
    let all_species = load_species_fixtures();
    let results = species_search::search("hewan_tidak_ada_xyz_123", &all_species);

    assert!(results.is_empty(), "Kata kunci tidak dikenal harus mengembalikan hasil kosong");
}

#[test]
fn test_recommend_related_queries() {
    let all_species = load_species_fixtures();
    let suggestions = species_search::recommend_related_queries("orangutan", &all_species);

    assert!(!suggestions.is_empty(), "Harus menghasilkan minimal satu saran query");
}

#[test]
fn test_normal_matching_query_completes_without_panic() {
    let all_species = load_species_fixtures();
    
    // Uji alur pencarian normal end-to-end (parse -> filter -> score -> rank)
    let results = species_search::search("orangutan kalimantan", &all_species);

    assert!(
        !results.is_empty(),
        "Pencarian query normal harus berhasil menemukan spesies"
    );

    // Buktikan setiap hasil memiliki skor valid dalam rentang (0.0, 1.0]
    for (species, score) in &results {
        assert!(
            *score > 0.0 && *score <= 1.0,
            "Skor relevansi untuk {} harus valid, didapat: {}",
            species.common_name,
            score
        );
    }

    // Pastikan terurut menurun (descending) sesuai skor tertinggi
    assert!(
        results.windows(2).all(|pair| pair[0].1 >= pair[1].1),
        "Hasil harus terurut descending berdasarkan skor relevansi"
    );
}

