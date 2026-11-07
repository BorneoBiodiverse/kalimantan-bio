//! Stage 1 — Query Parsing and Filter Extraction.
//!
//! Converts raw search tokens into a structured [`QueryFilters`] instance.

use crate::types::QueryFilters;

/// Classification of a search token.
#[derive(Debug, PartialEq, Eq)]
enum TokenKind {
    /// IUCN conservation status code (e.g. "CR", "EN", "VU").
    Iucn(String),
    /// Indonesian national protection status P.106 (e.g. "Dilindungi").
    P106(String),
    /// Species origin category (e.g. "Endemic", "Native", "Introduced").
    SpeciesType(String),
    /// CITES appendix designation (e.g. "Appendix I").
    Cites(String),
    /// Taxonomic kingdom identifier.
    KingdomId(u64),
    /// Taxonomic class identifier.
    ClassId(u64),
    /// Taxonomic kingdom name (e.g. "Animalia", "Plantae").
    Kingdom(String),
    /// Taxonomic class / category name (e.g. "Mammalia", "Reptilia", "Aves").
    Class(String),
    /// Taxonomic order name.
    Order(String),
    /// Taxonomic family name.
    Family(String),
    /// Taxonomic genus name.
    Genus(String),
    /// Record verification status.
    IsVerified(bool),
    /// Free-text keyword.
    FreeText(String),
}

/// Accumulator state used during functional folding over tokens.
#[derive(Default)]
struct ParseState {
    filters: QueryFilters,
    pending_negation: bool,
}

/// Maps a raw IUCN string to its standard uppercase representation.
fn normalize_iucn_code(val: &str) -> Option<String> {
    match val.to_lowercase().as_str() {
        "cr" | "critically-endangered" | "critically_endangered" | "kritis" => {
            Some("CR".to_string())
        }
        "en" | "endangered" | "genting" => Some("EN".to_string()),
        "vu" | "vulnerable" | "rentan" => Some("VU".to_string()),
        "nt" | "near-threatened" | "near_threatened" | "hampir-terancam" => {
            Some("NT".to_string())
        }
        "lc" | "least-concern" | "least_concern" | "risiko-rendah" => Some("LC".to_string()),
        "dd" | "data-deficient" | "data_deficient" | "kurang-data" => Some("DD".to_string()),
        "ew" | "extinct-in-the-wild" | "punah-di-alam" => Some("EW".to_string()),
        "ex" | "extinct" | "punah" => Some("EX".to_string()),
        other if other.len() == 2 && other.chars().all(|c| c.is_ascii_alphabetic()) => {
            Some(other.to_uppercase())
        }
        _ => None,
    }
}

/// Maps a raw CITES designation to standard representation.
fn normalize_cites(val: &str) -> Option<String> {
    match val.to_lowercase().as_str() {
        "i" | "1" | "appendix-i" | "appendix_i" | "appendix-1" => {
            Some("Appendix I".to_string())
        }
        "ii" | "2" | "appendix-ii" | "appendix_ii" | "appendix-2" => {
            Some("Appendix II".to_string())
        }
        "iii" | "3" | "appendix-iii" | "appendix_iii" | "appendix-3" => {
            Some("Appendix III".to_string())
        }
        other if !other.is_empty() => Some(other.to_string()),
        _ => None,
    }
}

/// Maps a raw species type to standard representation.
fn normalize_species_type(val: &str) -> Option<String> {
    match val.to_lowercase().as_str() {
        "endemik" | "endemic" => Some("Endemic".to_string()),
        "asli" | "native" => Some("Native".to_string()),
        "introduksi" | "introduced" => Some("Introduced".to_string()),
        other if !other.is_empty() => Some(other.to_string()),
        _ => None,
    }
}

/// Maps a raw P.106 protection value to standard representation.
fn normalize_p106(val: &str) -> Option<String> {
    match val.to_lowercase().as_str() {
        "dilindungi" | "protected" | "true" | "ya" => Some("Dilindungi".to_string()),
        "tidak-dilindungi" | "tidak_dilindungi" | "unprotected" | "false" | "tidak" | "bebas" => {
            Some("Tidak Dilindungi".to_string())
        }
        other if !other.is_empty() => Some(other.to_string()),
        _ => None,
    }
}

/// Maps a raw kingdom name (multilingual) to standard scientific name.
fn normalize_kingdom(val: &str) -> Option<String> {
    match val.to_lowercase().as_str() {
        "hewan" | "satwa" | "fauna" | "binatang" | "animalia" | "animal" | "animals" => {
            Some("Animalia".to_string())
        }
        "tumbuhan" | "tanaman" | "flora" | "pohon" | "plantae" | "plant" | "plants" => {
            Some("Plantae".to_string())
        }
        "jamur" | "fungi" | "fungus" | "mushroom" | "mushrooms" => {
            Some("Fungi".to_string())
        }
        _ => None,
    }
}

/// Maps a raw class / category name (multilingual) to standard scientific name.
fn normalize_class(val: &str) -> Option<String> {
    match val.to_lowercase().as_str() {
        "mamalia" | "mammalia" | "mammal" | "mammals" | "menyusui" => {
            Some("Mammalia".to_string())
        }
        "reptil" | "reptilia" | "reptile" | "reptiles" | "melata" => {
            Some("Reptilia".to_string())
        }
        "aves" | "burung" | "unggas" | "bird" | "birds" => {
            Some("Aves".to_string())
        }
        "amfibi" | "amphibia" | "amphibian" | "amphibians" => {
            Some("Amphibia".to_string())
        }
        "ikan" | "pisces" | "fish" | "fishes" | "actinopterygii" => {
            Some("Actinopterygii".to_string())
        }
        "serangga" | "insekta" | "insecta" | "insect" | "insects" => {
            Some("Insecta".to_string())
        }
        "arachnida" | "arachnid" | "arachnids" | "laba-laba" | "labalaba" | "spider" | "spiders" => {
            Some("Arachnida".to_string())
        }
        "magnoliopsida" | "dikotil" | "dicot" | "dicotyledon" => {
            Some("Magnoliopsida".to_string())
        }
        "liliopsida" | "monokotil" | "monocot" | "monocotyledon" => {
            Some("Liliopsida".to_string())
        }
        "pinopsida" | "konifer" | "conifer" | "gymnospermae" => {
            Some("Pinopsida".to_string())
        }
        _ => None,
    }
}

/// Parses a token with a known key-value prefix (e.g. `key:value`).
fn parse_prefixed_token(prefix: &str, value: &str) -> Option<TokenKind> {
    match prefix.to_lowercase().as_str() {
        // `status:` is ambiguous in Indonesian queries: fall back to the P.106
        // protection vocabulary when the value is not a valid IUCN code, so
        // `status:dilindungi` does not become an imaginary IUCN category.
        "iucn" | "status" => match normalize_iucn_code(value) {
            Some(code) => Some(TokenKind::Iucn(code)),
            None => match value.to_lowercase().as_str() {
                "dilindungi" | "protected" | "tidak-dilindungi" | "tidak_dilindungi" | "bebas"
                | "unprotected" => normalize_p106(value).map(TokenKind::P106),
                _ => None,
            },
        },
        "p106" | "protection" | "perlindungan" => normalize_p106(value).map(TokenKind::P106),
        "type" | "species_type" | "origin" | "tipe" | "asal" => {
            normalize_species_type(value).map(TokenKind::SpeciesType)
        }
        "cites" => normalize_cites(value).map(TokenKind::Cites),
        "kingdom_id" => value.parse::<u64>().ok().map(TokenKind::KingdomId),
        "class_id" => value.parse::<u64>().ok().map(TokenKind::ClassId),
        "kingdom" | "kerajaan" => {
            if let Ok(id) = value.parse::<u64>() {
                Some(TokenKind::KingdomId(id))
            } else if let Some(k) = normalize_kingdom(value) {
                Some(TokenKind::Kingdom(k))
            } else {
                let mut c = value.chars();
                Some(TokenKind::Kingdom(match c.next() {
                    None => value.to_string(),
                    Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                }))
            }
        }
        "class" | "kelas" => {
            if let Ok(id) = value.parse::<u64>() {
                Some(TokenKind::ClassId(id))
            } else if let Some(c) = normalize_class(value) {
                Some(TokenKind::Class(c))
            } else {
                let mut c = value.chars();
                Some(TokenKind::Class(match c.next() {
                    None => value.to_string(),
                    Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                }))
            }
        }
        "kategori" | "category" => {
            if let Some(c) = normalize_class(value) {
                Some(TokenKind::Class(c))
            } else if let Some(k) = normalize_kingdom(value) {
                Some(TokenKind::Kingdom(k))
            } else {
                let mut c = value.chars();
                Some(TokenKind::Class(match c.next() {
                    None => value.to_string(),
                    Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                }))
            }
        }
        "order" | "ordo" => Some(TokenKind::Order(value.to_string())),
        "family" | "famili" => Some(TokenKind::Family(value.to_string())),
        "genus" => Some(TokenKind::Genus(value.to_string())),
        "verified" | "is_verified" | "terverifikasi" => match value.to_lowercase().as_str() {
            "true" | "1" | "yes" | "ya" => Some(TokenKind::IsVerified(true)),
            "false" | "0" | "no" | "tidak" => Some(TokenKind::IsVerified(false)),
            _ => None,
        },
        _ => None,
    }
}

/// Classifies an individual token into a structured attribute or free text.
fn classify_token(token: &str) -> TokenKind {
    // 1. Check for key:value prefix syntax
    if let Some((prefix, value)) = token.split_once(':') {
        if let Some(kind) = parse_prefixed_token(prefix, value) {
            return kind;
        }
    }

    // 2. Check for standalone known keywords
    let lower = token.to_lowercase();
    match lower.as_str() {
        // IUCN status codes (multi-lang)
        "cr" | "kritis" | "critically-endangered" | "critically_endangered" => {
            TokenKind::Iucn("CR".to_string())
        }
        "en" | "genting" | "endangered" => TokenKind::Iucn("EN".to_string()),
        "vu" | "rentan" | "vulnerable" => TokenKind::Iucn("VU".to_string()),
        "nt" | "hampir-terancam" | "near-threatened" | "near_threatened" => {
            TokenKind::Iucn("NT".to_string())
        }
        "lc" | "risiko-rendah" | "least-concern" | "least_concern" => {
            TokenKind::Iucn("LC".to_string())
        }
        "dd" | "kurang-data" | "data-deficient" | "data_deficient" => {
            TokenKind::Iucn("DD".to_string())
        }
        "ew" | "punah-di-alam" | "extinct-in-the-wild" => TokenKind::Iucn("EW".to_string()),
        "ex" | "punah" | "extinct" => TokenKind::Iucn("EX".to_string()),

        // Protection status (P.106 multi-lang)
        "dilindungi" | "protected" => TokenKind::P106("Dilindungi".to_string()),
        "tidak-dilindungi" | "tidak_dilindungi" | "unprotected" | "bebas" => {
            TokenKind::P106("Tidak Dilindungi".to_string())
        }

        // Species origin type
        "endemik" | "endemic" => TokenKind::SpeciesType("Endemic".to_string()),
        "asli" | "native" => TokenKind::SpeciesType("Native".to_string()),
        "introduksi" | "introduced" => TokenKind::SpeciesType("Introduced".to_string()),

        // Categories & Classes (Natural Language multi-lang)
        "mamalia" | "mammalia" | "mammal" | "mammals" | "menyusui" => {
            TokenKind::Class("Mammalia".to_string())
        }
        "reptil" | "reptilia" | "reptile" | "reptiles" | "melata" => {
            TokenKind::Class("Reptilia".to_string())
        }
        "aves" | "burung" | "unggas" | "bird" | "birds" => {
            TokenKind::Class("Aves".to_string())
        }
        "amfibi" | "amphibia" | "amphibian" | "amphibians" => {
            TokenKind::Class("Amphibia".to_string())
        }
        "ikan" | "pisces" | "fish" | "fishes" | "actinopterygii" => {
            TokenKind::Class("Actinopterygii".to_string())
        }
        "serangga" | "insekta" | "insecta" | "insect" | "insects" => {
            TokenKind::Class("Insecta".to_string())
        }
        "arachnida" | "arachnid" | "arachnids" | "laba-laba" | "labalaba" | "spider" | "spiders" => {
            TokenKind::Class("Arachnida".to_string())
        }
        "magnoliopsida" | "dikotil" | "dicot" | "dicotyledon" => {
            TokenKind::Class("Magnoliopsida".to_string())
        }
        "liliopsida" | "monokotil" | "monocot" | "monocotyledon" => {
            TokenKind::Class("Liliopsida".to_string())
        }
        "pinopsida" | "konifer" | "conifer" | "gymnospermae" => {
            TokenKind::Class("Pinopsida".to_string())
        }

        // Kingdoms (Natural Language multi-lang)
        "hewan" | "satwa" | "fauna" | "binatang" | "animalia" | "animal" | "animals" => {
            TokenKind::Kingdom("Animalia".to_string())
        }
        "tumbuhan" | "tanaman" | "flora" | "pohon" | "plantae" | "plant" | "plants" => {
            TokenKind::Kingdom("Plantae".to_string())
        }
        "jamur" | "fungi" | "fungus" | "mushroom" | "mushrooms" => {
            TokenKind::Kingdom("Fungi".to_string())
        }

        // Verification status
        "terverifikasi" | "verified" => TokenKind::IsVerified(true),

        // Everything else is treated as free text
        _ => TokenKind::FreeText(token.to_string()),
    }
}

/// Multi-word phrases that must be matched as a single token.
///
/// Every entry maps a phrase (case-insensitive) to the hyphenated token that
/// [`classify_token`] already understands. Without this, a phrase such as
/// `punah di alam` would be split into `punah` (EX) plus stray free text.
const PHRASES: &[(&[&str], &str)] = &[
    (&["extinct", "in", "the", "wild"], "extinct-in-the-wild"),
    (&["critically", "endangered"], "critically-endangered"),
    (&["least", "concern"], "least-concern"),
    (&["data", "deficient"], "data-deficient"),
    (&["near", "threatened"], "near-threatened"),
    (&["punah", "di", "alam"], "punah-di-alam"),
    (&["hampir", "terancam"], "hampir-terancam"),
    (&["risiko", "rendah"], "risiko-rendah"),
    (&["kurang", "data"], "kurang-data"),
    (&["tidak", "dilindungi"], "tidak-dilindungi"),
    (&["appendix", "i"], "appendix-i"),
    (&["appendix", "ii"], "appendix-ii"),
    (&["appendix", "iii"], "appendix-iii"),
    (&["laba", "laba"], "laba-laba"),
];

/// Collapses known multi-word phrases into their single hyphenated token.
///
/// The longest matching phrase wins, and non-matching tokens are preserved.
fn merge_phrases(tokens: &[String]) -> Vec<String> {
    let mut merged = Vec::with_capacity(tokens.len());
    let mut index = 0;

    while index < tokens.len() {
        let mut best: Option<(usize, &str)> = None;

        for (phrase, canonical) in PHRASES {
            if index + phrase.len() > tokens.len() {
                continue;
            }
            let matches = phrase
                .iter()
                .enumerate()
                .all(|(offset, word)| tokens[index + offset].eq_ignore_ascii_case(word));
            if matches && best.map_or(true, |(len, _)| phrase.len() > len) {
                best = Some((phrase.len(), canonical));
            }
        }

        match best {
            Some((len, canonical)) => {
                merged.push(canonical.to_string());
                index += len;
            }
            None => {
                merged.push(tokens[index].clone());
                index += 1;
            }
        }
    }

    merged
}

/// Applies a single classified token to the `QueryFilters` accumulator.
fn apply_classification(mut filters: QueryFilters, kind: TokenKind) -> QueryFilters {
    match kind {
        TokenKind::Iucn(code) => filters.iucn = Some(code),
        TokenKind::P106(status) => filters.p106 = Some(status),
        TokenKind::SpeciesType(st) => filters.species_type = Some(st),
        TokenKind::Cites(cites) => filters.cites = Some(cites),
        TokenKind::KingdomId(id) => filters.kingdom_id = Some(id),
        TokenKind::ClassId(id) => filters.class_id = Some(id),
        TokenKind::Kingdom(k) => filters.kingdom = Some(k),
        TokenKind::Class(c) => filters.class = Some(c),
        TokenKind::Order(o) => filters.order = Some(o),
        TokenKind::Family(f) => filters.family = Some(f),
        TokenKind::Genus(g) => filters.genus = Some(g),
        TokenKind::IsVerified(verified) => filters.is_verified = Some(verified),
        TokenKind::FreeText(word) => filters.free_text.push(word),
    }
    filters
}

/// Removes punctuation attached to a token while keeping the characters that
/// carry meaning: `:` separates a prefix from its value (`iucn:cr`), `-` joins
/// the words of a known phrase (`hampir-terancam`), and `_` appears in prefix
/// names and vocabulary (`kingdom_id`, `tidak_dilindungi`).
fn clean_token(token: &str) -> String {
    token
        .chars()
        .filter(|c| c.is_alphanumeric() || matches!(c, ':' | '-' | '_'))
        .collect()
}

/// Extracts structured query filters from a slice of tokens.
///
/// Punctuation is stripped from each token first, then known multi-word phrases
/// (e.g. `hampir terancam`, `punah di alam`) are merged into one token before
/// classification, so `mamalia, kritis` parses exactly like `mamalia kritis`.
pub fn extract_filters(tokens: &[String]) -> QueryFilters {
    let cleaned_tokens: Vec<String> = tokens
        .iter()
        .map(|token| clean_token(token))
        .filter(|token| !token.is_empty())
        .collect();
    let merged_tokens = merge_phrases(&cleaned_tokens);
    let final_state = merged_tokens
        .iter()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .fold(ParseState::default(), |mut state, token| {
            if state.pending_negation {
                if token.eq_ignore_ascii_case("dilindungi") {
                    state.filters.p106 = Some("Tidak Dilindungi".to_string());
                    state.pending_negation = false;
                    return state;
                }
                // Flush pending negation as free text if the following token is not "dilindungi"
                state.filters.free_text.push("tidak".to_string());
                state.pending_negation = false;
            }

            if token.eq_ignore_ascii_case("tidak") {
                state.pending_negation = true;
                state
            } else {
                let kind = classify_token(token);
                state.filters = apply_classification(state.filters, kind);
                state
            }
        });

    let mut filters = final_state.filters;
    if final_state.pending_negation {
        filters.free_text.push("tidak".to_string());
    }

    filters
}

/// Parses a natural-language or structured raw query string into structured [`QueryFilters`].
///
/// # Arguments
///
/// * `raw_query` - The user-provided search text.
///
/// # Returns
///
/// A structured [`QueryFilters`] instance.
///
/// # Example
///
/// ```rust,ignore
/// let filters = parse_query("mamalia kritis bekantan");
/// assert_eq!(filters.class, Some("Mammalia".to_string()));
/// assert_eq!(filters.iucn, Some("CR".to_string()));
/// ```
pub fn parse_query(raw_query: &str) -> QueryFilters {
    let normalized = kalimantanbio_shared::text::normalize_text(raw_query);
    let tokens = kalimantanbio_shared::text::tokenize(&normalized);
    extract_filters(&tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_iucn_token() {
        let tokens = vec!["bekantan".to_string(), "cr".to_string()];
        let filters = extract_filters(&tokens);

        assert_eq!(filters.iucn, Some("CR".to_string()));
        assert_eq!(filters.free_text, vec!["bekantan".to_string()]);
    }

    #[test]
    fn test_extract_iucn_prefix() {
        let tokens = vec!["iucn:vu".to_string(), "kucing".to_string()];
        let filters = extract_filters(&tokens);

        assert_eq!(filters.iucn, Some("VU".to_string()));
        assert_eq!(filters.free_text, vec!["kucing".to_string()]);
    }

    #[test]
    fn test_extract_iucn_kritis() {
        let tokens = vec!["orangutan".to_string(), "kritis".to_string()];
        let filters = extract_filters(&tokens);

        assert_eq!(filters.iucn, Some("CR".to_string()));
        assert_eq!(filters.free_text, vec!["orangutan".to_string()]);
    }

    #[test]
    fn test_extract_protection_dilindungi() {
        let tokens = vec!["orangutan".to_string(), "dilindungi".to_string()];
        let filters = extract_filters(&tokens);

        assert_eq!(filters.p106, Some("Dilindungi".to_string()));
        assert_eq!(filters.free_text, vec!["orangutan".to_string()]);
    }

    #[test]
    fn test_extract_category_mamalia() {
        let tokens = vec!["mamalia".to_string(), "dilindungi".to_string()];
        let filters = extract_filters(&tokens);

        assert_eq!(filters.class, Some("Mammalia".to_string()));
        assert_eq!(filters.p106, Some("Dilindungi".to_string()));
        assert!(filters.free_text.is_empty());
    }

    #[test]
    fn test_extract_kingdom_tumbuhan_dan_hewan() {
        let filters_tumbuhan = parse_query("tumbuhan langka");
        assert_eq!(filters_tumbuhan.kingdom, Some("Plantae".to_string()));
        assert_eq!(filters_tumbuhan.free_text, vec!["langka".to_string()]);

        let filters_hewan = parse_query("kategori:hewan bekantan");
        assert_eq!(filters_hewan.kingdom, Some("Animalia".to_string()));
        assert_eq!(filters_hewan.free_text, vec!["bekantan".to_string()]);
    }

    #[test]
    fn test_extract_multilang_burung_reptil() {
        let filters_burung = parse_query("burung endemik");
        assert_eq!(filters_burung.class, Some("Aves".to_string()));
        assert_eq!(filters_burung.species_type, Some("Endemic".to_string()));

        let filters_reptil = parse_query("reptil kritis");
        assert_eq!(filters_reptil.class, Some("Reptilia".to_string()));
        assert_eq!(filters_reptil.iucn, Some("CR".to_string()));
    }

    #[test]
    fn test_extract_protection_tidak_dilindungi() {
        let tokens = vec![
            "burung".to_string(),
            "tidak".to_string(),
            "dilindungi".to_string(),
        ];
        let filters = extract_filters(&tokens);

        assert_eq!(filters.class, Some("Aves".to_string()));
        assert_eq!(filters.p106, Some("Tidak Dilindungi".to_string()));
        assert!(filters.free_text.is_empty());
    }

    #[test]
    fn test_extract_species_type_endemik_and_asli() {
        let tokens_endemik = vec!["endemik".to_string(), "kalimantan".to_string()];
        let filters_endemik = extract_filters(&tokens_endemik);
        assert_eq!(filters_endemik.species_type, Some("Endemic".to_string()));
        assert_eq!(filters_endemik.free_text, vec!["kalimantan".to_string()]);

        let tokens_asli = vec!["asli".to_string(), "borneo".to_string()];
        let filters_asli = extract_filters(&tokens_asli);
        assert_eq!(filters_asli.species_type, Some("Native".to_string()));
        assert_eq!(filters_asli.free_text, vec!["borneo".to_string()]);
    }

    #[test]
    fn test_extract_multi_attribute_combined() {
        let tokens = vec![
            "pongo".to_string(),
            "pygmaeus".to_string(),
            "cr".to_string(),
            "dilindungi".to_string(),
            "endemik".to_string(),
        ];
        let filters = extract_filters(&tokens);

        assert_eq!(
            filters.free_text,
            vec!["pongo".to_string(), "pygmaeus".to_string()]
        );
        assert_eq!(filters.iucn, Some("CR".to_string()));
        assert_eq!(filters.p106, Some("Dilindungi".to_string()));
        assert_eq!(filters.species_type, Some("Endemic".to_string()));
    }

    #[test]
    fn test_extract_prefixed_attributes() {
        let tokens = vec![
            "cites:appendix-i".to_string(),
            "kingdom_id:1".to_string(),
            "class_id:3".to_string(),
            "verified:true".to_string(),
        ];
        let filters = extract_filters(&tokens);

        assert_eq!(filters.cites, Some("Appendix I".to_string()));
        assert_eq!(filters.kingdom_id, Some(1));
        assert_eq!(filters.class_id, Some(3));
        assert_eq!(filters.is_verified, Some(true));
        assert!(filters.free_text.is_empty());
    }

    #[test]
    fn test_extract_empty_and_free_text_only() {
        let empty_tokens: Vec<String> = vec![];
        let filters_empty = extract_filters(&empty_tokens);
        assert_eq!(filters_empty, QueryFilters::default());

        let tokens = vec!["gajah".to_string(), "kerdil".to_string()];
        let filters = extract_filters(&tokens);
        assert_eq!(
            filters.free_text,
            vec!["gajah".to_string(), "kerdil".to_string()]
        );
        assert_eq!(filters.iucn, None);
        assert_eq!(filters.p106, None);
        assert_eq!(filters.species_type, None);
    }

    #[test]
    fn test_punctuation_is_ignored() {
        let punctuated = parse_query("mamalia, kritis");
        let plain = parse_query("mamalia kritis");
        assert_eq!(punctuated, plain);
        assert_eq!(punctuated.class, Some("Mammalia".to_string()));
        assert_eq!(punctuated.iucn, Some("CR".to_string()));

        assert_eq!(parse_query("iucn:cr,").iucn, Some("CR".to_string()));
        assert_eq!(parse_query("orangutan!").free_text, vec!["orangutan".to_string()]);
        assert_eq!(parse_query("\"orangutan\"").free_text, vec!["orangutan".to_string()]);
    }

    #[test]
    fn test_punctuation_does_not_break_phrases() {
        assert_eq!(
            parse_query("hampir terancam,").iucn,
            Some("NT".to_string())
        );
        assert_eq!(
            parse_query("punah di alam!").iucn,
            Some("EW".to_string())
        );
    }

    #[test]
    fn test_multiword_iucn_phrases() {
        assert_eq!(parse_query("hampir terancam").iucn, Some("NT".to_string()));
        assert_eq!(parse_query("risiko rendah").iucn, Some("LC".to_string()));
        assert_eq!(parse_query("kurang data").iucn, Some("DD".to_string()));
        assert_eq!(parse_query("near threatened").iucn, Some("NT".to_string()));
        assert_eq!(parse_query("least concern").iucn, Some("LC".to_string()));
        assert_eq!(parse_query("data deficient").iucn, Some("DD".to_string()));
    }

    #[test]
    fn test_extinct_in_the_wild_is_not_extinct() {
        let filters = parse_query("punah di alam");
        assert_eq!(filters.iucn, Some("EW".to_string()));
        assert!(filters.free_text.is_empty());

        let english = parse_query("extinct in the wild");
        assert_eq!(english.iucn, Some("EW".to_string()));
        assert!(english.free_text.is_empty());
    }

    #[test]
    fn test_natural_language_aliases_match_prefixed_form() {
        assert_eq!(parse_query("menyusui").class, Some("Mammalia".to_string()));
        assert_eq!(parse_query("melata").class, Some("Reptilia".to_string()));
        assert_eq!(
            parse_query("konifer").class,
            Some("Pinopsida".to_string())
        );
        assert_eq!(
            parse_query("kelas:menyusui").class,
            Some("Mammalia".to_string())
        );
        assert_eq!(
            parse_query("laba laba").class,
            Some("Arachnida".to_string())
        );
    }

    #[test]
    fn test_status_prefix_falls_back_to_protection() {
        let filters = parse_query("status:dilindungi");
        assert_eq!(filters.p106, Some("Dilindungi".to_string()));
        assert_eq!(filters.iucn, None);
    }

    #[test]
    fn test_extract_arachnid() {
        let tokens = vec!["arachnid".to_string(), "hutan".to_string()];
        let filters = extract_filters(&tokens);
        assert_eq!(filters.class, Some("Arachnida".to_string()));
        assert_eq!(filters.free_text, vec!["hutan".to_string()]);
    }
}

