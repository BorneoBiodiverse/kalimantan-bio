//! Core domain types shared across all modules.

use serde::{Deserialize, Serialize};

/// A biodiversity species record.
///
/// Mirrors the production KalimantanBio species schema.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Species {
    /// Database identifier.
    pub id: u64,
    /// Scientific name.
    pub scientific_name: String,
    /// Common name.
    pub common_name: String,
    /// Free-text description.
    pub description: String,
    /// Origin category: `Endemic`, `Native`, or `Introduced`.
    pub species_type: String,
    /// IUCN conservation status: `CR`, `EN`, `VU`, `NT`, `LC`, `DD`, or `-`.
    pub iucn: String,
    /// CITES appendix: `Appendix I`, `II`, `III`, or `-`.
    pub cites: String,
    /// National protection status: `Dilindungi`, `Tidak Dilindungi`, or `-`.
    pub p106: String,
    /// Whether the record is verified.
    pub is_verified: bool,
    /// Optional image URL.
    pub image_url: Option<String>,
    /// Taxonomic classification of the species.
    pub taxonomy: Taxonomy,
    /// Number of observation records.
    pub observation_count: u32,
    /// Total recorded individuals.
    pub recorded_individuals_total: u32,
    /// Latest observation year, when known.
    pub latest_observation_year: Option<u32>,
    /// Creation timestamp (ISO 8601).
    pub created_at: String,
    /// Last update timestamp (ISO 8601).
    pub updated_at: String,
}

/// Taxonomic classification levels for a species.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Taxonomy {
    /// Kingdom name.
    pub kingdom: String,
    /// Kingdom database identifier.
    pub kingdom_id: u64,
    /// Phylum/division name.
    pub phylum_division: String,
    /// Phylum/division database identifier.
    pub phylum_division_id: u64,
    /// Class name.
    pub class: String,
    /// Class database identifier.
    pub class_id: u64,
    /// Order name.
    pub order: String,
    /// Order database identifier.
    pub order_id: u64,
    /// Family name.
    pub family: String,
    /// Family database identifier.
    pub family_id: u64,
    /// Genus name.
    pub genus: String,
    /// Genus database identifier.
    pub genus_id: u64,
}

/// A taxonomic rank in the classification hierarchy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TaxonomicRank {
    /// Kingdom (rank 1).
    Kingdom = 1,
    /// Phylum/division (rank 2).
    Phylum = 2,
    /// Class (rank 3).
    Class = 3,
    /// Order (rank 4).
    Order = 4,
    /// Family (rank 5).
    Family = 5,
    /// Genus (rank 6).
    Genus = 6,
}

/// A verified observation record for a species.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Observation {
    /// Database identifier.
    pub id: u64,
    /// Identifier of the observed species.
    pub species_id: u64,
    /// Latitude of the observation.
    pub latitude: f64,
    /// Longitude of the observation.
    pub longitude: f64,
    /// Observation date (ISO 8601).
    pub date: String,
    /// Number of individuals observed.
    pub count: i32,
    /// Whether the observation is verified.
    pub is_verified: bool,
    /// Source or sampling method.
    pub source_method: String,
    /// Regency (kabupaten) identifier.
    pub kabupaten_id: u64,
    /// Regency (kabupaten) name.
    pub kabupaten_name: String,
    /// Province name.
    pub province_name: String,
}