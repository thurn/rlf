//! Integration tests for transform execution in the interpreter.

use rlf::interpreter::{EvalError, Locale, TransformKind, TransformRegistry};
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

// =============================================================================
// English Article Transform Integration Tests (Full Evaluation Path)
// =============================================================================

#[test]
fn english_a_in_template() {
    // Test: "Draw {@a card}." with card = :a "card"
    let source = r#"
        card = :a "card";
        event = :an "event";
        draw_card = "Draw {@a card}.";
        play_event = "Play {@a event}.";
        the_card = "{@the card}";
    "#;

    let mut locale = Locale::builder().language("en").build();
    locale.load_translations_str("en", source).unwrap();

    assert_eq!(
        locale.get_phrase("draw_card").unwrap().to_string(),
        "Draw a card."
    );
    assert_eq!(
        locale.get_phrase("play_event").unwrap().to_string(),
        "Play an event."
    );
    assert_eq!(
        locale.get_phrase("the_card").unwrap().to_string(),
        "the card"
    );
}

#[test]
fn english_a_with_cap() {
    // Test transform ordering: {@cap @a card} -> "A card"
    let source = r#"
        card = :a "card";
        a_card = "{@cap @a card}";
    "#;

    let mut locale = Locale::builder().language("en").build();
    locale.load_translations_str("en", source).unwrap();

    // Right-to-left: @a first ("a card"), then @cap ("A card")
    assert_eq!(locale.get_phrase("a_card").unwrap().to_string(), "A card");
}

#[test]
fn english_a_missing_tag_full_eval() {
    // Test error when tag missing in full evaluation
    let source = r#"
        thing = "thing";
        draw_thing = "Draw {@a thing}.";
    "#;

    let mut locale = Locale::builder().language("en").build();
    locale.load_translations_str("en", source).unwrap();

    let result = locale.get_phrase("draw_thing");
    assert!(matches!(result, Err(EvalError::MissingTag { .. })));
}

#[test]
fn english_the_with_cap_chained() {
    // Test {@cap @the card} -> "The card"
    let source = r#"
        card = :a "card";
        the_card_cap = "{@cap @the card}";
    "#;

    let mut locale = Locale::builder().language("en").build();
    locale.load_translations_str("en", source).unwrap();

    // Right-to-left: @the first ("the card"), then @cap ("The card")
    assert_eq!(
        locale.get_phrase("the_card_cap").unwrap().to_string(),
        "The card"
    );
}

#[test]
fn english_an_alias_in_template() {
    // Test: @an alias works in template evaluation
    let source = r#"
        event = :an "event";
        play_event = "Play {@an event}.";
    "#;

    let mut locale = Locale::builder().language("en").build();
    locale.load_translations_str("en", source).unwrap();

    assert_eq!(
        locale.get_phrase("play_event").unwrap().to_string(),
        "Play an event."
    );
}

#[test]
fn english_a_with_variant_phrase() {
    // Test @a with a phrase that has variants - uses default text
    let source = r#"
        card = :a { one: "card", other: "cards" };
        draw_card = "Draw {@a card}.";
    "#;

    let mut locale = Locale::builder().language("en").build();
    locale.load_translations_str("en", source).unwrap();

    // Default text is "card" (first variant), which has :a tag
    assert_eq!(
        locale.get_phrase("draw_card").unwrap().to_string(),
        "Draw a card."
    );
}

#[test]
fn english_a_after_selector_fails() {
    // When selector is applied, we get a String (losing tags), so @a fails
    let source = r#"
        card = :a { one: "card", other: "cards" };
        draw_n(n) = "Draw {@a card:n}.";
    "#;

    let mut locale = Locale::builder().language("en").build();
    locale.load_translations_str("en", source).unwrap();

    // After :n selector, card:one returns "card" as String, losing the :a tag
    let result = locale.call_phrase("draw_n", &[Value::from(1)]);
    assert!(
        matches!(result, Err(EvalError::MissingTag { .. })),
        "Expected MissingTag error when selector strips tags"
    );
}

#[test]
fn english_upper_a_card() {
    // Test {@upper @a card} -> "A CARD"
    let source = r#"
        card = :a "card";
        shouted = "{@upper @a card}";
    "#;

    let mut locale = Locale::builder().language("en").build();
    locale.load_translations_str("en", source).unwrap();

    // Right-to-left: @a first ("a card"), then @upper ("A CARD")
    assert_eq!(locale.get_phrase("shouted").unwrap().to_string(), "A CARD");
}

// =============================================================================
// German Article Transforms (@der/@die/@das, @ein/@eine)
// =============================================================================

#[test]
fn german_der_nominative_masculine() {
    let phrase = Phrase::builder()
        .text("Charakter".to_string())
        .tags(vec![Tag::new("masc")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::GermanDer;
    let result = transform.execute(&value, None, "de").unwrap();
    assert_eq!(result, "der Charakter");
}

#[test]
fn german_der_accusative_masculine() {
    let phrase = Phrase::builder()
        .text("Charakter".to_string())
        .tags(vec![Tag::new("masc")])
        .build();
    let value = Value::Phrase(phrase);
    let context = Value::String("acc".to_string());
    let transform = TransformKind::GermanDer;
    let result = transform.execute(&value, Some(&context), "de").unwrap();
    assert_eq!(result, "den Charakter");
}

#[test]
fn german_der_dative_feminine() {
    let phrase = Phrase::builder()
        .text("Karte".to_string())
        .tags(vec![Tag::new("fem")])
        .build();
    let value = Value::Phrase(phrase);
    let context = Value::String("dat".to_string());
    let transform = TransformKind::GermanDer;
    let result = transform.execute(&value, Some(&context), "de").unwrap();
    assert_eq!(result, "der Karte"); // feminine dative = "der"
}

#[test]
fn german_der_neuter() {
    let phrase = Phrase::builder()
        .text("Ereignis".to_string())
        .tags(vec![Tag::new("neut")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::GermanDer;
    let result = transform.execute(&value, None, "de").unwrap();
    assert_eq!(result, "das Ereignis");
}

#[test]
fn german_ein_accusative_masculine() {
    let phrase = Phrase::builder()
        .text("Charakter".to_string())
        .tags(vec![Tag::new("masc")])
        .build();
    let value = Value::Phrase(phrase);
    let context = Value::String("acc".to_string());
    let transform = TransformKind::GermanEin;
    let result = transform.execute(&value, Some(&context), "de").unwrap();
    assert_eq!(result, "einen Charakter");
}

#[test]
fn german_ein_nominative_feminine() {
    let phrase = Phrase::builder()
        .text("Karte".to_string())
        .tags(vec![Tag::new("fem")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::GermanEin;
    let result = transform.execute(&value, None, "de").unwrap();
    assert_eq!(result, "eine Karte");
}

#[test]
fn german_der_missing_gender_error() {
    let phrase = Phrase::builder().text("Ding".to_string()).build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::GermanDer;
    let result = transform.execute(&value, None, "de");
    assert!(matches!(result, Err(EvalError::MissingTag { .. })));
}

#[test]
fn german_transform_aliases() {
    let registry = TransformRegistry::new();
    assert_eq!(registry.get("die", "de"), Some(TransformKind::GermanDer));
    assert_eq!(registry.get("das", "de"), Some(TransformKind::GermanDer));
    assert_eq!(registry.get("eine", "de"), Some(TransformKind::GermanEin));
}

// =============================================================================
// German Article Transform Integration Tests (Full Evaluation Path)
// =============================================================================

#[test]
fn german_der_in_template() {
    let source = r#"
        karte = :fem "Karte";
        charakter = :masc "Charakter";
        ereignis = :neut "Ereignis";
        the_card = "{@der karte}";
        the_char = "{@der charakter}";
        the_event = "{@das ereignis}";
    "#;

    let mut locale = Locale::builder().language("de").build();
    locale.load_translations_str("de", source).unwrap();

    assert_eq!(
        locale.get_phrase("the_card").unwrap().to_string(),
        "die Karte"
    );
    assert_eq!(
        locale.get_phrase("the_char").unwrap().to_string(),
        "der Charakter"
    );
    assert_eq!(
        locale.get_phrase("the_event").unwrap().to_string(),
        "das Ereignis"
    );
}

#[test]
fn german_der_with_case_context() {
    // Test: "Zerstöre {@der:acc karte}." - accusative case
    let source = r#"
        karte = :fem "Karte";
        charakter = :masc "Charakter";
        destroy_card = "Zerstöre {@der:acc karte}.";
        destroy_char = "Zerstöre {@der:acc charakter}.";
    "#;

    let mut locale = Locale::builder().language("de").build();
    locale.load_translations_str("de", source).unwrap();

    // Feminine accusative = "die", masculine accusative = "den"
    assert_eq!(
        locale.get_phrase("destroy_card").unwrap().to_string(),
        "Zerstöre die Karte."
    );
    assert_eq!(
        locale.get_phrase("destroy_char").unwrap().to_string(),
        "Zerstöre den Charakter."
    );
}

#[test]
fn german_ein_in_template() {
    let source = r#"
        karte = :fem "Karte";
        charakter = :masc "Charakter";
        a_card = "{@ein karte}";
        a_char = "{@ein:acc charakter}";
    "#;

    let mut locale = Locale::builder().language("de").build();
    locale.load_translations_str("de", source).unwrap();

    assert_eq!(
        locale.get_phrase("a_card").unwrap().to_string(),
        "eine Karte"
    );
    assert_eq!(
        locale.get_phrase("a_char").unwrap().to_string(),
        "einen Charakter"
    );
}

#[test]
fn german_dative_case() {
    // Test dative case for prepositions like "mit" (with)
    let source = r#"
        karte = :fem "Karte";
        charakter = :masc "Charakter";
        with_card = "mit {@der:dat karte}";
        with_char = "mit {@ein:dat charakter}";
    "#;

    let mut locale = Locale::builder().language("de").build();
    locale.load_translations_str("de", source).unwrap();

    // Feminine dative definite = "der", masculine dative indefinite = "einem"
    assert_eq!(
        locale.get_phrase("with_card").unwrap().to_string(),
        "mit der Karte"
    );
    assert_eq!(
        locale.get_phrase("with_char").unwrap().to_string(),
        "mit einem Charakter"
    );
}

#[test]
fn german_genitive_case() {
    // Test genitive case
    let source = r#"
        karte = :fem "Karte";
        charakter = :masc "Charakter";
        of_card = "{@der:gen karte}";
        of_char = "{@ein:gen charakter}";
    "#;

    let mut locale = Locale::builder().language("de").build();
    locale.load_translations_str("de", source).unwrap();

    // Feminine genitive definite = "der", masculine genitive indefinite = "eines"
    assert_eq!(
        locale.get_phrase("of_card").unwrap().to_string(),
        "der Karte"
    );
    assert_eq!(
        locale.get_phrase("of_char").unwrap().to_string(),
        "eines Charakter"
    );
}

#[test]
fn german_all_cases_masculine() {
    // Test all 4 cases for masculine noun
    let source = r#"
        charakter = :masc "Charakter";
        nom = "{@der charakter}";
        acc = "{@der:acc charakter}";
        dat = "{@der:dat charakter}";
        gen = "{@der:gen charakter}";
    "#;

    let mut locale = Locale::builder().language("de").build();
    locale.load_translations_str("de", source).unwrap();

    assert_eq!(
        locale.get_phrase("nom").unwrap().to_string(),
        "der Charakter"
    );
    assert_eq!(
        locale.get_phrase("acc").unwrap().to_string(),
        "den Charakter"
    );
    assert_eq!(
        locale.get_phrase("dat").unwrap().to_string(),
        "dem Charakter"
    );
    assert_eq!(
        locale.get_phrase("gen").unwrap().to_string(),
        "des Charakter"
    );
}

#[test]
fn german_all_cases_neuter() {
    // Test all 4 cases for neuter noun
    let source = r#"
        ereignis = :neut "Ereignis";
        nom = "{@der ereignis}";
        acc = "{@der:acc ereignis}";
        dat = "{@der:dat ereignis}";
        gen = "{@der:gen ereignis}";
    "#;

    let mut locale = Locale::builder().language("de").build();
    locale.load_translations_str("de", source).unwrap();

    assert_eq!(
        locale.get_phrase("nom").unwrap().to_string(),
        "das Ereignis"
    );
    assert_eq!(
        locale.get_phrase("acc").unwrap().to_string(),
        "das Ereignis"
    );
    assert_eq!(
        locale.get_phrase("dat").unwrap().to_string(),
        "dem Ereignis"
    );
    assert_eq!(
        locale.get_phrase("gen").unwrap().to_string(),
        "des Ereignis"
    );
}

// =============================================================================
// Dutch Article Transforms (@de/@het, @een)
// =============================================================================

#[test]
fn dutch_de_with_de_tag() {
    // Phrase with :de tag (common gender) produces "de kaart"
    let phrase = Phrase::builder()
        .text("kaart".to_string())
        .tags(vec![Tag::new("de")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::DutchDe;
    let result = transform.execute(&value, None, "nl").unwrap();
    assert_eq!(result, "de kaart");
}

#[test]
fn dutch_de_with_het_tag() {
    // Phrase with :het tag (neuter gender) produces "het karakter"
    let phrase = Phrase::builder()
        .text("karakter".to_string())
        .tags(vec![Tag::new("het")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::DutchDe;
    let result = transform.execute(&value, None, "nl").unwrap();
    assert_eq!(result, "het karakter");
}

#[test]
fn dutch_de_missing_tag_error() {
    // Phrase without :de or :het tag produces error
    let phrase = Phrase::builder().text("ding".to_string()).build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::DutchDe;
    let result = transform.execute(&value, None, "nl");
    assert!(matches!(result, Err(EvalError::MissingTag { .. })));
}

#[test]
fn dutch_een_transform() {
    // @een always produces "een X" regardless of gender
    let value = Value::String("kaart".to_string());
    let transform = TransformKind::DutchEen;
    let result = transform.execute(&value, None, "nl").unwrap();
    assert_eq!(result, "een kaart");
}

#[test]
fn dutch_een_with_phrase() {
    // @een works with Phrase values too
    let phrase = Phrase::builder()
        .text("karakter".to_string())
        .tags(vec![Tag::new("het")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::DutchEen;
    let result = transform.execute(&value, None, "nl").unwrap();
    assert_eq!(result, "een karakter");
}

#[test]
fn dutch_transform_alias_het() {
    // @het resolves to DutchDe
    let registry = TransformRegistry::new();
    let transform = registry.get("het", "nl");
    assert_eq!(transform, Some(TransformKind::DutchDe));
}

#[test]
fn dutch_transform_de_lookup() {
    // @de resolves to DutchDe
    let registry = TransformRegistry::new();
    let transform = registry.get("de", "nl");
    assert_eq!(transform, Some(TransformKind::DutchDe));
}

#[test]
fn dutch_transform_een_lookup() {
    // @een resolves to DutchEen
    let registry = TransformRegistry::new();
    let transform = registry.get("een", "nl");
    assert_eq!(transform, Some(TransformKind::DutchEen));
}

#[test]
fn dutch_transform_not_available_for_other_languages() {
    // Dutch transforms should not be available for other languages
    let registry = TransformRegistry::new();
    assert_eq!(registry.get("de", "de"), None); // "de" is Dutch, not German
    assert_eq!(registry.get("het", "en"), None);
    assert_eq!(registry.get("een", "de"), None);
}

// =============================================================================
// Dutch Article Transform Integration Tests (Full Evaluation Path)
// =============================================================================

#[test]
fn dutch_de_in_template() {
    let source = r#"
        kaart = :de "kaart";
        karakter = :het "karakter";
        the_card = "{@de kaart}";
        the_char = "{@het karakter}";
    "#;

    let mut locale = Locale::builder().language("nl").build();
    locale.load_translations_str("nl", source).unwrap();

    assert_eq!(
        locale.get_phrase("the_card").unwrap().to_string(),
        "de kaart"
    );
    assert_eq!(
        locale.get_phrase("the_char").unwrap().to_string(),
        "het karakter"
    );
}

#[test]
fn dutch_een_in_template() {
    let source = r#"
        kaart = :de "kaart";
        karakter = :het "karakter";
        a_card = "{@een kaart}";
        a_char = "{@een karakter}";
    "#;

    let mut locale = Locale::builder().language("nl").build();
    locale.load_translations_str("nl", source).unwrap();

    // Indefinite is always "een" regardless of gender
    assert_eq!(
        locale.get_phrase("a_card").unwrap().to_string(),
        "een kaart"
    );
    assert_eq!(
        locale.get_phrase("a_char").unwrap().to_string(),
        "een karakter"
    );
}

#[test]
fn dutch_de_with_cap() {
    // Test transform ordering: {@cap @de kaart} -> "De kaart"
    let source = r#"
        kaart = :de "kaart";
        the_card = "{@cap @de kaart}";
    "#;

    let mut locale = Locale::builder().language("nl").build();
    locale.load_translations_str("nl", source).unwrap();

    // Right-to-left: @de first ("de kaart"), then @cap ("De kaart")
    assert_eq!(
        locale.get_phrase("the_card").unwrap().to_string(),
        "De kaart"
    );
}

#[test]
fn dutch_de_missing_tag_full_eval() {
    // Test error when tag missing in full evaluation
    let source = r#"
        ding = "ding";
        the_thing = "{@de ding}";
    "#;

    let mut locale = Locale::builder().language("nl").build();
    locale.load_translations_str("nl", source).unwrap();

    let result = locale.get_phrase("the_thing");
    assert!(matches!(result, Err(EvalError::MissingTag { .. })));
}

#[test]
fn dutch_het_alias_in_template() {
    // Test: @het alias works in template evaluation
    let source = r#"
        karakter = :het "karakter";
        the_char = "{@het karakter}";
    "#;

    let mut locale = Locale::builder().language("nl").build();
    locale.load_translations_str("nl", source).unwrap();

    assert_eq!(
        locale.get_phrase("the_char").unwrap().to_string(),
        "het karakter"
    );
}

#[test]
fn dutch_upper_de_card() {
    // Test {@upper @de kaart} -> "DE KAART"
    let source = r#"
        kaart = :de "kaart";
        shouted = "{@upper @de kaart}";
    "#;

    let mut locale = Locale::builder().language("nl").build();
    locale.load_translations_str("nl", source).unwrap();

    // Right-to-left: @de first ("de kaart"), then @upper ("DE KAART")
    assert_eq!(
        locale.get_phrase("shouted").unwrap().to_string(),
        "DE KAART"
    );
}
