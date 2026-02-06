//! Language-specific metadata for validation.
//!
//! Defines valid tags and variant key components per language, enabling
//! `validate_translations` to warn about unrecognized tags or case names.

/// Valid metadata tags for a language.
///
/// Returns `None` if the language has no tag validation rules (i.e., any tags
/// are accepted). Returns `Some(&[...])` with the list of recognized tags.
pub fn valid_tags(lang: &str) -> Option<&'static [&'static str]> {
    match lang {
        "pl" => Some(&["masc_anim", "masc_inan", "fem", "neut"]),
        "cs" => Some(&["masc_anim", "masc_inan", "fem", "neut"]),
        "ru" => Some(&["masc", "fem", "neut", "anim", "inan"]),
        "uk" => Some(&["masc", "fem", "neut", "anim", "inan"]),
        "de" => Some(&["masc", "fem", "neut"]),
        "es" => Some(&["masc", "fem"]),
        "fr" => Some(&["masc", "fem", "vowel"]),
        "pt" => Some(&["masc", "fem"]),
        "it" => Some(&["masc", "fem", "vowel", "s_imp"]),
        "en" => Some(&["a", "an"]),
        "nl" => Some(&["de", "het"]),
        "el" => Some(&["masc", "fem", "neut"]),
        "ro" => Some(&["masc", "fem", "neut"]),
        "ar" => Some(&["masc", "fem", "sun", "moon"]),
        "hi" => Some(&["masc", "fem"]),
        "tr" => Some(&["front", "back"]),
        "fi" => Some(&["front", "back"]),
        "hu" => Some(&["back", "front", "round"]),
        "fa" => Some(&["vowel"]),
        "zh" => Some(&["zhang", "ge", "ming", "wei", "tiao", "ben", "zhi"]),
        "ja" => Some(&["mai", "nin", "hiki", "hon", "ko", "satsu"]),
        "ko" => Some(&["jang", "myeong", "mari", "gae", "gwon"]),
        "vi" => Some(&["cai", "con", "nguoi", "chiec", "to"]),
        "th" => Some(&["bai", "tua", "khon", "an"]),
        "bn" => Some(&["ta", "ti", "khana", "jon"]),
        _ => None,
    }
}

/// Valid variant key components for a language.
///
/// Returns `None` if the language has no variant key validation rules.
/// Returns `Some(&[...])` with recognized case names and plural categories.
///
/// Variant keys use dot notation (e.g., "nom.one"). Each component is validated
/// independently against this list.
pub fn valid_variant_keys(lang: &str) -> Option<&'static [&'static str]> {
    match lang {
        // Polish: 7 cases + 4 plural categories
        "pl" => Some(&[
            "nom", "acc", "gen", "dat", "ins", "loc", "voc", "one", "few", "many", "other",
        ]),
        // Czech: same case system as Polish
        "cs" => Some(&[
            "nom", "acc", "gen", "dat", "ins", "loc", "voc", "one", "few", "many", "other",
        ]),
        // Russian: 6 cases + 4 plural categories
        "ru" => Some(&[
            "nom", "acc", "gen", "dat", "ins", "prep", "one", "few", "many", "other",
        ]),
        // Ukrainian: 7 cases + 4 plural categories
        "uk" => Some(&[
            "nom", "acc", "gen", "dat", "ins", "loc", "voc", "one", "few", "many", "other",
        ]),
        // German: 4 cases + 2 plural categories
        "de" => Some(&["nom", "acc", "dat", "gen", "one", "other"]),
        // Hindi: 3 case forms + 2 plural categories
        "hi" => Some(&["dir", "obl", "voc", "one", "other"]),
        // Arabic: 6 plural categories
        "ar" => Some(&["zero", "one", "two", "few", "many", "other"]),
        // Greek: 4 cases + 2 plural categories
        "el" => Some(&["nom", "acc", "gen", "voc", "one", "other"]),
        // Romanian: 2 cases + 3 plural categories
        "ro" => Some(&["nom", "gen", "one", "few", "other"]),
        _ => None,
    }
}
