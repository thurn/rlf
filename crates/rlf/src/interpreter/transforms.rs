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
    // German transforms (Phase 6)
    /// @der/@die/@das - German definite article with case context
    GermanDer,
    /// @ein/@eine - German indefinite article with case context
    GermanEin,
    // Dutch transforms (Phase 6)
    /// @de/@het - Dutch definite article from :de/:het tags
    DutchDe,
    /// @een - Dutch indefinite article "een"
    DutchEen,
}

impl TransformKind {
    /// Execute the transform on a value.
    ///
    /// Context is optional (used by language-specific transforms for case, etc.).
    /// Lang is used for locale-sensitive case mapping.
    pub fn execute(
        &self,
        value: &Value,
        context: Option<&Value>,
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
            // German transforms need Value (for tags) and context (for case)
            TransformKind::GermanDer => german_der_transform(value, context),
            TransformKind::GermanEin => german_ein_transform(value, context),
            // Dutch transforms need full Value to read tags
            TransformKind::DutchDe => dutch_de_transform(value),
            TransformKind::DutchEen => dutch_een_transform(value),
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

// =============================================================================
// German Transforms (Phase 6)
// =============================================================================

/// German grammatical gender.
#[derive(Clone, Copy)]
enum GermanGender {
    Masculine,
    Feminine,
    Neuter,
}

/// German grammatical case.
#[derive(Clone, Copy)]
enum GermanCase {
    Nominative,
    Accusative,
    Dative,
    Genitive,
}

/// Parse gender from Value's tags.
fn parse_german_gender(value: &Value) -> Option<GermanGender> {
    if value.has_tag("masc") {
        Some(GermanGender::Masculine)
    } else if value.has_tag("fem") {
        Some(GermanGender::Feminine)
    } else if value.has_tag("neut") {
        Some(GermanGender::Neuter)
    } else {
        None
    }
}

/// Parse case from context value.
///
/// Defaults to nominative if no context or unknown case string.
fn parse_german_case(context: Option<&Value>) -> GermanCase {
    match context {
        Some(Value::String(s)) => match s.as_str() {
            "acc" => GermanCase::Accusative,
            "dat" => GermanCase::Dative,
            "gen" => GermanCase::Genitive,
            _ => GermanCase::Nominative,
        },
        _ => GermanCase::Nominative,
    }
}

/// German definite article lookup table.
///
/// Returns the correct article for gender x case combination.
fn german_definite_article(gender: GermanGender, case: GermanCase) -> &'static str {
    match (gender, case) {
        // Masculine
        (GermanGender::Masculine, GermanCase::Nominative) => "der",
        (GermanGender::Masculine, GermanCase::Accusative) => "den",
        (GermanGender::Masculine, GermanCase::Dative) => "dem",
        (GermanGender::Masculine, GermanCase::Genitive) => "des",
        // Feminine
        (GermanGender::Feminine, GermanCase::Nominative) => "die",
        (GermanGender::Feminine, GermanCase::Accusative) => "die",
        (GermanGender::Feminine, GermanCase::Dative) => "der",
        (GermanGender::Feminine, GermanCase::Genitive) => "der",
        // Neuter
        (GermanGender::Neuter, GermanCase::Nominative) => "das",
        (GermanGender::Neuter, GermanCase::Accusative) => "das",
        (GermanGender::Neuter, GermanCase::Dative) => "dem",
        (GermanGender::Neuter, GermanCase::Genitive) => "des",
    }
}

/// German indefinite article lookup table.
///
/// Returns the correct article for gender x case combination.
fn german_indefinite_article(gender: GermanGender, case: GermanCase) -> &'static str {
    match (gender, case) {
        // Masculine
        (GermanGender::Masculine, GermanCase::Nominative) => "ein",
        (GermanGender::Masculine, GermanCase::Accusative) => "einen",
        (GermanGender::Masculine, GermanCase::Dative) => "einem",
        (GermanGender::Masculine, GermanCase::Genitive) => "eines",
        // Feminine
        (GermanGender::Feminine, GermanCase::Nominative) => "eine",
        (GermanGender::Feminine, GermanCase::Accusative) => "eine",
        (GermanGender::Feminine, GermanCase::Dative) => "einer",
        (GermanGender::Feminine, GermanCase::Genitive) => "einer",
        // Neuter
        (GermanGender::Neuter, GermanCase::Nominative) => "ein",
        (GermanGender::Neuter, GermanCase::Accusative) => "ein",
        (GermanGender::Neuter, GermanCase::Dative) => "einem",
        (GermanGender::Neuter, GermanCase::Genitive) => "eines",
    }
}

/// German definite article transform (@der/@die/@das).
///
/// Reads :masc/:fem/:neut tag from Value to determine gender.
/// Uses context for case (defaults to nominative).
fn german_der_transform(value: &Value, context: Option<&Value>) -> Result<String, EvalError> {
    let text = value.to_string();
    let gender = parse_german_gender(value).ok_or_else(|| EvalError::MissingTag {
        transform: "der".to_string(),
        expected: vec!["masc".to_string(), "fem".to_string(), "neut".to_string()],
        phrase: text.clone(),
    })?;
    let case = parse_german_case(context);
    let article = german_definite_article(gender, case);
    Ok(format!("{} {}", article, text))
}

/// German indefinite article transform (@ein/@eine).
///
/// Reads :masc/:fem/:neut tag from Value to determine gender.
/// Uses context for case (defaults to nominative).
fn german_ein_transform(value: &Value, context: Option<&Value>) -> Result<String, EvalError> {
    let text = value.to_string();
    let gender = parse_german_gender(value).ok_or_else(|| EvalError::MissingTag {
        transform: "ein".to_string(),
        expected: vec!["masc".to_string(), "fem".to_string(), "neut".to_string()],
        phrase: text.clone(),
    })?;
    let case = parse_german_case(context);
    let article = german_indefinite_article(gender, case);
    Ok(format!("{} {}", article, text))
}

// =============================================================================
// Dutch Transforms (Phase 6)
// =============================================================================

/// Dutch definite article transform (@de/@het).
///
/// Reads :de or :het tag from the Value to determine which article to prepend.
/// Dutch has only two grammatical genders for articles: common (de-words) and neuter (het-words).
/// Returns MissingTag error if neither tag is present.
fn dutch_de_transform(value: &Value) -> Result<String, EvalError> {
    let text = value.to_string();

    if value.has_tag("de") {
        return Ok(format!("de {}", text));
    }
    if value.has_tag("het") {
        return Ok(format!("het {}", text));
    }

    Err(EvalError::MissingTag {
        transform: "de".to_string(),
        expected: vec!["de".to_string(), "het".to_string()],
        phrase: text,
    })
}

/// Dutch indefinite article transform (@een).
///
/// Unconditionally prepends "een " to the value's text.
/// Dutch indefinite article is invariant - always "een" regardless of gender.
fn dutch_een_transform(value: &Value) -> Result<String, EvalError> {
    Ok(format!("een {value}"))
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
    /// 1. Resolve aliases (e.g., @an -> @a, @die/@das -> @der)
    /// 2. Universal transforms (@cap, @upper, @lower)
    /// 3. Language-specific transforms (@a, @the for English; @der, @ein for German)
    pub fn get(&self, name: &str, lang: &str) -> Option<TransformKind> {
        // Resolve aliases first
        let canonical = match name {
            "an" => "a",            // English alias: @an resolves to @a
            "die" | "das" => "der", // German aliases: @die/@das resolve to @der
            "eine" => "ein",        // German alias: @eine resolves to @ein
            "het" => "de",          // Dutch alias: @het resolves to @de
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
            ("de", "der") => Some(TransformKind::GermanDer),
            ("de", "ein") => Some(TransformKind::GermanEin),
            ("nl", "de") => Some(TransformKind::DutchDe),
            ("nl", "een") => Some(TransformKind::DutchEen),
            _ => None,
        }
    }

    /// Check if a transform exists for a language.
    pub fn has_transform(&self, name: &str, lang: &str) -> bool {
        self.get(name, lang).is_some()
    }
}
