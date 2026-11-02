//! Tahap 3 — Discovery, Ranking, dan Penjelasan Hubungan.
//!
//! Modul ini menyediakan fungsi-fungsi murni (pure functions) untuk:
//! - Menyaring spesies selain pusat (`candidate_species`).
//! - Menghasilkan penjelasan deterministik berbasis template untuk tiap dimensi bukti (`explain_relationship`).
//! - Menghitung skor dan menyusun penjelasan untuk seluruh kandidat (`score_candidates`).
//! - Menyaring, mengurutkan (ranking), dan membatasi rekomendasi spesies terkait (`rank_related`).

#![allow(dead_code)]

use kalimantanbio_shared::core::Species;

use crate::stage2::{collect_evidence, score_evidence};
use crate::types::{
    AttributeEvidence, AttributeKind, RelatedSpecies, RelationshipQuery, RelationshipScore,
    ScoreWeights, SpeciesId,
};

/// Toleransi komparasi floating-point untuk ambang skor dan kesetaraan ranking.
const FLOAT_TOLERANCE: f64 = 1e-9;

/// Mengambil semua kandidat spesies dari dataset selain spesies pusat.
///
/// Mengevaluasi seluruh kandidat untuk mempermudah verifikasi dan determinisme.
pub(crate) fn candidate_species<'a>(
    all: &'a [Species],
    center_id: &SpeciesId,
) -> Vec<&'a Species> {
    all.iter().filter(|s| s.id != *center_id).collect()
}

/// Menghasilkan penjelasan relasi berbasis template dari nilai atribut bersama,
/// kontribusi berbobot, dan keterbatasan data atribut.
///
/// Penjelasan dihasilkan deterministik dalam urutan kanonik:
/// taksonomi, habitat, lalu karakteristik.
pub(crate) fn explain_relationship(
    score: &RelationshipScore,
    weights: &ScoreWeights,
) -> Vec<String> {
    score
        .evidence
        .iter()
        .map(|ev| format_dimension_explanation(ev, weights))
        .collect()
}

/// Memformat string penjelasan untuk satu dimensi bukti.
fn format_dimension_explanation(evidence: &AttributeEvidence, weights: &ScoreWeights) -> String {
    match evidence.attribute {
        AttributeKind::Taxonomy => {
            let weight = weights.taxonomy;
            match evidence.similarity {
                Some(sim) if sim > FLOAT_TOLERANCE => {
                    let contrib = sim * weight;
                    
                    let match_desc = if let Some(genus) = find_prefix_value(&evidence.shared_values, "genus:") {
                        format!("Memiliki genus yang sama: {genus}")
                    } else if let Some(family) = find_prefix_value(&evidence.shared_values, "family:") {
                        format!("Memiliki famili yang sama: {family}")
                    } else if let Some(order) = find_prefix_value(&evidence.shared_values, "order:") {
                        format!("Memiliki ordo yang sama: {order}")
                    } else {
                        "Memiliki kesamaan taksonomi".to_string()
                    };
                    
                    format!("{match_desc}. Kontribusi taksonomi {contrib:.2}.")
                }
                Some(_) => {
                    "Taksonomi berbeda pada tingkat yang dapat dibandingkan; kontribusi taksonomi 0.00.".to_string()
                }
                None => {
                    "Data taksonomi belum tersedia pada salah satu atau kedua spesies; dimensi taksonomi tidak menambah skor.".to_string()
                }
            }
        }
        AttributeKind::Habitat => {
            let weight = weights.habitat;
            match evidence.similarity {
                Some(sim) if sim > FLOAT_TOLERANCE => {
                    let contrib = sim * weight;
                    let list = evidence.shared_values.join(", ");
                    
                    format!("Memiliki habitat yang sama: {list}. Kontribusi habitat {contrib:.2}.")
                }
                Some(_) => {
                    "Tidak ada habitat bersama; kontribusi habitat 0.00.".to_string()
                }
                None => {
                    "Data habitat belum tersedia pada salah satu atau kedua spesies; dimensi habitat tidak menambah skor.".to_string()
                }
            }
        }
        AttributeKind::Characteristic => {
            let weight = weights.characteristic;
            match evidence.similarity {
                Some(sim) if sim > FLOAT_TOLERANCE => {
                    let contrib = sim * weight;
                    let list = evidence.shared_values.join(", ");
                    
                    format!("Memiliki karakteristik yang sama: {list}. Kontribusi karakteristik {contrib:.2}.")
                }
                Some(_) => {
                    "Tidak ada karakteristik bersama; kontribusi karakteristik 0.00.".to_string()
                }
                None => {
                    "Data karakteristik belum tersedia pada salah satu atau kedua spesies; dimensi karakteristik tidak menambah skor.".to_string()
                }
            }
        }
    }
}

/// Mencari nilai atribut dari shared values taksonomi dengan prefix tertentu (misal `"genus:"`).
fn find_prefix_value<'a>(values: &'a [String], prefix: &str) -> Option<&'a str> {
    values
        .iter()
        .find(|v| v.starts_with(prefix))
        .map(|v| &v[prefix.len()..])
}

/// Mengomposisikan ekstraksi bukti, perhitungan skor, dan pembentukan penjelasan
/// untuk setiap kandidat spesies terhadap spesies pusat.
pub(crate) fn score_candidates(
    center: &Species,
    candidates: &[&Species],
    weights: &ScoreWeights,
) -> Vec<RelatedSpecies> {
    candidates
        .iter()
        .map(|candidate| {
            let evidence = collect_evidence(center, candidate);
            let score = score_evidence(&evidence, weights);
            let explanations = explain_relationship(&score, weights);
            
            RelatedSpecies {
                species: (*candidate).clone(),
                score,
                explanations,
            }
        })
        .collect()
}

/// Menyaring, mengurutkan, dan membatasi rekomendasi spesies terkait sesuai kueri.
///
/// Urutan aturan per spesifikasi §6:
/// 1. Simpan kandidat dengan `total > 0.0` dan `total >= min_score`.
/// 2. Jika `required_basis` terisi, simpan hanya kandidat dengan similarity `Some(value)` dan `value > 0.0` pada dimensi tersebut.
/// 3. Urutkan berdasarkan skor total menurun, lalu ID spesies secara numerik menaik untuk skor yang sama.
/// 4. Ambil maksimal `limit` kandidat setelah seluruh filter diterapkan.
pub(crate) fn rank_related(
    mut items: Vec<RelatedSpecies>,
    query: &RelationshipQuery,
) -> Vec<RelatedSpecies> {
    // Aturan 1: Filter kandidat berskor positif dan memenuhi ambang minimal
    items.retain(|item| {
        item.score.total > FLOAT_TOLERANCE && item.score.total >= (query.min_score - FLOAT_TOLERANCE)
    });

    // Aturan 2: Filter dimensi basis wajib (jika ditentukan)
    if let Some(required) = query.required_basis {
        items.retain(|item| {
            item.score
                .evidence
                .iter()
                .find(|ev| ev.attribute == required)
                .and_then(|ev| ev.similarity)
                .is_some_and(|sim| sim > FLOAT_TOLERANCE)
        });
    }

    // Aturan 3: Urutkan skor menurun; tie-break dengan ID spesies menaik
    items.sort_by(|a, b| {
        if (b.score.total - a.score.total).abs() < FLOAT_TOLERANCE {
            a.species.id.cmp(&b.species.id)
        } else {
            b.score.total.total_cmp(&a.score.total)
        }
    });

    // Aturan 4: Batasi hasil maksimal limit
    items.truncate(query.limit);

    items
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_fixtures::{default_weights, scenario_one, scenario_two};

    fn assert_close(actual: f64, expected: f64) {
        assert!(
            (actual - expected).abs() < FLOAT_TOLERANCE,
            "expected {expected}, got {actual}"
        );
    }

    #[test]
    fn candidate_species_excludes_center() {
        let species = scenario_one();
        let center_id = species[0].id; // 1

        let candidates = candidate_species(&species, &center_id);

        assert_eq!(candidates.len(), 3);
        assert!(candidates.iter().all(|s| s.id != center_id));
        assert_eq!(candidates[0].id, 2);
        assert_eq!(candidates[1].id, 3);
        assert_eq!(candidates[2].id, 4);
    }

    #[test]
    fn candidate_species_empty_when_only_center() {
        let species = scenario_one();
        let single = &species[0..1];

        let candidates = candidate_species(single, &single[0].id);
        assert!(candidates.is_empty());
    }

    #[test]
    fn score_candidates_scenario_one() {
        let species = scenario_one();
        let center = &species[0];
        let weights = default_weights();
        let candidates = candidate_species(&species, &center.id);

        let scored = score_candidates(center, &candidates, &weights);

        assert_eq!(scored.len(), 3);
        // sp-b: ID 2
        assert_eq!(scored[0].species.id, 2);
        assert_close(scored[0].score.total, 0.85);
        assert_close(scored[0].score.coverage, 1.0);
        assert_eq!(scored[0].explanations.len(), 3);
        assert!(scored[0].explanations[0].contains("Memiliki genus yang sama: genus-a"));
        assert!(scored[0].explanations[0].contains("Kontribusi taksonomi 0.50"));
        assert!(scored[0].explanations[1].contains("Memiliki habitat yang sama: wetland"));
        assert!(scored[0].explanations[1].contains("Kontribusi habitat 0.15"));
        assert!(scored[0].explanations[2].contains("Memiliki karakteristik yang sama: evergreen, woody"));
        assert!(scored[0].explanations[2].contains("Kontribusi karakteristik 0.20"));

        // sp-c: ID 3
        assert_eq!(scored[1].species.id, 3);
        assert_close(scored[1].score.total, 0.55);
        assert!(scored[1].explanations[0].contains("Memiliki famili yang sama: family-a"));
        assert!(scored[1].explanations[0].contains("Kontribusi taksonomi 0.30"));

        // sp-d: ID 4
        assert_eq!(scored[2].species.id, 4);
        assert_close(scored[2].score.total, 0.0);
        assert!(scored[2].explanations[0].contains("Taksonomi berbeda pada tingkat yang dapat dibandingkan"));
        assert!(scored[2].explanations[1].contains("Tidak ada habitat bersama"));
        assert!(scored[2].explanations[2].contains("Tidak ada karakteristik bersama"));
    }

    #[test]
    fn score_candidates_scenario_two_missing_data_explanations() {
        let species = scenario_two();
        let center = &species[0];
        let weights = default_weights();
        let candidates = candidate_species(&species, &center.id);

        let scored = score_candidates(center, &candidates, &weights);

        assert_eq!(scored.len(), 2);
        // sp-e: ID 5
        let sp_e = &scored[0];
        assert_eq!(sp_e.species.id, 5);
        assert_close(sp_e.score.total, 0.50);
        assert_close(sp_e.score.coverage, 0.50);
        assert!(sp_e.explanations[0].contains("Memiliki genus yang sama: genus-a"));
        assert!(sp_e.explanations[1].contains("Data habitat belum tersedia"));
        assert!(sp_e.explanations[2].contains("Data karakteristik belum tersedia"));

        // sp-f: ID 6
        let sp_f = &scored[1];
        assert_eq!(sp_f.species.id, 6);
        assert_close(sp_f.score.total, 0.0);
        assert_close(sp_f.score.coverage, 0.0);
        assert!(sp_f.explanations[0].contains("Data taksonomi belum tersedia"));
        assert!(sp_f.explanations[1].contains("Data habitat belum tersedia"));
        assert!(sp_f.explanations[2].contains("Data karakteristik belum tersedia"));
    }

    #[test]
    fn rank_related_scenario_one_ranking() {
        let species = scenario_one();
        let center = &species[0];
        let weights = default_weights();
        let candidates = candidate_species(&species, &center.id);
        let scored = score_candidates(center, &candidates, &weights);

        let query = RelationshipQuery {
            species_id: center.id,
            min_score: 0.20,
            limit: 10,
            required_basis: None,
        };

        let ranked = rank_related(scored, &query);

        // sp-d berskor 0.00 dikeluarkan; sp-b (0.85) peringkat 1, sp-c (0.55) peringkat 2
        assert_eq!(ranked.len(), 2);
        assert_eq!(ranked[0].species.id, 2);
        assert_close(ranked[0].score.total, 0.85);
        assert_eq!(ranked[1].species.id, 3);
        assert_close(ranked[1].score.total, 0.55);
    }

    #[test]
    fn rank_related_min_score_cutoff() {
        let species = scenario_one();
        let center = &species[0];
        let weights = default_weights();
        let candidates = candidate_species(&species, &center.id);
        let scored = score_candidates(center, &candidates, &weights);

        // Ambang dinaikkan ke 0.60 -> hanya sp-b (0.85) yang lolos
        let query = RelationshipQuery {
            species_id: center.id,
            min_score: 0.60,
            limit: 10,
            required_basis: None,
        };

        let ranked = rank_related(scored, &query);
        assert_eq!(ranked.len(), 1);
        assert_eq!(ranked[0].species.id, 2);
    }

    #[test]
    fn rank_related_zero_score_excluded_even_with_zero_min_score() {
        let species = scenario_one();
        let center = &species[0];
        let weights = default_weights();
        let candidates = candidate_species(&species, &center.id);
        let scored = score_candidates(center, &candidates, &weights);

        let query = RelationshipQuery {
            species_id: center.id,
            min_score: 0.0,
            limit: 10,
            required_basis: None,
        };

        let ranked = rank_related(scored, &query);
        // sp-d berskor 0.0 tetap tidak dimasukkan
        assert_eq!(ranked.len(), 2);
        assert!(ranked.iter().all(|r| r.species.id != 4));
    }

    #[test]
    fn rank_related_required_basis_filter() {
        let species = scenario_two();
        let center = &species[0];
        let weights = default_weights();
        let candidates = candidate_species(&species, &center.id);
        let scored = score_candidates(center, &candidates, &weights);

        // Kueri 1: required_basis = Taxonomy -> sp-e lolos karena similarity taksonomi 1.0 > 0
        let q_tax = RelationshipQuery {
            species_id: center.id,
            min_score: 0.20,
            limit: 10,
            required_basis: Some(AttributeKind::Taxonomy),
        };
        let ranked_tax = rank_related(scored.clone(), &q_tax);
        assert_eq!(ranked_tax.len(), 1);
        assert_eq!(ranked_tax[0].species.id, 5);

        // Kueri 2: required_basis = Habitat -> sp-e tidak punya bukti habitat -> hasil kosong
        let q_hab = RelationshipQuery {
            species_id: center.id,
            min_score: 0.20,
            limit: 10,
            required_basis: Some(AttributeKind::Habitat),
        };
        let ranked_hab = rank_related(scored, &q_hab);
        assert!(ranked_hab.is_empty());
    }

    #[test]
    fn rank_related_tie_break_by_id_ascending() {
        let species = scenario_one();
        let center = &species[0];
        let weights = default_weights();
        let candidates = candidate_species(&species, &center.id);

        // Buat dua kandidat dengan skor identik
        let mut scored = score_candidates(center, &candidates, &weights);
        // Set skor item[1] (ID 3) sama dengan item[0] (ID 2): 0.85
        scored[1].score.total = 0.85;

        // Balik urutan input untuk memverifikasi determinisme sorting
        scored.reverse();

        let query = RelationshipQuery {
            species_id: center.id,
            min_score: 0.20,
            limit: 10,
            required_basis: None,
        };

        let ranked = rank_related(scored, &query);
        assert_eq!(ranked.len(), 2);
        // ID 2 harus lebih dulu dari ID 3
        assert_eq!(ranked[0].species.id, 2);
        assert_eq!(ranked[1].species.id, 3);
    }

    #[test]
    fn rank_related_respects_limit() {
        let species = scenario_one();
        let center = &species[0];
        let weights = default_weights();
        let candidates = candidate_species(&species, &center.id);
        let scored = score_candidates(center, &candidates, &weights);

        let query = RelationshipQuery {
            species_id: center.id,
            min_score: 0.20,
            limit: 1, // Batasi 1
            required_basis: None,
        };

        let ranked = rank_related(scored, &query);
        assert_eq!(ranked.len(), 1);
        assert_eq!(ranked[0].species.id, 2);
    }
}
