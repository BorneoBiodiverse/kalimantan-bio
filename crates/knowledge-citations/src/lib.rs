#![warn(missing_docs)]
//! Module 5 — Biodiversity Knowledge & Citation Explorer.
//!
//! Connects biodiversity knowledge with scientific publications, research
//! topics, locations, and citations. Publication data is supplied by the
//! caller; this crate does not perform network or database I/O.

pub mod types;

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use kalimantanbio_shared::core::Species;
use kalimantanbio_shared::scoring::{filter_by_threshold, rank_by_score, top_n};
use kalimantanbio_shared::stats::{build_timeline, calculate_coverage_percentage, count_by_key};
use kalimantanbio_shared::text::normalize_text;

use crate::types::{
    CitationFormat, CoverageRecord, KnowledgeEdge, KnowledgeGraph, KnowledgeInput,
    KnowledgeNetwork, KnowledgeNode, KnowledgeNodeKind, KnowledgeQuery, KnowledgeReport,
    LocationType, ModuleError, Publication, PublicationQuery, ResearchLocation, ResearchTopic,
    SourceType,
};

/// Explore biodiversity knowledge across the supplied publications.
///
/// The pipeline validates, normalizes and deduplicates the dataset, filters
/// publications using every supplied criterion, and assembles timeline,
/// coverage, understudied-species and knowledge-network results. An empty
/// dataset is an error; a valid query with no matches returns an empty ranked
/// list alongside statistics for the complete dataset.
///
/// # Arguments
///
/// * `query` - Exploration criteria (species, topic, location, year and text).
/// * `publications` - The publication dataset, typically from a fixture or adapter.
///
/// # Returns
///
/// A report containing ranked matches and research coverage, or a validation
/// error if the query or any publication is invalid.
///
/// # Errors
///
/// Returns [`ModuleError::Empty`] for an empty dataset and
/// [`ModuleError::Validation`] for invalid query bounds or publication metadata.
///
/// # Example
///
/// ```rust
/// use knowledge_citations::explore_knowledge;
/// use knowledge_citations::types::{KnowledgeQuery, Publication, SourceType};
///
/// let query = KnowledgeQuery::default();
/// let publication = Publication {
///     id: String::from("pub-1"),
///     title: String::from("Borneo Flora"),
///     abstract_text: None,
///     publication_year: 2021,
///     authors: vec![],
///     venue: None,
///     doi: None,
///     url: None,
///     source_type: SourceType::JournalArticle,
///     species_ids: vec![1],
///     taxon_names: vec![],
///     topics: vec![],
///     locations: vec![],
///     citation_count: None,
/// };
/// let report = explore_knowledge(&query, &[publication]).unwrap();
/// assert_eq!(report.ranked_publications.len(), 1);
/// ```
pub fn explore_knowledge(
    query: &KnowledgeQuery,
    publications: &[Publication],
) -> Result<KnowledgeReport, ModuleError> {
    validate_knowledge_query(query)?;
    if publications.is_empty() {
        return Err(ModuleError::Empty(String::from(
            "publication dataset is empty",
        )));
    }

    let normalized = publications
        .iter()
        .cloned()
        .map(normalize_publication)
        .collect::<Vec<_>>();
    for publication in &normalized {
        validate_publication(publication)?;
    }
    let publications = deduplicate_publications(&normalized);
    let publication_query = PublicationQuery {
        text: query.text.clone(),
        species_id: if query.species_ids.len() == 1 {
            query.species_ids.first().copied()
        } else {
            None
        },
        taxon_name: query.taxon_name.clone(),
        topic: query.topic.clone(),
        location: query.location.clone(),
        year_from: query.year_from,
        year_to: query.year_to,
        source_type: query.source_type.clone(),
    };
    let species_scope = query
        .species_ids
        .iter()
        .copied()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let species_scope_set = species_scope.iter().copied().collect::<HashSet<_>>();
    let matches = query_publications(&publications, &publication_query)
        .into_iter()
        .filter(|publication| {
            species_scope.is_empty()
                || publication
                    .species_ids
                    .iter()
                    .any(|species_id| species_scope_set.contains(species_id))
        })
        .collect::<Vec<_>>();
    let ranked = rank_publications(&matches, &publication_query);

    let timeline = build_research_timeline(&publications);
    let topic_distribution = calculate_topic_distribution(&publications)
        .into_iter()
        .map(|(topic, count)| (topic_label(&topic), count))
        .collect::<BTreeMap<_, _>>();
    let location_distribution = calculate_location_distribution(&publications)
        .into_iter()
        .collect::<BTreeMap<_, _>>();
    let coverage = calculate_entity_coverage(&publications, &query.species_ids);
    let covered_species = coverage
        .iter()
        .filter(|record| record.publication_count > 0)
        .count();
    let coverage_score = calculate_coverage_score(covered_species, coverage.len());
    let understudied_threshold = query.understudied_threshold.unwrap_or(1);
    let understudied_records = find_understudied_species(&coverage, understudied_threshold);
    let understudied_species = understudied_records
        .iter()
        .filter_map(|record| record.entity_name.parse::<u64>().ok())
        .collect();
    let graph = build_knowledge_graph(&publications);

    Ok(KnowledgeReport {
        ranked_publications: ranked
            .into_iter()
            .map(|(publication, _)| publication.clone())
            .collect(),
        timeline,
        topic_distribution,
        location_distribution,
        coverage,
        coverage_score,
        understudied_species,
        graph,
    })
}

/// Convenience entry point for the knowledge exploration pipeline taking [`KnowledgeInput`] (§9).
///
/// Parses `input.raw_publications` as JSON and executes [`explore_knowledge`].
///
/// # Arguments
///
/// * `input` - Raw publication data string and exploration query.
///
/// # Errors
///
/// Returns [`ModuleError::Parse`] if publication JSON is invalid, or
/// [`ModuleError::Validation`] / [`ModuleError::Empty`].
///
/// # Example
///
/// ```rust
/// use knowledge_citations::explore_knowledge_from_input;
/// use knowledge_citations::types::{KnowledgeInput, KnowledgeQuery};
///
/// let input = KnowledgeInput {
///     raw_publications: String::from(r#"[{"id":"p1","title":"Forest Study","publication_year":2021,"authors":[],"source_type":"JournalArticle","species_ids":[1],"taxon_names":[],"topics":[],"locations":[]}]"#),
///     query: KnowledgeQuery::default(),
/// };
/// let report = explore_knowledge_from_input(&input).unwrap();
/// assert_eq!(report.ranked_publications.len(), 1);
/// ```
pub fn explore_knowledge_from_input(
    input: &KnowledgeInput,
) -> Result<KnowledgeReport, ModuleError> {
    let publications = parse_publications(&input.raw_publications)?;
    explore_knowledge(&input.query, &publications)
}

/// Recommend the most relevant publications for citation.
///
/// All specified query filters are combined with AND semantics. Text and
/// species matches contribute relevance scores; matching publications are
/// then ordered deterministically by score, publication year, citation count,
/// and publication ID.
///
/// # Arguments
///
/// * `items` - The publication dataset.
/// * `query` - The recommendation criteria.
/// * `limit` - Maximum number of recommendations.
///
/// # Returns
///
/// Up to `limit` references ordered by relevance.
///
/// # Example
///
/// ```rust
/// use knowledge_citations::recommend_citations;
/// use knowledge_citations::types::{Publication, PublicationQuery, SourceType};
///
/// let query = PublicationQuery::default();
/// let publication = Publication {
///     id: String::from("pub-1"),
///     title: String::from("Borneo Flora"),
///     abstract_text: None,
///     publication_year: 2021,
///     authors: vec![],
///     venue: None,
///     doi: None,
///     url: None,
///     source_type: SourceType::JournalArticle,
///     species_ids: vec![1],
///     taxon_names: vec![],
///     topics: vec![],
///     locations: vec![],
///     citation_count: None,
/// };
/// let publications = vec![publication];
/// let recommended = recommend_citations(&publications, &query, 5);
/// assert_eq!(recommended.len(), 1);
/// ```
pub fn recommend_citations<'a>(
    items: &'a [Publication],
    query: &PublicationQuery,
    limit: usize,
) -> Vec<&'a Publication> {
    let normalized = items
        .iter()
        .cloned()
        .map(normalize_publication)
        .collect::<Vec<_>>();
    let references = normalized.iter().collect::<Vec<_>>();
    let deduplicated = deduplicate_references(&references);
    let matched = deduplicated
        .into_iter()
        .filter(|publication| publication_matches(publication, query))
        .collect::<Vec<_>>();
    let ranked = rank_publications(&matched, query);
    let ranked = top_n(ranked, limit);
    let normalized_to_original = items
        .iter()
        .map(|publication| (normalized_id(publication), publication))
        .collect::<HashMap<_, _>>();

    ranked
        .into_iter()
        .filter_map(|(publication, _)| normalized_to_original.get(&publication.id).copied())
        .collect()
}

/// Export publications as citations in APA or BibTeX format.
///
/// Missing bibliographic fields are omitted rather than fabricated. Entries
/// are separated by a blank line; an empty input produces an empty string.
///
/// # Arguments
///
/// * `items` - The publications to export.
/// * `format` - Target citation format (APA or BibTeX).
///
/// # Returns
///
/// A citation string for all provided publications.
///
/// # Example
///
/// ```rust
/// use knowledge_citations::export_citations;
/// use knowledge_citations::types::{CitationFormat, Publication, SourceType};
///
/// let publication = Publication {
///     id: String::from("pub-1"),
///     title: String::from("Borneo Flora"),
///     abstract_text: None,
///     publication_year: 2021,
///     authors: vec![],
///     venue: None,
///     doi: None,
///     url: None,
///     source_type: SourceType::JournalArticle,
///     species_ids: vec![1],
///     taxon_names: vec![],
///     topics: vec![],
///     locations: vec![],
///     citation_count: None,
/// };
/// let apa = export_citations(&[publication], CitationFormat::Apa);
/// assert!(apa.contains("Borneo Flora"));
/// ```
pub fn export_citations(items: &[Publication], format: CitationFormat) -> String {
    items
        .iter()
        .map(|publication| match format {
            CitationFormat::Apa => format_apa(publication),
            CitationFormat::Bibtex => format_bibtex(publication),
        })
        .collect::<Vec<_>>()
        .join("\n\n")
}

/// Parse a JSON string into a vector of publications.
///
/// Supports parsing either a JSON array of publications or a single publication object.
///
/// # Arguments
///
/// * `raw` - JSON string containing publication data.
///
/// # Errors
///
/// Returns [`ModuleError::Parse`] if deserialization fails.
///
/// # Example
///
/// ```rust
/// use knowledge_citations::parse_publications;
///
/// let json = r#"[{"id":"p1","title":"Forest Study","publication_year":2020,"authors":[],"source_type":"JournalArticle","species_ids":[],"taxon_names":[],"topics":[],"locations":[]}]"#;
/// let publications = parse_publications(json).unwrap();
/// assert_eq!(publications.len(), 1);
/// ```
pub fn parse_publications(raw: &str) -> Result<Vec<Publication>, ModuleError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }
    if let Ok(publications) = serde_json::from_str::<Vec<Publication>>(trimmed) {
        return Ok(publications);
    }
    if let Ok(single) = serde_json::from_str::<Publication>(trimmed) {
        return Ok(vec![single]);
    }
    serde_json::from_str::<Vec<Publication>>(trimmed)
        .map_err(|error| ModuleError::Parse(error.to_string()))
}

/// Find publications that discuss a given species entity from `kalimantanbio-shared`.
///
/// Matches publication records by species ID, taxonomic genus or family, or
/// scientific name.
///
/// # Arguments
///
/// * `items` - The publication dataset.
/// * `species` - A species entity from `kalimantanbio-shared`.
///
/// # Returns
///
/// References to publications discussing the species.
///
/// # Example
///
/// ```rust
/// use kalimantanbio_shared::core::{Species, Taxonomy};
/// use knowledge_citations::publications_for_species_entity;
/// use knowledge_citations::types::{Publication, SourceType};
///
/// let species = Species {
///     id: 1,
///     scientific_name: String::from("Pongo pygmaeus"),
///     common_name: String::from("Bornean Orangutan"),
///     description: String::new(),
///     species_type: String::from("Endemic"),
///     iucn: String::from("CR"),
///     cites: String::from("Appendix I"),
///     p106: String::from("Dilindungi"),
///     is_verified: true,
///     image_url: None,
///     taxonomy: Taxonomy {
///         kingdom: String::from("Animalia"),
///         kingdom_id: 1,
///         phylum_division: String::from("Chordata"),
///         phylum_division_id: 2,
///         class: String::from("Mammalia"),
///         class_id: 3,
///         order: String::from("Primates"),
///         order_id: 4,
///         family: String::from("Hominidae"),
///         family_id: 5,
///         genus: String::from("Pongo"),
///         genus_id: 6,
///     },
///     observation_count: 5,
///     recorded_individuals_total: 10,
///     latest_observation_year: Some(2023),
///     created_at: String::new(),
///     updated_at: String::new(),
/// };
/// let publication = Publication {
///     id: String::from("pub-1"),
///     title: String::from("Pongo pygmaeus nesting patterns"),
///     abstract_text: None,
///     publication_year: 2021,
///     authors: vec![],
///     venue: None,
///     doi: None,
///     url: None,
///     source_type: SourceType::JournalArticle,
///     species_ids: vec![1],
///     taxon_names: vec![String::from("Pongo")],
///     topics: vec![],
///     locations: vec![],
///     citation_count: None,
/// };
/// let publications = vec![publication];
/// let matches = publications_for_species_entity(&publications, &species);
/// assert_eq!(matches.len(), 1);
/// ```
pub fn publications_for_species_entity<'a>(
    items: &'a [Publication],
    species: &Species,
) -> Vec<&'a Publication> {
    items
        .iter()
        .filter(|publication| {
            publication.species_ids.contains(&species.id)
                || publication.taxon_names.iter().any(|taxon| {
                    let norm = normalize_text(taxon);
                    norm == normalize_text(&species.taxonomy.genus)
                        || norm == normalize_text(&species.taxonomy.family)
                })
                || publication_matches_text(publication, &species.scientific_name)
        })
        .collect()
}

fn validate_knowledge_query(query: &KnowledgeQuery) -> Result<(), ModuleError> {
    if let (Some(from), Some(to)) = (query.year_from, query.year_to) {
        if from > to {
            return Err(ModuleError::Validation(String::from(
                "year_from must not be later than year_to",
            )));
        }
    }
    if query
        .text
        .as_deref()
        .is_some_and(|value| normalize_text(value).is_empty())
    {
        return Err(ModuleError::Validation(String::from(
            "text query must not be blank",
        )));
    }
    if query
        .taxon_name
        .as_deref()
        .is_some_and(|value| normalize_text(value).is_empty())
    {
        return Err(ModuleError::Validation(String::from(
            "taxon name query must not be blank",
        )));
    }
    if query
        .location
        .as_deref()
        .is_some_and(|value| normalize_text(value).is_empty())
    {
        return Err(ModuleError::Validation(String::from(
            "location query must not be blank",
        )));
    }
    Ok(())
}

fn validate_publication(publication: &Publication) -> Result<(), ModuleError> {
    if publication.id.trim().is_empty() {
        return Err(ModuleError::Validation(String::from(
            "publication ID must not be blank",
        )));
    }
    if publication.title.trim().is_empty() {
        return Err(ModuleError::Validation(format!(
            "publication {} has a blank title",
            publication.id
        )));
    }
    if publication.publication_year == 0 {
        return Err(ModuleError::Validation(format!(
            "publication {} has an invalid year",
            publication.id
        )));
    }
    if publication
        .authors
        .iter()
        .any(|author| author.name.trim().is_empty())
    {
        return Err(ModuleError::Validation(format!(
            "publication {} has an author with a blank name",
            publication.id
        )));
    }
    Ok(())
}

fn normalize_publication(mut publication: Publication) -> Publication {
    publication.id = clean_text(&publication.id);
    publication.title = clean_text(&publication.title);
    publication.abstract_text = clean_optional(publication.abstract_text);
    publication.venue = clean_optional(publication.venue);
    publication.doi = publication.doi.map(|doi| {
        let cleaned = clean_text(&doi).to_lowercase();
        cleaned
            .strip_prefix("https://doi.org/")
            .or_else(|| cleaned.strip_prefix("http://doi.org/"))
            .or_else(|| cleaned.strip_prefix("doi:"))
            .unwrap_or(&cleaned)
            .trim()
            .to_owned()
    });
    publication.url = publication
        .url
        .map(|url| url.trim().to_owned())
        .filter(|url| !url.is_empty());
    for author in &mut publication.authors {
        author.id = clean_optional(author.id.take());
        author.name = clean_text(&author.name);
        author.affiliation = clean_optional(author.affiliation.take());
    }
    publication.taxon_names = deduplicate_strings(
        publication
            .taxon_names
            .into_iter()
            .map(|name| clean_text(&name)),
    );
    publication.species_ids.sort_unstable();
    publication.species_ids.dedup();
    for topic in &mut publication.topics {
        if let ResearchTopic::Other(value) = topic {
            *value = clean_text(value);
        }
    }
    let mut seen_topics = HashSet::new();
    publication
        .topics
        .retain(|topic| seen_topics.insert(normalized_topic(topic)));
    let mut seen_locations = HashSet::new();
    publication.locations = publication
        .locations
        .into_iter()
        .map(normalize_location)
        .filter(|location| seen_locations.insert(location_key(location)))
        .collect();
    publication
}

fn normalize_location(mut location: ResearchLocation) -> ResearchLocation {
    location.name = clean_text(&location.name);
    location.province = clean_optional(location.province);
    location.regency = clean_optional(location.regency);
    if let LocationType::Other(value) = &mut location.location_type {
        *value = clean_text(value);
    }
    location
}

fn deduplicate_strings(values: impl IntoIterator<Item = String>) -> Vec<String> {
    let mut seen = HashSet::new();
    values
        .into_iter()
        .filter(|value| !value.is_empty() && seen.insert(normalize_text(value)))
        .collect()
}

fn deduplicate_publications(items: &[Publication]) -> Vec<Publication> {
    deduplicate_references(&items.iter().collect::<Vec<_>>())
        .into_iter()
        .cloned()
        .collect()
}

fn deduplicate_references<'a>(items: &[&'a Publication]) -> Vec<&'a Publication> {
    let mut ids = HashSet::new();
    let mut dois = HashSet::new();
    let mut fingerprints = HashSet::new();
    items
        .iter()
        .copied()
        .filter(|publication| {
            let id = normalized_id(publication);
            let doi = publication
                .doi
                .as_deref()
                .map(normalize_text)
                .filter(|value| !value.is_empty());
            let fingerprint = (
                normalize_text(&publication.title),
                publication.publication_year,
            );
            if ids.contains(&id)
                || doi.as_ref().is_some_and(|doi| dois.contains(doi))
                || fingerprints.contains(&fingerprint)
            {
                return false;
            }
            ids.insert(id);
            if let Some(doi) = doi {
                dois.insert(doi);
            }
            fingerprints.insert(fingerprint);
            true
        })
        .collect()
}

fn normalized_id(publication: &Publication) -> String {
    normalize_text(&publication.id)
}

fn query_publications<'a>(
    items: &'a [Publication],
    query: &PublicationQuery,
) -> Vec<&'a Publication> {
    items
        .iter()
        .filter(|publication| publication_matches(publication, query))
        .collect()
}

fn publication_matches(publication: &Publication, query: &PublicationQuery) -> bool {
    query
        .species_id
        .is_none_or(|id| !publications_for_species(std::slice::from_ref(publication), id).is_empty())
        && query.taxon_name.as_deref().is_none_or(|taxon| {
            !publications_for_taxon(std::slice::from_ref(publication), taxon).is_empty()
        })
        && query.topic.as_ref().is_none_or(|topic| {
            !publications_for_topic(std::slice::from_ref(publication), topic).is_empty()
        })
        && query.location.as_deref().is_none_or(|location| {
            !publications_for_location(std::slice::from_ref(publication), location).is_empty()
        })
        && query
            .year_from
            .is_none_or(|year| publication.publication_year >= year)
        && query
            .year_to
            .is_none_or(|year| publication.publication_year <= year)
        && query
            .source_type
            .as_ref()
            .is_none_or(|source_type| &publication.source_type == source_type)
        && query
            .text
            .as_deref()
            .is_none_or(|text| publication_matches_text(publication, text))
}

fn publications_for_species(
    items: &[Publication],
    species_id: u64,
) -> Vec<&Publication> {
    items
        .iter()
        .filter(|publication| publication.species_ids.contains(&species_id))
        .collect()
}

fn publications_for_taxon<'a>(items: &'a [Publication], taxon: &str) -> Vec<&'a Publication> {
    let query = normalize_text(taxon);
    items
        .iter()
        .filter(|publication| {
            publication
                .taxon_names
                .iter()
                .any(|name| normalize_text(name).contains(&query))
        })
        .collect()
}

fn publications_for_topic<'a>(
    items: &'a [Publication],
    topic: &ResearchTopic,
) -> Vec<&'a Publication> {
    items
        .iter()
        .filter(|publication| {
            publication
                .topics
                .iter()
                .any(|item| normalized_topic(item) == normalized_topic(topic))
        })
        .collect()
}

fn publications_for_location<'a>(items: &'a [Publication], location: &str) -> Vec<&'a Publication> {
    let query = normalize_text(location);
    items
        .iter()
        .filter(|publication| {
            publication.locations.iter().any(|item| {
                std::iter::once(item.name.as_str())
                    .chain(item.province.iter().map(String::as_str))
                    .chain(item.regency.iter().map(String::as_str))
                    .any(|value| normalize_text(value) == query)
            })
        })
        .collect()
}

fn publication_matches_text(publication: &Publication, query: &str) -> bool {
    let fields = searchable_fields(publication);
    let tokens = normalize_text(query)
        .split_whitespace()
        .map(str::to_owned)
        .collect::<Vec<_>>();
    !tokens.is_empty()
        && tokens.iter().all(|token| {
            fields
                .iter()
                .any(|field| normalize_text(field).contains(token))
        })
}

fn searchable_fields(publication: &Publication) -> Vec<String> {
    let mut fields = vec![publication.title.clone()];
    fields.extend(publication.abstract_text.iter().cloned());
    fields.extend(publication.authors.iter().map(|author| author.name.clone()));
    fields.extend(publication.venue.iter().cloned());
    fields.extend(publication.taxon_names.iter().cloned());
    fields.extend(publication.topics.iter().map(topic_label));
    for location in &publication.locations {
        fields.push(location.name.clone());
        fields.extend(location.province.iter().cloned());
        fields.extend(location.regency.iter().cloned());
    }
    fields
}

fn build_research_timeline(items: &[Publication]) -> BTreeMap<u16, usize> {
    build_timeline(items, |publication| publication.publication_year)
}

fn calculate_topic_distribution(items: &[Publication]) -> HashMap<ResearchTopic, usize> {
    let topics = items
        .iter()
        .flat_map(|publication| publication.topics.iter().map(canonical_topic))
        .collect::<Vec<_>>();
    count_by_key(&topics, Clone::clone)
}

fn calculate_location_distribution(items: &[Publication]) -> HashMap<String, usize> {
    let locations = items
        .iter()
        .flat_map(|publication| {
            publication
                .locations
                .iter()
                .map(|location| normalize_text(&location.name))
        })
        .collect::<Vec<_>>();
    count_by_key(&locations, Clone::clone)
}

fn calculate_coverage_score(available: usize, total: usize) -> f64 {
    calculate_coverage_percentage(available, total)
}

fn find_understudied_species(
    coverage: &[CoverageRecord],
    threshold: usize,
) -> Vec<&CoverageRecord> {
    let threshold_f64 = threshold as f64;
    filter_by_threshold(
        coverage
            .iter()
            .map(|record| (record, -(record.publication_count as f64)))
            .collect(),
        -threshold_f64,
    )
    .into_iter()
    .map(|(record, _)| record)
    .collect()
}

fn canonical_topic(topic: &ResearchTopic) -> ResearchTopic {
    match topic {
        ResearchTopic::Other(value) => ResearchTopic::Other(normalize_text(value)),
        _ => topic.clone(),
    }
}

fn calculate_entity_coverage(items: &[Publication], species_ids: &[u64]) -> Vec<CoverageRecord> {
    let species_index = build_species_index(items);
    let publication_years = items
        .iter()
        .map(|publication| (publication.id.as_str(), publication.publication_year))
        .collect::<HashMap<_, _>>();
    let ids = if species_ids.is_empty() {
        items
            .iter()
            .flat_map(|publication| publication.species_ids.iter().copied())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>()
    } else {
        species_ids
            .iter()
            .copied()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    };

    ids.into_iter()
        .map(|species_id| {
            let publication_ids = species_index
                .get(&species_id)
                .map(Vec::as_slice)
                .unwrap_or_default();
            let years = publication_ids
                .iter()
                .filter_map(|id| publication_years.get(id.as_str()).copied())
                .collect::<Vec<_>>();
            CoverageRecord {
                entity_name: species_id.to_string(),
                publication_count: publication_ids.len(),
                species_count: 1,
                first_year: years.iter().copied().min(),
                latest_year: years.into_iter().max(),
            }
        })
        .collect()
}

fn build_species_index(items: &[Publication]) -> HashMap<u64, Vec<String>> {
    let mut index = HashMap::new();
    for publication in items {
        for species_id in &publication.species_ids {
            index
                .entry(*species_id)
                .or_insert_with(Vec::new)
                .push(publication.id.clone());
        }
    }
    index
}

fn score_publication_relevance(
    publication: &Publication,
    query: &PublicationQuery,
) -> f64 {
    let mut weighted = Vec::new();
    if let Some(species_id) = query.species_id {
        weighted.push((
            f64::from(publication.species_ids.contains(&species_id)),
            4.0,
        ));
    }
    if let Some(text) = query.text.as_deref() {
        weighted.push((text_relevance(publication, text), 5.0));
    }
    if query.taxon_name.is_some() {
        weighted.push((1.0, 3.0));
    }
    if query.topic.is_some() {
        weighted.push((1.0, 3.0));
    }
    if query.location.is_some() {
        weighted.push((1.0, 3.0));
    }
    if query.year_from.is_some() || query.year_to.is_some() {
        weighted.push((1.0, 1.0));
    }
    if query.source_type.is_some() {
        weighted.push((1.0, 1.0));
    }
    let total_weight = weighted.iter().map(|(_, weight)| weight).sum::<f64>();
    if total_weight == 0.0 {
        return 0.0;
    }
    weighted
        .iter()
        .map(|(score, weight)| score * weight)
        .sum::<f64>()
        / total_weight
}

fn text_relevance(publication: &Publication, query: &str) -> f64 {
    let tokens = normalize_text(query)
        .split_whitespace()
        .map(str::to_owned)
        .collect::<Vec<_>>();
    if tokens.is_empty() {
        return 0.0;
    }
    let title = normalize_text(&publication.title);
    let abstract_text = publication
        .abstract_text
        .as_deref()
        .map(normalize_text)
        .unwrap_or_default();
    let other_fields = searchable_fields(publication)
        .into_iter()
        .skip(1)
        .map(|field| normalize_text(&field))
        .collect::<Vec<_>>();
    let total = tokens
        .iter()
        .map(|token| {
            if title.contains(token) {
                3.0
            } else if abstract_text.contains(token) {
                2.0
            } else if other_fields.iter().any(|field| field.contains(token)) {
                1.0
            } else {
                0.0
            }
        })
        .sum::<f64>();
    total / (tokens.len() as f64 * 3.0)
}

fn rank_publications<'a>(
    items: &[&'a Publication],
    query: &PublicationQuery,
) -> Vec<(&'a Publication, f64)> {
    let scored = items
        .iter()
        .map(|publication| {
            (
                *publication,
                score_publication_relevance(publication, query),
            )
        })
        .collect::<Vec<_>>();
    let mut ranked = rank_by_score(scored);
    ranked.sort_by(|(left, left_score), (right, right_score)| {
        right_score
            .total_cmp(left_score)
            .then_with(|| right.publication_year.cmp(&left.publication_year))
            .then_with(|| {
                right
                    .citation_count
                    .unwrap_or_default()
                    .cmp(&left.citation_count.unwrap_or_default())
            })
            .then_with(|| left.id.cmp(&right.id))
    });
    ranked
}

fn build_knowledge_graph(items: &[Publication]) -> KnowledgeGraph {
    let mut nodes = BTreeMap::<String, KnowledgeNode>::new();
    let mut edges = BTreeSet::<(String, String, String)>::new();
    for publication in items {
        let publication_id = format!("publication:{}", publication.id);
        nodes.insert(
            publication_id.clone(),
            KnowledgeNode {
                id: publication_id.clone(),
                kind: KnowledgeNodeKind::Publication,
                label: publication.title.clone(),
            },
        );
        for species_id in &publication.species_ids {
            let id = format!("species:{species_id}");
            nodes.entry(id.clone()).or_insert_with(|| KnowledgeNode {
                id: id.clone(),
                kind: KnowledgeNodeKind::Species,
                label: species_id.to_string(),
            });
            edges.insert((id, publication_id.clone(), String::from("discussed_in")));
        }
        for topic in &publication.topics {
            let label = topic_label(topic);
            let id = format!("topic:{}", normalize_text(&label));
            nodes.entry(id.clone()).or_insert_with(|| KnowledgeNode {
                id: id.clone(),
                kind: KnowledgeNodeKind::Topic,
                label,
            });
            edges.insert((publication_id.clone(), id, String::from("about")));
        }
        for location in &publication.locations {
            let label = location.name.clone();
            let id = format!("location:{}", normalize_text(&label));
            nodes.entry(id.clone()).or_insert_with(|| KnowledgeNode {
                id: id.clone(),
                kind: KnowledgeNodeKind::Location,
                label,
            });
            edges.insert((publication_id.clone(), id, String::from("studied_at")));
        }
        for author in &publication.authors {
            let label = author.name.clone();
            let identity = author
                .id
                .as_deref()
                .map(normalize_text)
                .unwrap_or_else(|| normalize_text(&label));
            let id = format!("researcher:{identity}");
            nodes.entry(id.clone()).or_insert_with(|| KnowledgeNode {
                id: id.clone(),
                kind: KnowledgeNodeKind::Researcher,
                label,
            });
            edges.insert((publication_id.clone(), id, String::from("written_by")));
        }
    }
    KnowledgeNetwork {
        nodes: nodes.into_values().collect(),
        edges: edges
            .into_iter()
            .map(|(source, target, relation)| KnowledgeEdge {
                source,
                target,
                relation,
            })
            .collect(),
    }
}

/// Find publications that share species, topics, study locations, or researchers
/// with the given publication in the knowledge network.
///
/// # Arguments
///
/// * `graph` - The biodiversity knowledge network.
/// * `publication_id` - Identifier of the source publication.
/// * `items` - Candidate publication records.
///
/// # Returns
///
/// References to related publications from `items`.
pub fn related_publications<'a>(
    graph: &KnowledgeGraph,
    publication_id: &str,
    items: &'a [Publication],
) -> Vec<&'a Publication> {
    let pub_node_id = format!("publication:{publication_id}");
    let mut connected_entities = HashSet::new();
    for edge in &graph.edges {
        if edge.target == pub_node_id {
            connected_entities.insert(&edge.source);
        } else if edge.source == pub_node_id {
            connected_entities.insert(&edge.target);
        }
    }
    if connected_entities.is_empty() {
        return Vec::new();
    }
    let mut related_pub_ids = HashSet::new();
    for edge in &graph.edges {
        if edge.target != pub_node_id
            && connected_entities.contains(&edge.source)
            && edge.target.starts_with("publication:")
        {
            if let Some(other_id) = edge.target.strip_prefix("publication:") {
                related_pub_ids.insert(other_id);
            }
        } else if edge.source != pub_node_id
            && connected_entities.contains(&edge.target)
            && edge.source.starts_with("publication:")
        {
            if let Some(other_id) = edge.source.strip_prefix("publication:") {
                related_pub_ids.insert(other_id);
            }
        }
    }
    items
        .iter()
        .filter(|publication| related_pub_ids.contains(publication.id.as_str()))
        .collect()
}

fn format_apa(publication: &Publication) -> String {
    let authors = publication
        .authors
        .iter()
        .map(|author| clean_text(&author.name))
        .filter(|name| !name.is_empty())
        .collect::<Vec<_>>()
        .join(", ");
    let mut parts = Vec::new();
    if !authors.is_empty() {
        parts.push(authors);
    }
    parts.push(format!("({}).", publication.publication_year));
    parts.push(format!("{}.", clean_text(&publication.title)));
    if let Some(venue) = publication
        .venue
        .as_deref()
        .map(clean_text)
        .filter(|value| !value.is_empty())
    {
        parts.push(format!("{venue}."));
    }
    if let Some(doi) = publication
        .doi
        .as_deref()
        .map(clean_text)
        .filter(|value| !value.is_empty())
    {
        parts.push(format!("https://doi.org/{doi}"));
    }
    if let Some(url) = publication
        .url
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        parts.push(url.to_owned());
    }
    parts.join(" ")
}

fn format_bibtex(publication: &Publication) -> String {
    let entry_type = match &publication.source_type {
        SourceType::JournalArticle => "article",
        SourceType::ConferencePaper => "inproceedings",
        SourceType::Thesis => "phdthesis",
        SourceType::Report => "techreport",
        SourceType::BookChapter => "incollection",
        SourceType::Dataset => "misc",
    };
    let venue_field = match &publication.source_type {
        SourceType::JournalArticle => "journal",
        SourceType::ConferencePaper | SourceType::BookChapter => "booktitle",
        SourceType::Thesis => "school",
        SourceType::Report => "institution",
        SourceType::Dataset => "howpublished",
    };
    let author = publication
        .authors
        .iter()
        .map(|author| escape_bibtex(&clean_text(&author.name)))
        .collect::<Vec<_>>()
        .join(" and ");
    let key_seed = publication
        .authors
        .first()
        .map(|author| author.name.as_str())
        .unwrap_or(&publication.id);
    let author_key = key_seed
        .split_whitespace()
        .last()
        .unwrap_or("publication")
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .collect::<String>()
        .to_lowercase();
    let id_key = publication
        .id
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .collect::<String>();
    let key = format!(
        "{}{}{}",
        if author_key.is_empty() {
            "publication"
        } else {
            &author_key
        },
        publication.publication_year,
        id_key
    );
    let mut fields = Vec::new();
    if !author.is_empty() {
        fields.push(format!("  author = {{{author}}}"));
    }
    fields.push(format!(
        "  title = {{{}}}",
        escape_bibtex(&clean_text(&publication.title))
    ));
    fields.push(format!("  year = {{{}}}", publication.publication_year));
    if let Some(venue) = publication
        .venue
        .as_deref()
        .map(clean_text)
        .filter(|value| !value.is_empty())
    {
        fields.push(format!("  {venue_field} = {{{}}}", escape_bibtex(&venue)));
    }
    if let Some(doi) = publication
        .doi
        .as_deref()
        .map(clean_text)
        .filter(|value| !value.is_empty())
    {
        fields.push(format!("  doi = {{{}}}", escape_bibtex(&doi)));
    }
    if let Some(url) = publication
        .url
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        fields.push(format!("  url = {{{}}}", escape_bibtex(url)));
    }
    if let Some(abstract_text) = publication
        .abstract_text
        .as_deref()
        .map(clean_text)
        .filter(|value| !value.is_empty())
    {
        fields.push(format!(
            "  abstract = {{{}}}",
            escape_bibtex(&abstract_text)
        ));
    }
    format!("@{entry_type}{{{key},\n{}\n}}", fields.join(",\n"))
}

fn escape_bibtex(value: &str) -> String {
    value
        .chars()
        .map(|character| match character {
            '\\' => String::from("\\textbackslash{}"),
            '{' => String::from("\\{"),
            '}' => String::from("\\}"),
            '$' => String::from("\\$"),
            '&' => String::from("\\&"),
            '%' => String::from("\\%"),
            '#' => String::from("\\#"),
            '_' => String::from("\\_"),
            '~' => String::from("\\textasciitilde{}"),
            '^' => String::from("\\textasciicircum{}"),
            other => other.to_string(),
        })
        .collect()
}

fn topic_label(topic: &ResearchTopic) -> String {
    match topic {
        ResearchTopic::Taxonomy => String::from("Taxonomy"),
        ResearchTopic::Ecology => String::from("Ecology"),
        ResearchTopic::Conservation => String::from("Conservation"),
        ResearchTopic::Ethnobotany => String::from("Ethnobotany"),
        ResearchTopic::Habitat => String::from("Habitat"),
        ResearchTopic::SpeciesIdentification => String::from("Species identification"),
        ResearchTopic::Other(value) => clean_text(value),
    }
}

fn normalized_topic(topic: &ResearchTopic) -> String {
    normalize_text(&topic_label(topic))
}

fn location_key(location: &ResearchLocation) -> String {
    format!(
        "{}|{}|{}|{}",
        normalize_text(&location.name),
        location
            .province
            .as_deref()
            .map(normalize_text)
            .unwrap_or_default(),
        location
            .regency
            .as_deref()
            .map(normalize_text)
            .unwrap_or_default(),
        match &location.location_type {
            LocationType::Province => String::from("province"),
            LocationType::Regency => String::from("regency"),
            LocationType::Forest => String::from("forest"),
            LocationType::ConservationArea => String::from("conservation-area"),
            LocationType::Other(value) => normalize_text(value),
        }
    )
}

fn clean_text(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn clean_optional(value: Option<String>) -> Option<String> {
    value
        .map(|value| clean_text(&value))
        .filter(|value| !value.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Author;

    fn publication(
        id: &str,
        title: &str,
        year: u16,
        species_ids: Vec<u64>,
        topics: Vec<ResearchTopic>,
    ) -> Publication {
        Publication {
            id: id.to_owned(),
            title: title.to_owned(),
            abstract_text: None,
            publication_year: year,
            authors: vec![Author {
                id: None,
                name: String::from("A. Researcher"),
                affiliation: None,
            }],
            venue: Some(String::from("Borneo Journal")),
            doi: None,
            url: None,
            source_type: SourceType::JournalArticle,
            species_ids,
            taxon_names: vec![String::from("Shorea")],
            topics,
            locations: vec![ResearchLocation {
                name: String::from("Gunung Palung"),
                province: Some(String::from("Kalimantan Barat")),
                regency: None,
                location_type: LocationType::ConservationArea,
            }],
            citation_count: Some(2),
        }
    }

    #[test]
    fn explores_and_reports_timeline_coverage_and_network() {
        let items = vec![
            publication(
                "p1",
                "Forest ecology",
                2020,
                vec![1, 1],
                vec![ResearchTopic::Ecology],
            ),
            publication(
                "p2",
                "Forest conservation",
                2022,
                vec![1],
                vec![ResearchTopic::Conservation],
            ),
        ];
        let query = KnowledgeQuery {
            species_ids: vec![1, 2],
            ..KnowledgeQuery::default()
        };

        let report = explore_knowledge(&query, &items).expect("valid exploration");
        assert_eq!(report.ranked_publications.len(), 2);
        assert_eq!(report.timeline.get(&2020), Some(&1));
        assert_eq!(report.timeline.get(&2022), Some(&1));
        assert_eq!(report.topic_distribution.get("Ecology"), Some(&1));
        assert_eq!(report.location_distribution.get("gunung palung"), Some(&2));
        assert_eq!(report.coverage_score, 0.5);
        assert_eq!(report.understudied_species, vec![2]);
        assert!(report.graph.nodes.iter().any(|node| node.id == "species:1"));
        assert!(report
            .graph
            .edges
            .iter()
            .any(|edge| edge.relation == "discussed_in"));
        assert!(serde_json::to_string(&report).is_ok());
    }

    #[test]
    fn scenario_1_species_to_publication_explorer() {
        let items = vec![
            publication("p1", "Orangutan study", 2020, vec![1], vec![]),
            publication("p2", "Clouded leopard survey", 2022, vec![4], vec![]),
            publication("p3", "Carnivore ecology", 2023, vec![4, 2], vec![]),
        ];

        let results = publications_for_species(&items, 4);
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|p| p.species_ids.contains(&4)));
        assert_eq!(results[0].id, "p2");
        assert_eq!(results[1].id, "p3");
    }

    #[test]
    fn scenario_2_research_topic_explorer() {
        let items = vec![
            publication("p1", "Ecology paper", 2020, vec![1], vec![ResearchTopic::Ecology]),
            publication("p2", "Conservation paper 1", 2021, vec![1], vec![ResearchTopic::Conservation]),
            publication("p3", "Conservation paper 2", 2022, vec![2], vec![ResearchTopic::Conservation]),
        ];

        let conservation = publications_for_topic(&items, &ResearchTopic::Conservation);
        assert_eq!(conservation.len(), 2);
        assert!(conservation.iter().all(|p| p.topics.contains(&ResearchTopic::Conservation)));
        // Check immutability of items
        assert_eq!(items.len(), 3);
    }

    #[test]
    fn scenario_3_research_location_explorer() {
        let p_barat = publication("p1", "West Borneo study", 2020, vec![1], vec![]);
        let mut p_selatan = publication("p2", "South Borneo study", 2021, vec![1], vec![]);
        p_selatan.locations = vec![ResearchLocation {
            name: String::from("Meratus"),
            province: Some(String::from("Kalimantan Selatan")),
            regency: None,
            location_type: LocationType::Forest,
        }];
        let items = vec![p_barat, p_selatan];

        let results = publications_for_location(&items, "Kalimantan Barat");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "p1");
    }

    #[test]
    fn scenario_4_coverage_and_understudied_species() {
        let items = vec![
            publication("p1", "Paper 1", 2020, vec![1, 2], vec![]),
            publication("p2", "Paper 2", 2021, vec![1], vec![]),
        ];
        let coverage = calculate_entity_coverage(&items, &[1, 2, 3]);
        assert_eq!(coverage.len(), 3);

        // Species 1 has 2 pubs, Species 2 has 1 pub, Species 3 has 0 pubs.
        // Threshold = 1: species with 0 or 1 publication are understudied.
        let understudied = find_understudied_species(&coverage, 1);
        let names = understudied
            .iter()
            .map(|r| r.entity_name.as_str())
            .collect::<Vec<_>>();
        assert_eq!(names, vec!["2", "3"]);
    }

    #[test]
    fn scenario_5_citation_export() {
        let mut item = publication("p_1", "Forest & Habitat in Borneo", 2020, vec![], vec![]);
        item.venue = Some(String::from("Journal of Asian Ecology"));
        item.doi = Some(String::from("10.1234/example"));
        let apa = export_citations(std::slice::from_ref(&item), CitationFormat::Apa);
        let bibtex = export_citations(std::slice::from_ref(&item), CitationFormat::Bibtex);

        assert!(apa.contains("Forest & Habitat in Borneo"));
        assert!(apa.contains("(2020)."));
        assert!(apa.contains("Journal of Asian Ecology."));
        assert!(apa.contains("https://doi.org/10.1234/example"));

        assert!(bibtex.contains("@article{"));
        assert!(bibtex.contains("Forest \\& Habitat in Borneo"));
        assert!(bibtex.contains("year = {2020}"));
        assert!(bibtex.contains("journal = {Journal of Asian Ecology}"));
        assert!(bibtex.contains("doi = {10.1234/example}"));
    }

    #[test]
    fn scenario_6_knowledge_network_and_related_publications() {
        let p1 = publication(
            "p1",
            "Orangutan Diet",
            2020,
            vec![1],
            vec![ResearchTopic::Ecology],
        );
        let p2 = publication(
            "p2",
            "Orangutan Ranging",
            2022,
            vec![1],
            vec![ResearchTopic::Habitat],
        );
        let p3 = publication(
            "p3",
            "Rafflesia Pollination",
            2023,
            vec![5],
            vec![ResearchTopic::Ecology],
        );
        let items = vec![p1, p2, p3];

        let graph = build_knowledge_graph(&items);
        assert!(graph.nodes.iter().any(|node| node.id == "species:1"));
        assert!(graph.nodes.iter().any(|node| node.id == "topic:ecology"));

        // Related to p1 via species:1 is p2; via topic:ecology is p3
        let related_to_p1 = related_publications(&graph, "p1", &items);
        let related_ids = related_to_p1.iter().map(|p| p.id.as_str()).collect::<Vec<_>>();
        assert!(related_ids.contains(&"p2"));
        assert!(related_ids.contains(&"p3"));
    }

    #[test]
    fn parses_fixture_publications() {
        let json_data = include_str!("../fixtures/publications.json");
        let parsed = parse_publications(json_data).expect("valid publications JSON");
        assert_eq!(parsed.len(), 4);
        assert_eq!(parsed[0].id, "pub-001");
        assert_eq!(parsed[1].id, "pub-002");
    }

    #[test]
    fn filters_and_ranks_text_matches_deterministically() {
        let exact_title = publication("b", "Orangutan habitat study", 2021, vec![3], vec![]);
        let abstract_match = Publication {
            abstract_text: Some(String::from("Orangutan habitat observations")),
            ..publication("a", "Wildlife notes", 2023, vec![3], vec![])
        };
        let items = vec![exact_title, abstract_match];
        let query = PublicationQuery {
            text: Some(String::from("orangutan habitat")),
            ..PublicationQuery::default()
        };

        let recommended = recommend_citations(&items, &query, 1);
        assert_eq!(recommended.len(), 1);
        assert_eq!(recommended[0].id, "b");
    }

    #[test]
    fn matches_locations_by_province_and_deduplicates_records() {
        let duplicate = publication(
            "other-id",
            "Forest ecology",
            2020,
            vec![1],
            vec![ResearchTopic::Ecology],
        );
        let items = vec![
            publication(
                "p1",
                "Forest ecology",
                2020,
                vec![1],
                vec![ResearchTopic::Ecology],
            ),
            duplicate,
        ];
        let query = KnowledgeQuery {
            location: Some(String::from("KALIMANTAN BARAT")),
            ..KnowledgeQuery::default()
        };

        let report = explore_knowledge(&query, &items).expect("valid exploration");
        assert_eq!(report.ranked_publications.len(), 1);
        assert_eq!(report.timeline.values().sum::<usize>(), 1);
    }

    #[test]
    fn rejects_invalid_year_range_and_empty_dataset() {
        let invalid_range = KnowledgeQuery {
            year_from: Some(2024),
            year_to: Some(2020),
            ..KnowledgeQuery::default()
        };
        assert!(matches!(
            explore_knowledge(
                &invalid_range,
                &[publication("p1", "Title", 2021, vec![], vec![])]
            ),
            Err(ModuleError::Validation(_))
        ));
        assert!(matches!(
            explore_knowledge(&KnowledgeQuery::default(), &[]),
            Err(ModuleError::Empty(_))
        ));
    }

    #[test]
    fn citation_exports_preserve_metadata_and_escape_bibtex() {
        let mut item = publication("p_1", "Forest & habitat", 2020, vec![], vec![]);
        item.doi = Some(String::from("10.1234/example"));
        let apa = export_citations(std::slice::from_ref(&item), CitationFormat::Apa);
        let bibtex = export_citations(std::slice::from_ref(&item), CitationFormat::Bibtex);
        assert!(apa.contains("Forest & habitat"));
        assert!(apa.contains("https://doi.org/10.1234/example"));
        assert!(bibtex.contains("Forest \\& habitat"));
        assert!(bibtex.contains("doi = {10.1234/example}"));
    }

    #[test]
    fn explores_knowledge_from_input_struct() {
        let json_data = include_str!("../fixtures/publications.json");
        let input = KnowledgeInput {
            raw_publications: json_data.to_owned(),
            query: KnowledgeQuery {
                species_ids: vec![1],
                ..KnowledgeQuery::default()
            },
        };
        let report = explore_knowledge_from_input(&input).expect("valid pipeline execution");
        assert_eq!(report.ranked_publications.len(), 2);
    }

    #[tokio::test]
    async fn fetch_publications_async_adapter_scenario() {
        async fn fetch_publications(source: &str) -> Result<Vec<Publication>, ModuleError> {
            if source.is_empty() {
                return Err(ModuleError::Validation(String::from("source cannot be empty")));
            }
            let data = include_str!("../fixtures/publications.json");
            parse_publications(data)
        }

        let pubs = fetch_publications("local://fixtures").await.expect("async fetch succeeds");
        assert_eq!(pubs.len(), 4);
    }

    #[test]
    fn publications_for_species_entity_matches_shared_species() {
        use kalimantanbio_shared::core::Taxonomy;

        let species = Species {
            id: 1,
            scientific_name: String::from("Pongo pygmaeus"),
            common_name: String::from("Bornean Orangutan"),
            description: String::new(),
            species_type: String::from("Endemic"),
            iucn: String::from("CR"),
            cites: String::from("Appendix I"),
            p106: String::from("Dilindungi"),
            is_verified: true,
            image_url: None,
            taxonomy: Taxonomy {
                kingdom: String::from("Animalia"),
                kingdom_id: 1,
                phylum_division: String::from("Chordata"),
                phylum_division_id: 2,
                class: String::from("Mammalia"),
                class_id: 3,
                order: String::from("Primates"),
                order_id: 4,
                family: String::from("Hominidae"),
                family_id: 5,
                genus: String::from("Pongo"),
                genus_id: 6,
            },
            observation_count: 10,
            recorded_individuals_total: 20,
            latest_observation_year: Some(2024),
            created_at: String::new(),
            updated_at: String::new(),
        };

        let items = vec![
            publication("p1", "Orangutan ecology", 2021, vec![1], vec![]),
            publication("p2", "Clouded leopard", 2022, vec![4], vec![]),
        ];

        let matched = publications_for_species_entity(&items, &species);
        assert_eq!(matched.len(), 1);
        assert_eq!(matched[0].id, "p1");
    }

    #[test]
    fn publication_score_falls_back_to_year_then_id() {
        let items = vec![
            publication("z", "Older title", 2020, vec![], vec![]),
            publication("b", "Beta title", 2022, vec![], vec![]),
            publication("a", "Alpha title", 2022, vec![], vec![]),
        ];
        let query = PublicationQuery::default();
        let recommended = recommend_citations(&items, &query, 3);
        assert_eq!(
            recommended
                .iter()
                .map(|item| item.id.as_str())
                .collect::<Vec<_>>(),
            vec!["a", "b", "z"]
        );
    }
}
