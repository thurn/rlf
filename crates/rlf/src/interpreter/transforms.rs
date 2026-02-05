//! Transform registry for string transformations.
//!
//! Transforms are functions that modify values (e.g., @cap, @upper, @lower).
//! This module provides the registry infrastructure and universal transform implementations.

use icu_casemap::CaseMapper;
use icu_locale_core::{LanguageIdentifier, langid};
use unicode_segmentation::UnicodeSegmentation;

use crate::interpreter::EvalError;
use crate::types::Value;

/// Transform types for static dispatch.
///
/// Per CONTEXT.md: static dispatch via enum, no trait objects or function pointers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransformKind {
    /// @cap - Capitalize first grapheme
    Cap,
    /// @upper - All uppercase
    Upper,
    /// @lower - All lowercase
    Lower,
    // English transforms (Phase 6)
    /// @a/@an - English indefinite article from :a/:an tags
    EnglishA,
    /// @the - English definite article "the"
    EnglishThe,
}

impl TransformKind {
    /// Execute the transform on a value.
    ///
    /// Context is optional (used by language-specific transforms in later phases).
    /// Lang is used for locale-sensitive case mapping.
    pub fn execute(
        &self,
        value: &Value,
        _context: Option<&Value>, // Unused for universal transforms
        lang: &str,
    ) -> Result<String, EvalError> {
        let text = value.to_string();
        let locale = parse_langid(lang);

        match self {
            TransformKind::Cap => cap_transform(&text, &locale),
            TransformKind::Upper => upper_transform(&text, &locale),
            TransformKind::Lower => lower_transform(&text, &locale),
            // English transforms need full Value to read tags
            TransformKind::EnglishA => english_a_transform(value),
            TransformKind::EnglishThe => english_the_transform(value),
        }
    }
}

/// Parse language code to ICU4X LanguageIdentifier.
///
/// Special handling for Turkish (tr) and Azerbaijani (az) which have
/// dotted-I case mapping rules different from other languages.
fn parse_langid(lang: &str) -> LanguageIdentifier {
    // Try to parse the language code, fall back to undetermined
    lang.parse().unwrap_or(langid!("und"))
}

/// Capitalize first grapheme, preserving rest of string.
///
/// Uses unicode-segmentation for grapheme-aware first character detection.
/// Handles combining characters correctly (e.g., "e\u{0301}" is one grapheme).
fn cap_transform(text: &str, locale: &LanguageIdentifier) -> Result<String, EvalError> {
    if text.is_empty() {
        return Ok(String::new());
    }

    let cm = CaseMapper::new();
    let mut graphemes = text.graphemes(true);

    match graphemes.next() {
        Some(first) => {
            let rest: String = graphemes.collect();
            // Uppercase the first grapheme (handles multi-codepoint graphemes)
            let capitalized = cm.uppercase_to_string(first, locale);
            Ok(format!("{capitalized}{rest}"))
        }
        None => Ok(String::new()),
    }
}

/// Convert entire string to uppercase.
fn upper_transform(text: &str, locale: &LanguageIdentifier) -> Result<String, EvalError> {
    let cm = CaseMapper::new();
    Ok(cm.uppercase_to_string(text, locale).into_owned())
}

/// Convert entire string to lowercase.
fn lower_transform(text: &str, locale: &LanguageIdentifier) -> Result<String, EvalError> {
    let cm = CaseMapper::new();
    Ok(cm.lowercase_to_string(text, locale).into_owned())
}

// =============================================================================
// English Transforms (Phase 6)
// =============================================================================

/// English indefinite article transform (@a/@an).
///
/// Reads :a or :an tag from the Value to determine which article to prepend.
/// Returns MissingTag error if neither tag is present.
fn english_a_transform(value: &Value) -> Result<String, EvalError> {
    let text = value.to_string();

    if value.has_tag("a") {
        return Ok(format!("a {}", text));
    }
    if value.has_tag("an") {
        return Ok(format!("an {}", text));
    }

    Err(EvalError::MissingTag {
        transform: "a".to_string(),
        expected: vec!["a".to_string(), "an".to_string()],
        phrase: text,
    })
}

/// English definite article transform (@the).
///
/// Unconditionally prepends "the " to the value's text.
fn english_the_transform(value: &Value) -> Result<String, EvalError> {
    Ok(format!("the {value}"))
}

/// Registry for transform functions.
///
/// Transforms are registered per-language with universal transforms available to all.
/// Language-specific transforms take precedence over universal transforms.
#[derive(Default)]
pub struct TransformRegistry {
    // Reserved for future language-specific transform registration.
    // Currently all transforms are resolved via TransformKind::get().
}

impl TransformRegistry {
    /// Create a new registry with universal transforms registered.
    pub fn new() -> Self {
        Self {}
    }

    /// Get a transform by name for a language.
    ///
    /// Resolution order:
    /// 1. Resolve aliases (e.g., @an -> @a)
    /// 2. Universal transforms (@cap, @upper, @lower)
    /// 3. Language-specific transforms (@a, @the for English)
    pub fn get(&self, name: &str, lang: &str) -> Option<TransformKind> {
        // Resolve aliases first
        let canonical = match name {
            "an" => "a", // English alias: @an resolves to @a
            other => other,
        };

        // Universal transforms are always available
        match canonical {
            "cap" => return Some(TransformKind::Cap),
            "upper" => return Some(TransformKind::Upper),
            "lower" => return Some(TransformKind::Lower),
            _ => {}
        }

        // Language-specific transforms
        match (lang, canonical) {
            ("en", "a") => Some(TransformKind::EnglishA),
            ("en", "the") => Some(TransformKind::EnglishThe),
            _ => None,
        }
    }

    /// Check if a transform exists for a language.
    pub fn has_transform(&self, name: &str, lang: &str) -> bool {
        self.get(name, lang).is_some()
    }
}
