use kalimantanbio_shared::core::{Observation, Species, Taxonomy};
use serde::de::DeserializeOwned;
use std::collections::{HashMap, HashSet};

// Read and parse `crates/shared/fixtures/<file>`.
fn load_file<T: DeserializeOwned>(file: &str) -> T {
    let path = format!("{}/fixtures/{}", env!("CARGO_MANIFEST_DIR"), file);
    let raw_json =
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("Cannot read {path}: {e}"));

    serde_json::from_str(&raw_json).unwrap_or_else(|e| panic!("Cannot parse {path}: {e}"))
}

fn kabupaten_by_species(observations: &[Observation]) -> HashMap<u64, HashSet<u64>> {
    observations
        .iter()
        .map(|o| (o.species_id, o.kabupaten_id))
        .fold(HashMap::new(), |mut acc, (species_id, kabupaten_id)| {
            acc.entry(species_id).or_default().insert(kabupaten_id);
            acc
        })
}

#[test]
fn load_species_returns_all_entries() {
    let species: Vec<Species> = load_file("species.json");
    println!("Species: {:?}", species);
    assert!(!species.is_empty());
}

#[test]
fn species_fixtures_loads_five_species() {
    let species: Vec<Species> = load_file("species.json");
    assert_eq!(species.len(), 5, "Fixture must hold five species");

    let unique_ids: HashSet<u64> = species.iter().map(|s| s.id).collect();

    assert_eq!(
        unique_ids.len(),
        species.len(),
        "Species ids must be unique, got {unique_ids:?}"
    )
}

#[test]
fn exactly_one_same_genus_pair_and_one_same_family_pair() {
    let species: Vec<Species> = load_file("species.json");

    let mut groups_by_genus: HashMap<u64, Vec<u64>> = HashMap::new();
    for s in &species {
        groups_by_genus
            .entry(s.taxonomy.genus_id)
            .or_default()
            .push(s.id);
    }

    let same_genus_groups: Vec<&Vec<u64>> = groups_by_genus
        .values()
        .filter(|ids| ids.len() > 1)
        .collect();

    assert_eq!(
        same_genus_groups.len(),
        1,
        "Expected exactly one group sharing a genus"
    );

    let mut same_genus_ids = same_genus_groups[0].clone();
    same_genus_ids.sort_unstable();

    assert_eq!(
        same_genus_ids,
        vec![1, 3],
        "Expected the Pongo pair 1 and 3"
    );

    let mut groups_by_family: HashMap<u64, Vec<&Species>> = HashMap::new();
    for s in &species {
        groups_by_family
            .entry(s.taxonomy.family_id)
            .or_default()
            .push(s);
    }

    let same_family_groups: Vec<&Vec<&Species>> = groups_by_family
        .values()
        .filter(|members| {
            let distinct_genera: HashSet<u64> =
                members.iter().map(|s| s.taxonomy.genus_id).collect();

            distinct_genera.len() > 1
        })
        .collect();

    assert_eq!(
        same_family_groups.len(),
        1,
        "Expected exactly one family group containing different genera"
    );

    assert_eq!(
        same_family_groups[0].len(),
        2,
        "Expected exactly one pair sharing a family but differing in genus"
    );

    let mut same_family_ids: Vec<u64> = same_family_groups[0].iter().map(|s| s.id).collect();

    same_family_ids.sort_unstable();
    assert_eq!(
        same_family_ids,
        vec![2, 4],
        "Expected the Felidae pair 2 and 4"
    );
}

#[test]
fn species_five_has_no_observations() {
    let observations: Vec<Observation> = load_file("observations.json");

    let count_for = |species_id: u64| {
        observations
            .iter()
            .filter(|o| o.species_id == species_id)
            .count()
    };

    assert_eq!(count_for(5), 0, "Species 5 must have no observation rows");

    for species_id in 1..=4 {
        assert!(
            count_for(species_id) > 0,
            "Species {species_id} must have at least one observation row"
        );
    }
}

#[test]
fn kabupaten_overlap_cases_exist() {
    let observations: Vec<Observation> = load_file("observations.json");
    let by_species = kabupaten_by_species(&observations);

    let set_of = |species_id: u64| -> HashSet<u64> {
        by_species.get(&species_id).cloned().unwrap_or_default()
    };

    let shared = |a: u64, b: u64| -> HashSet<u64> {
        let (set_a, set_b) = (set_of(a), set_of(b));
        set_a.intersection(&set_b).copied().collect()
    };

    assert_eq!(
        shared(1, 3),
        HashSet::from([6201]),
        "Species 1 and 3 must overlap exactly on Kotawaringin Barat"
    );

    assert_eq!(
        shared(2, 4),
        HashSet::from([6402]),
        "Species 2 and 4 must overlap exactly on Kutai Kartanegara"
    );

    assert!(
        shared(1, 2).is_empty(),
        "Species 1 and 2 must not share any kabupaten"
    );

    assert!(
        set_of(5).is_empty(),
        "Species 5 has no observations, so its kabupaten set must be empty"
    );
}

#[test]
fn taxonomy_fixture_mirrors_species_taxonomies() {
    let species: Vec<Species> = load_file("species.json");
    let taxonomies: Vec<Taxonomy> = load_file("taxonomy.json");

    let from_species: HashSet<Taxonomy> = species.iter().map(|s| s.taxonomy.clone()).collect();
    let from_fixture: HashSet<Taxonomy> = taxonomies.into_iter().collect();

    assert_eq!(
        from_species.len(),
        4,
        "Five species must yield four distinct taxonomies"
    );

    assert_eq!(
        from_species, from_fixture,
        "taxonomy.json must mirror every distinct taxonomy in species.json"
    );
}

#[test]
fn unknown_conservation_value_and_valid_species_type() {
    let species: Vec<Species> = load_file("species.json");

    let unknown_cites: Vec<u64> = species
        .iter()
        .filter(|s| s.cites == "-")
        .map(|s| s.id)
        .collect();

    assert_eq!(
        unknown_cites,
        vec![5],
        "Only species 5 must carry the unknown value cites = \"-\""
    );

    let allowed_types = ["Endemic", "Native", "Introduced"];

    for s in &species {
        assert!(
            allowed_types.contains(&s.species_type.as_str()),
            "Species {} has invalid species_type '{}'",
            s.id,
            s.species_type
        );
    }
}
