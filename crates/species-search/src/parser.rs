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
        "nt" | "near-threatened" | "near_threatened" | "hampir-terancam" => Some("NT".to_string()),
        "lc" | "least-concern" | "least_concern" | "risiko-rendah" => Some("LC".to_string()),
        "dd" | "data-deficient" | "data_deficient" | "kurang-data" => Some("DD".to_string()),
        "ew" | "extinct-in-the-wild" => Some("EW".to_string()),
        "ex" | "extinct" | "punah" => Some("EX".to_string()),
        other if !other.is_empty() => Some(other.to_uppercase()),
        _ => None,
    }
}

/// Maps a raw CITES designation to standard representation.
fn normalize_cites(val: &str) -> Option<String> {
    match val.to_lowercase().as_str() {
        "i" | "1" | "appendix-i" | "appendix_i" | "appendix-1" => Some("Appendix I".to_string()),
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
        "tidak-dilindungi" | "tidak_dilindungi" | "unprotected" | "false" | "tidak" => {
            Some("Tidak Dilindungi".to_string())
        }
        other if !other.is_empty() => Some(other.to_string()),
        _ => None,
    }
}

/// Parses a token with a known key-value prefix (e.g. `key:value`).
fn parse_prefixed_token(prefix: &str, value: &str) -> Option<TokenKind> {
    match prefix.to_lowercase().as_str() {
        "iucn" => normalize_iucn_code(value).map(TokenKind::Iucn),
        "p106" | "protection" => normalize_p106(value).map(TokenKind::P106),
        "type" | "species_type" | "origin" => {
            normalize_species_type(value).map(TokenKind::SpeciesType)
        }
        "cites" => normalize_cites(value).map(TokenKind::Cites),
        "kingdom_id" | "kingdom" => value.parse::<u64>().ok().map(TokenKind::KingdomId),
        "class_id" | "class" => value.parse::<u64>().ok().map(TokenKind::ClassId),
        "verified" | "is_verified" => match value.to_lowercase().as_str() {
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
        // IUCN status codes
        "cr" | "kritis" => TokenKind::Iucn("CR".to_string()),
        "en" => TokenKind::Iucn("EN".to_string()),
        "vu" => TokenKind::Iucn("VU".to_string()),
        "nt" => TokenKind::Iucn("NT".to_string()),
        "lc" => TokenKind::Iucn("LC".to_string()),
        "dd" => TokenKind::Iucn("DD".to_string()),
        "ew" => TokenKind::Iucn("EW".to_string()),
        "ex" => TokenKind::Iucn("EX".to_string()),

        // Protection status (P.106)
        "dilindungi" | "protected" => TokenKind::P106("Dilindungi".to_string()),
        "tidak-dilindungi" | "tidak_dilindungi" | "unprotected" => {
            TokenKind::P106("Tidak Dilindungi".to_string())
        }

        // Species origin type
        "endemik" | "endemic" => TokenKind::SpeciesType("Endemic".to_string()),
        "asli" | "native" => TokenKind::SpeciesType("Native".to_string()),
        "introduksi" | "introduced" => TokenKind::SpeciesType("Introduced".to_string()),

        // Everything else is treated as free text
        _ => TokenKind::FreeText(token.to_string()),
    }
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
        TokenKind::IsVerified(verified) => filters.is_verified = Some(verified),
        TokenKind::FreeText(word) => filters.free_text.push(word),
    }
    filters
}

/// ```
pub(crate) fn extract_filters(tokens: &[String]) -> QueryFilters {
    let final_state = tokens
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
        let tokens = vec!["mamalia".to_string(), "dilindungi".to_string()];
        let filters = extract_filters(&tokens);

        assert_eq!(filters.p106, Some("Dilindungi".to_string()));
        assert_eq!(filters.free_text, vec!["mamalia".to_string()]);
    }

    #[test]
    fn test_extract_protection_tidak_dilindungi() {
        let tokens = vec![
            "burung".to_string(),
            "tidak".to_string(),
            "dilindungi".to_string(),
        ];
        let filters = extract_filters(&tokens);

        assert_eq!(filters.p106, Some("Tidak Dilindungi".to_string()));
        assert_eq!(filters.free_text, vec!["burung".to_string()]);
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
}
