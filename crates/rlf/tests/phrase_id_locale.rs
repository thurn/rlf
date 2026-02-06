//! Tests for PhraseId integration with Locale.

use rlf::{EvalError, Locale, PhraseId, Value};

// =========================================================================
// resolve(&Locale)
// =========================================================================

#[test]
fn resolve_parameterless_phrase() {
    let mut locale = Locale::new();
    locale
        .load_translations_str("en", r#"hello = "Hello, world!";"#)
        .unwrap();

    let id = PhraseId::from_name("hello");
    let phrase = id.resolve(&locale).unwrap();
    assert_eq!(phrase.to_string(), "Hello, world!");
}

#[test]
fn resolve_phrase_with_variants() {
    let mut locale = Locale::new();
    locale
        .load_translations_str("en", r#"card = { one: "card", other: "cards" };"#)
        .unwrap();

    let id = PhraseId::from_name("card");
    let phrase = id.resolve(&locale).unwrap();
    assert_eq!(phrase.to_string(), "card");
    assert_eq!(phrase.variant("other"), "cards");
}

#[test]
fn resolve_phrase_not_found() {
    let mut locale = Locale::new();
    locale
        .load_translations_str("en", r#"hello = "Hello!";"#)
        .unwrap();

    let id = PhraseId::from_name("nonexistent");
    let result = id.resolve(&locale);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        EvalError::PhraseNotFoundById { .. }
    ));
}

#[test]
fn resolve_no_language_loaded() {
    let locale = Locale::new();

    let id = PhraseId::from_name("hello");
    let result = id.resolve(&locale);
    assert!(result.is_err());
}

#[test]
fn resolve_wrong_language() {
    let mut locale = Locale::builder().language("ru").build();
    locale
        .load_translations_str("en", r#"hello = "Hello!";"#)
        .unwrap();

    let id = PhraseId::from_name("hello");
    let result = id.resolve(&locale);
    assert!(result.is_err());
}

#[test]
fn resolve_after_language_switch() {
    let mut locale = Locale::new();
    locale
        .load_translations_str("en", r#"hello = "Hello!";"#)
        .unwrap();
    locale
        .load_translations_str("ru", r#"hello = "Привет!";"#)
        .unwrap();

    let id = PhraseId::from_name("hello");

    let phrase = id.resolve(&locale).unwrap();
    assert_eq!(phrase.to_string(), "Hello!");

    locale.set_language("ru");
    let phrase = id.resolve(&locale).unwrap();
    assert_eq!(phrase.to_string(), "Привет!");
}

// =========================================================================
// call(&Locale, &[Value])
// =========================================================================

#[test]
fn call_phrase_with_args() {
    let mut locale = Locale::new();
    locale
        .load_translations_str("en", r#"greet(name) = "Hello, {name}!";"#)
        .unwrap();

    let id = PhraseId::from_name("greet");
    let phrase = id.call(&locale, &[Value::from("World")]).unwrap();
    assert_eq!(phrase.to_string(), "Hello, World!");
}

#[test]
fn call_phrase_with_numeric_arg() {
    let mut locale = Locale::new();
    locale
        .load_translations_str(
            "en",
            r#"
        card = { one: "card", other: "cards" };
        draw(n) = "Draw {n} {card:n}.";
    "#,
        )
        .unwrap();

    let id = PhraseId::from_name("draw");
    let phrase = id.call(&locale, &[Value::from(3)]).unwrap();
    assert_eq!(phrase.to_string(), "Draw 3 cards.");
}

#[test]
fn call_parameterless_with_empty_slice() {
    let mut locale = Locale::new();
    locale
        .load_translations_str("en", r#"hello = "Hello!";"#)
        .unwrap();

    let id = PhraseId::from_name("hello");
    let phrase = id.call(&locale, &[]).unwrap();
    assert_eq!(phrase.to_string(), "Hello!");
}

#[test]
fn call_phrase_not_found() {
    let mut locale = Locale::new();
    locale
        .load_translations_str("en", r#"hello = "Hello!";"#)
        .unwrap();

    let id = PhraseId::from_name("nonexistent");
    let result = id.call(&locale, &[]);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        EvalError::PhraseNotFoundById { .. }
    ));
}

#[test]
fn call_wrong_arg_count() {
    let mut locale = Locale::new();
    locale
        .load_translations_str("en", r#"greet(name) = "Hello, {name}!";"#)
        .unwrap();

    let id = PhraseId::from_name("greet");
    let result = id.call(&locale, &[]);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        EvalError::ArgumentCount {
            expected: 1,
            got: 0,
            ..
        }
    ));
}

// =========================================================================
// name(&Locale)
// =========================================================================

#[test]
fn name_returns_phrase_name() {
    let mut locale = Locale::new();
    locale
        .load_translations_str("en", r#"hello = "Hello!";"#)
        .unwrap();

    let id = PhraseId::from_name("hello");
    assert_eq!(id.name(&locale), Some("hello"));
}

#[test]
fn name_returns_none_for_unknown() {
    let mut locale = Locale::new();
    locale
        .load_translations_str("en", r#"hello = "Hello!";"#)
        .unwrap();

    let id = PhraseId::from_name("nonexistent");
    assert_eq!(id.name(&locale), None);
}

#[test]
fn name_returns_none_when_no_language_loaded() {
    let locale = Locale::new();

    let id = PhraseId::from_name("hello");
    assert_eq!(id.name(&locale), None);
}

// =========================================================================
// has_parameters(&Locale) and parameter_count(&Locale)
// =========================================================================

#[test]
fn has_parameters_true_for_parameterized_phrase() {
    let mut locale = Locale::new();
    locale
        .load_translations_str("en", r#"greet(name) = "Hello, {name}!";"#)
        .unwrap();

    let id = PhraseId::from_name("greet");
    assert!(id.has_parameters(&locale));
    assert_eq!(id.parameter_count(&locale), 1);
}

#[test]
fn has_parameters_false_for_parameterless_phrase() {
    let mut locale = Locale::new();
    locale
        .load_translations_str("en", r#"hello = "Hello!";"#)
        .unwrap();

    let id = PhraseId::from_name("hello");
    assert!(!id.has_parameters(&locale));
    assert_eq!(id.parameter_count(&locale), 0);
}

#[test]
fn parameter_count_multiple_params() {
    let mut locale = Locale::new();
    locale
        .load_translations_str("en", r#"msg(a, b, c) = "{a} {b} {c}";"#)
        .unwrap();

    let id = PhraseId::from_name("msg");
    assert!(id.has_parameters(&locale));
    assert_eq!(id.parameter_count(&locale), 3);
}

#[test]
fn parameter_count_returns_zero_for_unknown() {
    let mut locale = Locale::new();
    locale
        .load_translations_str("en", r#"hello = "Hello!";"#)
        .unwrap();

    let id = PhraseId::from_name("nonexistent");
    assert!(!id.has_parameters(&locale));
    assert_eq!(id.parameter_count(&locale), 0);
}

// =========================================================================
// Const PhraseId construction
// =========================================================================

#[test]
fn const_phrase_id_works_with_locale() {
    const HELLO: PhraseId = PhraseId::from_name("hello");

    let mut locale = Locale::new();
    locale
        .load_translations_str("en", r#"hello = "Hello!";"#)
        .unwrap();

    let phrase = HELLO.resolve(&locale).unwrap();
    assert_eq!(phrase.to_string(), "Hello!");
}

// =========================================================================
// resolve_with_registry (existing API still works)
// =========================================================================

#[test]
fn resolve_with_registry_still_works() {
    let mut registry = rlf::PhraseRegistry::new();
    registry.load_phrases(r#"hello = "Hello!";"#).unwrap();

    let id = PhraseId::from_name("hello");
    let phrase = id.resolve_with_registry(&registry, "en").unwrap();
    assert_eq!(phrase.to_string(), "Hello!");
}

#[test]
fn call_with_registry_still_works() {
    let mut registry = rlf::PhraseRegistry::new();
    registry
        .load_phrases(r#"greet(name) = "Hello, {name}!";"#)
        .unwrap();

    let id = PhraseId::from_name("greet");
    let phrase = id
        .call_with_registry(&registry, "en", &[Value::from("World")])
        .unwrap();
    assert_eq!(phrase.to_string(), "Hello, World!");
}
