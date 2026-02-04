use serde::{Deserialize, Serialize};

/// A key identifying a specific variant of a phrase.
///
/// Variant keys can be simple (e.g., "one", "other") or multi-dimensional
/// using dot notation (e.g., "nom.one", "acc.few").
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VariantKey(String);

impl VariantKey {
    /// Create a new variant key from any string-like value.
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    /// Get the variant key as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::ops::Deref for VariantKey {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<&str> for VariantKey {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for VariantKey {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl std::fmt::Display for VariantKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
