//! CLDR plural category resolution.
//!
//! This module provides plural category resolution following CLDR rules.
//! Different languages have different plural rules - English has "one" and "other",
//! while Russian has "one", "few", "many", and "other", and Arabic uses all six
//! categories: "zero", "one", "two", "few", "many", "other".

use icu_locale_core::locale;
use icu_plurals::{PluralCategory, PluralRuleType, PluralRules};

/// Get CLDR plural category for a number in a given language.
///
/// Returns one of: "zero", "one", "two", "few", "many", "other"
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
        _ => locale!("en"), // fallback to English
    };

    let rules = PluralRules::try_new(loc.into(), PluralRuleType::Cardinal.into())
        .expect("locale should be supported");

    match rules.category_for(n) {
        PluralCategory::Zero => "zero",
        PluralCategory::One => "one",
        PluralCategory::Two => "two",
        PluralCategory::Few => "few",
        PluralCategory::Many => "many",
        PluralCategory::Other => "other",
    }
}
