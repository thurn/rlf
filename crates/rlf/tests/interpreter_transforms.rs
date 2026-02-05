//! Integration tests for transform execution in the interpreter.

use rlf::interpreter::{EvalError, TransformKind, TransformRegistry};
use rlf::{Phrase, PhraseRegistry, Tag, Value};
use std::collections::HashMap;

// =============================================================================
// Basic Case Transforms
// =============================================================================

#[test]
fn test_cap_basic() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(r#"greeting(name) = "Hello, {@cap name}!";"#)
        .unwrap();
    let result = registry
        .call_phrase("en", "greeting", &[Value::from("world")])
        .unwrap();
    assert_eq!(result.to_string(), "Hello, World!");
}

#[test]
fn test_upper_basic() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(r#"shout(text) = "{@upper text}";"#)
        .unwrap();
    let result = registry
        .call_phrase("en", "shout", &[Value::from("hello")])
        .unwrap();
    assert_eq!(result.to_string(), "HELLO");
}

#[test]
fn test_lower_basic() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(r#"quiet(text) = "{@lower text}";"#)
        .unwrap();
    let result = registry
        .call_phrase("en", "quiet", &[Value::from("HELLO")])
        .unwrap();
    assert_eq!(result.to_string(), "hello");
}

// =============================================================================
// Empty String Edge Cases
// =============================================================================

#[test]
fn test_cap_empty() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(r#"empty_cap(text) = "{@cap text}";"#)
        .unwrap();
    let result = registry
        .call_phrase("en", "empty_cap", &[Value::from("")])
        .unwrap();
    assert_eq!(result.to_string(), "");
}

#[test]
fn test_upper_empty() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(r#"empty_upper(text) = "{@upper text}";"#)
        .unwrap();
    let result = registry
        .call_phrase("en", "empty_upper", &[Value::from("")])
        .unwrap();
    assert_eq!(result.to_string(), "");
}

#[test]
fn test_lower_empty() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(r#"empty_lower(text) = "{@lower text}";"#)
        .unwrap();
    let result = registry
        .call_phrase("en", "empty_lower", &[Value::from("")])
        .unwrap();
    assert_eq!(result.to_string(), "");
}

// =============================================================================
// Unicode and Grapheme Handling
// =============================================================================

#[test]
fn test_cap_unicode_cyrillic() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(r#"cap_cyrillic(text) = "{@cap text}";"#)
        .unwrap();
    let result = registry
        .call_phrase(
            "ru",
            "cap_cyrillic",
            &[Value::from(
                "\u{043f}\u{0440}\u{0438}\u{0432}\u{0435}\u{0442}",
            )],
        ) // "привет"
        .unwrap();
    // First grapheme capitalized: "Привет"
    assert_eq!(
        result.to_string(),
        "\u{041f}\u{0440}\u{0438}\u{0432}\u{0435}\u{0442}"
    );
}

#[test]
fn test_cap_combining_character() {
    // Test e + combining acute accent (U+0301)
    // This is one grapheme but two codepoints
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(r#"cap_combining(text) = "{@cap text}";"#)
        .unwrap();
    let result = registry
        .call_phrase("en", "cap_combining", &[Value::from("e\u{0301}xample")])
        .unwrap();
    // The first grapheme (e + combining acute) should be capitalized as a unit
    assert_eq!(result.to_string(), "E\u{0301}xample");
}

#[test]
fn test_upper_greek() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(r#"upper_greek(text) = "{@upper text}";"#)
        .unwrap();
    let result = registry
        .call_phrase(
            "el",
            "upper_greek",
            &[Value::from("\u{03b1}\u{03b2}\u{03b3}")],
        ) // "αβγ"
        .unwrap();
    // Greek uppercase: "ΑΒΓ"
    assert_eq!(result.to_string(), "\u{0391}\u{0392}\u{0393}");
}

#[test]
fn test_lower_greek() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(r#"lower_greek(text) = "{@lower text}";"#)
        .unwrap();
    let result = registry
        .call_phrase(
            "el",
            "lower_greek",
            &[Value::from("\u{0391}\u{0392}\u{0393}")],
        ) // "ΑΒΓ"
        .unwrap();
    // Greek lowercase: "αβγ"
    assert_eq!(result.to_string(), "\u{03b1}\u{03b2}\u{03b3}");
}

// =============================================================================
// Turkish Locale-Sensitive Case Mapping
// =============================================================================

#[test]
fn test_upper_turkish_dotted_i() {
    // Turkish: "i" (dotted lowercase) should uppercase to "I" (dotted capital I, U+0130)
    // Not the standard "I" which other languages would produce
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(r#"upper_tr(text) = "{@upper text}";"#)
        .unwrap();
    let result = registry
        .call_phrase("tr", "upper_tr", &[Value::from("istanbul")])
        .unwrap();
    // Turkish uppercase of "istanbul" should have dotted capital I: "İSTANBUL"
    assert_eq!(result.to_string(), "\u{0130}STANBUL");
}

#[test]
fn test_lower_turkish_capital_i() {
    // Turkish: "I" (undotted capital) should lowercase to "ı" (dotless lowercase i, U+0131)
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(r#"lower_tr(text) = "{@lower text}";"#)
        .unwrap();
    let result = registry
        .call_phrase("tr", "lower_tr", &[Value::from("ISTANBUL")])
        .unwrap();
    // Turkish lowercase of "ISTANBUL" should have dotless i: "ıstanbul"
    assert_eq!(result.to_string(), "\u{0131}stanbul");
}

#[test]
fn test_cap_turkish() {
    // Turkish: "istanbul" -> "Istanbul" with dotted capital I
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(r#"cap_tr(text) = "{@cap text}";"#)
        .unwrap();
    let result = registry
        .call_phrase("tr", "cap_tr", &[Value::from("istanbul")])
        .unwrap();
    // Turkish capitalize: "İstanbul" (dotted capital I)
    assert_eq!(result.to_string(), "\u{0130}stanbul");
}

#[test]
fn test_upper_english_i_for_comparison() {
    // English: "i" uppercases to regular "I", not dotted
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(r#"upper_en(text) = "{@upper text}";"#)
        .unwrap();
    let result = registry
        .call_phrase("en", "upper_en", &[Value::from("istanbul")])
        .unwrap();
    // English uppercase produces standard "I": "ISTANBUL"
    assert_eq!(result.to_string(), "ISTANBUL");
}

// =============================================================================
// Transform Execution Order (Right-to-Left)
// =============================================================================

#[test]
fn test_transform_chain_right_to_left() {
    // {@upper @cap x} should: cap first, then upper
    // "hello" -> "Hello" (cap) -> "HELLO" (upper)
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(r#"chain(text) = "{@upper @cap text}";"#)
        .unwrap();
    let result = registry
        .call_phrase("en", "chain", &[Value::from("hello")])
        .unwrap();
    assert_eq!(result.to_string(), "HELLO");
}

#[test]
fn test_transform_chain_cap_lower() {
    // {@cap @lower x} should: lower first, then cap
    // "HELLO WORLD" -> "hello world" (lower) -> "Hello world" (cap)
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(r#"chain_cap_lower(text) = "{@cap @lower text}";"#)
        .unwrap();
    let result = registry
        .call_phrase("en", "chain_cap_lower", &[Value::from("HELLO WORLD")])
        .unwrap();
    assert_eq!(result.to_string(), "Hello world");
}

#[test]
fn test_transform_chain_three_transforms() {
    // {@cap @lower @upper x} should: upper first, then lower, then cap
    // "HeLLo" -> "HELLO" (upper) -> "hello" (lower) -> "Hello" (cap)
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(r#"chain_three(text) = "{@cap @lower @upper text}";"#)
        .unwrap();
    let result = registry
        .call_phrase("en", "chain_three", &[Value::from("HeLLo")])
        .unwrap();
    assert_eq!(result.to_string(), "Hello");
}

// =============================================================================
// Unknown Transform Error
// =============================================================================

#[test]
fn test_unknown_transform_error() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(r#"unknown(text) = "{@nonexistent text}";"#)
        .unwrap();
    let err = registry
        .call_phrase("en", "unknown", &[Value::from("test")])
        .unwrap_err();
    assert!(
        matches!(err, EvalError::UnknownTransform { ref name } if name == "nonexistent"),
        "Expected UnknownTransform error, got: {:?}",
        err
    );
}

// =============================================================================
// Integration with Templates
// =============================================================================

#[test]
fn test_transform_in_template_with_phrase_reference() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
            card = "card";
            display_card = "The {@cap card}.";
        "#,
        )
        .unwrap();
    let result = registry.get_phrase("en", "display_card").unwrap();
    assert_eq!(result.to_string(), "The Card.");
}

#[test]
fn test_transform_with_variant_phrase() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
            card = { one: "card", other: "cards" };
            display(n) = "The {@cap card:n}.";
        "#,
        )
        .unwrap();
    let one = registry
        .call_phrase("en", "display", &[Value::from(1)])
        .unwrap();
    assert_eq!(one.to_string(), "The Card.");

    let many = registry
        .call_phrase("en", "display", &[Value::from(5)])
        .unwrap();
    assert_eq!(many.to_string(), "The Cards.");
}

#[test]
fn test_transform_preserves_surrounding_text() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(r#"sentence(name) = "Hello {@cap name}, welcome to {@upper name}!";"#)
        .unwrap();
    let result = registry
        .call_phrase("en", "sentence", &[Value::from("world")])
        .unwrap();
    assert_eq!(result.to_string(), "Hello World, welcome to WORLD!");
}

#[test]
fn test_transform_in_eval_str() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
            card = "card";
        "#,
        )
        .unwrap();

    let params: HashMap<String, Value> = [("name".to_string(), Value::from("world"))]
        .into_iter()
        .collect();
    let result = registry
        .eval_str("Hello {@cap name}, see {@upper card}!", "en", params)
        .unwrap();
    assert_eq!(result.to_string(), "Hello World, see CARD!");
}

// =============================================================================
// Edge Cases
// =============================================================================

#[test]
fn test_cap_single_character() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(r#"cap_single(text) = "{@cap text}";"#)
        .unwrap();
    let result = registry
        .call_phrase("en", "cap_single", &[Value::from("a")])
        .unwrap();
    assert_eq!(result.to_string(), "A");
}

#[test]
fn test_upper_already_upper() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(r#"upper_already(text) = "{@upper text}";"#)
        .unwrap();
    let result = registry
        .call_phrase("en", "upper_already", &[Value::from("HELLO")])
        .unwrap();
    assert_eq!(result.to_string(), "HELLO");
}

#[test]
fn test_lower_already_lower() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(r#"lower_already(text) = "{@lower text}";"#)
        .unwrap();
    let result = registry
        .call_phrase("en", "lower_already", &[Value::from("hello")])
        .unwrap();
    assert_eq!(result.to_string(), "hello");
}

#[test]
fn test_cap_already_capitalized() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(r#"cap_already(text) = "{@cap text}";"#)
        .unwrap();
    let result = registry
        .call_phrase("en", "cap_already", &[Value::from("Hello")])
        .unwrap();
    assert_eq!(result.to_string(), "Hello");
}

#[test]
fn test_transform_with_numbers() {
    // Transforms on strings that contain numbers
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(r#"mixed(text) = "{@upper text}";"#)
        .unwrap();
    let result = registry
        .call_phrase("en", "mixed", &[Value::from("test123")])
        .unwrap();
    assert_eq!(result.to_string(), "TEST123");
}

#[test]
fn test_transform_with_punctuation() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(r#"punct(text) = "{@cap text}";"#)
        .unwrap();
    let result = registry
        .call_phrase("en", "punct", &[Value::from("hello, world!")])
        .unwrap();
    assert_eq!(result.to_string(), "Hello, world!");
}

#[test]
fn test_cap_whitespace_start() {
    // Cap on string starting with whitespace
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(r#"cap_ws(text) = "{@cap text}";"#)
        .unwrap();
    let result = registry
        .call_phrase("en", "cap_ws", &[Value::from("  hello")])
        .unwrap();
    // The first grapheme is a space, which doesn't change
    assert_eq!(result.to_string(), "  hello");
}

// =============================================================================
// Azerbaijani (similar to Turkish)
// =============================================================================

#[test]
fn test_upper_azerbaijani_dotted_i() {
    // Azerbaijani also has Turkish-style dotted I handling
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(r#"upper_az(text) = "{@upper text}";"#)
        .unwrap();
    let result = registry
        .call_phrase("az", "upper_az", &[Value::from("istanbul")])
        .unwrap();
    // Azerbaijani uppercase of "istanbul" should have dotted capital I: "İSTANBUL"
    assert_eq!(result.to_string(), "\u{0130}STANBUL");
}

// =============================================================================
// English Article Transforms (@a/@an, @the)
// =============================================================================

#[test]
fn english_a_with_a_tag() {
    // Phrase with :a tag produces "a card"
    let phrase = Phrase::builder()
        .text("card".to_string())
        .tags(vec![Tag::new("a")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::EnglishA;
    let result = transform.execute(&value, None, "en").unwrap();
    assert_eq!(result, "a card");
}

#[test]
fn english_a_with_an_tag() {
    // Phrase with :an tag produces "an event"
    let phrase = Phrase::builder()
        .text("event".to_string())
        .tags(vec![Tag::new("an")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::EnglishA;
    let result = transform.execute(&value, None, "en").unwrap();
    assert_eq!(result, "an event");
}

#[test]
fn english_a_missing_tag_error() {
    // Phrase without :a or :an tag produces error
    let phrase = Phrase::builder().text("thing".to_string()).build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::EnglishA;
    let result = transform.execute(&value, None, "en");
    assert!(matches!(result, Err(EvalError::MissingTag { .. })));
}

#[test]
fn english_a_string_value_error() {
    // String values (no tags) produce error
    let value = Value::String("card".to_string());
    let transform = TransformKind::EnglishA;
    let result = transform.execute(&value, None, "en");
    assert!(matches!(result, Err(EvalError::MissingTag { .. })));
}

#[test]
fn english_the_transform() {
    // @the always produces "the X"
    let value = Value::String("card".to_string());
    let transform = TransformKind::EnglishThe;
    let result = transform.execute(&value, None, "en").unwrap();
    assert_eq!(result, "the card");
}

#[test]
fn english_the_with_phrase() {
    // @the works with Phrase values too
    let phrase = Phrase::builder()
        .text("card".to_string())
        .tags(vec![Tag::new("a")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::EnglishThe;
    let result = transform.execute(&value, None, "en").unwrap();
    assert_eq!(result, "the card");
}

#[test]
fn english_transform_alias_an() {
    // @an resolves to EnglishA
    let registry = TransformRegistry::new();
    let transform = registry.get("an", "en");
    assert_eq!(transform, Some(TransformKind::EnglishA));
}

#[test]
fn english_transform_a_lookup() {
    // @a resolves to EnglishA
    let registry = TransformRegistry::new();
    let transform = registry.get("a", "en");
    assert_eq!(transform, Some(TransformKind::EnglishA));
}

#[test]
fn english_transform_the_lookup() {
    // @the resolves to EnglishThe
    let registry = TransformRegistry::new();
    let transform = registry.get("the", "en");
    assert_eq!(transform, Some(TransformKind::EnglishThe));
}

#[test]
fn english_transform_not_available_for_other_languages() {
    // English transforms should not be available for other languages
    let registry = TransformRegistry::new();
    assert_eq!(registry.get("a", "de"), None);
    assert_eq!(registry.get("the", "de"), None);
}
