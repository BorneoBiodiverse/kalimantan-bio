//! Text normalization and tokenization utilities.

/// Normalize text: lowercase, trim, remove extra whitespace.
///
/// # Arguments
///
/// * `input` - Raw input text.
///
/// # Returns
///
/// A normalized string.
///
/// # Used By
///
/// - **Module 1 (species-search)**: Normalize user query before matching
/// - **Module 2 (species-relationships)**: Normalize habitat/characteristic labels for comparison
/// - **Module 5 (knowledge-citations)**: Normalize titles, topics for matching
///
/// # Example
///
/// ```rust,ignore
/// let normalized = normalize_text("  Orangutan   Kalimantan ");
/// // Returns: "orangutan kalimantan"
/// ```
pub fn normalize_text(input: &str) -> String {
    todo!("shared text phase 3: normalize_text")
}

/// Sanitize a label: trim and remove extra spaces.
///
/// # Arguments
///
/// * `input` - Raw label text.
///
/// # Returns
///
/// A sanitized label.
///
/// # Used By
///
/// - **Module 2 (species-relationships)**: Clean habitat/characteristic labels
/// - **Module 3 (taxonomy)**: Clean genus/family names for comparison
///
/// # Example
///
/// ```rust,ignore
/// let label = sanitize_label("  Family   Dipterocarpaceae ");
/// // Returns: "Family Dipterocarpaceae"
/// ```
pub fn sanitize_label(input: &str) -> String {
    todo!("shared text phase 3: sanitize_label")
}

/// Tokenize text by whitespace.
///
/// # Arguments
///
/// * `text` - Text to split into tokens.
///
/// # Returns
///
/// A vector of whitespace-separated tokens.
///
/// # Used By
///
/// - **Module 1 (species-search)**: Extract search terms from query
/// - **Module 5 (knowledge-citations)**: Parse publication topics for matching
///
/// # Example
///
/// ```rust,ignore
/// let tokens = tokenize("orangutan kalimantan");
/// // Returns: vec!["orangutan", "kalimantan"]
/// ```
pub fn tokenize(text: &str) -> Vec<String> {
    todo!("shared text phase 3: tokenize")
}