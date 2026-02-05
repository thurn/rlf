use std::fmt::{Display, Formatter, Result as FmtResult};
use std::ops::Deref;

use serde::{Deserialize, Serialize};

/// A key identifying a specific variant of a phrase.
///
/// Variant keys can be simple (e.g., "one", "other") or multi-dimensional
/// using dot notation (e.g., "nom.one", "acc.few").
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
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

impl Deref for VariantKey {
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

impl Display for VariantKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.0)
    }
}
