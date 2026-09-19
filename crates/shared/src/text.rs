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
/// # Example
///
/// ```rust,ignore
/// let normalized = normalize_text("  Orangutan   Kalimantan ");
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
/// # Example
///
/// ```rust,ignore
/// let label = sanitize_label("  Family   Dipterocarpaceae ");
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
/// # Example
///
/// ```rust,ignore
/// let tokens = tokenize("orangutan kalimantan");
/// ```
pub fn tokenize(text: &str) -> Vec<String> {
    todo!("shared text phase 3: tokenize")
}