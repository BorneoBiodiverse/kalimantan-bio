//! Module-specific types for the Biodiversity Knowledge & Citation Explorer.

use serde::{Deserialize, Serialize};

/// An author of a publication.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Author {
    /// Author identifier, when available.
    pub id: Option<String>,
    /// Author name.
    pub name: String,
    /// Author affiliation, when available.
    pub affiliation: Option<String>,
}

/// Source type of a publication.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum SourceType {
    /// Journal article.
    JournalArticle,
    /// Conference paper.
    ConferencePaper,
    /// Thesis.
    Thesis,
    /// Report.
    Report,
    /// Book chapter.
    BookChapter,
    /// Dataset.
    Dataset,
}

/// Research topic category.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum ResearchTopic {
    /// Taxonomy.
    Taxonomy,
    /// Ecology.
    Ecology,
    /// Conservation.
    Conservation,
    /// Ethnobotany.
    Ethnobotany,
    /// Habitat.
    Habitat,
    /// Species identification.
    SpeciesIdentification,
    /// Other topic.
    Other(String),
}

/// Location type of a research study.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum LocationType {
    /// Province.
    Province,
    /// Regency.
    Regency,
    /// Forest.
    Forest,
    /// Conservation area.
    ConservationArea,
    /// Other location type.
    Other(String),
}

/// A research study location.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ResearchLocation {
    /// Location name.
    pub name: String,
    /// Province, when available.
    pub province: Option<String>,
    /// Regency, when available.
    pub regency: Option<String>,
    /// Location type.
    pub location_type: LocationType,
}

/// A scientific publication record.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Publication {
    /// Publication identifier.
    pub id: String,
    /// Title.
    pub title: String,
    /// Abstract, when available.
    pub abstract_text: Option<String>,
    /// Publication year.
    pub publication_year: u16,
    /// Authors.
    pub authors: Vec<Author>,
    /// Venue, when available.
    pub venue: Option<String>,
    /// DOI, when available.
    pub doi: Option<String>,
    /// URL, when available.
    pub url: Option<String>,
    /// Source type.
    pub source_type: SourceType,
    /// Species identifiers discussed in the publication.
    pub species_ids: Vec<u64>,
    /// Taxon names discussed in the publication.
    pub taxon_names: Vec<String>,
    /// Research topics covered.
    pub topics: Vec<ResearchTopic>,
    /// Study locations covered.
    pub locations: Vec<ResearchLocation>,
    /// Citation count, when available.
    pub citation_count: Option<u32>,
}

/// Filter criteria for querying publications.
#[derive(Debug, Clone, Default, Serialize)]
pub struct PublicationQuery {
    /// Free-text filter.
    pub text: Option<String>,
    /// Species identifier filter.
    pub species_id: Option<u64>,
    /// Taxon name filter.
    pub taxon_name: Option<String>,
    /// Topic filter.
    pub topic: Option<ResearchTopic>,
    /// Location name filter.
    pub location: Option<String>,
    /// Earliest publication year.
    pub year_from: Option<u16>,
    /// Latest publication year.
    pub year_to: Option<u16>,
    /// Source type filter.
    pub source_type: Option<SourceType>,
}

/// Coverage record for an exploration entity.
#[derive(Debug, Clone, Default, Serialize)]
pub struct CoverageRecord {
    /// Entity name.
    pub entity_name: String,
    /// Number of linked publications.
    pub publication_count: usize,
    /// Number of linked species.
    pub species_count: usize,
    /// First publication year, when known.
    pub first_year: Option<u16>,
    /// Latest publication year, when known.
    pub latest_year: Option<u16>,
}

/// Criteria and thresholds for a knowledge exploration.
#[derive(Debug, Clone, Default, Serialize)]
pub struct KnowledgeQuery {
    /// Free-text filter.
    pub text: Option<String>,
    /// Species identifiers to scope the exploration.
    pub species_ids: Vec<u64>,
    /// Taxon name filter.
    pub taxon_name: Option<String>,
    /// Topic filter.
    pub topic: Option<ResearchTopic>,
    /// Location name filter.
    pub location: Option<String>,
    /// Earliest publication year.
    pub year_from: Option<u16>,
    /// Latest publication year.
    pub year_to: Option<u16>,
    /// Source type filter.
    pub source_type: Option<SourceType>,
    /// Understudied threshold for species coverage.
    pub understudied_threshold: Option<usize>,
}

/// Node kind in the knowledge network.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub enum KnowledgeNodeKind {
    /// A publication.
    Publication,
    /// A species.
    Species,
    /// A research topic.
    Topic,
    /// A location.
    Location,
    /// A researcher.
    Researcher,
}

/// A node in the knowledge network.
#[derive(Debug, Clone, Serialize)]
pub struct KnowledgeNode {
    /// Node identifier.
    pub id: String,
    /// Node kind.
    pub kind: KnowledgeNodeKind,
    /// Display label.
    pub label: String,
}

/// An edge in the knowledge network.
#[derive(Debug, Clone, Serialize)]
pub struct KnowledgeEdge {
    /// Source node identifier.
    pub source: String,
    /// Target node identifier.
    pub target: String,
    /// Relationship label.
    pub relation: String,
}

/// The biodiversity knowledge network.
#[derive(Debug, Clone, Default, Serialize)]
pub struct KnowledgeNetwork {
    /// Network nodes.
    pub nodes: Vec<KnowledgeNode>,
    /// Network edges.
    pub edges: Vec<KnowledgeEdge>,
}

/// Full knowledge exploration report.
#[derive(Debug, Clone, Default, Serialize)]
pub struct KnowledgeReport {
    /// Publications ordered by relevance.
    pub ranked_publications: Vec<Publication>,
    /// Research output timeline per year.
    pub timeline: std::collections::BTreeMap<u16, usize>,
    /// Entity coverage records.
    pub coverage: Vec<CoverageRecord>,
    /// Understudied species identifiers.
    pub understudied_species: Vec<u64>,
    /// The knowledge network.
    pub graph: KnowledgeNetwork,
}

/// Citation export format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum CitationFormat {
    /// APA style.
    Apa,
    /// BibTeX style.
    Bibtex,
}

/// Errors reported by the knowledge module.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum ModuleError {
    /// Failed to parse publication input.
    Parse(String),
    /// Invalid query or dataset.
    Validation(String),
    /// The dataset produced no results.
    Empty(String),
    /// A requested entity was not found.
    NotFound(String),
}