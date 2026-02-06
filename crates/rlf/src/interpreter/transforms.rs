//! Transform registry for string transformations.
//!
//! Transforms are functions that modify values (e.g., @cap, @upper, @lower).
//! This module provides the registry infrastructure and universal transform implementations.

use hangeul::ends_with_jongseong;
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
    // Spanish transforms (Phase 7)
    /// @el/@la - Spanish definite article with plural context
    SpanishEl,
    /// @un/@una - Spanish indefinite article with plural context
    SpanishUn,
    // Portuguese transforms (Phase 7)
    /// @o/@a - Portuguese definite article with plural context
    PortugueseO,
    /// @um/@uma - Portuguese indefinite article
    PortugueseUm,
    /// @de - Portuguese "de" + article contraction
    PortugueseDe,
    /// @em - Portuguese "em" + article contraction
    PortugueseEm,
    // French transforms (Phase 7)
    /// @le/@la - French definite article with elision
    FrenchLe,
    /// @un/@une - French indefinite article
    FrenchUn,
    /// @de - French "de" + article contraction
    FrenchDe,
    /// @au - French "a" + article contraction
    FrenchAu,
    /// @liaison - French prevocalic form selection
    FrenchLiaison,
    // Italian transforms (Phase 7)
    /// @il/@lo/@la - Italian definite article with sound rules
    ItalianIl,
    /// @un/@uno/@una - Italian indefinite article with sound rules
    ItalianUn,
    /// @di - Italian "di" + article contraction
    ItalianDi,
    /// @a - Italian "a" + article contraction
    ItalianA,
    // Greek transforms (Phase 8)
    /// @o/@i/@to - Greek definite article with case and number
    GreekO,
    /// @enas/@mia/@ena - Greek indefinite article with case
    GreekEnas,
    // Romanian transforms (Phase 8)
    /// @def - Romanian postposed definite article (suffix)
    RomanianDef,
    // Arabic transforms (Phase 8)
    /// @al - Arabic definite article with sun/moon letter assimilation
    ArabicAl,
    // Persian transforms (Phase 8)
    /// @ezafe - Persian ezafe connector (-e/-ye)
    PersianEzafe,
    // CJK transforms (Phase 9)
    /// @count - Chinese count with classifier
    ChineseCount,
    /// @count - Japanese count with counter
    JapaneseCount,
    /// @count - Korean count with counter
    KoreanCount,
    // Southeast Asian transforms (Phase 9)
    /// @count - Vietnamese count with classifier
    VietnameseCount,
    /// @count - Thai count with classifier
    ThaiCount,
    /// @count - Bengali count with classifier
    BengaliCount,
    /// @plural - Indonesian reduplication plural
    IndonesianPlural,
    // Korean particle transform (Phase 9)
    /// @particle - Korean particle selection based on final sound
    KoreanParticle,
    // Turkish inflection transform (Phase 9)
    /// @inflect - Turkish suffix chain with vowel harmony
    TurkishInflect,
    // Finnish inflection transform
    /// @inflect - Finnish suffix chain with vowel harmony
    FinnishInflect,
    // Hungarian inflection transform
    /// @inflect - Hungarian suffix chain with vowel harmony
    HungarianInflect,
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
            // Spanish transforms need Value (for tags) and context (for plural)
            TransformKind::SpanishEl => spanish_el_transform(value, context),
            TransformKind::SpanishUn => spanish_un_transform(value, context),
            // Portuguese transforms need Value (for tags) and context (for plural)
            TransformKind::PortugueseO => portuguese_o_transform(value, context),
            TransformKind::PortugueseUm => portuguese_um_transform(value),
            TransformKind::PortugueseDe => portuguese_de_transform(value, context),
            TransformKind::PortugueseEm => portuguese_em_transform(value, context),
            // French transforms need Value (for tags) and context (for plural/vowel)
            TransformKind::FrenchLe => french_le_transform(value, context),
            TransformKind::FrenchUn => french_un_transform(value),
            TransformKind::FrenchDe => french_de_transform(value, context),
            TransformKind::FrenchAu => french_au_transform(value, context),
            TransformKind::FrenchLiaison => french_liaison_transform(value, context),
            // Italian transforms need Value (for tags) and context (for plural)
            TransformKind::ItalianIl => italian_il_transform(value, context),
            TransformKind::ItalianUn => italian_un_transform(value),
            TransformKind::ItalianDi => italian_di_transform(value, context),
            TransformKind::ItalianA => italian_a_transform(value, context),
            // Greek transforms need Value (for tags) and context (for case/plural)
            TransformKind::GreekO => greek_o_transform(value, context),
            TransformKind::GreekEnas => greek_enas_transform(value, context),
            // Romanian transforms need Value (for tags) and context (for plural)
            TransformKind::RomanianDef => romanian_def_transform(value, context),
            // Arabic transforms need Value (for tags)
            TransformKind::ArabicAl => arabic_al_transform(value),
            // Persian transforms need Value (for tags)
            TransformKind::PersianEzafe => persian_ezafe_transform(value),
            // CJK transforms need Value (for tags) and context (for count)
            TransformKind::ChineseCount => chinese_count_transform(value, context),
            TransformKind::JapaneseCount => japanese_count_transform(value, context),
            TransformKind::KoreanCount => korean_count_transform(value, context),
            // Southeast Asian transforms need Value (for tags) and context (for count)
            TransformKind::VietnameseCount => vietnamese_count_transform(value, context),
            TransformKind::ThaiCount => thai_count_transform(value, context),
            TransformKind::BengaliCount => bengali_count_transform(value, context),
            // Indonesian @plural doesn't need context
            TransformKind::IndonesianPlural => indonesian_plural_transform(value),
            // Korean @particle needs Value (for text) and context (for particle type)
            TransformKind::KoreanParticle => korean_particle_transform(value, context),
            // Turkish @inflect needs Value (for tags) and context (for suffix chain)
            TransformKind::TurkishInflect => turkish_inflect_transform(value, context),
            // Finnish @inflect needs Value (for tags) and context (for suffix chain)
            TransformKind::FinnishInflect => finnish_inflect_transform(value, context),
            // Hungarian @inflect needs Value (for tags) and context (for suffix chain)
            TransformKind::HungarianInflect => hungarian_inflect_transform(value, context),
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

// =============================================================================
// Romance Language Transforms (Phase 7)
// =============================================================================

/// Romance grammatical gender (shared by Spanish, French, Portuguese, Italian).
#[derive(Clone, Copy)]
enum RomanceGender {
    Masculine,
    Feminine,
}

/// Romance plural category (singular/plural).
#[derive(Clone, Copy, PartialEq, Eq)]
enum RomancePlural {
    One,   // Singular
    Other, // Plural
}

/// Parse gender from Value's tags for Romance languages.
/// Returns error if neither :masc nor :fem tag is present.
fn parse_romance_gender(value: &Value, transform: &str) -> Result<RomanceGender, EvalError> {
    if value.has_tag("masc") {
        Ok(RomanceGender::Masculine)
    } else if value.has_tag("fem") {
        Ok(RomanceGender::Feminine)
    } else {
        Err(EvalError::MissingTag {
            transform: transform.to_string(),
            expected: vec!["masc".to_string(), "fem".to_string()],
            phrase: value.to_string(),
        })
    }
}

/// Parse plural category from context value.
/// Supports both string context (:one/:other) and numeric context.
/// Defaults to singular (One) if no context provided.
fn parse_romance_plural(context: Option<&Value>) -> RomancePlural {
    match context {
        Some(Value::String(s)) => match s.as_str() {
            "other" => RomancePlural::Other,
            _ => RomancePlural::One,
        },
        Some(Value::Number(n)) => {
            if *n == 1 {
                RomancePlural::One
            } else {
                RomancePlural::Other
            }
        }
        _ => RomancePlural::One,
    }
}

// =============================================================================
// Spanish Transforms (Phase 7)
// =============================================================================

/// Spanish definite article lookup table.
/// Gender x Plural -> article (el/la/los/las)
fn spanish_definite_article(gender: RomanceGender, plural: RomancePlural) -> &'static str {
    match (gender, plural) {
        (RomanceGender::Masculine, RomancePlural::One) => "el",
        (RomanceGender::Masculine, RomancePlural::Other) => "los",
        (RomanceGender::Feminine, RomancePlural::One) => "la",
        (RomanceGender::Feminine, RomancePlural::Other) => "las",
    }
}

/// Spanish indefinite article lookup table.
/// Gender x Plural -> article (un/una/unos/unas)
fn spanish_indefinite_article(gender: RomanceGender, plural: RomancePlural) -> &'static str {
    match (gender, plural) {
        (RomanceGender::Masculine, RomancePlural::One) => "un",
        (RomanceGender::Masculine, RomancePlural::Other) => "unos",
        (RomanceGender::Feminine, RomancePlural::One) => "una",
        (RomanceGender::Feminine, RomancePlural::Other) => "unas",
    }
}

/// Spanish definite article transform (@el/@la).
fn spanish_el_transform(value: &Value, context: Option<&Value>) -> Result<String, EvalError> {
    let text = value.to_string();
    let gender = parse_romance_gender(value, "el")?;
    let plural = parse_romance_plural(context);
    let article = spanish_definite_article(gender, plural);
    Ok(format!("{} {}", article, text))
}

/// Spanish indefinite article transform (@un/@una).
fn spanish_un_transform(value: &Value, context: Option<&Value>) -> Result<String, EvalError> {
    let text = value.to_string();
    let gender = parse_romance_gender(value, "un")?;
    let plural = parse_romance_plural(context);
    let article = spanish_indefinite_article(gender, plural);
    Ok(format!("{} {}", article, text))
}

// =============================================================================
// Portuguese Transforms (Phase 7)
// =============================================================================

/// Portuguese definite article lookup table.
/// Gender x Plural -> article (o/a/os/as)
fn portuguese_definite_article(gender: RomanceGender, plural: RomancePlural) -> &'static str {
    match (gender, plural) {
        (RomanceGender::Masculine, RomancePlural::One) => "o",
        (RomanceGender::Masculine, RomancePlural::Other) => "os",
        (RomanceGender::Feminine, RomancePlural::One) => "a",
        (RomanceGender::Feminine, RomancePlural::Other) => "as",
    }
}

/// Portuguese indefinite article lookup table.
/// Gender only (no plural for indefinite in Portuguese).
fn portuguese_indefinite_article(gender: RomanceGender) -> &'static str {
    match gender {
        RomanceGender::Masculine => "um",
        RomanceGender::Feminine => "uma",
    }
}

/// Portuguese "de" + article contraction lookup table.
/// de + o = do, de + a = da, de + os = dos, de + as = das
fn portuguese_de_contraction(gender: RomanceGender, plural: RomancePlural) -> &'static str {
    match (gender, plural) {
        (RomanceGender::Masculine, RomancePlural::One) => "do",
        (RomanceGender::Masculine, RomancePlural::Other) => "dos",
        (RomanceGender::Feminine, RomancePlural::One) => "da",
        (RomanceGender::Feminine, RomancePlural::Other) => "das",
    }
}

/// Portuguese "em" + article contraction lookup table.
/// em + o = no, em + a = na, em + os = nos, em + as = nas
fn portuguese_em_contraction(gender: RomanceGender, plural: RomancePlural) -> &'static str {
    match (gender, plural) {
        (RomanceGender::Masculine, RomancePlural::One) => "no",
        (RomanceGender::Masculine, RomancePlural::Other) => "nos",
        (RomanceGender::Feminine, RomancePlural::One) => "na",
        (RomanceGender::Feminine, RomancePlural::Other) => "nas",
    }
}

/// Portuguese definite article transform (@o/@a).
fn portuguese_o_transform(value: &Value, context: Option<&Value>) -> Result<String, EvalError> {
    let text = value.to_string();
    let gender = parse_romance_gender(value, "o")?;
    let plural = parse_romance_plural(context);
    let article = portuguese_definite_article(gender, plural);
    Ok(format!("{} {}", article, text))
}

/// Portuguese indefinite article transform (@um/@uma).
fn portuguese_um_transform(value: &Value) -> Result<String, EvalError> {
    let text = value.to_string();
    let gender = parse_romance_gender(value, "um")?;
    let article = portuguese_indefinite_article(gender);
    Ok(format!("{} {}", article, text))
}

/// Portuguese "de" + article contraction transform (@de).
fn portuguese_de_transform(value: &Value, context: Option<&Value>) -> Result<String, EvalError> {
    let text = value.to_string();
    let gender = parse_romance_gender(value, "de")?;
    let plural = parse_romance_plural(context);
    let contracted = portuguese_de_contraction(gender, plural);
    Ok(format!("{} {}", contracted, text))
}

/// Portuguese "em" + article contraction transform (@em).
fn portuguese_em_transform(value: &Value, context: Option<&Value>) -> Result<String, EvalError> {
    let text = value.to_string();
    let gender = parse_romance_gender(value, "em")?;
    let plural = parse_romance_plural(context);
    let contracted = portuguese_em_contraction(gender, plural);
    Ok(format!("{} {}", contracted, text))
}

// =============================================================================
// French Transforms (Phase 7)
// =============================================================================

/// French definite article lookup table with elision support.
/// Elision produces l' before vowels (singular only).
/// Always returns lowercase - capitalization handled by @cap transform.
fn french_definite_article(
    gender: RomanceGender,
    has_vowel: bool,
    plural: RomancePlural,
) -> &'static str {
    match (gender, has_vowel, plural) {
        // Elision before vowel (singular only)
        (_, true, RomancePlural::One) => "l'",
        // Masculine singular
        (RomanceGender::Masculine, false, RomancePlural::One) => "le",
        // Feminine singular
        (RomanceGender::Feminine, false, RomancePlural::One) => "la",
        // Plural (same for both genders, no elision)
        (_, _, RomancePlural::Other) => "les",
    }
}

/// French indefinite article lookup table.
/// No plural forms per APPENDIX_STDLIB.
/// Always returns lowercase - capitalization handled by @cap transform.
fn french_indefinite_article(gender: RomanceGender) -> &'static str {
    match gender {
        RomanceGender::Masculine => "un",
        RomanceGender::Feminine => "une",
    }
}

/// French "de" + article contraction lookup table with elision.
/// Always returns lowercase - capitalization handled by @cap transform.
fn french_de_contraction(
    gender: RomanceGender,
    has_vowel: bool,
    plural: RomancePlural,
) -> &'static str {
    match (gender, has_vowel, plural) {
        // Elision: de + l' -> de l' (no contraction, but elided article)
        (_, true, RomancePlural::One) => "de l'",
        // Masculine singular: de + le -> du
        (RomanceGender::Masculine, false, RomancePlural::One) => "du",
        // Feminine singular: de + la -> de la (no contraction)
        (RomanceGender::Feminine, false, RomancePlural::One) => "de la",
        // Plural: de + les -> des
        (_, _, RomancePlural::Other) => "des",
    }
}

/// French "a" + article contraction lookup table with elision.
/// Always returns lowercase - capitalization handled by @cap transform.
fn french_au_contraction(
    gender: RomanceGender,
    has_vowel: bool,
    plural: RomancePlural,
) -> &'static str {
    match (gender, has_vowel, plural) {
        // Elision: a + l' -> a l' (no contraction, but elided article)
        (_, true, RomancePlural::One) => "a l'",
        // Masculine singular: a + le -> au
        (RomanceGender::Masculine, false, RomancePlural::One) => "au",
        // Feminine singular: a + la -> a la (no contraction)
        (RomanceGender::Feminine, false, RomancePlural::One) => "a la",
        // Plural: a + les -> aux
        (_, _, RomancePlural::Other) => "aux",
    }
}

/// French definite article transform (@le/@la).
/// Handles elision before vowels via :vowel tag.
fn french_le_transform(value: &Value, context: Option<&Value>) -> Result<String, EvalError> {
    let text = value.to_string();
    let gender = parse_romance_gender(value, "le")?;
    let has_vowel = value.has_tag("vowel");
    let plural = parse_romance_plural(context);
    let article = french_definite_article(gender, has_vowel, plural);

    // Elided article (l') attaches directly, no space
    if article.ends_with('\'') {
        Ok(format!("{}{}", article, text))
    } else {
        Ok(format!("{} {}", article, text))
    }
}

/// French indefinite article transform (@un/@une).
fn french_un_transform(value: &Value) -> Result<String, EvalError> {
    let text = value.to_string();
    let gender = parse_romance_gender(value, "un")?;
    let article = french_indefinite_article(gender);
    Ok(format!("{} {}", article, text))
}

/// French "de" + article contraction transform (@de).
fn french_de_transform(value: &Value, context: Option<&Value>) -> Result<String, EvalError> {
    let text = value.to_string();
    let gender = parse_romance_gender(value, "de")?;
    let has_vowel = value.has_tag("vowel");
    let plural = parse_romance_plural(context);
    let contracted = french_de_contraction(gender, has_vowel, plural);

    // "de l'" has apostrophe - attach directly
    if contracted.ends_with('\'') {
        Ok(format!("{}{}", contracted, text))
    } else {
        Ok(format!("{} {}", contracted, text))
    }
}

/// French "a" + article contraction transform (@au).
fn french_au_transform(value: &Value, context: Option<&Value>) -> Result<String, EvalError> {
    let text = value.to_string();
    let gender = parse_romance_gender(value, "au")?;
    let has_vowel = value.has_tag("vowel");
    let plural = parse_romance_plural(context);
    let contracted = french_au_contraction(gender, has_vowel, plural);

    // "a l'" has apostrophe - attach directly
    if contracted.ends_with('\'') {
        Ok(format!("{}{}", contracted, text))
    } else {
        Ok(format!("{} {}", contracted, text))
    }
}

/// French liaison transform (@liaison).
/// Selects between standard and prevocalic forms based on context's :vowel tag.
/// The input value should have variants "standard" and "vowel".
/// Output is just the selected variant - context is only used to determine selection.
fn french_liaison_transform(value: &Value, context: Option<&Value>) -> Result<String, EvalError> {
    use crate::types::VariantKey;

    // Context should be a phrase with :vowel tag (or not)
    let has_vowel = match context {
        Some(v) => v.has_tag("vowel"),
        None => false,
    };

    // Select the appropriate variant from the liaison word
    let variant_key = if has_vowel { "vowel" } else { "standard" };

    // Try to get the variant from the value
    if let Value::Phrase(phrase) = value
        && let Some(variant_text) = phrase.variants.get(&VariantKey::new(variant_key))
    {
        return Ok(variant_text.clone());
    }

    // Fallback: just use the text as-is (default variant)
    Ok(value.to_string())
}

// =============================================================================
// Italian Transforms (Phase 7)
// =============================================================================

/// Italian sound category for article selection.
#[derive(Clone, Copy)]
enum ItalianSound {
    /// Standard consonant (il, un, del, al)
    Normal,
    /// Starts with vowel (l', un, dell', all')
    Vowel,
    /// Starts with s+consonant, z, gn, ps, x (lo, uno, dello, allo)
    SImpura,
}

/// Parse Italian sound category from tags.
fn parse_italian_sound(value: &Value) -> ItalianSound {
    if value.has_tag("vowel") {
        ItalianSound::Vowel
    } else if value.has_tag("s_imp") {
        ItalianSound::SImpura
    } else {
        ItalianSound::Normal
    }
}

/// Italian definite article lookup table.
/// Gender x Sound x Plural -> article
/// Always returns lowercase - capitalization handled by @cap transform.
fn italian_definite_article(
    gender: RomanceGender,
    sound: ItalianSound,
    plural: RomancePlural,
) -> &'static str {
    match (gender, sound, plural) {
        // Masculine singular
        (RomanceGender::Masculine, ItalianSound::Normal, RomancePlural::One) => "il",
        (RomanceGender::Masculine, ItalianSound::Vowel, RomancePlural::One) => "l'",
        (RomanceGender::Masculine, ItalianSound::SImpura, RomancePlural::One) => "lo",
        // Masculine plural
        (RomanceGender::Masculine, ItalianSound::Normal, RomancePlural::Other) => "i",
        (RomanceGender::Masculine, ItalianSound::Vowel, RomancePlural::Other) => "gli",
        (RomanceGender::Masculine, ItalianSound::SImpura, RomancePlural::Other) => "gli",
        // Feminine singular
        (RomanceGender::Feminine, ItalianSound::Vowel, RomancePlural::One) => "l'",
        (RomanceGender::Feminine, _, RomancePlural::One) => "la",
        // Feminine plural
        (RomanceGender::Feminine, _, RomancePlural::Other) => "le",
    }
}

/// Italian indefinite article lookup table.
/// Gender x Sound -> article
/// Always returns lowercase - capitalization handled by @cap transform.
fn italian_indefinite_article(gender: RomanceGender, sound: ItalianSound) -> &'static str {
    match (gender, sound) {
        // Masculine
        (RomanceGender::Masculine, ItalianSound::Normal) => "un",
        (RomanceGender::Masculine, ItalianSound::Vowel) => "un",
        (RomanceGender::Masculine, ItalianSound::SImpura) => "uno",
        // Feminine
        (RomanceGender::Feminine, ItalianSound::Vowel) => "un'",
        (RomanceGender::Feminine, _) => "una",
    }
}

/// Italian "di" + article contraction lookup table.
/// Always returns lowercase - capitalization handled by @cap transform.
fn italian_di_contraction(
    gender: RomanceGender,
    sound: ItalianSound,
    plural: RomancePlural,
) -> &'static str {
    match (gender, sound, plural) {
        // Masculine singular
        (RomanceGender::Masculine, ItalianSound::Normal, RomancePlural::One) => "del",
        (RomanceGender::Masculine, ItalianSound::Vowel, RomancePlural::One) => "dell'",
        (RomanceGender::Masculine, ItalianSound::SImpura, RomancePlural::One) => "dello",
        // Masculine plural
        (RomanceGender::Masculine, ItalianSound::Normal, RomancePlural::Other) => "dei",
        (RomanceGender::Masculine, ItalianSound::Vowel, RomancePlural::Other) => "degli",
        (RomanceGender::Masculine, ItalianSound::SImpura, RomancePlural::Other) => "degli",
        // Feminine singular
        (RomanceGender::Feminine, ItalianSound::Vowel, RomancePlural::One) => "dell'",
        (RomanceGender::Feminine, _, RomancePlural::One) => "della",
        // Feminine plural
        (RomanceGender::Feminine, _, RomancePlural::Other) => "delle",
    }
}

/// Italian "a" + article contraction lookup table.
/// Always returns lowercase - capitalization handled by @cap transform.
fn italian_a_contraction(
    gender: RomanceGender,
    sound: ItalianSound,
    plural: RomancePlural,
) -> &'static str {
    match (gender, sound, plural) {
        // Masculine singular
        (RomanceGender::Masculine, ItalianSound::Normal, RomancePlural::One) => "al",
        (RomanceGender::Masculine, ItalianSound::Vowel, RomancePlural::One) => "all'",
        (RomanceGender::Masculine, ItalianSound::SImpura, RomancePlural::One) => "allo",
        // Masculine plural
        (RomanceGender::Masculine, ItalianSound::Normal, RomancePlural::Other) => "ai",
        (RomanceGender::Masculine, ItalianSound::Vowel, RomancePlural::Other) => "agli",
        (RomanceGender::Masculine, ItalianSound::SImpura, RomancePlural::Other) => "agli",
        // Feminine singular
        (RomanceGender::Feminine, ItalianSound::Vowel, RomancePlural::One) => "all'",
        (RomanceGender::Feminine, _, RomancePlural::One) => "alla",
        // Feminine plural
        (RomanceGender::Feminine, _, RomancePlural::Other) => "alle",
    }
}

/// Italian definite article transform (@il/@lo/@la).
fn italian_il_transform(value: &Value, context: Option<&Value>) -> Result<String, EvalError> {
    let text = value.to_string();
    let gender = parse_romance_gender(value, "il")?;
    let sound = parse_italian_sound(value);
    let plural = parse_romance_plural(context);
    let article = italian_definite_article(gender, sound, plural);

    // Apostrophe articles attach directly
    if article.ends_with('\'') {
        Ok(format!("{}{}", article, text))
    } else {
        Ok(format!("{} {}", article, text))
    }
}

/// Italian indefinite article transform (@un/@uno/@una).
fn italian_un_transform(value: &Value) -> Result<String, EvalError> {
    let text = value.to_string();
    let gender = parse_romance_gender(value, "un")?;
    let sound = parse_italian_sound(value);
    let article = italian_indefinite_article(gender, sound);

    // Apostrophe articles attach directly (un'amica)
    if article.ends_with('\'') {
        Ok(format!("{}{}", article, text))
    } else {
        Ok(format!("{} {}", article, text))
    }
}

/// Italian "di" + article contraction transform (@di).
fn italian_di_transform(value: &Value, context: Option<&Value>) -> Result<String, EvalError> {
    let text = value.to_string();
    let gender = parse_romance_gender(value, "di")?;
    let sound = parse_italian_sound(value);
    let plural = parse_romance_plural(context);
    let contracted = italian_di_contraction(gender, sound, plural);

    if contracted.ends_with('\'') {
        Ok(format!("{}{}", contracted, text))
    } else {
        Ok(format!("{} {}", contracted, text))
    }
}

/// Italian "a" + article contraction transform (@a).
fn italian_a_transform(value: &Value, context: Option<&Value>) -> Result<String, EvalError> {
    let text = value.to_string();
    let gender = parse_romance_gender(value, "a")?;
    let sound = parse_italian_sound(value);
    let plural = parse_romance_plural(context);
    let contracted = italian_a_contraction(gender, sound, plural);

    if contracted.ends_with('\'') {
        Ok(format!("{}{}", contracted, text))
    } else {
        Ok(format!("{} {}", contracted, text))
    }
}

// =============================================================================
// Greek Transforms (Phase 8)
// =============================================================================

/// Greek grammatical gender.
#[derive(Clone, Copy)]
enum GreekGender {
    Masculine,
    Feminine,
    Neuter,
}

/// Greek grammatical case.
#[derive(Clone, Copy)]
enum GreekCase {
    Nominative,
    Accusative,
    Genitive,
    Dative, // Archaic in modern Greek, but supported per spec
}

/// Parse gender from Value's tags for Greek.
/// Returns error if no gender tag is present.
fn parse_greek_gender(value: &Value, transform: &str) -> Result<GreekGender, EvalError> {
    if value.has_tag("masc") {
        Ok(GreekGender::Masculine)
    } else if value.has_tag("fem") {
        Ok(GreekGender::Feminine)
    } else if value.has_tag("neut") {
        Ok(GreekGender::Neuter)
    } else {
        Err(EvalError::MissingTag {
            transform: transform.to_string(),
            expected: vec!["masc".to_string(), "fem".to_string(), "neut".to_string()],
            phrase: value.to_string(),
        })
    }
}

/// Parse case from context value.
/// Defaults to nominative if no context or unknown case string.
fn parse_greek_case(context: Option<&Value>) -> GreekCase {
    match context {
        Some(Value::String(s)) => match s.as_str() {
            "acc" => GreekCase::Accusative,
            "gen" => GreekCase::Genitive,
            "dat" => GreekCase::Dative,
            _ => GreekCase::Nominative,
        },
        _ => GreekCase::Nominative,
    }
}

/// Greek definite article lookup table - singular.
/// Returns the correct article for gender x case combination.
fn greek_definite_article_singular(gender: GreekGender, case: GreekCase) -> &'static str {
    match (gender, case) {
        // Masculine: ο/τον/του/τω
        (GreekGender::Masculine, GreekCase::Nominative) => "ο",
        (GreekGender::Masculine, GreekCase::Accusative) => "τον",
        (GreekGender::Masculine, GreekCase::Genitive) => "του",
        (GreekGender::Masculine, GreekCase::Dative) => "τω",
        // Feminine: η/την/της/τη
        (GreekGender::Feminine, GreekCase::Nominative) => "η",
        (GreekGender::Feminine, GreekCase::Accusative) => "την",
        (GreekGender::Feminine, GreekCase::Genitive) => "της",
        (GreekGender::Feminine, GreekCase::Dative) => "τη",
        // Neuter: το/το/του/τω
        (GreekGender::Neuter, GreekCase::Nominative) => "το",
        (GreekGender::Neuter, GreekCase::Accusative) => "το",
        (GreekGender::Neuter, GreekCase::Genitive) => "του",
        (GreekGender::Neuter, GreekCase::Dative) => "τω",
    }
}

/// Greek definite article lookup table - plural.
/// Returns the correct article for gender x case combination.
fn greek_definite_article_plural(gender: GreekGender, case: GreekCase) -> &'static str {
    match (gender, case) {
        // Masculine: οι/τους/των/τοις
        (GreekGender::Masculine, GreekCase::Nominative) => "οι",
        (GreekGender::Masculine, GreekCase::Accusative) => "τους",
        (GreekGender::Masculine, GreekCase::Genitive) => "των",
        (GreekGender::Masculine, GreekCase::Dative) => "τοις",
        // Feminine: οι/τις/των/ταις
        (GreekGender::Feminine, GreekCase::Nominative) => "οι",
        (GreekGender::Feminine, GreekCase::Accusative) => "τις",
        (GreekGender::Feminine, GreekCase::Genitive) => "των",
        (GreekGender::Feminine, GreekCase::Dative) => "ταις",
        // Neuter: τα/τα/των/τοις
        (GreekGender::Neuter, GreekCase::Nominative) => "τα",
        (GreekGender::Neuter, GreekCase::Accusative) => "τα",
        (GreekGender::Neuter, GreekCase::Genitive) => "των",
        (GreekGender::Neuter, GreekCase::Dative) => "τοις",
    }
}

/// Greek indefinite article lookup table (singular only).
/// Returns the correct article for gender x case combination.
fn greek_indefinite_article(gender: GreekGender, case: GreekCase) -> &'static str {
    match (gender, case) {
        // Masculine: ένας/έναν/ενός/ενί
        (GreekGender::Masculine, GreekCase::Nominative) => "ένας",
        (GreekGender::Masculine, GreekCase::Accusative) => "έναν",
        (GreekGender::Masculine, GreekCase::Genitive) => "ενός",
        (GreekGender::Masculine, GreekCase::Dative) => "ενί",
        // Feminine: μία/μία/μιας/μια
        (GreekGender::Feminine, GreekCase::Nominative) => "μία",
        (GreekGender::Feminine, GreekCase::Accusative) => "μία",
        (GreekGender::Feminine, GreekCase::Genitive) => "μιας",
        (GreekGender::Feminine, GreekCase::Dative) => "μια",
        // Neuter: ένα/ένα/ενός/ενί
        (GreekGender::Neuter, GreekCase::Nominative) => "ένα",
        (GreekGender::Neuter, GreekCase::Accusative) => "ένα",
        (GreekGender::Neuter, GreekCase::Genitive) => "ενός",
        (GreekGender::Neuter, GreekCase::Dative) => "ενί",
    }
}

/// Greek definite article transform (@o/@i/@to).
///
/// Reads :masc/:fem/:neut tag from Value to determine gender.
/// Uses context for case (defaults to nominative) and plural.
fn greek_o_transform(value: &Value, context: Option<&Value>) -> Result<String, EvalError> {
    let text = value.to_string();
    let gender = parse_greek_gender(value, "o")?;
    let case = parse_greek_case(context);
    let plural = parse_romance_plural(context);

    let article = if plural == RomancePlural::One {
        greek_definite_article_singular(gender, case)
    } else {
        greek_definite_article_plural(gender, case)
    };

    Ok(format!("{} {}", article, text))
}

/// Greek indefinite article transform (@enas/@mia/@ena).
///
/// Reads :masc/:fem/:neut tag from Value to determine gender.
/// Uses context for case (defaults to nominative).
/// Note: Greek indefinite articles exist only in singular form.
fn greek_enas_transform(value: &Value, context: Option<&Value>) -> Result<String, EvalError> {
    let text = value.to_string();
    let gender = parse_greek_gender(value, "enas")?;
    let case = parse_greek_case(context);
    let article = greek_indefinite_article(gender, case);

    Ok(format!("{} {}", article, text))
}

// =============================================================================
// Romanian Transforms (Phase 8)
// =============================================================================

/// Romanian grammatical gender.
/// Note: Neuter behaves as masculine in singular, feminine in plural.
#[derive(Clone, Copy)]
enum RomanianGender {
    Masculine,
    Feminine,
    Neuter,
}

/// Parse gender from Value's tags for Romanian.
/// Returns error if no gender tag is present.
fn parse_romanian_gender(value: &Value, transform: &str) -> Result<RomanianGender, EvalError> {
    if value.has_tag("masc") {
        Ok(RomanianGender::Masculine)
    } else if value.has_tag("fem") {
        Ok(RomanianGender::Feminine)
    } else if value.has_tag("neut") {
        Ok(RomanianGender::Neuter)
    } else {
        Err(EvalError::MissingTag {
            transform: transform.to_string(),
            expected: vec!["masc".to_string(), "fem".to_string(), "neut".to_string()],
            phrase: value.to_string(),
        })
    }
}

/// Romanian definite article suffix lookup table (nominative/accusative).
/// APPENDS suffix to word, not prepends.
fn romanian_definite_suffix(gender: RomanianGender, plural: RomancePlural) -> &'static str {
    match (gender, plural) {
        // Masculine: -ul (sg), -ii (pl)
        (RomanianGender::Masculine, RomancePlural::One) => "-ul",
        (RomanianGender::Masculine, RomancePlural::Other) => "-ii",
        // Feminine: -a (sg), -le (pl)
        (RomanianGender::Feminine, RomancePlural::One) => "-a",
        (RomanianGender::Feminine, RomancePlural::Other) => "-le",
        // Neuter: like masculine singular, like feminine plural
        (RomanianGender::Neuter, RomancePlural::One) => "-ul",
        (RomanianGender::Neuter, RomancePlural::Other) => "-le",
    }
}

/// Romanian postposed definite article transform (@def).
///
/// APPENDS article suffix to word (unique among Romance languages).
/// Per CONTEXT.md: neuter singular -> masculine suffix, neuter plural -> feminine suffix.
fn romanian_def_transform(value: &Value, context: Option<&Value>) -> Result<String, EvalError> {
    let text = value.to_string();
    let gender = parse_romanian_gender(value, "def")?;
    let plural = parse_romance_plural(context);
    let suffix = romanian_definite_suffix(gender, plural);

    // Remove the leading dash from the suffix for display
    let suffix_text = suffix.trim_start_matches('-');

    // APPEND suffix, not prepend (Romanian postposed article)
    Ok(format!("{}{}", text, suffix_text))
}

// =============================================================================
// Arabic Transforms (Phase 8)
// =============================================================================

/// Arabic shadda (consonant doubling mark).
/// Unicode: U+0651 (ARABIC SHADDA)
const SHADDA: char = '\u{0651}';

/// Arabic definite article transform (@al).
///
/// Handles sun/moon letter assimilation via :sun/:moon tags.
/// - Sun letters: assimilation occurs, first consonant doubles (shadda added)
/// - Moon letters: no assimilation, plain "ال" prefix
///
/// Per CONTEXT.md: uses :sun/:moon tags, no automatic detection.
fn arabic_al_transform(value: &Value) -> Result<String, EvalError> {
    let text = value.to_string();

    if value.has_tag("sun") {
        // Sun letter: assimilation occurs
        // Get first character and add shadda after it to indicate doubling
        // Output: "ال" + first_char + shadda + rest_of_text
        if let Some(first_char) = text.chars().next() {
            let rest: String = text.chars().skip(1).collect();
            // Per RESEARCH.md pitfall: shadda goes AFTER the consonant, not before
            return Ok(format!("ال{}{}{}", first_char, SHADDA, rest));
        }
        // Fallback if empty text
        return Ok(format!("ال{}", text));
    }

    if value.has_tag("moon") {
        // Moon letter: no assimilation, plain prefix
        return Ok(format!("ال{}", text));
    }

    Err(EvalError::MissingTag {
        transform: "al".to_string(),
        expected: vec!["sun".to_string(), "moon".to_string()],
        phrase: text,
    })
}

// =============================================================================
// Persian Transforms (Phase 8)
// =============================================================================

/// Persian kasra (short 'e' vowel mark).
/// Unicode: U+0650 (ARABIC KASRA)
const KASRA: char = '\u{0650}';

/// Persian zero-width non-joiner.
/// Unicode: U+200C (ZERO WIDTH NON-JOINER)
/// Used to prevent letter joining before ezafe connector.
const ZWNJ: &str = "\u{200C}";

/// Persian ye character for ezafe.
/// Unicode: U+06CC (ARABIC LETTER FARSI YEH)
const PERSIAN_YE: char = '\u{06CC}';

/// Persian ezafe connector transform (@ezafe).
///
/// Connects nouns to modifiers with -e or -ye based on word ending.
/// - Words ending in vowel (:vowel tag): use -ye connector with ZWNJ
/// - Words ending in consonant (no :vowel tag): use -e (kasra)
///
/// Per CONTEXT.md: no gender system in Persian.
fn persian_ezafe_transform(value: &Value) -> Result<String, EvalError> {
    let text = value.to_string();

    if value.has_tag("vowel") {
        // Word ends in vowel: use -ye connector
        // Per RESEARCH.md: include ZWNJ before ye for proper rendering
        Ok(format!("{}{}{}", text, ZWNJ, PERSIAN_YE))
    } else {
        // Word ends in consonant: use -e (kasra)
        // Kasra is placed after the final letter
        Ok(format!("{}{}", text, KASRA))
    }
}

// =============================================================================
// CJK Transforms (Phase 9)
// =============================================================================

/// Chinese measure word classifiers.
/// Tag name -> classifier character.
const CHINESE_CLASSIFIERS: &[(&str, &str)] = &[
    ("zhang", "张"), // Flat objects (cards, paper)
    ("ge", "个"),    // General classifier
    ("ming", "名"),  // People (formal)
    ("wei", "位"),   // People (respectful)
    ("tiao", "条"),  // Long thin objects
    ("ben", "本"),   // Books, volumes
    ("zhi", "只"),   // Animals, hands
];

/// Japanese counters.
/// Tag name -> counter character.
const JAPANESE_COUNTERS: &[(&str, &str)] = &[
    ("mai", "枚"),   // Flat objects
    ("nin", "人"),   // People
    ("hiki", "匹"),  // Small animals
    ("hon", "本"),   // Long objects
    ("ko", "個"),    // General small objects
    ("satsu", "冊"), // Books
];

/// Korean counters.
/// Tag name -> counter character.
const KOREAN_COUNTERS: &[(&str, &str)] = &[
    ("jang", "장"),   // Flat objects
    ("myeong", "명"), // People (formal)
    ("mari", "마리"), // Animals
    ("gae", "개"),    // General objects
    ("gwon", "권"),   // Books
];

/// Vietnamese classifiers.
/// Tag name -> classifier word (Vietnamese uses Latin script).
const VIETNAMESE_CLASSIFIERS: &[(&str, &str)] = &[
    ("cai", "cai"),     // General objects (cái)
    ("con", "con"),     // Animals, some objects
    ("nguoi", "nguoi"), // People (người)
    ("chiec", "chiec"), // Vehicles, single items (chiếc)
    ("to", "to"),       // Flat paper items (tờ)
];

/// Thai classifiers.
/// Tag name -> classifier character.
const THAI_CLASSIFIERS: &[(&str, &str)] = &[
    ("bai", "ใบ"),  // Flat objects, cards
    ("tua", "ตัว"),  // Animals, letters, characters
    ("khon", "คน"), // People
    ("an", "อัน"),   // General small objects
];

/// Bengali classifiers.
/// Tag name -> classifier character.
const BENGALI_CLASSIFIERS: &[(&str, &str)] = &[
    ("ta", "টা"),     // General classifier
    ("ti", "টি"),    // Formal classifier
    ("khana", "খানা"), // For flat objects
    ("jon", "জন"),   // For people
];

/// Extract count value from context.
fn context_to_count(context: Option<&Value>) -> i64 {
    match context {
        Some(Value::Number(n)) => *n,
        Some(Value::String(s)) => s.parse().unwrap_or(1),
        _ => 1,
    }
}

/// Find classifier/counter from a lookup table based on value tags.
fn find_classifier<'a>(value: &Value, classifiers: &'a [(&str, &str)]) -> Option<&'a str> {
    for (tag, classifier) in classifiers {
        if value.has_tag(tag) {
            return Some(classifier);
        }
    }
    None
}

/// Chinese @count transform.
///
/// Produces "{count}{classifier}{noun}" format.
/// Requires classifier tag (zhang, ge, ming, wei, tiao, ben, zhi).
fn chinese_count_transform(value: &Value, context: Option<&Value>) -> Result<String, EvalError> {
    let text = value.to_string();
    let count = context_to_count(context);

    let classifier =
        find_classifier(value, CHINESE_CLASSIFIERS).ok_or_else(|| EvalError::MissingTag {
            transform: "count".to_string(),
            expected: CHINESE_CLASSIFIERS
                .iter()
                .map(|(t, _)| t.to_string())
                .collect(),
            phrase: text.clone(),
        })?;

    Ok(format!("{}{}{}", count, classifier, text))
}

/// Japanese @count transform.
///
/// Produces "{count}{counter}{noun}" format.
/// Requires counter tag (mai, nin, hiki, hon, ko, satsu).
fn japanese_count_transform(value: &Value, context: Option<&Value>) -> Result<String, EvalError> {
    let text = value.to_string();
    let count = context_to_count(context);

    let counter =
        find_classifier(value, JAPANESE_COUNTERS).ok_or_else(|| EvalError::MissingTag {
            transform: "count".to_string(),
            expected: JAPANESE_COUNTERS
                .iter()
                .map(|(t, _)| t.to_string())
                .collect(),
            phrase: text.clone(),
        })?;

    Ok(format!("{}{}{}", count, counter, text))
}

/// Korean @count transform.
///
/// Produces "{count}{counter}{noun}" format.
/// Requires counter tag (jang, myeong, mari, gae, gwon).
fn korean_count_transform(value: &Value, context: Option<&Value>) -> Result<String, EvalError> {
    let text = value.to_string();
    let count = context_to_count(context);

    let counter = find_classifier(value, KOREAN_COUNTERS).ok_or_else(|| EvalError::MissingTag {
        transform: "count".to_string(),
        expected: KOREAN_COUNTERS.iter().map(|(t, _)| t.to_string()).collect(),
        phrase: text.clone(),
    })?;

    Ok(format!("{}{}{}", count, counter, text))
}

// =============================================================================
// Southeast Asian Transforms (Phase 9)
// =============================================================================

/// Vietnamese @count transform.
///
/// Produces "{count} {classifier} {noun}" format (spaces between elements).
/// Requires classifier tag (cai, con, nguoi, chiec, to).
fn vietnamese_count_transform(value: &Value, context: Option<&Value>) -> Result<String, EvalError> {
    let text = value.to_string();
    let count = context_to_count(context);

    let classifier =
        find_classifier(value, VIETNAMESE_CLASSIFIERS).ok_or_else(|| EvalError::MissingTag {
            transform: "count".to_string(),
            expected: VIETNAMESE_CLASSIFIERS
                .iter()
                .map(|(t, _)| t.to_string())
                .collect(),
            phrase: text.clone(),
        })?;

    // Vietnamese uses spaces between elements
    Ok(format!("{} {} {}", count, classifier, text))
}

/// Thai @count transform.
///
/// Produces "{count}{classifier}{noun}" format (no spaces).
/// Requires classifier tag (bai, tua, khon, an).
fn thai_count_transform(value: &Value, context: Option<&Value>) -> Result<String, EvalError> {
    let text = value.to_string();
    let count = context_to_count(context);

    let classifier =
        find_classifier(value, THAI_CLASSIFIERS).ok_or_else(|| EvalError::MissingTag {
            transform: "count".to_string(),
            expected: THAI_CLASSIFIERS
                .iter()
                .map(|(t, _)| t.to_string())
                .collect(),
            phrase: text.clone(),
        })?;

    // Thai uses no spaces between elements
    Ok(format!("{}{}{}", count, classifier, text))
}

/// Bengali @count transform.
///
/// Produces "{count}{classifier} {noun}" format (classifier attached to number, space before noun).
/// Requires classifier tag (ta, ti, khana, jon).
fn bengali_count_transform(value: &Value, context: Option<&Value>) -> Result<String, EvalError> {
    let text = value.to_string();
    let count = context_to_count(context);

    let classifier =
        find_classifier(value, BENGALI_CLASSIFIERS).ok_or_else(|| EvalError::MissingTag {
            transform: "count".to_string(),
            expected: BENGALI_CLASSIFIERS
                .iter()
                .map(|(t, _)| t.to_string())
                .collect(),
            phrase: text.clone(),
        })?;

    // Bengali: classifier immediately after number, then space, then noun
    Ok(format!("{}{} {}", count, classifier, text))
}

/// Indonesian @plural transform.
///
/// Produces "{text}-{text}" format (reduplication).
/// No tags required, no context needed.
fn indonesian_plural_transform(value: &Value) -> Result<String, EvalError> {
    let text = value.to_string();
    Ok(format!("{}-{}", text, text))
}

// =============================================================================
// Korean Particle Transform (Phase 9)
// =============================================================================

/// Korean particle type for @particle transform.
#[derive(Clone, Copy)]
enum KoreanParticleType {
    /// Subject particle: 가 (vowel-final) / 이 (consonant-final)
    Subject,
    /// Object particle: 를 (vowel-final) / 을 (consonant-final)
    Object,
    /// Topic particle: 는 (vowel-final) / 은 (consonant-final)
    Topic,
}

/// Korean @particle transform.
///
/// Selects the appropriate particle form based on whether the preceding word
/// ends in a vowel or consonant (jongseong/batchim).
///
/// Particle types from context:
/// - "subj" -> Subject (가/이)
/// - "obj" -> Object (를/을)
/// - "topic" -> Topic (는/은)
/// - Default to Subject if no context
///
/// Returns ONLY the particle (not prepended to text).
fn korean_particle_transform(value: &Value, context: Option<&Value>) -> Result<String, EvalError> {
    let text = value.to_string();

    let particle_type = match context {
        Some(Value::String(s)) => match s.as_str() {
            "subj" => KoreanParticleType::Subject,
            "obj" => KoreanParticleType::Object,
            "topic" => KoreanParticleType::Topic,
            _ => KoreanParticleType::Subject,
        },
        _ => KoreanParticleType::Subject,
    };

    // Check if text ends in consonant (has jongseong/batchim)
    // For non-Hangul text or errors, treat as vowel-ending (returns false)
    let consonant_ending = ends_with_jongseong(&text).unwrap_or(false);

    let particle = match (particle_type, consonant_ending) {
        (KoreanParticleType::Subject, false) => "가",
        (KoreanParticleType::Subject, true) => "이",
        (KoreanParticleType::Object, false) => "를",
        (KoreanParticleType::Object, true) => "을",
        (KoreanParticleType::Topic, false) => "는",
        (KoreanParticleType::Topic, true) => "은",
    };

    Ok(particle.to_string())
}

// =============================================================================
// Turkish Inflect Transform (Phase 9)
// =============================================================================

/// Turkish vowel harmony type.
#[derive(Clone, Copy)]
enum TurkishHarmony {
    /// Front vowels: e, i, ö, ü
    Front,
    /// Back vowels: a, ı, o, u
    Back,
}

/// Turkish suffix types for @inflect transform.
#[derive(Clone, Copy)]
enum TurkishSuffix {
    /// Plural: -ler/-lar
    Plural,
    /// Nominative: no suffix (unmarked case)
    Nominative,
    /// Accusative: -i/-ı (definite object)
    Accusative,
    /// Genitive: -in/-ın (possession)
    Genitive,
    /// Dative: -e/-a (direction/recipient)
    Dative,
    /// Locative: -de/-da (location)
    Locative,
    /// Ablative: -den/-dan (source/origin)
    Ablative,
    /// Possessive 1st person singular: -im/-ım (my)
    Poss1Sg,
    /// Possessive 2nd person singular: -in/-ın (your)
    Poss2Sg,
    /// Possessive 3rd person singular: -i/-ı (his/her/its)
    Poss3Sg,
    /// Possessive 1st person plural: -imiz/-ımız (our)
    Poss1Pl,
    /// Possessive 2nd person plural: -iniz/-ınız (your, pl.)
    Poss2Pl,
    /// Possessive 3rd person plural: -leri/-ları (their)
    Poss3Pl,
}

/// Parse suffix chain from context value.
///
/// Parses dot-separated suffix names: "pl.poss1sg.abl" -> [Plural, Poss1Sg, Ablative]
fn parse_turkish_suffix_chain(context: Option<&Value>) -> Vec<TurkishSuffix> {
    let Some(Value::String(s)) = context else {
        return Vec::new();
    };

    s.split('.')
        .filter_map(|part| match part {
            "pl" => Some(TurkishSuffix::Plural),
            "nom" => Some(TurkishSuffix::Nominative),
            "acc" => Some(TurkishSuffix::Accusative),
            "gen" => Some(TurkishSuffix::Genitive),
            "dat" => Some(TurkishSuffix::Dative),
            "loc" => Some(TurkishSuffix::Locative),
            "abl" => Some(TurkishSuffix::Ablative),
            "poss1sg" => Some(TurkishSuffix::Poss1Sg),
            "poss2sg" => Some(TurkishSuffix::Poss2Sg),
            "poss3sg" => Some(TurkishSuffix::Poss3Sg),
            "poss1pl" => Some(TurkishSuffix::Poss1Pl),
            "poss2pl" => Some(TurkishSuffix::Poss2Pl),
            "poss3pl" => Some(TurkishSuffix::Poss3Pl),
            _ => None,
        })
        .collect()
}

/// Get the 2-way harmony suffix text for the given suffix and harmony class.
fn turkish_suffix_form(suffix: TurkishSuffix, harmony: TurkishHarmony) -> &'static str {
    match (suffix, harmony) {
        (TurkishSuffix::Plural, TurkishHarmony::Front) => "ler",
        (TurkishSuffix::Plural, TurkishHarmony::Back) => "lar",
        (TurkishSuffix::Nominative, _) => "",
        (TurkishSuffix::Accusative, TurkishHarmony::Front) => "i",
        (TurkishSuffix::Accusative, TurkishHarmony::Back) => "\u{0131}",
        (TurkishSuffix::Genitive, TurkishHarmony::Front) => "in",
        (TurkishSuffix::Genitive, TurkishHarmony::Back) => "\u{0131}n",
        (TurkishSuffix::Dative, TurkishHarmony::Front) => "e",
        (TurkishSuffix::Dative, TurkishHarmony::Back) => "a",
        (TurkishSuffix::Locative, TurkishHarmony::Front) => "de",
        (TurkishSuffix::Locative, TurkishHarmony::Back) => "da",
        (TurkishSuffix::Ablative, TurkishHarmony::Front) => "den",
        (TurkishSuffix::Ablative, TurkishHarmony::Back) => "dan",
        (TurkishSuffix::Poss1Sg, TurkishHarmony::Front) => "im",
        (TurkishSuffix::Poss1Sg, TurkishHarmony::Back) => "\u{0131}m",
        (TurkishSuffix::Poss2Sg, TurkishHarmony::Front) => "in",
        (TurkishSuffix::Poss2Sg, TurkishHarmony::Back) => "\u{0131}n",
        (TurkishSuffix::Poss3Sg, TurkishHarmony::Front) => "i",
        (TurkishSuffix::Poss3Sg, TurkishHarmony::Back) => "\u{0131}",
        (TurkishSuffix::Poss1Pl, TurkishHarmony::Front) => "imiz",
        (TurkishSuffix::Poss1Pl, TurkishHarmony::Back) => "\u{0131}m\u{0131}z",
        (TurkishSuffix::Poss2Pl, TurkishHarmony::Front) => "iniz",
        (TurkishSuffix::Poss2Pl, TurkishHarmony::Back) => "\u{0131}n\u{0131}z",
        (TurkishSuffix::Poss3Pl, TurkishHarmony::Front) => "leri",
        (TurkishSuffix::Poss3Pl, TurkishHarmony::Back) => "lar\u{0131}",
    }
}

/// Turkish @inflect transform.
///
/// Applies suffix chain with vowel harmony based on :front/:back tag.
///
/// Context specifies suffix chain as dot-separated names:
/// - "pl" -> Plural (-ler/-lar)
/// - "nom" -> Nominative (no suffix)
/// - "acc" -> Accusative (-i/-ı)
/// - "gen" -> Genitive (-in/-ın)
/// - "dat" -> Dative (-e/-a)
/// - "loc" -> Locative (-de/-da)
/// - "abl" -> Ablative (-den/-dan)
/// - "poss1sg" -> 1st person singular possessive (-im/-ım)
/// - "poss2sg" -> 2nd person singular possessive (-in/-ın)
/// - "poss3sg" -> 3rd person singular possessive (-i/-ı)
/// - "poss1pl" -> 1st person plural possessive (-imiz/-ımız)
/// - "poss2pl" -> 2nd person plural possessive (-iniz/-ınız)
/// - "poss3pl" -> 3rd person plural possessive (-leri/-ları)
///
/// Example: "pl.poss1sg.abl" on :front "ev" -> "evlerimden"
fn turkish_inflect_transform(value: &Value, context: Option<&Value>) -> Result<String, EvalError> {
    let text = value.to_string();

    // Get initial harmony from tag
    let harmony = if value.has_tag("front") {
        TurkishHarmony::Front
    } else if value.has_tag("back") {
        TurkishHarmony::Back
    } else {
        return Err(EvalError::MissingTag {
            transform: "inflect".to_string(),
            expected: vec!["front".to_string(), "back".to_string()],
            phrase: text,
        });
    };

    // Parse suffix chain from context
    let suffixes = parse_turkish_suffix_chain(context);

    // Apply each suffix left-to-right, harmony persists through chain
    let mut result = text;
    for suffix in suffixes {
        let suffix_text = turkish_suffix_form(suffix, harmony);
        result.push_str(suffix_text);
    }

    Ok(result)
}

// =============================================================================
// Finnish Inflect Transform
// =============================================================================

/// Finnish vowel harmony type.
#[derive(Clone, Copy)]
enum FinnishHarmony {
    /// Front vowels: ä, ö, y
    Front,
    /// Back vowels: a, o, u
    Back,
}

/// Finnish suffix types for @inflect transform.
#[derive(Clone, Copy)]
enum FinnishSuffix {
    /// Plural marker: -t (nominative), -i- (other cases)
    Plural,
    /// Nominative: no suffix (unmarked case)
    Nominative,
    /// Genitive: -n
    Genitive,
    /// Partitive: -a/-ä (vowel harmony)
    Partitive,
    /// Inessive: -ssa/-ssä (in/inside)
    Inessive,
    /// Elative: -sta/-stä (out of)
    Elative,
    /// Illative: -Vn (into; V copies the preceding vowel)
    Illative,
    /// Adessive: -lla/-llä (on/at)
    Adessive,
    /// Ablative: -lta/-ltä (from off)
    Ablative,
    /// Allative: -lle (to/onto)
    Allative,
    /// Essive: -na/-nä (as/being)
    Essive,
    /// Translative: -ksi (becoming/into)
    Translative,
    /// Accusative: -n (same as genitive in singular)
    Accusative,
    /// Possessive 1st person singular: -ni (my)
    Poss1Sg,
    /// Possessive 2nd person singular: -si (your)
    Poss2Sg,
    /// Possessive 3rd person singular: -nsa/-nsä (his/her/its)
    Poss3Sg,
    /// Possessive 1st person plural: -mme (our)
    Poss1Pl,
    /// Possessive 2nd person plural: -nne (your, pl.)
    Poss2Pl,
    /// Possessive 3rd person plural: -nsa/-nsä (their)
    Poss3Pl,
}

/// Parse suffix chain from context value for Finnish.
///
/// Parses dot-separated suffix names: "pl.ine" -> [Plural, Inessive]
fn parse_finnish_suffix_chain(context: Option<&Value>) -> Vec<FinnishSuffix> {
    let Some(Value::String(s)) = context else {
        return Vec::new();
    };

    s.split('.')
        .filter_map(|part| match part {
            "pl" => Some(FinnishSuffix::Plural),
            "nom" => Some(FinnishSuffix::Nominative),
            "gen" => Some(FinnishSuffix::Genitive),
            "par" => Some(FinnishSuffix::Partitive),
            "ine" => Some(FinnishSuffix::Inessive),
            "ela" => Some(FinnishSuffix::Elative),
            "ill" => Some(FinnishSuffix::Illative),
            "ade" => Some(FinnishSuffix::Adessive),
            "abl" => Some(FinnishSuffix::Ablative),
            "all" => Some(FinnishSuffix::Allative),
            "ess" => Some(FinnishSuffix::Essive),
            "tra" => Some(FinnishSuffix::Translative),
            "acc" => Some(FinnishSuffix::Accusative),
            "poss1sg" => Some(FinnishSuffix::Poss1Sg),
            "poss2sg" => Some(FinnishSuffix::Poss2Sg),
            "poss3sg" => Some(FinnishSuffix::Poss3Sg),
            "poss1pl" => Some(FinnishSuffix::Poss1Pl),
            "poss2pl" => Some(FinnishSuffix::Poss2Pl),
            "poss3pl" => Some(FinnishSuffix::Poss3Pl),
            _ => None,
        })
        .collect()
}

/// Get the suffix text for the given Finnish suffix and harmony class.
///
/// Finnish vowel harmony affects suffixes containing a/ä. Suffixes with only
/// neutral vowels (i, e) or no vowels are invariant across harmony classes.
fn finnish_suffix_form(suffix: FinnishSuffix, harmony: FinnishHarmony) -> &'static str {
    match (suffix, harmony) {
        (FinnishSuffix::Plural, _) => "t",
        (FinnishSuffix::Nominative, _) => "",
        (FinnishSuffix::Genitive, _) => "n",
        (FinnishSuffix::Partitive, FinnishHarmony::Front) => "\u{00e4}",
        (FinnishSuffix::Partitive, FinnishHarmony::Back) => "a",
        (FinnishSuffix::Inessive, FinnishHarmony::Front) => "ss\u{00e4}",
        (FinnishSuffix::Inessive, FinnishHarmony::Back) => "ssa",
        (FinnishSuffix::Elative, FinnishHarmony::Front) => "st\u{00e4}",
        (FinnishSuffix::Elative, FinnishHarmony::Back) => "sta",
        (FinnishSuffix::Illative, _) => "n",
        (FinnishSuffix::Adessive, FinnishHarmony::Front) => "ll\u{00e4}",
        (FinnishSuffix::Adessive, FinnishHarmony::Back) => "lla",
        (FinnishSuffix::Ablative, FinnishHarmony::Front) => "lt\u{00e4}",
        (FinnishSuffix::Ablative, FinnishHarmony::Back) => "lta",
        (FinnishSuffix::Allative, _) => "lle",
        (FinnishSuffix::Essive, FinnishHarmony::Front) => "n\u{00e4}",
        (FinnishSuffix::Essive, FinnishHarmony::Back) => "na",
        (FinnishSuffix::Translative, _) => "ksi",
        (FinnishSuffix::Accusative, _) => "n",
        (FinnishSuffix::Poss1Sg, _) => "ni",
        (FinnishSuffix::Poss2Sg, _) => "si",
        (FinnishSuffix::Poss3Sg, FinnishHarmony::Front) => "ns\u{00e4}",
        (FinnishSuffix::Poss3Sg, FinnishHarmony::Back) => "nsa",
        (FinnishSuffix::Poss1Pl, _) => "mme",
        (FinnishSuffix::Poss2Pl, _) => "nne",
        (FinnishSuffix::Poss3Pl, FinnishHarmony::Front) => "ns\u{00e4}",
        (FinnishSuffix::Poss3Pl, FinnishHarmony::Back) => "nsa",
    }
}

/// Apply Finnish illative suffix by duplicating the last vowel and appending -n.
///
/// The illative case in Finnish works by lengthening the final vowel: talo -> taloon.
/// If the word ends in a consonant, falls back to appending -Vn where V is the
/// last vowel found in the word.
fn apply_finnish_illative(word: &mut String) {
    let last_vowel = word.chars().rev().find(|c| "aeiouyäöAEIOUYÄÖ".contains(*c));
    if let Some(v) = last_vowel {
        if word.ends_with(v) {
            // Word ends in a vowel: duplicate it, then append 'n'
            word.push(v);
            word.push('n');
        } else {
            // Word ends in consonant: append the last vowel + 'n'
            word.push(v);
            word.push('n');
        }
    } else {
        // No vowel found, just append 'n'
        word.push('n');
    }
}

/// Finnish @inflect transform.
///
/// Applies suffix chain with vowel harmony based on :front/:back tag.
///
/// Context specifies suffix chain as dot-separated names:
/// - "pl" -> Plural (-t nominative plural marker)
/// - "nom" -> Nominative (no suffix)
/// - "gen" -> Genitive (-n)
/// - "par" -> Partitive (-a/-ä)
/// - "ine" -> Inessive (-ssa/-ssä)
/// - "ela" -> Elative (-sta/-stä)
/// - "ill" -> Illative (vowel lengthening + -n)
/// - "ade" -> Adessive (-lla/-llä)
/// - "abl" -> Ablative (-lta/-ltä)
/// - "all" -> Allative (-lle)
/// - "ess" -> Essive (-na/-nä)
/// - "tra" -> Translative (-ksi)
/// - "acc" -> Accusative (-n)
/// - "poss1sg" -> 1st person singular possessive (-ni)
/// - "poss2sg" -> 2nd person singular possessive (-si)
/// - "poss3sg" -> 3rd person singular possessive (-nsa/-nsä)
/// - "poss1pl" -> 1st person plural possessive (-mme)
/// - "poss2pl" -> 2nd person plural possessive (-nne)
/// - "poss3pl" -> 3rd person plural possessive (-nsa/-nsä)
///
/// Example: "ine" on :front "tyttö" -> "tytössä" (simplified; stem changes are
/// the responsibility of the .rlf translation author)
fn finnish_inflect_transform(value: &Value, context: Option<&Value>) -> Result<String, EvalError> {
    let text = value.to_string();

    let harmony = if value.has_tag("front") {
        FinnishHarmony::Front
    } else if value.has_tag("back") {
        FinnishHarmony::Back
    } else {
        return Err(EvalError::MissingTag {
            transform: "inflect".to_string(),
            expected: vec!["front".to_string(), "back".to_string()],
            phrase: text,
        });
    };

    let suffixes = parse_finnish_suffix_chain(context);

    let mut result = text;
    for suffix in suffixes {
        if matches!(suffix, FinnishSuffix::Illative) {
            apply_finnish_illative(&mut result);
        } else {
            let suffix_text = finnish_suffix_form(suffix, harmony);
            result.push_str(suffix_text);
        }
    }

    Ok(result)
}

// =============================================================================
// Hungarian Inflect Transform
// =============================================================================

/// Hungarian vowel harmony type.
///
/// Hungarian has a 3-way harmony system for some suffixes:
/// - Back vowels: a, o, u
/// - Front unrounded vowels: e, i
/// - Front rounded vowels: ö, ü
#[derive(Clone, Copy)]
enum HungarianHarmony {
    /// Back vowels: a, á, o, ó, u, ú
    Back,
    /// Front unrounded vowels: e, é, i, í
    Front,
    /// Front rounded vowels: ö, ő, ü, ű
    Round,
}

/// Hungarian suffix types for @inflect transform.
#[derive(Clone, Copy)]
enum HungarianSuffix {
    /// Plural: -k (with linking vowel -ok/-ek/-ök)
    Plural,
    /// Nominative: no suffix (unmarked case)
    Nominative,
    /// Accusative: -t (with linking vowel -ot/-et/-öt)
    Accusative,
    /// Dative: -nak/-nek
    Dative,
    /// Inessive (in): -ban/-ben
    Inessive,
    /// Illative (into): -ba/-be
    Illative,
    /// Elative (out of): -ból/-ből
    Elative,
    /// Superessive (on): -n/-on/-en/-ön
    Superessive,
    /// Sublative (onto): -ra/-re
    Sublative,
    /// Delative (off of): -ról/-ről
    Delative,
    /// Adessive (at/near): -nál/-nél
    Adessive,
    /// Ablative (from): -tól/-től
    Ablative,
    /// Allative (towards): -hoz/-hez/-höz
    Allative,
    /// Instrumental: -val/-vel
    Instrumental,
    /// Translative (becoming): -vá/-vé
    Translative,
    /// Causal-final (for): -ért
    CausalFinal,
    /// Terminative (until): -ig
    Terminative,
    /// Essive-formal (as): -ként
    EssiveFormal,
    /// Possessive 1st person singular: -m/-om/-em/-öm (my)
    Poss1Sg,
    /// Possessive 2nd person singular: -d/-od/-ed/-öd (your)
    Poss2Sg,
    /// Possessive 3rd person singular: -a/-e/-ja/-je (his/her/its)
    Poss3Sg,
    /// Possessive 1st person plural: -unk/-ünk (our)
    Poss1Pl,
    /// Possessive 2nd person plural: -tok/-tek/-tök (your, pl.)
    Poss2Pl,
    /// Possessive 3rd person plural: -uk/-ük/-juk/-jük (their)
    Poss3Pl,
}

/// Parse suffix chain from context value for Hungarian.
///
/// Parses dot-separated suffix names: "pl.dat" -> [Plural, Dative]
fn parse_hungarian_suffix_chain(context: Option<&Value>) -> Vec<HungarianSuffix> {
    let Some(Value::String(s)) = context else {
        return Vec::new();
    };

    s.split('.')
        .filter_map(|part| match part {
            "pl" => Some(HungarianSuffix::Plural),
            "nom" => Some(HungarianSuffix::Nominative),
            "acc" => Some(HungarianSuffix::Accusative),
            "dat" => Some(HungarianSuffix::Dative),
            "ine" => Some(HungarianSuffix::Inessive),
            "ill" => Some(HungarianSuffix::Illative),
            "ela" => Some(HungarianSuffix::Elative),
            "sup" => Some(HungarianSuffix::Superessive),
            "sub" => Some(HungarianSuffix::Sublative),
            "del" => Some(HungarianSuffix::Delative),
            "ade" => Some(HungarianSuffix::Adessive),
            "abl" => Some(HungarianSuffix::Ablative),
            "all" => Some(HungarianSuffix::Allative),
            "ins" => Some(HungarianSuffix::Instrumental),
            "tra" => Some(HungarianSuffix::Translative),
            "cau" => Some(HungarianSuffix::CausalFinal),
            "ter" => Some(HungarianSuffix::Terminative),
            "ess" => Some(HungarianSuffix::EssiveFormal),
            "poss1sg" => Some(HungarianSuffix::Poss1Sg),
            "poss2sg" => Some(HungarianSuffix::Poss2Sg),
            "poss3sg" => Some(HungarianSuffix::Poss3Sg),
            "poss1pl" => Some(HungarianSuffix::Poss1Pl),
            "poss2pl" => Some(HungarianSuffix::Poss2Pl),
            "poss3pl" => Some(HungarianSuffix::Poss3Pl),
            _ => None,
        })
        .collect()
}

/// Get the Hungarian suffix text for the given suffix and harmony class.
///
/// Hungarian uses 2-way harmony (back/front) for most suffixes, with 3-way
/// harmony (back/front-unrounded/front-rounded) for allative, superessive,
/// plural linking vowel, accusative linking vowel, and possessives.
fn hungarian_suffix_form(suffix: HungarianSuffix, harmony: HungarianHarmony) -> &'static str {
    match (suffix, harmony) {
        (HungarianSuffix::Plural, HungarianHarmony::Back) => "ok",
        (HungarianSuffix::Plural, HungarianHarmony::Front) => "ek",
        (HungarianSuffix::Plural, HungarianHarmony::Round) => "\u{00f6}k",
        (HungarianSuffix::Nominative, _) => "",
        (HungarianSuffix::Accusative, HungarianHarmony::Back) => "ot",
        (HungarianSuffix::Accusative, HungarianHarmony::Front) => "et",
        (HungarianSuffix::Accusative, HungarianHarmony::Round) => "\u{00f6}t",
        (HungarianSuffix::Dative, HungarianHarmony::Back) => "nak",
        (HungarianSuffix::Dative, HungarianHarmony::Front | HungarianHarmony::Round) => "nek",
        (HungarianSuffix::Inessive, HungarianHarmony::Back) => "ban",
        (HungarianSuffix::Inessive, HungarianHarmony::Front | HungarianHarmony::Round) => "ben",
        (HungarianSuffix::Illative, HungarianHarmony::Back) => "ba",
        (HungarianSuffix::Illative, HungarianHarmony::Front | HungarianHarmony::Round) => "be",
        (HungarianSuffix::Elative, HungarianHarmony::Back) => "b\u{00f3}l",
        (HungarianSuffix::Elative, HungarianHarmony::Front | HungarianHarmony::Round) => {
            "b\u{0151}l"
        }
        (HungarianSuffix::Superessive, HungarianHarmony::Back) => "on",
        (HungarianSuffix::Superessive, HungarianHarmony::Front) => "en",
        (HungarianSuffix::Superessive, HungarianHarmony::Round) => "\u{00f6}n",
        (HungarianSuffix::Sublative, HungarianHarmony::Back) => "ra",
        (HungarianSuffix::Sublative, HungarianHarmony::Front | HungarianHarmony::Round) => "re",
        (HungarianSuffix::Delative, HungarianHarmony::Back) => "r\u{00f3}l",
        (HungarianSuffix::Delative, HungarianHarmony::Front | HungarianHarmony::Round) => {
            "r\u{0151}l"
        }
        (HungarianSuffix::Adessive, HungarianHarmony::Back) => "n\u{00e1}l",
        (HungarianSuffix::Adessive, HungarianHarmony::Front | HungarianHarmony::Round) => {
            "n\u{00e9}l"
        }
        (HungarianSuffix::Ablative, HungarianHarmony::Back) => "t\u{00f3}l",
        (HungarianSuffix::Ablative, HungarianHarmony::Front | HungarianHarmony::Round) => {
            "t\u{0151}l"
        }
        (HungarianSuffix::Allative, HungarianHarmony::Back) => "hoz",
        (HungarianSuffix::Allative, HungarianHarmony::Front) => "hez",
        (HungarianSuffix::Allative, HungarianHarmony::Round) => "h\u{00f6}z",
        (HungarianSuffix::Instrumental, HungarianHarmony::Back) => "val",
        (HungarianSuffix::Instrumental, HungarianHarmony::Front | HungarianHarmony::Round) => "vel",
        (HungarianSuffix::Translative, HungarianHarmony::Back) => "v\u{00e1}",
        (HungarianSuffix::Translative, HungarianHarmony::Front | HungarianHarmony::Round) => {
            "v\u{00e9}"
        }
        (HungarianSuffix::CausalFinal, _) => "\u{00e9}rt",
        (HungarianSuffix::Terminative, _) => "ig",
        (HungarianSuffix::EssiveFormal, _) => "k\u{00e9}nt",
        (HungarianSuffix::Poss1Sg, HungarianHarmony::Back) => "om",
        (HungarianSuffix::Poss1Sg, HungarianHarmony::Front) => "em",
        (HungarianSuffix::Poss1Sg, HungarianHarmony::Round) => "\u{00f6}m",
        (HungarianSuffix::Poss2Sg, HungarianHarmony::Back) => "od",
        (HungarianSuffix::Poss2Sg, HungarianHarmony::Front) => "ed",
        (HungarianSuffix::Poss2Sg, HungarianHarmony::Round) => "\u{00f6}d",
        (HungarianSuffix::Poss3Sg, HungarianHarmony::Back) => "a",
        (HungarianSuffix::Poss3Sg, HungarianHarmony::Front | HungarianHarmony::Round) => "e",
        (HungarianSuffix::Poss1Pl, HungarianHarmony::Back) => "unk",
        (HungarianSuffix::Poss1Pl, HungarianHarmony::Front | HungarianHarmony::Round) => {
            "\u{00fc}nk"
        }
        (HungarianSuffix::Poss2Pl, HungarianHarmony::Back) => "tok",
        (HungarianSuffix::Poss2Pl, HungarianHarmony::Front) => "tek",
        (HungarianSuffix::Poss2Pl, HungarianHarmony::Round) => "t\u{00f6}k",
        (HungarianSuffix::Poss3Pl, HungarianHarmony::Back) => "uk",
        (HungarianSuffix::Poss3Pl, HungarianHarmony::Front | HungarianHarmony::Round) => {
            "\u{00fc}k"
        }
    }
}

/// Hungarian @inflect transform.
///
/// Applies suffix chain with vowel harmony based on `:back`/`:front`/`:round` tags.
///
/// Hungarian uses 3-way vowel harmony for some suffixes:
/// - `:back` — back vowels (a, o, u): -hoz, -ok, -om, etc.
/// - `:front` — front unrounded vowels (e, i): -hez, -ek, -em, etc.
/// - `:round` — front rounded vowels (ö, ü): -höz, -ök, -öm, etc.
///
/// Context specifies suffix chain as dot-separated names:
/// - "pl" -> Plural (-ok/-ek/-ök)
/// - "nom" -> Nominative (no suffix)
/// - "acc" -> Accusative (-ot/-et/-öt)
/// - "dat" -> Dative (-nak/-nek)
/// - "ine" -> Inessive (-ban/-ben)
/// - "ill" -> Illative (-ba/-be)
/// - "ela" -> Elative (-ból/-ből)
/// - "sup" -> Superessive (-on/-en/-ön)
/// - "sub" -> Sublative (-ra/-re)
/// - "del" -> Delative (-ról/-ről)
/// - "ade" -> Adessive (-nál/-nél)
/// - "abl" -> Ablative (-tól/-től)
/// - "all" -> Allative (-hoz/-hez/-höz)
/// - "ins" -> Instrumental (-val/-vel)
/// - "tra" -> Translative (-vá/-vé)
/// - "cau" -> Causal-final (-ért)
/// - "ter" -> Terminative (-ig)
/// - "ess" -> Essive-formal (-ként)
/// - "poss1sg" -> 1st person sg possessive (-om/-em/-öm)
/// - "poss2sg" -> 2nd person sg possessive (-od/-ed/-öd)
/// - "poss3sg" -> 3rd person sg possessive (-a/-e)
/// - "poss1pl" -> 1st person pl possessive (-unk/-ünk)
/// - "poss2pl" -> 2nd person pl possessive (-tok/-tek/-tök)
/// - "poss3pl" -> 3rd person pl possessive (-uk/-ük)
///
/// Example: "pl.dat" on :back "ház" -> "házaknak"
fn hungarian_inflect_transform(
    value: &Value,
    context: Option<&Value>,
) -> Result<String, EvalError> {
    let text = value.to_string();

    let harmony = if value.has_tag("back") {
        HungarianHarmony::Back
    } else if value.has_tag("front") {
        HungarianHarmony::Front
    } else if value.has_tag("round") {
        HungarianHarmony::Round
    } else {
        return Err(EvalError::MissingTag {
            transform: "inflect".to_string(),
            expected: vec!["back".to_string(), "front".to_string(), "round".to_string()],
            phrase: text,
        });
    };

    let suffixes = parse_hungarian_suffix_chain(context);

    let mut result = text;
    for suffix in suffixes {
        let suffix_text = hungarian_suffix_form(suffix, harmony);
        result.push_str(suffix_text);
    }

    Ok(result)
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
        // Resolve aliases first (some are language-specific)
        // Order matters: more specific patterns (with lang) before wildcards
        let canonical = match (name, lang) {
            ("an", _) => "a",                // English alias: @an resolves to @a
            ("die" | "das", _) => "der",     // German aliases: @die/@das resolve to @der
            ("eine", _) => "ein",            // German alias: @eine resolves to @ein
            ("het", _) => "de",              // Dutch alias: @het resolves to @de
            ("la", "es") => "el",            // Spanish alias: @la resolves to @el
            ("una", "es") => "un",           // Spanish alias: @una resolves to @un
            ("a", "pt") => "o",              // Portuguese alias: @a resolves to @o
            ("uma", _) => "um",              // Portuguese alias: @uma resolves to @um
            ("la", "fr") => "le",            // French alias: @la resolves to @le
            ("une", "fr") => "un",           // French alias: @une resolves to @un
            ("lo" | "la", "it") => "il",     // Italian aliases: @lo/@la resolve to @il
            ("uno" | "una", "it") => "un",   // Italian aliases: @uno/@una resolve to @un
            ("i" | "to", "el") => "o",       // Greek aliases: @i/@to resolve to @o
            ("mia" | "ena", "el") => "enas", // Greek aliases: @mia/@ena resolve to @enas
            (other, _) => other,
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
            ("es", "el") => Some(TransformKind::SpanishEl),
            ("es", "un") => Some(TransformKind::SpanishUn),
            ("pt", "o") => Some(TransformKind::PortugueseO),
            ("pt", "um") => Some(TransformKind::PortugueseUm),
            ("pt", "de") => Some(TransformKind::PortugueseDe),
            ("pt", "em") => Some(TransformKind::PortugueseEm),
            ("fr", "le") => Some(TransformKind::FrenchLe),
            ("fr", "un") => Some(TransformKind::FrenchUn),
            ("fr", "de") => Some(TransformKind::FrenchDe),
            ("fr", "au") => Some(TransformKind::FrenchAu),
            ("fr", "liaison") => Some(TransformKind::FrenchLiaison),
            ("it", "il") => Some(TransformKind::ItalianIl),
            ("it", "un") => Some(TransformKind::ItalianUn),
            ("it", "di") => Some(TransformKind::ItalianDi),
            ("it", "a") => Some(TransformKind::ItalianA),
            ("el", "o") => Some(TransformKind::GreekO),
            ("el", "enas") => Some(TransformKind::GreekEnas),
            ("ro", "def") => Some(TransformKind::RomanianDef),
            ("ar", "al") => Some(TransformKind::ArabicAl),
            ("fa", "ezafe") => Some(TransformKind::PersianEzafe),
            ("zh", "count") => Some(TransformKind::ChineseCount),
            ("ja", "count") => Some(TransformKind::JapaneseCount),
            ("ko", "count") => Some(TransformKind::KoreanCount),
            ("vi", "count") => Some(TransformKind::VietnameseCount),
            ("th", "count") => Some(TransformKind::ThaiCount),
            ("bn", "count") => Some(TransformKind::BengaliCount),
            ("id", "plural") => Some(TransformKind::IndonesianPlural),
            ("ko", "particle") => Some(TransformKind::KoreanParticle),
            ("tr", "inflect") => Some(TransformKind::TurkishInflect),
            ("fi", "inflect") => Some(TransformKind::FinnishInflect),
            ("hu", "inflect") => Some(TransformKind::HungarianInflect),
            _ => None,
        }
    }

    /// Check if a transform exists for a language.
    pub fn has_transform(&self, name: &str, lang: &str) -> bool {
        self.get(name, lang).is_some()
    }
}
