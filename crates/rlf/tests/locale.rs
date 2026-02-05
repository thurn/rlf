//! Integration tests for Locale management.

use rlf::{EvalError, LoadError, Locale, Value};
use std::collections::HashMap;
use std::io::{Seek, Write};
use tempfile::NamedTempFile;

// =========================================================================
// Builder and Basic API
// =========================================================================

#[test]
fn locale_default_is_english() {
    let locale = Locale::new();
    assert_eq!(locale.language(), "en");
}

#[test]
fn locale_builder_sets_language() {
    let locale = Locale::builder().language("ru").build();
    assert_eq!(locale.language(), "ru");
}

#[test]
fn locale_with_language_shorthand() {
    let locale = Locale::with_language("de");
    assert_eq!(locale.language(), "de");
}

#[test]
fn locale_set_language_changes_current() {
    let mut locale = Locale::new();
    assert_eq!(locale.language(), "en");

    locale.set_language("ru");
    assert_eq!(locale.language(), "ru");
}

// =========================================================================
// Translation Loading from String
// =========================================================================

#[test]
fn load_translations_str_parses_phrases() {
    let mut locale = Locale::new();
    let count = locale
        .load_translations_str(
            "en",
            r#"
        hello = "Hello!";
        card = { one: "card", other: "cards" };
    "#,
        )
        .unwrap();

    assert_eq!(count, 2);
}

#[test]
fn load_translations_str_replaces_on_reload() {
    let mut locale = Locale::new();

    // First load
    locale
        .load_translations_str("en", r#"hello = "Hello!";"#)
        .unwrap();
    let phrase = locale.get_phrase("hello").unwrap();
    assert_eq!(phrase.to_string(), "Hello!");

    // Second load should replace all phrases for that language
    locale
        .load_translations_str("en", r#"goodbye = "Goodbye!";"#)
        .unwrap();

    // Old phrase should be gone
    let result = locale.get_phrase("hello");
    assert!(result.is_err());

    // New phrase should exist
    let phrase = locale.get_phrase("goodbye").unwrap();
    assert_eq!(phrase.to_string(), "Goodbye!");
}

#[test]
fn load_translations_str_returns_parse_error() {
    let mut locale = Locale::new();
    let result = locale.load_translations_str("en", r#"invalid syntax here"#);

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, LoadError::Parse { .. }));
}

// =========================================================================
// Per-Language Storage
// =========================================================================

#[test]
fn translations_stored_per_language() {
    let mut locale = Locale::new();

    // Load English
    locale
        .load_translations_str("en", r#"hello = "Hello!";"#)
        .unwrap();

    // Load Russian
    locale
        .load_translations_str("ru", r#"hello = "Привет!";"#)
        .unwrap();

    // English lookup
    locale.set_language("en");
    let phrase = locale.get_phrase("hello").unwrap();
    assert_eq!(phrase.to_string(), "Hello!");

    // Russian lookup
    locale.set_language("ru");
    let phrase = locale.get_phrase("hello").unwrap();
    assert_eq!(phrase.to_string(), "Привет!");
}

#[test]
fn reloading_language_only_affects_that_language() {
    let mut locale = Locale::new();

    // Load both languages
    locale
        .load_translations_str("en", r#"hello = "Hello!";"#)
        .unwrap();
    locale
        .load_translations_str("ru", r#"privet = "Привет!";"#)
        .unwrap();

    // Reload English with different content
    locale
        .load_translations_str("en", r#"goodbye = "Goodbye!";"#)
        .unwrap();

    // Russian should be unaffected
    locale.set_language("ru");
    let phrase = locale.get_phrase("privet").unwrap();
    assert_eq!(phrase.to_string(), "Привет!");
}

// =========================================================================
// Translation Loading from File
// =========================================================================

#[test]
fn load_translations_from_file() {
    let mut locale = Locale::new();

    // Create temp file with translation content
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, r#"hello = "Hello from file!";"#).unwrap();

    let count = locale.load_translations("en", file.path()).unwrap();
    assert_eq!(count, 1);
}

#[test]
fn load_translations_file_not_found() {
    let mut locale = Locale::new();
    let result = locale.load_translations("en", "/nonexistent/path/file.rlf");

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, LoadError::Io { .. }));
}

// =========================================================================
// Hot Reload
// =========================================================================

#[test]
fn reload_translations_rereads_file() {
    let mut locale = Locale::new();

    // Create temp file
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, r#"hello = "Original";"#).unwrap();
    file.flush().unwrap();

    // Initial load
    locale.load_translations("en", file.path()).unwrap();
    let phrase = locale.get_phrase("hello").unwrap();
    assert_eq!(phrase.to_string(), "Original");

    // Modify file
    file.rewind().unwrap();
    file.as_file_mut().set_len(0).unwrap();
    writeln!(file, r#"hello = "Modified";"#).unwrap();
    file.flush().unwrap();

    // Reload
    let count = locale.reload_translations("en").unwrap();
    assert_eq!(count, 1);

    let phrase = locale.get_phrase("hello").unwrap();
    assert_eq!(phrase.to_string(), "Modified");
}

#[test]
fn reload_string_loaded_returns_error() {
    let mut locale = Locale::new();

    // Load from string
    locale
        .load_translations_str("en", r#"hello = "Hello!";"#)
        .unwrap();

    // Attempt reload should fail
    let result = locale.reload_translations("en");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, LoadError::NoPathForReload { .. }));
}

#[test]
fn reload_unloaded_language_returns_error() {
    let mut locale = Locale::new();
    let result = locale.reload_translations("ru");

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        LoadError::NoPathForReload { .. }
    ));
}

// =========================================================================
// Phrase Evaluation
// =========================================================================

#[test]
fn get_phrase_returns_phrase() {
    let mut locale = Locale::new();
    locale
        .load_translations_str("en", r#"hello = "Hello, world!";"#)
        .unwrap();

    let phrase = locale.get_phrase("hello").unwrap();
    assert_eq!(phrase.to_string(), "Hello, world!");
}

#[test]
fn get_phrase_not_found_returns_error() {
    let mut locale = Locale::new();
    locale
        .load_translations_str("en", r#"hello = "Hello!";"#)
        .unwrap();

    let result = locale.get_phrase("nonexistent");
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        EvalError::PhraseNotFound { .. }
    ));
}

#[test]
fn call_phrase_with_args() {
    let mut locale = Locale::new();
    locale
        .load_translations_str(
            "en",
            r#"
        greet(name) = "Hello, {name}!";
    "#,
        )
        .unwrap();

    let phrase = locale
        .call_phrase("greet", &[Value::from("World")])
        .unwrap();
    assert_eq!(phrase.to_string(), "Hello, World!");
}

#[test]
fn eval_str_evaluates_template() {
    let mut locale = Locale::new();
    locale
        .load_translations_str(
            "en",
            r#"
        card = { one: "card", other: "cards" };
    "#,
        )
        .unwrap();

    let params: HashMap<String, Value> = [("n".to_string(), Value::from(3))].into_iter().collect();
    let phrase = locale.eval_str("Draw {n} {card:n}.", params).unwrap();
    assert_eq!(phrase.to_string(), "Draw 3 cards.");
}

// =========================================================================
// Fallback Language
// =========================================================================

#[test]
fn fallback_language_used_when_primary_missing() {
    let mut locale = Locale::builder()
        .language("ru")
        .fallback_language("en".to_string())
        .build();

    // Load English only
    locale
        .load_translations_str("en", r#"hello = "Hello!";"#)
        .unwrap();

    // Russian phrase lookup should fall back to English
    let phrase = locale.get_phrase("hello").unwrap();
    assert_eq!(phrase.to_string(), "Hello!");
}

#[test]
fn fallback_not_used_when_primary_has_phrase() {
    let mut locale = Locale::builder()
        .language("ru")
        .fallback_language("en".to_string())
        .build();

    locale
        .load_translations_str("en", r#"hello = "Hello!";"#)
        .unwrap();
    locale
        .load_translations_str("ru", r#"hello = "Привет!";"#)
        .unwrap();

    // Should use Russian, not fallback to English
    let phrase = locale.get_phrase("hello").unwrap();
    assert_eq!(phrase.to_string(), "Привет!");
}

#[test]
fn no_fallback_by_default() {
    let mut locale = Locale::builder().language("ru").build();

    // Load English only (no fallback configured)
    locale
        .load_translations_str("en", r#"hello = "Hello!";"#)
        .unwrap();

    // Russian phrase lookup should fail (no fallback)
    let result = locale.get_phrase("hello");
    assert!(result.is_err());
}

#[test]
fn fallback_works_for_call_phrase() {
    let mut locale = Locale::builder()
        .language("ru")
        .fallback_language("en".to_string())
        .build();

    locale
        .load_translations_str("en", r#"greet(name) = "Hello, {name}!";"#)
        .unwrap();

    // Should fall back to English
    let phrase = locale
        .call_phrase("greet", &[Value::from("World")])
        .unwrap();
    assert_eq!(phrase.to_string(), "Hello, World!");
}

// =========================================================================
// Transform Registry Access
// =========================================================================

#[test]
fn transforms_accessible_from_locale() {
    let locale = Locale::new();

    // Should have default transforms available
    let transforms = locale.transforms();
    // TransformRegistry exists and is accessible
    assert!(std::ptr::eq(transforms, locale.transforms()));
}

// =========================================================================
// Registry Access
// =========================================================================

#[test]
fn registry_for_returns_language_registry() {
    let mut locale = Locale::new();
    locale
        .load_translations_str("en", r#"hello = "Hello!";"#)
        .unwrap();
    locale
        .load_translations_str("ru", r#"privet = "Привет!";"#)
        .unwrap();

    let en_registry = locale.registry_for("en").unwrap();
    assert!(en_registry.get("hello").is_some());
    assert!(en_registry.get("privet").is_none());

    let ru_registry = locale.registry_for("ru").unwrap();
    assert!(ru_registry.get("privet").is_some());
    assert!(ru_registry.get("hello").is_none());
}

#[test]
fn registry_returns_current_language_registry() {
    let mut locale = Locale::new();
    locale
        .load_translations_str("en", r#"hello = "Hello!";"#)
        .unwrap();

    let registry = locale.registry().unwrap();
    assert!(registry.get("hello").is_some());
}
