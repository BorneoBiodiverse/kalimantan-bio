#![warn(missing_docs)]
//! Module 3 — Taxonomy & Classification Explorer.
//!
//! Provides interactive exploration of biological taxonomy and classification:
//! taxonomic tree construction, diversity analysis, gap analysis, and endemic
//! taxa exploration.
//!
//! Public API: `generate_taxonomy_report`, `calculate_diversity`,
//! `analyze_taxonomic_gap`. Internal helpers stay private.

pub mod diversity;
pub mod gap_analysis;
pub mod ingestion;
pub mod taxonomictree;
pub mod types;

use std::collections::{HashMap, HashSet};

use kalimantanbio_shared::core::{Species, TaxonomicRank, Taxonomy};

use crate::ingestion::{parse_species, parse_taxons, validate_hierarchy};
use crate::taxonomictree::{build_taxonomy_map, get_lineage};
use crate::types::{
    BioDataInput, DiversityStats, ExplorationReport, GapReport, PipelineError, RawSpeciesRecord,
    Taxon, TaxonId,
};

/// Analyze which genera are under-represented versus a reference set.
///
/// # Arguments
///
/// * `our_genera` - Genera present in the platform dataset.
/// * `reference_genera` - Genera expected from the reference taxonomy.
///
/// # Returns
///
/// Missing taxa and the coverage score.
///
/// # Example
///
/// ```rust,ignore
/// let report = analyze_taxonomic_gap(&our_genera, &reference_genera);
/// ```
pub fn analyze_taxonomic_gap(
    our_genera: &HashSet<String>,
    reference_genera: &HashSet<String>,
) -> GapReport {
    gap_analysis::analyze_taxonomic_gap(our_genera, reference_genera)
}

/// Calculate diversity statistics for a species list.
///
/// # Arguments
///
/// * `species_subset` - The species dataset.
///
/// # Returns
///
/// Family/genus/species counts and endemic species count.
///
/// # Example
///
/// ```rust,ignore
/// let stats = calculate_diversity(&species);
/// ```
pub fn calculate_diversity(species_subset: &[Species]) -> DiversityStats {
    diversity::calculate_diversity(species_subset)
}

/// Generate the full taxonomy exploration report.
///
/// Pipeline: parse → validate → build tree → diversity → endemic → gap
/// analysis → assemble the report.
///
/// # Arguments
///
/// * `input` - Raw taxon data, raw species data, and reference genera.
///
/// # Returns
///
/// The exploration report, or a pipeline error.
///
/// # Example
///
/// ```rust,ignore
/// let report = generate_taxonomy_report(&input)?;
/// ```
pub fn generate_taxonomy_report(input: &BioDataInput) -> Result<ExplorationReport, PipelineError> {
    // Tahap 1: parsing dan validasi.
    let taxons = parse_taxons(&input.raw_taxons)?;
    let raw_species = parse_species(&input.raw_species)?;

    if !validate_hierarchy(&taxons) {
        return Err(PipelineError::Validation(
            "Taxonomy hierarchy is invalid: a taxon refers to a missing parent or forms a cycle"
                .to_string(),
        ));
    }

    // Tahap 2: struktur pohon dan pencarian takson berdasarkan ID.
    let taxonomy_map = build_taxonomy_map(&taxons);
    let taxon_by_id: HashMap<TaxonId, Taxon> = taxons
        .iter()
        .cloned()
        .map(|taxon| (taxon.id, taxon))
        .collect();

    // Penghubung: RawSpeciesRecord -> Species (tipe yang dibutuhkan Tahap 3).
    let species = raw_species
        .iter()
        .map(|raw| to_species(raw, &taxon_by_id))
        .collect::<Result<Vec<Species>, PipelineError>>()?;

    // Tahap 3: endemik (hanya rank Genus) dan peringkat kekayaan famili.
    let endemic_genera = diversity::filter_endemic_taxa(&taxons, &species)
        .into_iter()
        .filter(|taxon| taxon.rank == TaxonomicRank::Genus)
        .collect();
    let richness_rank = diversity::rank_taxon_richness(&taxonomy_map, &species);

    // Tahap 4: gap analysis terhadap genus referensi.
    let gap = gap_analysis::analyze_taxonomic_gap_from_taxons(&taxons, &input.reference_genera);

    Ok(ExplorationReport {
        endemic_genera,
        richness_rank,
        missing_taxa: gap.missing_taxa,
        coverage_score: gap.coverage_score,
    })
}

/// Links a raw species record to the taxonomy and builds the shared [`Species`] type.
///
/// The genus must exist as a taxon of rank `Genus` and must have a family in its
/// lineage. Kingdom, phylum, class and order are filled from the lineage when present
/// (otherwise left empty), and every field that the analytics never read
/// (common name, description, IUCN status, ...) stays empty.
fn to_species(
    raw: &RawSpeciesRecord,
    taxon_by_id: &HashMap<TaxonId, Taxon>,
) -> Result<Species, PipelineError> {
    let genus = taxon_by_id
        .get(&raw.genus_id)
        .filter(|taxon| taxon.rank == TaxonomicRank::Genus)
        .ok_or_else(|| {
            PipelineError::Parse(format!(
                "Species {} refers to genus_id {} which is not a genus taxon",
                raw.id, raw.genus_id.0
            ))
        })?;

    let lineage = get_lineage(&genus.id, taxon_by_id);

    if !lineage
        .iter()
        .any(|taxon| taxon.rank == TaxonomicRank::Family)
    {
        return Err(PipelineError::Validation(format!(
            "Genus {} (species {}) has no family in its lineage",
            genus.name, raw.id
        )));
    }

    let (kingdom_id, kingdom) = lineage_entry(&lineage, TaxonomicRank::Kingdom);
    let (phylum_division_id, phylum_division) = lineage_entry(&lineage, TaxonomicRank::Phylum);
    let (class_id, class) = lineage_entry(&lineage, TaxonomicRank::Class);
    let (order_id, order) = lineage_entry(&lineage, TaxonomicRank::Order);
    let (family_id, family) = lineage_entry(&lineage, TaxonomicRank::Family);

    Ok(Species {
        id: raw.id,
        scientific_name: raw.scientific_name.clone(),
        common_name: String::new(),
        description: String::new(),
        species_type: raw.species_type.clone(),
        iucn: String::new(),
        cites: String::new(),
        p106: String::new(),
        is_verified: false,
        image_url: None,
        taxonomy: Taxonomy {
            kingdom,
            kingdom_id,
            phylum_division,
            phylum_division_id,
            class,
            class_id,
            order,
            order_id,
            family,
            family_id,
            genus: genus.name.clone(),
            genus_id: genus.id.0,
        },
        observation_count: 0,
        recorded_individuals_total: 0,
        latest_observation_year: None,
        created_at: String::new(),
        updated_at: String::new(),
    })
}

/// Returns the `(id, name)` of the lineage entry with the given rank, or `(0, "")` when absent.
fn lineage_entry(lineage: &[Taxon], rank: TaxonomicRank) -> (u64, String) {
    lineage
        .iter()
        .find(|taxon| taxon.rank == rank)
        .map(|taxon| (taxon.id.0, taxon.name.clone()))
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::RawTaxonRecord;

    fn raw_taxon(id: u64, name: &str, rank: TaxonomicRank, parent: Option<u64>) -> RawTaxonRecord {
        RawTaxonRecord {
            id: TaxonId(id),
            name: name.to_string(),
            rank,
            parent_id: parent.map(TaxonId),
        }
    }

    fn raw_species(id: u64, name: &str, genus_id: u64, species_type: &str) -> RawSpeciesRecord {
        RawSpeciesRecord {
            id,
            scientific_name: name.to_string(),
            genus_id: TaxonId(genus_id),
            species_type: species_type.to_string(),
        }
    }

    fn names(items: &[&str]) -> HashSet<String> {
        items.iter().map(|item| item.to_string()).collect()
    }

    /// Plantae > {Dipterocarpaceae > {Shorea, Hopea}, Lauraceae > Litsea}
    fn sample_taxons() -> Vec<RawTaxonRecord> {
        vec![
            raw_taxon(1, "Plantae", TaxonomicRank::Kingdom, None),
            raw_taxon(2, "Dipterocarpaceae", TaxonomicRank::Family, Some(1)),
            raw_taxon(3, "Shorea", TaxonomicRank::Genus, Some(2)),
            raw_taxon(4, "Hopea", TaxonomicRank::Genus, Some(2)),
            raw_taxon(5, "Lauraceae", TaxonomicRank::Family, Some(1)),
            raw_taxon(6, "Litsea", TaxonomicRank::Genus, Some(5)),
        ]
    }

    fn sample_species() -> Vec<RawSpeciesRecord> {
        vec![
            raw_species(1, "Shorea leprosula", 3, "Endemic"),
            raw_species(2, "Shorea parvifolia", 3, "Native"),
            raw_species(3, "Hopea dryobalanoides", 4, "Endemic"),
            raw_species(4, "Litsea firma", 6, "Introduced"),
        ]
    }

    fn sample_input() -> BioDataInput {
        BioDataInput {
            raw_taxons: sample_taxons(),
            raw_species: sample_species(),
            // Litsea sengaja tidak ada di referensi; Vatica dan Dipterocarpus belum kita punya.
            reference_genera: names(&["Shorea", "Hopea", "Vatica", "Dipterocarpus"]),
        }
    }

    #[test]
    fn test_generate_taxonomy_report_end_to_end() {
        let report = generate_taxonomy_report(&sample_input()).unwrap();

        let endemic: Vec<&str> = report
            .endemic_genera
            .iter()
            .map(|t| t.name.as_str())
            .collect();
        assert_eq!(endemic, vec!["Shorea", "Hopea"]);
        assert!(report
            .endemic_genera
            .iter()
            .all(|taxon| taxon.rank == TaxonomicRank::Genus));

        let richness: Vec<(&str, usize)> = report
            .richness_rank
            .iter()
            .map(|(taxon, count)| (taxon.name.as_str(), *count))
            .collect();
        assert_eq!(richness, vec![("Dipterocarpaceae", 3), ("Lauraceae", 1)]);
        assert_eq!(report.richness_rank[0].0.parent_id, Some(TaxonId(1)));

        assert_eq!(
            report.missing_taxa,
            vec!["Dipterocarpus".to_string(), "Vatica".to_string()]
        );
        // 2 dari 4 genus referensi tercakup. Rumus planning §9 (3 genus kita / 4) akan salah (0.75).
        assert!((report.coverage_score - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_generate_taxonomy_report_without_endemic_species() {
        let mut input = sample_input();
        input.raw_species = vec![raw_species(1, "Shorea leprosula", 3, "Native")];

        let report = generate_taxonomy_report(&input).unwrap();

        assert!(report.endemic_genera.is_empty());
        assert_eq!(report.richness_rank.len(), 1);
    }

    #[test]
    fn test_generate_taxonomy_report_rejects_unknown_genus_id() {
        let mut input = sample_input();
        input
            .raw_species
            .push(raw_species(5, "Unknown species", 999, "Endemic"));

        assert!(matches!(
            generate_taxonomy_report(&input),
            Err(PipelineError::Parse(_))
        ));
    }

    #[test]
    fn test_generate_taxonomy_report_rejects_species_pointing_to_non_genus() {
        let mut input = sample_input();
        // ID 2 adalah Famili, bukan Genus.
        input
            .raw_species
            .push(raw_species(5, "Wrong link", 2, "Endemic"));

        assert!(matches!(
            generate_taxonomy_report(&input),
            Err(PipelineError::Parse(_))
        ));
    }

    #[test]
    fn test_generate_taxonomy_report_rejects_orphan_taxon() {
        let mut input = sample_input();
        input
            .raw_taxons
            .push(raw_taxon(7, "Orphan", TaxonomicRank::Genus, Some(999)));

        assert!(matches!(
            generate_taxonomy_report(&input),
            Err(PipelineError::Validation(_))
        ));
    }

    #[test]
    fn test_generate_taxonomy_report_rejects_genus_without_family() {
        let input = BioDataInput {
            raw_taxons: vec![
                raw_taxon(1, "Plantae", TaxonomicRank::Kingdom, None),
                raw_taxon(3, "Shorea", TaxonomicRank::Genus, Some(1)),
            ],
            raw_species: vec![raw_species(1, "Shorea leprosula", 3, "Endemic")],
            reference_genera: names(&["Shorea"]),
        };

        assert!(matches!(
            generate_taxonomy_report(&input),
            Err(PipelineError::Validation(_))
        ));
    }

    #[test]
    fn test_generate_taxonomy_report_propagates_parse_errors() {
        let mut empty_taxons = sample_input();
        empty_taxons.raw_taxons.clear();
        assert!(matches!(
            generate_taxonomy_report(&empty_taxons),
            Err(PipelineError::Empty(_))
        ));

        let mut empty_type = sample_input();
        empty_type.raw_species[0].species_type = "  ".to_string();
        assert!(matches!(
            generate_taxonomy_report(&empty_type),
            Err(PipelineError::Validation(_))
        ));
    }

    #[test]
    fn test_get_lineage_with_dummy_data() {
        let taxons = parse_taxons(&sample_taxons()).unwrap();
        let by_id: HashMap<TaxonId, Taxon> =
            taxons.into_iter().map(|taxon| (taxon.id, taxon)).collect();

        let lineage: Vec<String> = get_lineage(&TaxonId(3), &by_id)
            .into_iter()
            .map(|taxon| taxon.name)
            .collect();

        assert_eq!(lineage, vec!["Shorea", "Dipterocarpaceae", "Plantae"]);
    }

    #[test]
    fn test_find_missing_taxa_with_dummy_data() {
        let reference = names(&["Shorea", "Hopea", "Vatica"]);
        let ours = names(&["Shorea", "Litsea"]);

        let missing = gap_analysis::find_missing_taxa(&reference, &ours);

        assert_eq!(missing, vec!["Hopea".to_string(), "Vatica".to_string()]);
    }

    #[test]
    fn test_calculate_diversity_with_dummy_data() {
        let taxons = parse_taxons(&sample_taxons()).unwrap();
        let by_id: HashMap<TaxonId, Taxon> =
            taxons.into_iter().map(|taxon| (taxon.id, taxon)).collect();
        let species: Vec<Species> = parse_species(&sample_species())
            .unwrap()
            .iter()
            .map(|raw| to_species(raw, &by_id).unwrap())
            .collect();

        let stats = calculate_diversity(&species);

        assert_eq!(stats.total_species, 4);
        assert_eq!(stats.total_genera, 3);
        assert_eq!(stats.total_families, 2);
        assert_eq!(stats.endemic_species_count, 2);
    }
}
