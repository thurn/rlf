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
            LoadWarning::ParameterCountMismatch { .. } => None,
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
