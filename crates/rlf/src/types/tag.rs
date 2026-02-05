use std::fmt::{Display, Formatter, Result as FmtResult};
use std::ops::Deref;

use serde::{Deserialize, Serialize};

/// A metadata tag attached to a phrase.
///
/// Tags provide grammatical information like gender (`:masc`, `:fem`),
/// article hints (`:a`, `:an`), or other language-specific metadata.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Tag(String);

impl Tag {
    /// Create a new tag from any string-like value.
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    /// Get the tag as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Deref for Tag {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<&str> for Tag {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for Tag {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl Display for Tag {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.0)
    }
}
