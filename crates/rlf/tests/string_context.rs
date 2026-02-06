//! Integration tests for StringContext format variant selection.

use rlf::{Locale, Value};
use std::collections::HashMap;

// =========================================================================
// Basic StringContext
// =========================================================================

#[test]
fn string_context_defaults_to_none() {
    let locale = Locale::new();
    assert!(locale.string_context().is_none());
}

#[test]
fn string_context_set_and_get() {
    let mut locale = Locale::new();
    locale.set_string_context(Some("card_text"));
    assert_eq!(locale.string_context(), Some("card_text"));
}

#[test]
fn string_context_clear() {
    let mut locale = Locale::new();
    locale.set_string_context(Some("card_text"));
    locale.set_string_context(None::<String>);
    assert!(locale.string_context().is_none());
}

#[test]
fn string_context_builder() {
    let locale = Locale::builder()
        .language("en")
        .string_context("interface".to_string())
        .build();
    assert_eq!(locale.string_context(), Some("interface"));
}

// =========================================================================
// Variant Selection with StringContext
// =========================================================================

#[test]
fn string_context_selects_variant_as_default_text() {
    let mut locale = Locale::new();
    locale
        .load_translations_str(
            "en",
            r#"
        energy = {
            interface: "E",
            card_text: "<b>E</b>",
        };
    "#,
        )
        .unwrap();

    // Without context: default text is first variant
    let phrase = locale.get_phrase("energy").unwrap();
    assert_eq!(phrase.to_string(), "E");

    // With context: default text matches the context variant
    locale.set_string_context(Some("card_text"));
    let phrase = locale.get_phrase("energy").unwrap();
    assert_eq!(phrase.to_string(), "<b>E</b>");
}

#[test]
fn string_context_preserves_all_variants() {
    let mut locale = Locale::new();
    locale
        .load_translations_str(
            "en",
            r#"
        energy = {
            interface: "E",
            card_text: "<b>E</b>",
        };
    "#,
        )
        .unwrap();

    locale.set_string_context(Some("card_text"));
    let phrase = locale.get_phrase("energy").unwrap();

    // Default text is card_text variant
    assert_eq!(phrase.to_string(), "<b>E</b>");
    // But all variants are still accessible
    assert_eq!(phrase.variant("interface"), "E");
    assert_eq!(phrase.variant("card_text"), "<b>E</b>");
}

#[test]
fn string_context_falls_back_when_no_match() {
    let mut locale = Locale::new();
    locale
        .load_translations_str(
            "en",
            r#"
        greeting = {
            formal: "Good day",
            casual: "Hey",
        };
    "#,
        )
        .unwrap();

    // Context doesn't match any variant: use default (first variant)
    locale.set_string_context(Some("card_text"));
    let phrase = locale.get_phrase("greeting").unwrap();
    assert_eq!(phrase.to_string(), "Good day");
}

#[test]
fn string_context_no_effect_on_simple_phrases() {
    let mut locale = Locale::new();
    locale
        .load_translations_str("en", r#"hello = "Hello!";"#)
        .unwrap();

    locale.set_string_context(Some("card_text"));
    let phrase = locale.get_phrase("hello").unwrap();
    assert_eq!(phrase.to_string(), "Hello!");
}

#[test]
fn string_context_with_plural_variants() {
    let mut locale = Locale::new();
    locale
        .load_translations_str(
            "en",
            r#"
        card = {
            one: "card",
            other: "cards",
        };
    "#,
        )
        .unwrap();

    // Plural variants are not affected by string context (no match)
    locale.set_string_context(Some("card_text"));
    let phrase = locale.get_phrase("card").unwrap();
    assert_eq!(phrase.to_string(), "card");
    assert_eq!(phrase.variant("one"), "card");
    assert_eq!(phrase.variant("other"), "cards");
}

// =========================================================================
// StringContext with Phrase Calls
// =========================================================================

#[test]
fn string_context_applies_to_phrase_calls() {
    let mut locale = Locale::new();
    locale
        .load_translations_str(
            "en",
            r#"
        icon = {
            interface: "E",
            card_text: "<b>E</b>",
        };
        cost(n) = "{n}{icon}";
    "#,
        )
        .unwrap();

    // Without context
    let phrase = locale.call_phrase("cost", &[Value::from(3)]).unwrap();
    assert_eq!(phrase.to_string(), "3E");

    // With context: nested phrase references also use the context
    locale.set_string_context(Some("card_text"));
    let phrase = locale.call_phrase("cost", &[Value::from(3)]).unwrap();
    assert_eq!(phrase.to_string(), "3<b>E</b>");
}

#[test]
fn string_context_applies_to_eval_str() {
    let mut locale = Locale::new();
    locale
        .load_translations_str(
            "en",
            r#"
        icon = {
            interface: "E",
            card_text: "<b>E</b>",
        };
    "#,
        )
        .unwrap();

    let params = HashMap::new();

    // Without context
    let phrase = locale.eval_str("{icon}", params.clone()).unwrap();
    assert_eq!(phrase.to_string(), "E");

    // With context
    locale.set_string_context(Some("card_text"));
    let phrase = locale.eval_str("{icon}", params).unwrap();
    assert_eq!(phrase.to_string(), "<b>E</b>");
}

// =========================================================================
// StringContext Switching
// =========================================================================

#[test]
fn string_context_can_be_switched() {
    let mut locale = Locale::new();
    locale
        .load_translations_str(
            "en",
            r#"
        energy_symbol = {
            interface: "●",
            card_text: "<color=#00838F>●</color>",
        };
    "#,
        )
        .unwrap();

    locale.set_string_context(Some("interface"));
    let phrase = locale.get_phrase("energy_symbol").unwrap();
    assert_eq!(phrase.to_string(), "●");

    locale.set_string_context(Some("card_text"));
    let phrase = locale.get_phrase("energy_symbol").unwrap();
    assert_eq!(phrase.to_string(), "<color=#00838F>●</color>");

    locale.set_string_context(None::<String>);
    let phrase = locale.get_phrase("energy_symbol").unwrap();
    assert_eq!(phrase.to_string(), "●"); // Falls back to first variant
}

// =========================================================================
// StringContext with Translations
// =========================================================================

#[test]
fn string_context_works_across_languages() {
    let mut locale = Locale::new();
    locale
        .load_translations_str(
            "en",
            r#"
        energy = {
            interface: "E",
            card_text: "<b>E</b>",
        };
    "#,
        )
        .unwrap();
    locale
        .load_translations_str(
            "ru",
            r#"
        energy = {
            interface: "Э",
            card_text: "<b>Э</b>",
        };
    "#,
        )
        .unwrap();

    locale.set_string_context(Some("card_text"));

    let en = locale.get_phrase("energy").unwrap();
    assert_eq!(en.to_string(), "<b>E</b>");

    locale.set_language("ru");
    let ru = locale.get_phrase("energy").unwrap();
    assert_eq!(ru.to_string(), "<b>Э</b>");
}

// =========================================================================
// Edge Cases
// =========================================================================

#[test]
fn string_context_with_single_variant_matching() {
    let mut locale = Locale::new();
    locale
        .load_translations_str(
            "en",
            r#"
        msg = {
            card_text: "styled",
        };
    "#,
        )
        .unwrap();

    // With matching context: uses the only variant
    locale.set_string_context(Some("card_text"));
    let phrase = locale.get_phrase("msg").unwrap();
    assert_eq!(phrase.to_string(), "styled");

    // Without context: first variant is still "styled"
    locale.set_string_context(None::<String>);
    let phrase = locale.get_phrase("msg").unwrap();
    assert_eq!(phrase.to_string(), "styled");
}

#[test]
fn string_context_with_multi_key_variants() {
    let mut locale = Locale::new();
    locale
        .load_translations_str(
            "en",
            r#"
        symbol = {
            interface, tooltip: "●",
            card_text: "<b>●</b>",
        };
    "#,
        )
        .unwrap();

    locale.set_string_context(Some("tooltip"));
    let phrase = locale.get_phrase("symbol").unwrap();
    assert_eq!(phrase.to_string(), "●");

    locale.set_string_context(Some("card_text"));
    let phrase = locale.get_phrase("symbol").unwrap();
    assert_eq!(phrase.to_string(), "<b>●</b>");
}
