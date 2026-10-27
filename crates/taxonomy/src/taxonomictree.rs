
use std::collections::{HashMap, HashSet};
use kalimantanbio_shared::core::Species;
use crate::types::{Taxon, TaxonId};

/// Membuat peta yang menghubungkan ID induk dengan ID anak-anaknya.
pub fn build_taxonomy_map(
    taxons: &[Taxon],
) -> HashMap<TaxonId, Vec<TaxonId>> {
    taxons.iter().fold(
        HashMap::new(),
        |mut taxonomy_map: HashMap<TaxonId, Vec<TaxonId>>, taxon| {
            if let Some(parent_id) = taxon.parent_id {
                taxonomy_map.entry(parent_id).or_default().push(taxon.id);
            }
            taxonomy_map
        },
    )
}

/// Mendapatkan garis keturunan takson dari dirinya hingga akar.
///
/// Penelusuran berhenti jika bertemu node yang sudah dikunjungi, sehingga data
/// dengan siklus induk tidak membuat fungsi ini berputar tanpa henti.

pub fn get_lineage(
    taxon_id: &TaxonId,
    map: &HashMap<TaxonId, Taxon>,
) -> Vec<Taxon> {
    let mut lineage = Vec::new();
    let mut visited: HashSet<TaxonId> = HashSet::new();
    let mut current_id = Some(*taxon_id);

    while let Some(id) = current_id {
        if !visited.insert(id) {
            break;
        }
        match map.get(&id) {
            Some(taxon) => {
                lineage.push(taxon.clone());
                current_id = taxon.parent_id;
            }
            None => break,
        }
    }

    lineage
}

/// Mendapatkan semua spesies yang berada di bawah takson target.
pub fn get_subtree_species(
    target: &TaxonId,
    map: &HashMap<TaxonId, Vec<TaxonId>>,
    species: &[Species],
) -> Vec<Species> {
    let mut descendant_ids = std::collections::HashSet::new();
    let mut stack = vec![*target];

    // Telusuri semua takson di bawah target.
    while let Some(current_id) = stack.pop() {
        if !descendant_ids.insert(current_id) {
            continue;
        }

        if let Some(children) = map.get(&current_id) {
            stack.extend(children.iter().copied());
        }
    }

    // Ambil spesies yang genus-nya berada di dalam subtree.
    species
        .iter()
        .filter(|sp| {
            descendant_ids.contains(&TaxonId(sp.taxonomy.genus_id))
        })
        .cloned()
        .collect()
}

/// Mengambil nama semua takson yang memiliki rank Genus.
///
/// Di-export ulang dari [`crate::gap_analysis`] agar hanya ada satu implementasi.
pub use crate::gap_analysis::extract_our_genera;

#[cfg(test)]
mod tests {
    use super::*;
    use kalimantanbio_shared::core::TaxonomicRank;

    #[test]
    fn test_build_taxonomy_map() {
        let taxons = vec![
            Taxon {
                id: TaxonId(2),
                name: "Dipterocarpaceae".to_string(),
                rank: TaxonomicRank::Family,
                parent_id: None,
            },
            Taxon {
                id: TaxonId(3),
                name: "Shorea".to_string(),
                rank: TaxonomicRank::Genus,
                parent_id: Some(TaxonId(2)),
            },
            Taxon {
                id: TaxonId(4),
                name: "Hopea".to_string(),
                rank: TaxonomicRank::Genus,
                parent_id: Some(TaxonId(2)),
            },
        ];

        let result = build_taxonomy_map(&taxons);

        assert_eq!(
            result.get(&TaxonId(2)),
            Some(&vec![TaxonId(3), TaxonId(4)])
        );

        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_get_lineage() {
        let taxons = vec![
            Taxon {
                id: TaxonId(1),
                name: "Plantae".to_string(),
                rank: TaxonomicRank::Kingdom,
                parent_id: None,
            },
            Taxon {
                id: TaxonId(2),
                name: "Dipterocarpaceae".to_string(),
                rank: TaxonomicRank::Family,
                parent_id: Some(TaxonId(1)),
            },
            Taxon {
                id: TaxonId(3),
                name: "Shorea".to_string(),
                rank: TaxonomicRank::Genus,
                parent_id: Some(TaxonId(2)),
            },
        ];

        let map: HashMap<TaxonId, Taxon> = taxons
            .into_iter()
            .map(|taxon| (taxon.id, taxon))
            .collect();

        let lineage = get_lineage(&TaxonId(3), &map);

        let names: Vec<&str> = lineage
            .iter()
            .map(|taxon| taxon.name.as_str())
            .collect();

        assert_eq!(
            names,
            vec!["Shorea", "Dipterocarpaceae", "Plantae"]
        );

        assert!(get_lineage(&TaxonId(999), &map).is_empty());
    }
    
    #[test]
    fn test_extract_our_genera() {
        let taxa = vec![
            Taxon {
                id: TaxonId(1),
                name: "Dipterocarpaceae".to_string(),
                rank: TaxonomicRank::Family,
                parent_id: None,
            },
            Taxon {
                id: TaxonId(2),
                name: "Shorea".to_string(),
                rank: TaxonomicRank::Genus,
                parent_id: Some(TaxonId(1)),
            },
            Taxon {
                id: TaxonId(3),
                name: "Hopea".to_string(),
                rank: TaxonomicRank::Genus,
                parent_id: Some(TaxonId(1)),
            },

    #[test]
    fn test_get_lineage_stops_on_cycle() {
        let taxons = vec![
            Taxon {
                id: TaxonId(1),
                name: "A".to_string(),
                rank: TaxonomicRank::Family,
                parent_id: Some(TaxonId(2)),
            },
            Taxon {
                id: TaxonId(2),
                name: "B".to_string(),
                rank: TaxonomicRank::Genus,
                parent_id: Some(TaxonId(1)),
            },
        ];

        let map: HashMap<TaxonId, Taxon> =
            taxons.into_iter().map(|taxon| (taxon.id, taxon)).collect();

        let lineage = get_lineage(&TaxonId(2), &map);

        assert_eq!(lineage.len(), 2);
            }
        ];

        let result = extract_our_genera(&taxa);

        assert_eq!(result.len(), 2);
        assert!(result.contains("Shorea"));
        assert!(result.contains("Hopea"));
        assert!(!result.contains("Dipterocarpaceae"));
    }
}