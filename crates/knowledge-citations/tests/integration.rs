//! Integration tests for knowledge-citations crate.

use knowledge_citations::types::{
    CitationFormat, KnowledgeInput, KnowledgeQuery, Publication, PublicationQuery, ResearchTopic,
};
use knowledge_citations::{
    explore_knowledge_from_input, export_citations, parse_publications, recommend_citations,
};

#[test]
fn test_end_to_end_knowledge_exploration_flow() {
    let fixture_raw = include_str!("../fixtures/publications.json");
    let input = KnowledgeInput {
        raw_publications: fixture_raw.to_owned(),
        query: KnowledgeQuery {
            species_ids: vec![1, 4, 5],
            understudied_threshold: Some(1),
            ..KnowledgeQuery::default()
        },
    };

    let report = explore_knowledge_from_input(&input).expect("exploration succeeded");

    // All publications in the fixture (pub-001, pub-002, pub-003, pub-004) match the queried species
    assert_eq!(report.ranked_publications.len(), 4);

    // Verify timeline
    assert_eq!(report.timeline.get(&2021), Some(&1));
    assert_eq!(report.timeline.get(&2022), Some(&1));
    assert_eq!(report.timeline.get(&2023), Some(&1));
    assert_eq!(report.timeline.get(&2024), Some(&1));

    // Verify topic distribution
    assert!(report.topic_distribution.contains_key("Ecology"));
    assert!(report.topic_distribution.contains_key("Conservation"));

    // Verify understudied species (species 4 has 1 pub, species 5 has 1 pub -> <= 1 threshold)
    assert!(report.understudied_species.contains(&4));
    assert!(report.understudied_species.contains(&5));

    // Verify graph construction
    assert!(report.graph.nodes.iter().any(|node| node.id == "species:1"));
    assert!(report.graph.nodes.iter().any(|node| node.id == "publication:pub-001"));
}

#[test]
fn test_recommend_and_export_citations() {
    let fixture_raw = include_str!("../fixtures/publications.json");
    let publications = parse_publications(fixture_raw).expect("valid publications");

    let query = PublicationQuery {
        topic: Some(ResearchTopic::Conservation),
        ..PublicationQuery::default()
    };

    let recommended = recommend_citations(&publications, &query, 2);
    assert_eq!(recommended.len(), 2);

    let owned_recommended: Vec<Publication> = recommended.into_iter().cloned().collect();

    let apa = export_citations(&owned_recommended, CitationFormat::Apa);
    assert!(apa.contains("https://doi.org/"));

    let bibtex = export_citations(&owned_recommended, CitationFormat::Bibtex);
    assert!(bibtex.contains("@article{"));
}
