//! Tests for load-time translation validation warnings.

use rlf::{LoadWarning, Locale};

// =========================================================================
// Unknown Phrase Warnings
// =========================================================================

#[test]
fn unknown_phrase_produces_warning() {
    let mut locale = Locale::new();
    locale
        .load_translations_str("en", r#"hello = "Hello!";"#)
        .unwrap();
    locale
        .load_translations_str("ru", r#"hello = "Привет!"; extra = "Лишнее";"#)
        .unwrap();

    let warnings = locale.validate_translations("en", "ru");

    assert_eq!(warnings.len(), 1);
    assert_eq!(
        warnings[0],
        LoadWarning::UnknownPhrase {
            name: "extra".to_string(),
            language: "ru".to_string(),
        }
    );
}

#[test]
fn multiple_unknown_phrases_produce_multiple_warnings() {
    let mut locale = Locale::new();
    locale
        .load_translations_str("en", r#"hello = "Hello!";"#)
        .unwrap();
    locale
        .load_translations_str(
            "ru",
            r#"
            hello = "Привет!";
            extra_one = "Один";
            extra_two = "Два";
        "#,
        )
        .unwrap();

    let warnings = locale.validate_translations("en", "ru");

    assert_eq!(warnings.len(), 2);
    let names: Vec<&str> = warnings
        .iter()
        .filter_map(|w| match w {
            LoadWarning::UnknownPhrase { name, .. } => Some(name.as_str()),
            _ => None,
        })
        .collect();
    assert!(names.contains(&"extra_one"));
    assert!(names.contains(&"extra_two"));
}

// =========================================================================
// Parameter Count Mismatch Warnings
// =========================================================================

#[test]
fn parameter_count_mismatch_produces_warning() {
    let mut locale = Locale::new();
    locale
        .load_translations_str("en", r#"greet(name) = "Hello, {name}!";"#)
        .unwrap();
    locale
        .load_translations_str(
            "ru",
            r#"greet(first_name, last_name) = "{first_name} {last_name}";"#,
        )
        .unwrap();

    let warnings = locale.validate_translations("en", "ru");

    assert_eq!(warnings.len(), 1);
    assert_eq!(
        warnings[0],
        LoadWarning::ParameterCountMismatch {
            name: "greet".to_string(),
            language: "ru".to_string(),
            source_count: 1,
            translation_count: 2,
        }
    );
}

#[test]
fn translation_has_fewer_params_than_source() {
    let mut locale = Locale::new();
    locale
        .load_translations_str("en", r#"greet(first, last) = "Hello, {first} {last}!";"#)
        .unwrap();
    locale
        .load_translations_str("ru", r#"greet(name) = "Привет, {name}!";"#)
        .unwrap();

    let warnings = locale.validate_translations("en", "ru");

    assert_eq!(warnings.len(), 1);
    assert_eq!(
        warnings[0],
        LoadWarning::ParameterCountMismatch {
            name: "greet".to_string(),
            language: "ru".to_string(),
            source_count: 2,
            translation_count: 1,
        }
    );
}

#[test]
fn translation_adds_params_to_parameterless_phrase() {
    let mut locale = Locale::new();
    locale
        .load_translations_str("en", r#"hello = "Hello!";"#)
        .unwrap();
    locale
        .load_translations_str("ru", r#"hello(name) = "Привет, {name}!";"#)
        .unwrap();

    let warnings = locale.validate_translations("en", "ru");

    assert_eq!(warnings.len(), 1);
    assert_eq!(
        warnings[0],
        LoadWarning::ParameterCountMismatch {
            name: "hello".to_string(),
            language: "ru".to_string(),
            source_count: 0,
            translation_count: 1,
        }
    );
}

// =========================================================================
// Valid Translation Files
// =========================================================================

#[test]
fn valid_translations_produce_no_warnings() {
    let mut locale = Locale::new();
    locale
        .load_translations_str(
            "en",
            r#"
            hello = "Hello!";
            greet(name) = "Hello, {name}!";
            card = { one: "card", other: "cards" };
        "#,
        )
        .unwrap();
    locale
        .load_translations_str(
            "ru",
            r#"
            hello = "Привет!";
            greet(name) = "Привет, {name}!";
            card = { one: "карта", few: "карты", many: "карт", other: "карт" };
        "#,
        )
        .unwrap();

    let warnings = locale.validate_translations("en", "ru");

    assert!(warnings.is_empty());
}

#[test]
fn subset_of_source_phrases_produces_no_warnings() {
    let mut locale = Locale::new();
    locale
        .load_translations_str(
            "en",
            r#"
            hello = "Hello!";
            goodbye = "Goodbye!";
        "#,
        )
        .unwrap();
    locale
        .load_translations_str("ru", r#"hello = "Привет!";"#)
        .unwrap();

    let warnings = locale.validate_translations("en", "ru");

    assert!(warnings.is_empty());
}

// =========================================================================
// Mixed Warnings
// =========================================================================

#[test]
fn mixed_unknown_and_mismatch_warnings() {
    let mut locale = Locale::new();
    locale
        .load_translations_str(
            "en",
            r#"
            hello = "Hello!";
            greet(name) = "Hello, {name}!";
        "#,
        )
        .unwrap();
    locale
        .load_translations_str(
            "ru",
            r#"
            hello = "Привет!";
            greet(first, last) = "Привет, {first} {last}!";
            extra = "Лишнее";
        "#,
        )
        .unwrap();

    let warnings = locale.validate_translations("en", "ru");

    assert_eq!(warnings.len(), 2);

    let has_unknown = warnings
        .iter()
        .any(|w| matches!(w, LoadWarning::UnknownPhrase { name, .. } if name == "extra"));
    let has_mismatch = warnings.iter().any(|w| {
        matches!(w, LoadWarning::ParameterCountMismatch { name, source_count: 1, translation_count: 2, .. } if name == "greet")
    });
    assert!(has_unknown);
    assert!(has_mismatch);
}

// =========================================================================
// Edge Cases
// =========================================================================

#[test]
fn unloaded_source_language_returns_empty() {
    let mut locale = Locale::new();
    locale
        .load_translations_str("ru", r#"hello = "Привет!";"#)
        .unwrap();

    let warnings = locale.validate_translations("en", "ru");

    assert!(warnings.is_empty());
}

#[test]
fn unloaded_target_language_returns_empty() {
    let mut locale = Locale::new();
    locale
        .load_translations_str("en", r#"hello = "Hello!";"#)
        .unwrap();

    let warnings = locale.validate_translations("en", "ru");

    assert!(warnings.is_empty());
}

#[test]
fn both_languages_unloaded_returns_empty() {
    let locale = Locale::new();

    let warnings = locale.validate_translations("en", "ru");

    assert!(warnings.is_empty());
}

#[test]
fn warning_display_format() {
    let unknown = LoadWarning::UnknownPhrase {
        name: "extra".to_string(),
        language: "ru".to_string(),
    };
    assert_eq!(
        unknown.to_string(),
        "warning: translation 'ru' defines unknown phrase 'extra' not found in source"
    );

    let mismatch = LoadWarning::ParameterCountMismatch {
        name: "greet".to_string(),
        language: "ru".to_string(),
        source_count: 1,
        translation_count: 2,
    };
    assert_eq!(
        mismatch.to_string(),
        "warning: phrase 'greet' in 'ru' has 2 parameter(s), but source has 1"
    );
}

// =========================================================================
// Registry phrase_names / len / is_empty
// =========================================================================

#[test]
fn registry_len_and_is_empty() {
    use rlf::PhraseRegistry;

    let mut registry = PhraseRegistry::new();
    assert!(registry.is_empty());
    assert_eq!(registry.len(), 0);

    registry.load_phrases(r#"hello = "Hello!";"#).unwrap();
    assert!(!registry.is_empty());
    assert_eq!(registry.len(), 1);
}

#[test]
fn registry_phrase_names_returns_all_names() {
    use rlf::PhraseRegistry;

    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        hello = "Hello!";
        goodbye = "Goodbye!";
        greet(name) = "Hi, {name}!";
    "#,
        )
        .unwrap();

    let mut names: Vec<&str> = registry.phrase_names().collect();
    names.sort();
    assert_eq!(names, vec!["goodbye", "greet", "hello"]);
}

// =========================================================================
// Polish Tag Validation
// =========================================================================

#[test]
fn polish_valid_tags_produce_no_warnings() {
    let mut locale = Locale::new();
    locale
        .load_translations_str("en", r#"card = "card";"#)
        .unwrap();
    locale
        .load_translations_str("pl", r#"card = :fem "karta";"#)
        .unwrap();

    let warnings = locale.validate_translations("en", "pl");
    assert!(warnings.is_empty());
}

#[test]
fn polish_all_valid_gender_tags() {
    let mut locale = Locale::new();
    locale
        .load_translations_str(
            "en",
            r#"
            card = "card";
            enemy = "enemy";
            sword = "sword";
            kingdom = "kingdom";
        "#,
        )
        .unwrap();
    locale
        .load_translations_str(
            "pl",
            r#"
            card = :fem "karta";
            enemy = :masc_anim "wróg";
            sword = :masc_inan "miecz";
            kingdom = :neut "królestwo";
        "#,
        )
        .unwrap();

    let warnings = locale.validate_translations("en", "pl");
    assert!(warnings.is_empty());
}

#[test]
fn polish_invalid_tag_produces_warning() {
    let mut locale = Locale::new();
    locale
        .load_translations_str("en", r#"card = "card";"#)
        .unwrap();
    // Using Russian-style :masc instead of Polish :masc_anim or :masc_inan
    locale
        .load_translations_str("pl", r#"card = :masc "karta";"#)
        .unwrap();

    let warnings = locale.validate_translations("en", "pl");

    let invalid_tags: Vec<_> = warnings
        .iter()
        .filter(|w| matches!(w, LoadWarning::InvalidTag { .. }))
        .collect();
    assert_eq!(invalid_tags.len(), 1);
    assert!(matches!(
        &invalid_tags[0],
        LoadWarning::InvalidTag { name, language, tag, .. }
        if name == "card" && language == "pl" && tag == "masc"
    ));
}

#[test]
fn polish_multiple_invalid_tags_produce_multiple_warnings() {
    let mut locale = Locale::new();
    locale
        .load_translations_str(
            "en",
            r#"
            card = "card";
            enemy = "enemy";
        "#,
        )
        .unwrap();
    locale
        .load_translations_str(
            "pl",
            r#"
            card = :masc "karta";
            enemy = :anim "wróg";
        "#,
        )
        .unwrap();

    let warnings = locale.validate_translations("en", "pl");

    let invalid_tags: Vec<_> = warnings
        .iter()
        .filter(|w| matches!(w, LoadWarning::InvalidTag { .. }))
        .collect();
    assert_eq!(invalid_tags.len(), 2);
}

// =========================================================================
// Polish Variant Key Validation
// =========================================================================

#[test]
fn polish_valid_case_variants_produce_no_warnings() {
    let mut locale = Locale::new();
    locale
        .load_translations_str("en", r#"card = "card";"#)
        .unwrap();
    locale
        .load_translations_str(
            "pl",
            r#"
            card = :fem {
                nom: "karta",
                gen: "karty",
                dat: "karcie",
                acc: "kartę",
                ins: "kartą",
                loc: "karcie",
                voc: "karto"
            };
        "#,
        )
        .unwrap();

    let warnings = locale.validate_translations("en", "pl");
    assert!(warnings.is_empty());
}

#[test]
fn polish_valid_case_plural_variants_produce_no_warnings() {
    let mut locale = Locale::new();
    locale
        .load_translations_str("en", r#"card = "card";"#)
        .unwrap();
    locale
        .load_translations_str(
            "pl",
            r#"
            card = :fem {
                nom.one: "karta",
                nom.few: "karty",
                nom.many: "kart",
                nom.other: "karty",
                acc.one: "kartę",
                acc.many: "kart"
            };
        "#,
        )
        .unwrap();

    let warnings = locale.validate_translations("en", "pl");
    assert!(warnings.is_empty());
}

#[test]
fn polish_invalid_case_variant_produces_warning() {
    let mut locale = Locale::new();
    locale
        .load_translations_str("en", r#"card = "card";"#)
        .unwrap();
    // "prep" is Russian's prepositional case; Polish uses "loc" (locative)
    locale
        .load_translations_str(
            "pl",
            r#"
            card = :fem {
                nom: "karta",
                prep: "karcie"
            };
        "#,
        )
        .unwrap();

    let warnings = locale.validate_translations("en", "pl");

    let invalid_keys: Vec<_> = warnings
        .iter()
        .filter(|w| matches!(w, LoadWarning::InvalidVariantKey { .. }))
        .collect();
    assert_eq!(invalid_keys.len(), 1);
    assert!(matches!(
        &invalid_keys[0],
        LoadWarning::InvalidVariantKey { name, language, key, .. }
        if name == "card" && language == "pl" && key == "prep"
    ));
}

#[test]
fn polish_invalid_compound_variant_key_produces_warning() {
    let mut locale = Locale::new();
    locale
        .load_translations_str("en", r#"card = "card";"#)
        .unwrap();
    // "prep" is invalid; "two" is not a Polish plural category
    locale
        .load_translations_str(
            "pl",
            r#"
            card = :fem {
                nom: "karta",
                prep.two: "karcie"
            };
        "#,
        )
        .unwrap();

    let warnings = locale.validate_translations("en", "pl");

    let invalid_keys: Vec<_> = warnings
        .iter()
        .filter(|w| matches!(w, LoadWarning::InvalidVariantKey { .. }))
        .collect();
    // Both "prep" and "two" are invalid for Polish
    assert_eq!(invalid_keys.len(), 2);
}

// =========================================================================
// Polish Warning Display Format
// =========================================================================

#[test]
fn polish_invalid_tag_display_format() {
    let warning = LoadWarning::InvalidTag {
        name: "card".to_string(),
        language: "pl".to_string(),
        tag: "masc".to_string(),
        valid_tags: vec![
            "masc_anim".to_string(),
            "masc_inan".to_string(),
            "fem".to_string(),
            "neut".to_string(),
        ],
    };
    assert_eq!(
        warning.to_string(),
        "warning: phrase 'card' in 'pl' has unrecognized tag ':masc'; valid tags: masc_anim, masc_inan, fem, neut"
    );
}

#[test]
fn polish_invalid_variant_key_display_format() {
    let warning = LoadWarning::InvalidVariantKey {
        name: "card".to_string(),
        language: "pl".to_string(),
        key: "prep".to_string(),
        valid_keys: vec![
            "nom".to_string(),
            "acc".to_string(),
            "gen".to_string(),
            "dat".to_string(),
            "ins".to_string(),
            "loc".to_string(),
            "voc".to_string(),
            "one".to_string(),
            "few".to_string(),
            "many".to_string(),
            "other".to_string(),
        ],
    };
    assert_eq!(
        warning.to_string(),
        "warning: phrase 'card' in 'pl' has unrecognized variant key 'prep'; valid keys: nom, acc, gen, dat, ins, loc, voc, one, few, many, other"
    );
}

// =========================================================================
// Russian Tag and Variant Key Validation (cross-check existing language)
// =========================================================================

#[test]
fn russian_valid_tags_produce_no_warnings() {
    let mut locale = Locale::new();
    locale
        .load_translations_str("en", r#"card = "card";"#)
        .unwrap();
    locale
        .load_translations_str("ru", r#"card = :fem :inan "карта";"#)
        .unwrap();

    let warnings = locale.validate_translations("en", "ru");
    assert!(warnings.is_empty());
}

#[test]
fn russian_valid_case_variants_produce_no_warnings() {
    let mut locale = Locale::new();
    locale
        .load_translations_str("en", r#"card = "card";"#)
        .unwrap();
    locale
        .load_translations_str(
            "ru",
            r#"
            card = :fem :inan {
                nom: "карта",
                acc: "карту",
                gen: "карты",
                dat: "карте",
                ins: "картой",
                prep: "карте"
            };
        "#,
        )
        .unwrap();

    let warnings = locale.validate_translations("en", "ru");
    assert!(warnings.is_empty());
}

#[test]
fn russian_invalid_polish_case_produces_warning() {
    let mut locale = Locale::new();
    locale
        .load_translations_str("en", r#"card = "card";"#)
        .unwrap();
    // "loc" and "voc" are Polish cases, not Russian (Russian uses "prep")
    locale
        .load_translations_str(
            "ru",
            r#"
            card = :fem {
                nom: "карта",
                loc: "карте"
            };
        "#,
        )
        .unwrap();

    let warnings = locale.validate_translations("en", "ru");

    let invalid_keys: Vec<_> = warnings
        .iter()
        .filter(|w| matches!(w, LoadWarning::InvalidVariantKey { .. }))
        .collect();
    assert_eq!(invalid_keys.len(), 1);
    assert!(matches!(
        &invalid_keys[0],
        LoadWarning::InvalidVariantKey { key, .. } if key == "loc"
    ));
}

// =========================================================================
// Languages Without Validation Rules
// =========================================================================

#[test]
fn unknown_language_tags_not_validated() {
    let mut locale = Locale::new();
    locale
        .load_translations_str("en", r#"card = "card";"#)
        .unwrap();
    // "xx" is unknown; any tags should pass without warning
    locale
        .load_translations_str("xx", r#"card = :foo :bar "card";"#)
        .unwrap();

    let warnings = locale.validate_translations("en", "xx");
    assert!(warnings.is_empty());
}
