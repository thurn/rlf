//! CLDR plural category resolution.
//!
//! This module provides plural category resolution following CLDR rules.
//! Different languages have different plural rules - English has "one" and "other",
//! while Russian has "one", "few", "many", and "other", and Arabic uses all six
//! categories: "zero", "one", "two", "few", "many", "other".
//!
//! Plural rules are cached per thread per language to avoid re-creating
//! `PluralRules` instances on every call. The cache is initialized lazily
//! on first access within each thread.

use std::cell::RefCell;

use icu_locale_core::locale;
use icu_plurals::{PluralCategory, PluralRuleType, PluralRules};

/// Supported language codes for plural rule resolution.
const SUPPORTED_LANGUAGES: &[&str] = &[
    "ar", "bn", "de", "el", "en", "es", "fa", "fr", "he", "hi", "id", "it", "ja", "ko", "nl", "pl",
    "pt", "ro", "ru", "th", "tr", "uk", "vi", "zh",
];

thread_local! {
    /// Per-thread cache of `PluralRules` keyed by language code.
    static PLURAL_RULES_CACHE: RefCell<Vec<(&'static str, PluralRules)>> = const { RefCell::new(Vec::new()) };
}

/// Normalize a language code to a supported static string reference.
///
/// Returns the canonical `&'static str` for the language, or `"en"` for
/// unrecognized codes.
fn normalize_lang(lang: &str) -> &'static str {
    SUPPORTED_LANGUAGES
        .iter()
        .find(|&&code| code == lang)
        .copied()
        .unwrap_or("en")
}

/// Build `PluralRules` for a normalized language code.
fn build_rules(lang: &'static str) -> PluralRules {
    let loc = match lang {
        "en" => locale!("en"),
        "ru" => locale!("ru"),
        "ar" => locale!("ar"),
        "de" => locale!("de"),
        "es" => locale!("es"),
        "fr" => locale!("fr"),
        "it" => locale!("it"),
        "pt" => locale!("pt"),
        "ja" => locale!("ja"),
        "zh" => locale!("zh"),
        "ko" => locale!("ko"),
        "nl" => locale!("nl"),
        "pl" => locale!("pl"),
        "tr" => locale!("tr"),
        "uk" => locale!("uk"),
        "vi" => locale!("vi"),
        "th" => locale!("th"),
        "id" => locale!("id"),
        "el" => locale!("el"),
        "ro" => locale!("ro"),
        "fa" => locale!("fa"),
        "bn" => locale!("bn"),
        "hi" => locale!("hi"),
        "he" => locale!("he"),
        _ => locale!("en"),
    };
    PluralRules::try_new(loc.into(), PluralRuleType::Cardinal.into())
        .expect("locale should be supported")
}

/// Translate a `PluralCategory` enum to its string representation.
fn category_str(category: PluralCategory) -> &'static str {
    match category {
        PluralCategory::Zero => "zero",
        PluralCategory::One => "one",
        PluralCategory::Two => "two",
        PluralCategory::Few => "few",
        PluralCategory::Many => "many",
        PluralCategory::Other => "other",
    }
}

/// Get CLDR plural category for a number in a given language.
///
/// Returns one of: "zero", "one", "two", "few", "many", "other".
/// Rules are cached per thread per language, so repeated calls with the same
/// language code reuse the previously constructed `PluralRules`.
///
/// # Arguments
///
/// * `lang` - Language code (e.g., "en", "ru", "ar")
/// * `n` - The number to categorize
///
/// # Examples
///
/// ```
/// use rlf::interpreter::plural_category;
///
/// // English: 1 = "one", everything else = "other"
/// assert_eq!(plural_category("en", 1), "one");
/// assert_eq!(plural_category("en", 2), "other");
///
/// // Russian: complex rules for "one", "few", "many", "other"
/// assert_eq!(plural_category("ru", 1), "one");
/// assert_eq!(plural_category("ru", 2), "few");
/// assert_eq!(plural_category("ru", 5), "many");
/// ```
pub fn plural_category(lang: &str, n: i64) -> &'static str {
    let lang = normalize_lang(lang);
    PLURAL_RULES_CACHE.with_borrow_mut(|cache| {
        if let Some(entry) = cache.iter().find(|(code, _)| *code == lang) {
            return category_str(entry.1.category_for(n));
        }
        let rules = build_rules(lang);
        let category = category_str(rules.category_for(n));
        cache.push((lang, rules));
        category
    })
}
