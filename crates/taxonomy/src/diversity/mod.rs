//! Module 3 Tahap 3 — Diversity & Endemic Analytics.
//!
//! Responsible for aggregating taxonomic data into diversity statistics and for
//! isolating the taxa that contain Kalimantan endemic species, using higher-order
//! functions (`map`, `filter`, `fold`) together with the shared library aggregators
//! [`count_by_key`] and [`frequency_distribution`].
//!
//! # Planning Reference
//!
//! - Document: `Planning_Modul_3_Taxonomy.md` §6 (Tahap 3 — Diversity & Endemic Analytics)
//! - PIC: Anggota 3 — Analisis keanekaragaman dan endemik (ella - 11241021)
//!
//! # Shared Library Usage
//!
//! | Function | Shared helper |
//! | --- | --- |
//! | [`calculate_diversity`] | [`count_by_key`] |
//! | [`filter_endemic_taxa`] | — (module-specific, uses `.filter()`) |
//! | [`rank_taxon_richness`] | [`frequency_distribution`] |
//! | [`get_subtree_species`] | — (Tahap 2, reused from [`crate::taxonomictree`]) |

use std::collections::{HashMap, HashSet};

use kalimantanbio_shared::core::{Species, TaxonomicRank};
use kalimantanbio_shared::stats::{count_by_key, frequency_distribution};

use crate::types::{DiversityStats, Taxon, TaxonId};

/// Species classification value that marks a species as endemic to Kalimantan.
const ENDEMIC_SPECIES_TYPE: &str = "endemic";

/// Collects every species located under `target` in the taxonomy tree.
///
/// This function belongs to the Tahap 3 contract but is implemented once in
/// Tahap 2 ([`crate::taxonomictree::get_subtree_species`]); it is re-exported here
/// so the diversity analytics of Tahap 3 can be read from a single module.
///
/// # Arguments
///
/// * `target` - Identifier of the root taxon whose subtree is collected.
/// * `map` - Parent to children map produced by [`crate::taxonomictree::build_taxonomy_map`].
/// * `species` - The species dataset to filter.
///
/// # Returns
///
/// Every [`Species`] whose `genus_id` sits under the target node.
///
/// # Example
///
/// ```rust,ignore
/// let subtree = get_subtree_species(&TaxonId(2), &map, &species);
/// ```
#[doc(inline)]
pub use crate::taxonomictree::get_subtree_species;

/// Returns whether a species is flagged as endemic to Kalimantan.
///
/// The comparison is case-insensitive and tolerant of surrounding whitespace so
/// that `"Endemic"`, `"endemic"`, and `" Endemic "` are treated alike.
///
/// # Arguments
///
/// * `species` - The species record to inspect.
///
/// # Returns
///
/// `true` when `species_type` is `"Endemic"` (case-insensitive), otherwise `false`.
///
/// # Example
///
/// ```rust,ignore
/// assert!(is_endemic(&species));
/// ```
fn is_endemic(species: &Species) -> bool {
    species
        .species_type
        .trim()
        .eq_ignore_ascii_case(ENDEMIC_SPECIES_TYPE)
}

/// Computes diversity statistics for a species subset.
///
/// Uses the shared [`count_by_key`] aggregator to count distinct families and
/// genera by their identifiers, then counts the endemic species in a single
/// `filter` pass. The input slice is never mutated.
///
/// # Arguments
///
/// * `species_subset` - The species records to aggregate.
///
/// # Returns
///
/// [`DiversityStats`] with `total_families`, `total_genera`, `total_species`, and
/// `endemic_species_count`. An empty subset yields all zeros.
///
/// # Example
///
/// ```rust,ignore
/// let stats = calculate_diversity(&species_subset);
/// assert_eq!(stats.total_species, species_subset.len());
/// assert!(stats.endemic_species_count <= stats.total_species);
/// ```
pub fn calculate_diversity(species_subset: &[Species]) -> DiversityStats {
    let families = count_by_key(species_subset, |species| species.taxonomy.family_id);
    let genera = count_by_key(species_subset, |species| species.taxonomy.genus_id);

    let endemic_species_count = species_subset
        .iter()
        .filter(|species| is_endemic(species))
        .count();

    DiversityStats {
        total_families: families.len(),
        total_genera: genera.len(),
        total_species: species_subset.len(),
        endemic_species_count,
    }
}

/// Keeps only the genus and family taxa that hold at least one endemic species.
///
/// Module-specific implementation: the endemic genus and family identifiers are
/// gathered once into [`HashSet`]s with `filter` + `map` + `collect`, then the
/// taxon slice is narrowed with a single `.filter()`. Ranks above family (kingdom,
/// phylum, class, order, species) are never returned.
///
/// # Arguments
///
/// * `taxons` - Candidate taxon nodes of the taxonomy tree.
/// * `species` - The species dataset used to decide which taxa are endemic.
///
/// # Returns
///
/// The matching [`Taxon`] nodes in their original input order.
///
/// # Example
///
/// ```rust,ignore
/// let endemic_taxa = filter_endemic_taxa(&taxons, &species);
/// assert!(endemic_taxa
///     .iter()
///     .all(|taxon| taxon.rank == TaxonomicRank::Genus || taxon.rank == TaxonomicRank::Family));
/// ```
pub fn filter_endemic_taxa(taxons: &[Taxon], species: &[Species]) -> Vec<Taxon> {
    let endemic_genus_ids: HashSet<u64> = species
        .iter()
        .filter(|species| is_endemic(species))
        .map(|species| species.taxonomy.genus_id)
        .collect();

    let endemic_family_ids: HashSet<u64> = species
        .iter()
        .filter(|species| is_endemic(species))
        .map(|species| species.taxonomy.family_id)
        .collect();

    taxons
        .iter()
        .filter(|taxon| match taxon.rank {
            TaxonomicRank::Genus => endemic_genus_ids.contains(&taxon.id.0),
            TaxonomicRank::Family => endemic_family_ids.contains(&taxon.id.0),
            _ => false,
        })
        .cloned()
        .collect()
}

/// Ranks families by species richness, most diverse first.
///
/// Uses the shared [`frequency_distribution`] aggregator to count species per
/// family, then re-sorts ties alphabetically so the output is deterministic.
/// The family to children map supplied in `map` contributes the optional
/// `parent_id` of every returned family taxon: a family that appears as a child
/// in the map keeps its parent node.
///
/// # Arguments
///
/// * `map` - Parent to children map produced by [`crate::taxonomictree::build_taxonomy_map`].
/// * `species` - The species dataset to rank.
///
/// # Returns
///
/// `(family, species_count)` pairs sorted by count descending, then by family
/// name ascending. An empty dataset yields an empty vector.
///
/// # Example
///
/// ```rust,ignore
/// let rank = rank_taxon_richness(&map, &species);
/// let (top_family, richness) = &rank[0];
/// println!("{} memiliki {} spesies", top_family.name, richness);
/// ```
pub fn rank_taxon_richness(
    map: &HashMap<TaxonId, Vec<TaxonId>>,
    species: &[Species],
) -> Vec<(Taxon, usize)> {
    // Peta anak -> induk, dipakai untuk mengisi `parent_id` famili.
    let parent_of: HashMap<TaxonId, TaxonId> = map
        .iter()
        .flat_map(|(parent, children)| children.iter().map(move |child| (*child, *parent)))
        .collect();

    // Metadata famili diambil dari data spesies karena `map` hanya menyimpan ID.
    let family_meta: HashMap<u64, Taxon> = species.iter().fold(HashMap::new(), |mut acc, species| {
        acc.entry(species.taxonomy.family_id).or_insert_with(|| Taxon {
            id: TaxonId(species.taxonomy.family_id),
            name: species.taxonomy.family.trim().to_string(),
            rank: TaxonomicRank::Family,
            parent_id: parent_of.get(&TaxonId(species.taxonomy.family_id)).copied(),
        });
        acc
    });

    let mut distribution = frequency_distribution(species, |species| species.taxonomy.family_id);
    // Urutkan: jumlah spesies terbanyak dulu; jika sama, nama famili (A-Z); jika nama
    // juga sama, ID famili agar hasil tetap deterministik.
    distribution.sort_by(|a, b| {
        let name_a = family_meta.get(&a.0).map(|family| family.name.as_str());
        let name_b = family_meta.get(&b.0).map(|family| family.name.as_str());

        b.1.cmp(&a.1)
            .then_with(|| name_a.cmp(&name_b))
            .then_with(|| a.0.cmp(&b.0))
    });

    distribution
        .into_iter()
        .filter_map(|(family_id, species_count)| {
            family_meta
                .get(&family_id)
                .cloned()
                .map(|family| (family, species_count))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use kalimantanbio_shared::core::Taxonomy;

    const DIPTEROCARPACEAE: u64 = 10;
    const LAURACEAE: u64 = 20;
    const SHOREA: u64 = 100;
    const HOPEA: u64 = 101;
    const LITSEA: u64 = 200;

    fn sample_species(
        id: u64,
        family_id: u64,
        family: &str,
        genus_id: u64,
        genus: &str,
        species_type: &str,
    ) -> Species {
        Species {
            id,
            scientific_name: format!("{genus} example{id}"),
            common_name: "Contoh".to_string(),
            description: String::new(),
            species_type: species_type.to_string(),
            iucn: "LC".to_string(),
            cites: "-".to_string(),
            p106: "-".to_string(),
            is_verified: true,
            image_url: None,
            taxonomy: Taxonomy {
                kingdom: "Plantae".to_string(),
                kingdom_id: 1,
                phylum_division: "Tracheophyta".to_string(),
                phylum_division_id: 2,
                class: "Magnoliopsida".to_string(),
                class_id: 3,
                order: "Malvales".to_string(),
                order_id: 4,
                family: family.to_string(),
                family_id,
                genus: genus.to_string(),
                genus_id,
            },
            observation_count: 1,
            recorded_individuals_total: 1,
            latest_observation_year: Some(2024),
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
        }
    }

    fn sample_taxon(id: u64, name: &str, rank: TaxonomicRank, parent_id: Option<u64>) -> Taxon {
        Taxon {
            id: TaxonId(id),
            name: name.to_string(),
            rank,
            parent_id: parent_id.map(TaxonId),
        }
    }

    fn dataset() -> Vec<Species> {
        vec![
            sample_species(1, DIPTEROCARPACEAE, "Dipterocarpaceae", SHOREA, "Shorea", "Endemic"),
            sample_species(2, DIPTEROCARPACEAE, "Dipterocarpaceae", SHOREA, "Shorea", "Native"),
            sample_species(3, DIPTEROCARPACEAE, "Dipterocarpaceae", HOPEA, "Hopea", "Endemic"),
            sample_species(4, LAURACEAE, "Lauraceae", LITSEA, "Litsea", "Introduced"),
        ]
    }

    #[test]
    fn test_calculate_diversity_counts_families_genera_and_endemics() {
        let stats = calculate_diversity(&dataset());

        assert_eq!(stats.total_families, 2);
        assert_eq!(stats.total_genera, 3);
        assert_eq!(stats.total_species, 4);
        assert_eq!(stats.endemic_species_count, 2);
    }

    #[test]
    fn test_calculate_diversity_empty_subset() {
        let stats = calculate_diversity(&[]);

        assert_eq!(stats, DiversityStats::default());
    }

    #[test]
    fn test_calculate_diversity_ignores_species_type_case_and_whitespace() {
        let species = vec![
            sample_species(1, DIPTEROCARPACEAE, "Dipterocarpaceae", SHOREA, "Shorea", " ENDEMIC "),
            sample_species(2, DIPTEROCARPACEAE, "Dipterocarpaceae", SHOREA, "Shorea", "Endemi"),
        ];

        assert_eq!(calculate_diversity(&species).endemic_species_count, 1);
    }

    #[test]
    fn test_filter_endemic_taxa_keeps_only_affected_genus_and_family() {
        let taxons = vec![
            sample_taxon(1, "Plantae", TaxonomicRank::Kingdom, None),
            sample_taxon(DIPTEROCARPACEAE, "Dipterocarpaceae", TaxonomicRank::Family, Some(1)),
            sample_taxon(LAURACEAE, "Lauraceae", TaxonomicRank::Family, Some(1)),
            sample_taxon(SHOREA, "Shorea", TaxonomicRank::Genus, Some(DIPTEROCARPACEAE)),
            sample_taxon(HOPEA, "Hopea", TaxonomicRank::Genus, Some(DIPTEROCARPACEAE)),
            sample_taxon(LITSEA, "Litsea", TaxonomicRank::Genus, Some(LAURACEAE)),
        ];

        let endemic = filter_endemic_taxa(&taxons, &dataset());

        let names: Vec<&str> = endemic.iter().map(|taxon| taxon.name.as_str()).collect();
        assert_eq!(names, vec!["Dipterocarpaceae", "Shorea", "Hopea"]);
        assert!(endemic
            .iter()
            .all(|taxon| taxon.rank == TaxonomicRank::Genus || taxon.rank == TaxonomicRank::Family));
    }

    #[test]
    fn test_filter_endemic_taxa_without_endemic_species_is_empty() {
        let taxons = vec![
            sample_taxon(DIPTEROCARPACEAE, "Dipterocarpaceae", TaxonomicRank::Family, None),
            sample_taxon(SHOREA, "Shorea", TaxonomicRank::Genus, Some(DIPTEROCARPACEAE)),
        ];
        let species = vec![sample_species(
            1,
            DIPTEROCARPACEAE,
            "Dipterocarpaceae",
            SHOREA,
            "Shorea",
            "Native",
        )];

        assert!(filter_endemic_taxa(&taxons, &species).is_empty());
    }

    #[test]
    fn test_rank_taxon_richness_sorts_by_species_count() {
        let map: HashMap<TaxonId, Vec<TaxonId>> = HashMap::from([
            (TaxonId(1), vec![TaxonId(DIPTEROCARPACEAE), TaxonId(LAURACEAE)]),
            (
                TaxonId(DIPTEROCARPACEAE),
                vec![TaxonId(SHOREA), TaxonId(HOPEA)],
            ),
            (TaxonId(LAURACEAE), vec![TaxonId(LITSEA)]),
        ]);

        let rank = rank_taxon_richness(&map, &dataset());

        assert_eq!(rank.len(), 2);
        assert_eq!(rank[0].0.name, "Dipterocarpaceae");
        assert_eq!(rank[0].1, 3);
        assert_eq!(rank[0].0.rank, TaxonomicRank::Family);
        assert_eq!(rank[0].0.id, TaxonId(DIPTEROCARPACEAE));
        assert_eq!(rank[0].0.parent_id, Some(TaxonId(1)));
        assert_eq!(rank[1].0.name, "Lauraceae");
        assert_eq!(rank[1].1, 1);
    }

    #[test]
    fn test_rank_taxon_richness_ties_are_sorted_by_name() {

        // ID famili sengaja dibuat berlawanan dengan urutan abjad:
        // "Zingiberaceae" (ID 5) < "Lauraceae" (ID 20) jika diurutkan menurut ID.
        const ZINGIBERACEAE: u64 = 5;
        const ALPINIA: u64 = 300;

        let species = vec![
            sample_species(1, ZINGIBERACEAE, "Zingiberaceae", ALPINIA, "Alpinia", "Native"),
            sample_species(2, LAURACEAE, "Lauraceae", LITSEA, "Litsea", "Native"),
        ];
        
        let map: HashMap<TaxonId, Vec<TaxonId>> = HashMap::new();

        let rank = rank_taxon_richness(&map, &species);

        let names: Vec<&str> = rank.iter().map(|(taxon, _)| taxon.name.as_str()).collect();
        assert_eq!(names, vec!["Lauraceae", "Zingiberaceae"]);
        assert!(rank.iter().all(|(taxon, _)| taxon.parent_id.is_none()));
    }

    #[test]
    fn test_rank_taxon_richness_empty_dataset() {
        let map: HashMap<TaxonId, Vec<TaxonId>> = HashMap::new();

        assert!(rank_taxon_richness(&map, &[]).is_empty());
    }

    #[test]
    fn test_get_subtree_species_re_export_collects_family_species() {
        let map: HashMap<TaxonId, Vec<TaxonId>> = HashMap::from([(
            TaxonId(DIPTEROCARPACEAE),
            vec![TaxonId(SHOREA), TaxonId(HOPEA)],
        )]);

        let subtree = get_subtree_species(&TaxonId(DIPTEROCARPACEAE), &map, &dataset());

        assert_eq!(subtree.len(), 3);
        assert!(subtree
            .iter()
            .all(|species| species.taxonomy.family_id == DIPTEROCARPACEAE));
    }
}
