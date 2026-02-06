use std::collections::HashMap;
use std::fmt::{Display, Formatter, Result as FmtResult};

use bon::Builder;

use super::{Tag, VariantKey};

/// A localized phrase that can have multiple variants and metadata tags.
///
/// Phrases are the primary output type of RLF phrase functions. They carry:
/// - A default text representation
/// - Optional variants for different grammatical forms (e.g., singular/plural)
/// - Metadata tags for grammatical information (e.g., gender, article hints)
///
/// # Example
///
/// ```
/// use rlf::{Phrase, VariantKey, Tag};
/// use std::collections::HashMap;
///
/// let card = Phrase::builder()
///     .text("card".to_string())
///     .variants(HashMap::from([
///         (VariantKey::new("one"), "card".to_string()),
///         (VariantKey::new("other"), "cards".to_string()),
///     ]))
///     .tags(vec![Tag::new("a")])
///     .build();
///
/// assert_eq!(card.to_string(), "card");
/// assert_eq!(card.variant("one"), "card");
/// assert_eq!(card.variant("other"), "cards");
/// ```
#[derive(Debug, Clone, Builder)]
pub struct Phrase {
    /// Default text when the phrase is displayed.
    pub text: String,

    /// Variant key to variant text mapping.
    ///
    /// Keys can be simple (e.g., "one", "other") or multi-dimensional
    /// using dot notation (e.g., "nom.one", "acc.few").
    #[builder(default)]
    pub variants: HashMap<VariantKey, String>,

    /// Metadata tags attached to this phrase.
    ///
    /// Tags provide grammatical information like gender (`:masc`, `:fem`),
    /// article hints (`:a`, `:an`), or other language-specific metadata.
    #[builder(default)]
    pub tags: Vec<Tag>,
}

impl Phrase {
    /// Get a specific variant by key, with fallback resolution.
    ///
    /// Resolution order:
    /// 1. Exact match (e.g., "nom.one")
    /// 2. Progressively shorter keys by removing the last segment (e.g., "nom.one" -> "nom")
    ///
    /// # Panics
    ///
    /// Panics if no matching variant is found. This is intentional - missing
    /// variants indicate a programming error in the RLF definition.
    ///
    /// # Example
    ///
    /// ```
    /// use rlf::{Phrase, VariantKey};
    /// use std::collections::HashMap;
    ///
    /// let card = Phrase::builder()
    ///     .text("card".to_string())
    ///     .variants(HashMap::from([
    ///         (VariantKey::new("nom"), "card".to_string()),
    ///         (VariantKey::new("nom.other"), "cards".to_string()),
    ///     ]))
    ///     .build();
    ///
    /// // Exact match
    /// assert_eq!(card.variant("nom.other"), "cards");
    ///
    /// // Fallback: "nom.one" -> "nom"
    /// assert_eq!(card.variant("nom.one"), "card");
    /// ```
    pub fn variant(&self, key: &str) -> &str {
        // Try exact match
        if let Some(v) = self.variants.get(&VariantKey::new(key)) {
            return v;
        }

        // Try progressively shorter keys (fallback resolution)
        let mut current = key;
        while let Some(dot_pos) = current.rfind('.') {
            current = &current[..dot_pos];
            if let Some(v) = self.variants.get(&VariantKey::new(current)) {
                return v;
            }
        }

        // No match - panic with helpful error
        panic!(
            "No variant '{}' in phrase. Available: {:?}",
            key,
            self.variants.keys().collect::<Vec<_>>()
        );
    }

    /// Check if this phrase has a specific tag.
    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.iter().any(|t| t.as_str() == tag)
    }

    /// Get the first tag, if any.
    pub fn first_tag(&self) -> Option<&Tag> {
        self.tags.first()
    }
}

impl Display for Phrase {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.text)
    }
}

impl From<Phrase> for String {
    fn from(phrase: Phrase) -> Self {
        phrase.text
    }
}
