//! Integration tests for transform execution in the interpreter.

use rlf::interpreter::{EvalError, Locale, TransformKind, TransformRegistry};
use rlf::{Phrase, PhraseRegistry, Tag, Value, VariantKey};
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

// =============================================================================
// Cross-Language Verification (Phase 6 Complete)
// =============================================================================

#[test]
fn transforms_are_language_scoped() {
    // @a only works for English, not German or Dutch
    let registry = TransformRegistry::new();

    assert!(registry.get("a", "en").is_some());
    assert!(registry.get("a", "de").is_none()); // German has @der/@ein, not @a
    assert!(registry.get("a", "nl").is_none()); // Dutch has @de/@een, not @a

    assert!(registry.get("der", "de").is_some());
    assert!(registry.get("der", "en").is_none());

    assert!(registry.get("de", "nl").is_some());
    assert!(registry.get("de", "de").is_none()); // "de" is Dutch, not German
}

#[test]
fn universal_transforms_work_in_all_languages() {
    let registry = TransformRegistry::new();

    // @cap, @upper, @lower should work for all languages
    for lang in &["en", "de", "nl", "es", "fr"] {
        assert!(
            registry.get("cap", lang).is_some(),
            "cap should work for {}",
            lang
        );
        assert!(
            registry.get("upper", lang).is_some(),
            "upper should work for {}",
            lang
        );
        assert!(
            registry.get("lower", lang).is_some(),
            "lower should work for {}",
            lang
        );
    }
}

#[test]
fn unknown_transform_returns_none() {
    let registry = TransformRegistry::new();

    // @foo doesn't exist in any language
    assert!(registry.get("foo", "en").is_none());
    assert!(registry.get("foo", "de").is_none());
    assert!(registry.get("foo", "nl").is_none());
}

// =============================================================================
// Spanish Transform Tests (Phase 7)
// =============================================================================

#[test]
fn spanish_el_masculine_singular() {
    let phrase = Phrase::builder()
        .text("enemigo".to_string())
        .tags(vec![Tag::new("masc")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::SpanishEl;
    let result = transform.execute(&value, None, "es").unwrap();
    assert_eq!(result, "el enemigo");
}

#[test]
fn spanish_el_feminine_singular() {
    let phrase = Phrase::builder()
        .text("carta".to_string())
        .tags(vec![Tag::new("fem")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::SpanishEl;
    let result = transform.execute(&value, None, "es").unwrap();
    assert_eq!(result, "la carta");
}

#[test]
fn spanish_el_masculine_plural() {
    let phrase = Phrase::builder()
        .text("enemigos".to_string())
        .tags(vec![Tag::new("masc")])
        .build();
    let value = Value::Phrase(phrase);
    let context = Value::String("other".to_string());
    let transform = TransformKind::SpanishEl;
    let result = transform.execute(&value, Some(&context), "es").unwrap();
    assert_eq!(result, "los enemigos");
}

#[test]
fn spanish_el_feminine_plural() {
    let phrase = Phrase::builder()
        .text("cartas".to_string())
        .tags(vec![Tag::new("fem")])
        .build();
    let value = Value::Phrase(phrase);
    let context = Value::String("other".to_string());
    let transform = TransformKind::SpanishEl;
    let result = transform.execute(&value, Some(&context), "es").unwrap();
    assert_eq!(result, "las cartas");
}

#[test]
fn spanish_un_masculine_singular() {
    let phrase = Phrase::builder()
        .text("enemigo".to_string())
        .tags(vec![Tag::new("masc")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::SpanishUn;
    let result = transform.execute(&value, None, "es").unwrap();
    assert_eq!(result, "un enemigo");
}

#[test]
fn spanish_un_feminine_singular() {
    let phrase = Phrase::builder()
        .text("carta".to_string())
        .tags(vec![Tag::new("fem")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::SpanishUn;
    let result = transform.execute(&value, None, "es").unwrap();
    assert_eq!(result, "una carta");
}

#[test]
fn spanish_un_masculine_plural() {
    let phrase = Phrase::builder()
        .text("enemigos".to_string())
        .tags(vec![Tag::new("masc")])
        .build();
    let value = Value::Phrase(phrase);
    let context = Value::String("other".to_string());
    let transform = TransformKind::SpanishUn;
    let result = transform.execute(&value, Some(&context), "es").unwrap();
    assert_eq!(result, "unos enemigos");
}

#[test]
fn spanish_el_missing_gender() {
    let phrase = Phrase::builder().text("cosa".to_string()).build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::SpanishEl;
    let result = transform.execute(&value, None, "es");
    assert!(matches!(result, Err(EvalError::MissingTag { .. })));
}

#[test]
fn spanish_transform_aliases() {
    let registry = TransformRegistry::new();
    assert_eq!(registry.get("la", "es"), Some(TransformKind::SpanishEl));
    assert_eq!(registry.get("una", "es"), Some(TransformKind::SpanishUn));
}

#[test]
fn spanish_el_numeric_context() {
    // Numeric context uses plural rules: 1=one, else=other
    let phrase = Phrase::builder()
        .text("carta".to_string())
        .tags(vec![Tag::new("fem")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::SpanishEl;

    let ctx_one = Value::Number(1);
    let result_one = transform.execute(&value, Some(&ctx_one), "es").unwrap();
    assert_eq!(result_one, "la carta");

    let ctx_three = Value::Number(3);
    let result_three = transform.execute(&value, Some(&ctx_three), "es").unwrap();
    assert_eq!(result_three, "las carta");
}

// =============================================================================
// Portuguese Transform Tests (Phase 7)
// =============================================================================

#[test]
fn portuguese_o_masculine_singular() {
    let phrase = Phrase::builder()
        .text("inimigo".to_string())
        .tags(vec![Tag::new("masc")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::PortugueseO;
    let result = transform.execute(&value, None, "pt").unwrap();
    assert_eq!(result, "o inimigo");
}

#[test]
fn portuguese_o_feminine_singular() {
    let phrase = Phrase::builder()
        .text("carta".to_string())
        .tags(vec![Tag::new("fem")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::PortugueseO;
    let result = transform.execute(&value, None, "pt").unwrap();
    assert_eq!(result, "a carta");
}

#[test]
fn portuguese_o_masculine_plural() {
    let phrase = Phrase::builder()
        .text("inimigos".to_string())
        .tags(vec![Tag::new("masc")])
        .build();
    let value = Value::Phrase(phrase);
    let context = Value::String("other".to_string());
    let transform = TransformKind::PortugueseO;
    let result = transform.execute(&value, Some(&context), "pt").unwrap();
    assert_eq!(result, "os inimigos");
}

#[test]
fn portuguese_um_masculine() {
    let phrase = Phrase::builder()
        .text("inimigo".to_string())
        .tags(vec![Tag::new("masc")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::PortugueseUm;
    let result = transform.execute(&value, None, "pt").unwrap();
    assert_eq!(result, "um inimigo");
}

#[test]
fn portuguese_um_feminine() {
    let phrase = Phrase::builder()
        .text("carta".to_string())
        .tags(vec![Tag::new("fem")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::PortugueseUm;
    let result = transform.execute(&value, None, "pt").unwrap();
    assert_eq!(result, "uma carta");
}

#[test]
fn portuguese_de_contraction_masculine() {
    let phrase = Phrase::builder()
        .text("vazio".to_string())
        .tags(vec![Tag::new("masc")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::PortugueseDe;
    let result = transform.execute(&value, None, "pt").unwrap();
    assert_eq!(result, "do vazio"); // de + o = do
}

#[test]
fn portuguese_de_contraction_feminine() {
    let phrase = Phrase::builder()
        .text("carta".to_string())
        .tags(vec![Tag::new("fem")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::PortugueseDe;
    let result = transform.execute(&value, None, "pt").unwrap();
    assert_eq!(result, "da carta"); // de + a = da
}

#[test]
fn portuguese_de_contraction_plural() {
    let phrase = Phrase::builder()
        .text("cartas".to_string())
        .tags(vec![Tag::new("fem")])
        .build();
    let value = Value::Phrase(phrase);
    let context = Value::String("other".to_string());
    let transform = TransformKind::PortugueseDe;
    let result = transform.execute(&value, Some(&context), "pt").unwrap();
    assert_eq!(result, "das cartas"); // de + as = das
}

#[test]
fn portuguese_em_contraction_masculine() {
    let phrase = Phrase::builder()
        .text("vazio".to_string())
        .tags(vec![Tag::new("masc")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::PortugueseEm;
    let result = transform.execute(&value, None, "pt").unwrap();
    assert_eq!(result, "no vazio"); // em + o = no
}

#[test]
fn portuguese_em_contraction_feminine() {
    let phrase = Phrase::builder()
        .text("mao".to_string())
        .tags(vec![Tag::new("fem")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::PortugueseEm;
    let result = transform.execute(&value, None, "pt").unwrap();
    assert_eq!(result, "na mao"); // em + a = na
}

#[test]
fn portuguese_transform_aliases() {
    let registry = TransformRegistry::new();
    assert_eq!(registry.get("a", "pt"), Some(TransformKind::PortugueseO));
    assert_eq!(registry.get("uma", "pt"), Some(TransformKind::PortugueseUm));
}

#[test]
fn portuguese_o_missing_gender() {
    let phrase = Phrase::builder().text("coisa".to_string()).build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::PortugueseO;
    let result = transform.execute(&value, None, "pt");
    assert!(matches!(result, Err(EvalError::MissingTag { .. })));
}

// =============================================================================
// Spanish/Portuguese Integration Tests (Phase 7)
// =============================================================================

#[test]
fn spanish_el_in_template() {
    let source = r#"
        carta = :fem "carta";
        enemigo = :masc "enemigo";
        the_card = "{@el carta}";
        the_enemy = "{@la enemigo}";
    "#;

    let mut locale = Locale::builder().language("es").build();
    locale.load_translations_str("es", source).unwrap();

    assert_eq!(
        locale.get_phrase("the_card").unwrap().to_string(),
        "la carta"
    );
    // Note: @la alias resolves to @el, then looks up fem tag
    assert_eq!(
        locale.get_phrase("the_enemy").unwrap().to_string(),
        "el enemigo"
    );
}

#[test]
fn spanish_el_with_plural_context() {
    // Test: @el:other uses context to select plural article form
    let source = r#"
        carta = :fem "carta";
        cartas = :fem "cartas";
        return_all = "devuelve {@el:other cartas}";
    "#;

    let mut locale = Locale::builder().language("es").build();
    locale.load_translations_str("es", source).unwrap();

    // @el:other uses "other" context for plural article (las)
    let result = locale.get_phrase("return_all").unwrap();
    assert_eq!(result.to_string(), "devuelve las cartas");
}

#[test]
fn spanish_un_in_template() {
    let source = r#"
        carta = :fem "carta";
        enemigo = :masc "enemigo";
        draw_one = "Roba {@un carta}.";
        draw_enemy = "Roba {@una enemigo}.";
    "#;

    let mut locale = Locale::builder().language("es").build();
    locale.load_translations_str("es", source).unwrap();

    assert_eq!(
        locale.get_phrase("draw_one").unwrap().to_string(),
        "Roba una carta."
    );
    // @una resolves to @un
    assert_eq!(
        locale.get_phrase("draw_enemy").unwrap().to_string(),
        "Roba un enemigo."
    );
}

#[test]
fn portuguese_o_in_template() {
    let source = r#"
        carta = :fem "carta";
        inimigo = :masc "inimigo";
        the_card = "{@o carta}";
        the_enemy = "{@a inimigo}";
    "#;

    let mut locale = Locale::builder().language("pt").build();
    locale.load_translations_str("pt", source).unwrap();

    assert_eq!(
        locale.get_phrase("the_card").unwrap().to_string(),
        "a carta"
    );
    // @a alias resolves to @o
    assert_eq!(
        locale.get_phrase("the_enemy").unwrap().to_string(),
        "o inimigo"
    );
}

#[test]
fn portuguese_contractions_in_template() {
    let source = r#"
        vazio = :masc "vazio";
        mao = :fem "mao";
        from_void = "{@de vazio}";
        in_hand = "{@em mao}";
    "#;

    let mut locale = Locale::builder().language("pt").build();
    locale.load_translations_str("pt", source).unwrap();

    assert_eq!(
        locale.get_phrase("from_void").unwrap().to_string(),
        "do vazio" // de + o = do
    );
    assert_eq!(
        locale.get_phrase("in_hand").unwrap().to_string(),
        "na mao" // em + a = na
    );
}

#[test]
fn spanish_portuguese_cross_language() {
    // Verify same phrase structure works in both languages
    let es_source = r#"
        carta = :fem "carta";
        draw_card = "Roba {@un carta}.";
    "#;

    let pt_source = r#"
        carta = :fem "carta";
        draw_card = "Compre {@um carta}.";
    "#;

    // Spanish locale
    let mut es_locale = Locale::builder().language("es").build();
    es_locale.load_translations_str("es", es_source).unwrap();

    // Portuguese locale
    let mut pt_locale = Locale::builder().language("pt").build();
    pt_locale.load_translations_str("pt", pt_source).unwrap();

    assert_eq!(
        es_locale.get_phrase("draw_card").unwrap().to_string(),
        "Roba una carta."
    );
    assert_eq!(
        pt_locale.get_phrase("draw_card").unwrap().to_string(),
        "Compre uma carta."
    );
}

#[test]
fn all_phase6_transforms_work() {
    // English
    let en_source = r#"
        card = :a "card";
        event = :an "event";
        test = "Draw {@a card}, play {@an event}, get {@the card}.";
    "#;

    // German
    let de_source = r#"
        karte = :fem "Karte";
        test = "Nimm {@der:acc karte}, benutze {@ein karte}.";
    "#;

    // Dutch
    let nl_source = r#"
        kaart = :de "kaart";
        karakter = :het "karakter";
        test = "Pak {@de kaart}, krijg {@een karakter}.";
    "#;

    let mut en_locale = Locale::builder().language("en").build();
    en_locale.load_translations_str("en", en_source).unwrap();

    let mut de_locale = Locale::builder().language("de").build();
    de_locale.load_translations_str("de", de_source).unwrap();

    let mut nl_locale = Locale::builder().language("nl").build();
    nl_locale.load_translations_str("nl", nl_source).unwrap();

    assert_eq!(
        en_locale.get_phrase("test").unwrap().to_string(),
        "Draw a card, play an event, get the card."
    );
    assert_eq!(
        de_locale.get_phrase("test").unwrap().to_string(),
        "Nimm die Karte, benutze eine Karte."
    );
    assert_eq!(
        nl_locale.get_phrase("test").unwrap().to_string(),
        "Pak de kaart, krijg een karakter."
    );
}

// =============================================================================
// French Transform Tests (Phase 7)
// =============================================================================

#[test]
fn french_le_masculine_no_elision() {
    let phrase = Phrase::builder()
        .text("livre".to_string())
        .tags(vec![Tag::new("masc")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::FrenchLe;
    let result = transform.execute(&value, None, "fr").unwrap();
    assert_eq!(result, "le livre");
}

#[test]
fn french_le_feminine_no_elision() {
    let phrase = Phrase::builder()
        .text("carte".to_string())
        .tags(vec![Tag::new("fem")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::FrenchLe;
    let result = transform.execute(&value, None, "fr").unwrap();
    assert_eq!(result, "la carte");
}

#[test]
fn french_le_elision_masculine() {
    let phrase = Phrase::builder()
        .text("ennemi".to_string())
        .tags(vec![Tag::new("masc"), Tag::new("vowel")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::FrenchLe;
    let result = transform.execute(&value, None, "fr").unwrap();
    assert_eq!(result, "l'ennemi"); // No space after apostrophe
}

#[test]
fn french_le_elision_feminine() {
    let phrase = Phrase::builder()
        .text("amie".to_string())
        .tags(vec![Tag::new("fem"), Tag::new("vowel")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::FrenchLe;
    let result = transform.execute(&value, None, "fr").unwrap();
    assert_eq!(result, "l'amie");
}

#[test]
fn french_le_plural_no_elision() {
    // Plural never elides
    let phrase = Phrase::builder()
        .text("ennemis".to_string())
        .tags(vec![Tag::new("masc"), Tag::new("vowel")])
        .build();
    let value = Value::Phrase(phrase);
    let context = Value::String("other".to_string());
    let transform = TransformKind::FrenchLe;
    let result = transform.execute(&value, Some(&context), "fr").unwrap();
    assert_eq!(result, "les ennemis"); // No elision in plural
}

#[test]
fn french_un_masculine() {
    let phrase = Phrase::builder()
        .text("livre".to_string())
        .tags(vec![Tag::new("masc")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::FrenchUn;
    let result = transform.execute(&value, None, "fr").unwrap();
    assert_eq!(result, "un livre");
}

#[test]
fn french_un_feminine() {
    let phrase = Phrase::builder()
        .text("carte".to_string())
        .tags(vec![Tag::new("fem")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::FrenchUn;
    let result = transform.execute(&value, None, "fr").unwrap();
    assert_eq!(result, "une carte");
}

#[test]
fn french_de_contraction_masculine() {
    let phrase = Phrase::builder()
        .text("vide".to_string())
        .tags(vec![Tag::new("masc")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::FrenchDe;
    let result = transform.execute(&value, None, "fr").unwrap();
    assert_eq!(result, "du vide"); // de + le = du
}

#[test]
fn french_de_contraction_feminine() {
    let phrase = Phrase::builder()
        .text("main".to_string())
        .tags(vec![Tag::new("fem")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::FrenchDe;
    let result = transform.execute(&value, None, "fr").unwrap();
    assert_eq!(result, "de la main"); // No contraction
}

#[test]
fn french_de_elision() {
    let phrase = Phrase::builder()
        .text("ennemi".to_string())
        .tags(vec![Tag::new("masc"), Tag::new("vowel")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::FrenchDe;
    let result = transform.execute(&value, None, "fr").unwrap();
    assert_eq!(result, "de l'ennemi"); // de + l' (elided)
}

#[test]
fn french_de_plural() {
    let phrase = Phrase::builder()
        .text("cartes".to_string())
        .tags(vec![Tag::new("fem")])
        .build();
    let value = Value::Phrase(phrase);
    let context = Value::String("other".to_string());
    let transform = TransformKind::FrenchDe;
    let result = transform.execute(&value, Some(&context), "fr").unwrap();
    assert_eq!(result, "des cartes"); // de + les = des
}

#[test]
fn french_au_contraction_masculine() {
    let phrase = Phrase::builder()
        .text("marche".to_string())
        .tags(vec![Tag::new("masc")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::FrenchAu;
    let result = transform.execute(&value, None, "fr").unwrap();
    assert_eq!(result, "au marche"); // a + le = au
}

#[test]
fn french_au_contraction_feminine() {
    let phrase = Phrase::builder()
        .text("main".to_string())
        .tags(vec![Tag::new("fem")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::FrenchAu;
    let result = transform.execute(&value, None, "fr").unwrap();
    assert_eq!(result, "a la main"); // No contraction
}

#[test]
fn french_au_elision() {
    let phrase = Phrase::builder()
        .text("ami".to_string())
        .tags(vec![Tag::new("masc"), Tag::new("vowel")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::FrenchAu;
    let result = transform.execute(&value, None, "fr").unwrap();
    assert_eq!(result, "a l'ami"); // a + l' (elided)
}

#[test]
fn french_au_plural() {
    let phrase = Phrase::builder()
        .text("marches".to_string())
        .tags(vec![Tag::new("masc")])
        .build();
    let value = Value::Phrase(phrase);
    let context = Value::String("other".to_string());
    let transform = TransformKind::FrenchAu;
    let result = transform.execute(&value, Some(&context), "fr").unwrap();
    assert_eq!(result, "aux marches"); // a + les = aux
}

#[test]
fn french_le_missing_gender() {
    let phrase = Phrase::builder().text("chose".to_string()).build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::FrenchLe;
    let result = transform.execute(&value, None, "fr");
    assert!(matches!(result, Err(EvalError::MissingTag { .. })));
}

#[test]
fn french_transform_aliases() {
    let registry = TransformRegistry::new();
    assert_eq!(registry.get("la", "fr"), Some(TransformKind::FrenchLe));
    assert_eq!(registry.get("une", "fr"), Some(TransformKind::FrenchUn));
}

// =============================================================================
// French Contraction Lowercase Tests (Phase 7)
// Per locked decision: "Capitalization handled via separate @cap transform
// (contractions always lowercase)"
// =============================================================================

#[test]
fn french_de_contraction_preserves_lowercase() {
    // Contraction output is always lowercase regardless of input text case
    let phrase = Phrase::builder()
        .text("Vide".to_string()) // Input starts with capital
        .tags(vec![Tag::new("masc")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::FrenchDe;
    let result = transform.execute(&value, None, "fr").unwrap();
    // Contraction "du" must be lowercase, input text preserved as-is
    assert_eq!(result, "du Vide");
    assert!(result.starts_with("du"), "Contraction must be lowercase");
}

#[test]
fn french_au_contraction_preserves_lowercase() {
    let phrase = Phrase::builder()
        .text("Marche".to_string()) // Input starts with capital
        .tags(vec![Tag::new("masc")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::FrenchAu;
    let result = transform.execute(&value, None, "fr").unwrap();
    // Contraction "au" must be lowercase
    assert_eq!(result, "au Marche");
    assert!(result.starts_with("au"), "Contraction must be lowercase");
}

#[test]
fn french_de_elision_preserves_lowercase() {
    let phrase = Phrase::builder()
        .text("Ennemi".to_string()) // Input starts with capital
        .tags(vec![Tag::new("masc"), Tag::new("vowel")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::FrenchDe;
    let result = transform.execute(&value, None, "fr").unwrap();
    // Elided form "de l'" must be lowercase
    assert_eq!(result, "de l'Ennemi");
    assert!(
        result.starts_with("de l'"),
        "Elided contraction must be lowercase"
    );
}

#[test]
fn french_le_article_preserves_lowercase() {
    let phrase = Phrase::builder()
        .text("Livre".to_string()) // Input starts with capital
        .tags(vec![Tag::new("masc")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::FrenchLe;
    let result = transform.execute(&value, None, "fr").unwrap();
    // Article "le" must be lowercase
    assert_eq!(result, "le Livre");
    assert!(result.starts_with("le"), "Article must be lowercase");
}

// =============================================================================
// Italian Transform Tests (Phase 7)
// =============================================================================

#[test]
fn italian_il_masculine_normal() {
    let phrase = Phrase::builder()
        .text("libro".to_string())
        .tags(vec![Tag::new("masc")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::ItalianIl;
    let result = transform.execute(&value, None, "it").unwrap();
    assert_eq!(result, "il libro");
}

#[test]
fn italian_il_masculine_vowel() {
    let phrase = Phrase::builder()
        .text("amico".to_string())
        .tags(vec![Tag::new("masc"), Tag::new("vowel")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::ItalianIl;
    let result = transform.execute(&value, None, "it").unwrap();
    assert_eq!(result, "l'amico"); // Elision
}

#[test]
fn italian_il_masculine_s_impura() {
    let phrase = Phrase::builder()
        .text("studente".to_string())
        .tags(vec![Tag::new("masc"), Tag::new("s_imp")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::ItalianIl;
    let result = transform.execute(&value, None, "it").unwrap();
    assert_eq!(result, "lo studente");
}

#[test]
fn italian_il_feminine_normal() {
    let phrase = Phrase::builder()
        .text("carta".to_string())
        .tags(vec![Tag::new("fem")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::ItalianIl;
    let result = transform.execute(&value, None, "it").unwrap();
    assert_eq!(result, "la carta");
}

#[test]
fn italian_il_feminine_vowel() {
    let phrase = Phrase::builder()
        .text("amica".to_string())
        .tags(vec![Tag::new("fem"), Tag::new("vowel")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::ItalianIl;
    let result = transform.execute(&value, None, "it").unwrap();
    assert_eq!(result, "l'amica"); // Elision
}

#[test]
fn italian_il_plural_normal() {
    let phrase = Phrase::builder()
        .text("libri".to_string())
        .tags(vec![Tag::new("masc")])
        .build();
    let value = Value::Phrase(phrase);
    let context = Value::String("other".to_string());
    let transform = TransformKind::ItalianIl;
    let result = transform.execute(&value, Some(&context), "it").unwrap();
    assert_eq!(result, "i libri");
}

#[test]
fn italian_il_plural_vowel() {
    let phrase = Phrase::builder()
        .text("amici".to_string())
        .tags(vec![Tag::new("masc"), Tag::new("vowel")])
        .build();
    let value = Value::Phrase(phrase);
    let context = Value::String("other".to_string());
    let transform = TransformKind::ItalianIl;
    let result = transform.execute(&value, Some(&context), "it").unwrap();
    assert_eq!(result, "gli amici");
}

#[test]
fn italian_il_plural_s_impura() {
    let phrase = Phrase::builder()
        .text("studenti".to_string())
        .tags(vec![Tag::new("masc"), Tag::new("s_imp")])
        .build();
    let value = Value::Phrase(phrase);
    let context = Value::String("other".to_string());
    let transform = TransformKind::ItalianIl;
    let result = transform.execute(&value, Some(&context), "it").unwrap();
    assert_eq!(result, "gli studenti");
}

#[test]
fn italian_un_masculine_normal() {
    let phrase = Phrase::builder()
        .text("libro".to_string())
        .tags(vec![Tag::new("masc")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::ItalianUn;
    let result = transform.execute(&value, None, "it").unwrap();
    assert_eq!(result, "un libro");
}

#[test]
fn italian_un_masculine_s_impura() {
    let phrase = Phrase::builder()
        .text("studente".to_string())
        .tags(vec![Tag::new("masc"), Tag::new("s_imp")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::ItalianUn;
    let result = transform.execute(&value, None, "it").unwrap();
    assert_eq!(result, "uno studente");
}

#[test]
fn italian_un_feminine_normal() {
    let phrase = Phrase::builder()
        .text("carta".to_string())
        .tags(vec![Tag::new("fem")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::ItalianUn;
    let result = transform.execute(&value, None, "it").unwrap();
    assert_eq!(result, "una carta");
}

#[test]
fn italian_un_feminine_vowel() {
    let phrase = Phrase::builder()
        .text("amica".to_string())
        .tags(vec![Tag::new("fem"), Tag::new("vowel")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::ItalianUn;
    let result = transform.execute(&value, None, "it").unwrap();
    assert_eq!(result, "un'amica"); // Feminine elision with apostrophe
}

#[test]
fn italian_di_contraction_normal() {
    let phrase = Phrase::builder()
        .text("libro".to_string())
        .tags(vec![Tag::new("masc")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::ItalianDi;
    let result = transform.execute(&value, None, "it").unwrap();
    assert_eq!(result, "del libro");
}

#[test]
fn italian_di_contraction_vowel() {
    let phrase = Phrase::builder()
        .text("amico".to_string())
        .tags(vec![Tag::new("masc"), Tag::new("vowel")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::ItalianDi;
    let result = transform.execute(&value, None, "it").unwrap();
    assert_eq!(result, "dell'amico");
}

#[test]
fn italian_di_contraction_s_impura() {
    let phrase = Phrase::builder()
        .text("studente".to_string())
        .tags(vec![Tag::new("masc"), Tag::new("s_imp")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::ItalianDi;
    let result = transform.execute(&value, None, "it").unwrap();
    assert_eq!(result, "dello studente");
}

#[test]
fn italian_a_contraction_normal() {
    let phrase = Phrase::builder()
        .text("libro".to_string())
        .tags(vec![Tag::new("masc")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::ItalianA;
    let result = transform.execute(&value, None, "it").unwrap();
    assert_eq!(result, "al libro");
}

#[test]
fn italian_a_contraction_vowel() {
    let phrase = Phrase::builder()
        .text("amico".to_string())
        .tags(vec![Tag::new("masc"), Tag::new("vowel")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::ItalianA;
    let result = transform.execute(&value, None, "it").unwrap();
    assert_eq!(result, "all'amico");
}

#[test]
fn italian_il_missing_gender() {
    let phrase = Phrase::builder().text("cosa".to_string()).build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::ItalianIl;
    let result = transform.execute(&value, None, "it");
    assert!(matches!(result, Err(EvalError::MissingTag { .. })));
}

#[test]
fn italian_transform_aliases() {
    let registry = TransformRegistry::new();
    assert_eq!(registry.get("lo", "it"), Some(TransformKind::ItalianIl));
    assert_eq!(registry.get("la", "it"), Some(TransformKind::ItalianIl));
    assert_eq!(registry.get("uno", "it"), Some(TransformKind::ItalianUn));
    assert_eq!(registry.get("una", "it"), Some(TransformKind::ItalianUn));
}

// =============================================================================
// Italian Contraction Lowercase Tests (Phase 7)
// Per locked decision: "Capitalization handled via separate @cap transform
// (contractions always lowercase)"
// =============================================================================

#[test]
fn italian_di_contraction_preserves_lowercase() {
    // Contraction output is always lowercase regardless of input text case
    let phrase = Phrase::builder()
        .text("Libro".to_string()) // Input starts with capital
        .tags(vec![Tag::new("masc")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::ItalianDi;
    let result = transform.execute(&value, None, "it").unwrap();
    // Contraction "del" must be lowercase, input text preserved as-is
    assert_eq!(result, "del Libro");
    assert!(result.starts_with("del"), "Contraction must be lowercase");
}

#[test]
fn italian_a_contraction_preserves_lowercase() {
    let phrase = Phrase::builder()
        .text("Libro".to_string()) // Input starts with capital
        .tags(vec![Tag::new("masc")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::ItalianA;
    let result = transform.execute(&value, None, "it").unwrap();
    // Contraction "al" must be lowercase
    assert_eq!(result, "al Libro");
    assert!(result.starts_with("al"), "Contraction must be lowercase");
}

#[test]
fn italian_di_elision_preserves_lowercase() {
    let phrase = Phrase::builder()
        .text("Amico".to_string()) // Input starts with capital
        .tags(vec![Tag::new("masc"), Tag::new("vowel")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::ItalianDi;
    let result = transform.execute(&value, None, "it").unwrap();
    // Elided contraction "dell'" must be lowercase
    assert_eq!(result, "dell'Amico");
    assert!(
        result.starts_with("dell'"),
        "Elided contraction must be lowercase"
    );
}

#[test]
fn italian_il_article_preserves_lowercase() {
    let phrase = Phrase::builder()
        .text("Libro".to_string()) // Input starts with capital
        .tags(vec![Tag::new("masc")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::ItalianIl;
    let result = transform.execute(&value, None, "it").unwrap();
    // Article "il" must be lowercase
    assert_eq!(result, "il Libro");
    assert!(result.starts_with("il"), "Article must be lowercase");
}

#[test]
fn italian_dello_contraction_preserves_lowercase() {
    let phrase = Phrase::builder()
        .text("Studente".to_string()) // Input starts with capital
        .tags(vec![Tag::new("masc"), Tag::new("s_imp")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::ItalianDi;
    let result = transform.execute(&value, None, "it").unwrap();
    // Contraction "dello" must be lowercase
    assert_eq!(result, "dello Studente");
    assert!(result.starts_with("dello"), "Contraction must be lowercase");
}

// =============================================================================
// French/Italian Integration Tests (Phase 7)
// =============================================================================

#[test]
fn french_le_in_template() {
    let source = r#"
        livre = :masc "livre";
        maison = :fem "maison";
        the_book = "{@le livre}";
        the_house = "{@la maison}";
    "#;

    let mut locale = Locale::builder().language("fr").build();
    locale.load_translations_str("fr", source).unwrap();

    assert_eq!(
        locale.get_phrase("the_book").unwrap().to_string(),
        "le livre"
    );
    // @la alias resolves to @le, then looks up fem tag
    assert_eq!(
        locale.get_phrase("the_house").unwrap().to_string(),
        "la maison"
    );
}

#[test]
fn french_le_with_elision_in_template() {
    let source = r#"
        ami = :masc :vowel "ami";
        ecole = :fem :vowel "ecole";
        the_friend = "{@le ami}";
        the_school = "{@la ecole}";
    "#;

    let mut locale = Locale::builder().language("fr").build();
    locale.load_translations_str("fr", source).unwrap();

    // Elision before vowels (singular only)
    assert_eq!(
        locale.get_phrase("the_friend").unwrap().to_string(),
        "l'ami"
    );
    assert_eq!(
        locale.get_phrase("the_school").unwrap().to_string(),
        "l'ecole"
    );
}

#[test]
fn french_le_plural_no_elision_in_template() {
    // Test: @le:other uses context for plural (les) which doesn't elide
    let source = r#"
        amis = :masc :vowel "amis";
        get_friends = "{@le:other amis}";
    "#;

    let mut locale = Locale::builder().language("fr").build();
    locale.load_translations_str("fr", source).unwrap();

    // Plural "les" does NOT elide, even before vowel
    assert_eq!(
        locale.get_phrase("get_friends").unwrap().to_string(),
        "les amis"
    );
}

#[test]
fn french_un_in_template() {
    let source = r#"
        livre = :masc "livre";
        maison = :fem "maison";
        a_book = "{@un livre}";
        a_house = "{@une maison}";
    "#;

    let mut locale = Locale::builder().language("fr").build();
    locale.load_translations_str("fr", source).unwrap();

    assert_eq!(locale.get_phrase("a_book").unwrap().to_string(), "un livre");
    // @une alias resolves to @un
    assert_eq!(
        locale.get_phrase("a_house").unwrap().to_string(),
        "une maison"
    );
}

// NOTE: French @un has no plural forms per APPENDIX_STDLIB.
// For plural indefinite, use natural language constructs.

#[test]
fn french_de_contractions_in_template() {
    let source = r#"
        livre = :masc "livre";
        ami = :masc :vowel "ami";
        maison = :fem "maison";
        livres = :masc "livres";

        of_book = "{@de livre}";
        of_friend = "{@de ami}";
        of_house = "{@de maison}";
        of_books = "{@de:other livres}";
    "#;

    let mut locale = Locale::builder().language("fr").build();
    locale.load_translations_str("fr", source).unwrap();

    // de + le = du
    assert_eq!(
        locale.get_phrase("of_book").unwrap().to_string(),
        "du livre"
    );
    // de + l' = de l'
    assert_eq!(
        locale.get_phrase("of_friend").unwrap().to_string(),
        "de l'ami"
    );
    // de + la = de la
    assert_eq!(
        locale.get_phrase("of_house").unwrap().to_string(),
        "de la maison"
    );
    // de + les = des
    assert_eq!(
        locale.get_phrase("of_books").unwrap().to_string(),
        "des livres"
    );
}

#[test]
fn french_au_contractions_in_template() {
    let source = r#"
        marche = :masc "marche";
        ami = :masc :vowel "ami";
        maison = :fem "maison";
        marches = :masc "marches";

        to_market = "{@au marche}";
        to_friend = "{@au ami}";
        to_house = "{@au maison}";
        to_markets = "{@au:other marches}";
    "#;

    let mut locale = Locale::builder().language("fr").build();
    locale.load_translations_str("fr", source).unwrap();

    // a + le = au
    assert_eq!(
        locale.get_phrase("to_market").unwrap().to_string(),
        "au marche"
    );
    // a + l' = a l'
    assert_eq!(
        locale.get_phrase("to_friend").unwrap().to_string(),
        "a l'ami"
    );
    // a + la = a la
    assert_eq!(
        locale.get_phrase("to_house").unwrap().to_string(),
        "a la maison"
    );
    // a + les = aux
    assert_eq!(
        locale.get_phrase("to_markets").unwrap().to_string(),
        "aux marches"
    );
}

#[test]
fn french_liaison_transform_direct() {
    // Test @liaison directly with Value - selects between standard and vowel variants
    // based on context's :vowel tag

    // Create adjective with liaison variants
    let mut variants = HashMap::new();
    variants.insert(VariantKey::new("standard"), "beau".to_string());
    variants.insert(VariantKey::new("vowel"), "bel".to_string());
    let beau = Phrase::builder()
        .text("beau".to_string())
        .tags(vec![Tag::new("masc")])
        .variants(variants)
        .build();
    let beau_value = Value::Phrase(beau);

    // Create vowel-starting noun (context)
    let ami = Phrase::builder()
        .text("ami".to_string())
        .tags(vec![Tag::new("masc"), Tag::new("vowel")])
        .build();
    let ami_value = Value::Phrase(ami);

    // Create consonant-starting noun (context)
    let livre = Phrase::builder()
        .text("livre".to_string())
        .tags(vec![Tag::new("masc")])
        .build();
    let livre_value = Value::Phrase(livre);

    let transform = TransformKind::FrenchLiaison;

    // Before vowel: use "bel"
    let result = transform
        .execute(&beau_value, Some(&ami_value), "fr")
        .unwrap();
    assert_eq!(result, "bel");

    // Before consonant: use "beau"
    let result = transform
        .execute(&beau_value, Some(&livre_value), "fr")
        .unwrap();
    assert_eq!(result, "beau");

    // No context: use default text
    let result = transform.execute(&beau_value, None, "fr").unwrap();
    assert_eq!(result, "beau");
}

#[test]
fn italian_il_in_template() {
    let source = r#"
        libro = :masc "libro";
        casa = :fem "casa";
        the_book = "{@il libro}";
        the_house = "{@la casa}";
    "#;

    let mut locale = Locale::builder().language("it").build();
    locale.load_translations_str("it", source).unwrap();

    assert_eq!(
        locale.get_phrase("the_book").unwrap().to_string(),
        "il libro"
    );
    // @la alias resolves to @il
    assert_eq!(
        locale.get_phrase("the_house").unwrap().to_string(),
        "la casa"
    );
}

#[test]
fn italian_il_sound_variants_in_template() {
    let source = r#"
        libro = :masc "libro";
        amico = :masc :vowel "amico";
        studente = :masc :s_imp "studente";

        the_book = "{@il libro}";
        the_friend = "{@il amico}";
        the_student = "{@il studente}";
    "#;

    let mut locale = Locale::builder().language("it").build();
    locale.load_translations_str("it", source).unwrap();

    // Normal: il
    assert_eq!(
        locale.get_phrase("the_book").unwrap().to_string(),
        "il libro"
    );
    // Vowel: l'
    assert_eq!(
        locale.get_phrase("the_friend").unwrap().to_string(),
        "l'amico"
    );
    // S-impura: lo
    assert_eq!(
        locale.get_phrase("the_student").unwrap().to_string(),
        "lo studente"
    );
}

#[test]
fn italian_un_in_template() {
    let source = r#"
        libro = :masc "libro";
        amico = :masc :vowel "amico";
        casa = :fem "casa";
        studente = :masc :s_imp "studente";

        a_book = "{@un libro}";
        a_friend = "{@un amico}";
        a_house = "{@una casa}";
        a_student = "{@uno studente}";
    "#;

    let mut locale = Locale::builder().language("it").build();
    locale.load_translations_str("it", source).unwrap();

    // Masculine normal: un
    assert_eq!(locale.get_phrase("a_book").unwrap().to_string(), "un libro");
    // Masculine vowel: un (same as normal)
    assert_eq!(
        locale.get_phrase("a_friend").unwrap().to_string(),
        "un amico"
    );
    // Feminine: una
    assert_eq!(
        locale.get_phrase("a_house").unwrap().to_string(),
        "una casa"
    );
    // S-impura: uno
    assert_eq!(
        locale.get_phrase("a_student").unwrap().to_string(),
        "uno studente"
    );
}

#[test]
fn italian_di_contractions_in_template() {
    let source = r#"
        libro = :masc "libro";
        amico = :masc :vowel "amico";
        casa = :fem "casa";
        studente = :masc :s_imp "studente";
        libri = :masc "libri";
        amici = :masc :vowel "amici";

        of_book = "{@di libro}";
        of_friend = "{@di amico}";
        of_house = "{@di casa}";
        of_student = "{@di studente}";
        of_books = "{@di:other libri}";
        of_friends = "{@di:other amici}";
    "#;

    let mut locale = Locale::builder().language("it").build();
    locale.load_translations_str("it", source).unwrap();

    // di + il = del
    assert_eq!(
        locale.get_phrase("of_book").unwrap().to_string(),
        "del libro"
    );
    // di + l' = dell'
    assert_eq!(
        locale.get_phrase("of_friend").unwrap().to_string(),
        "dell'amico"
    );
    // di + la = della
    assert_eq!(
        locale.get_phrase("of_house").unwrap().to_string(),
        "della casa"
    );
    // di + lo = dello
    assert_eq!(
        locale.get_phrase("of_student").unwrap().to_string(),
        "dello studente"
    );
    // di + i = dei
    assert_eq!(
        locale.get_phrase("of_books").unwrap().to_string(),
        "dei libri"
    );
    // di + gli = degli (plural vowel)
    assert_eq!(
        locale.get_phrase("of_friends").unwrap().to_string(),
        "degli amici"
    );
}

#[test]
fn italian_a_contractions_in_template() {
    let source = r#"
        mercato = :masc "mercato";
        amico = :masc :vowel "amico";
        casa = :fem "casa";
        stadio = :masc :s_imp "stadio";

        to_market = "{@a mercato}";
        to_friend = "{@a amico}";
        to_house = "{@a casa}";
        to_stadium = "{@a stadio}";
    "#;

    let mut locale = Locale::builder().language("it").build();
    locale.load_translations_str("it", source).unwrap();

    // a + il = al
    assert_eq!(
        locale.get_phrase("to_market").unwrap().to_string(),
        "al mercato"
    );
    // a + l' = all'
    assert_eq!(
        locale.get_phrase("to_friend").unwrap().to_string(),
        "all'amico"
    );
    // a + la = alla
    assert_eq!(
        locale.get_phrase("to_house").unwrap().to_string(),
        "alla casa"
    );
    // a + lo = allo
    assert_eq!(
        locale.get_phrase("to_stadium").unwrap().to_string(),
        "allo stadio"
    );
}

#[test]
fn french_italian_cross_language() {
    // Verify same phrase structure works in both French and Italian
    let fr_source = r#"
        livre = :masc "livre";
        draw_book = "Prends {@le livre}.";
    "#;

    let it_source = r#"
        libro = :masc "libro";
        draw_book = "Prendi {@il libro}.";
    "#;

    // French locale
    let mut fr_locale = Locale::builder().language("fr").build();
    fr_locale.load_translations_str("fr", fr_source).unwrap();

    // Italian locale
    let mut it_locale = Locale::builder().language("it").build();
    it_locale.load_translations_str("it", it_source).unwrap();

    assert_eq!(
        fr_locale.get_phrase("draw_book").unwrap().to_string(),
        "Prends le livre."
    );
    assert_eq!(
        it_locale.get_phrase("draw_book").unwrap().to_string(),
        "Prendi il libro."
    );
}

#[test]
fn all_phase7_transforms_work() {
    // Spanish
    let es_source = r#"
        carta = :fem "carta";
        test = "Roba {@el carta}, compra {@un carta}.";
    "#;

    // Portuguese
    let pt_source = r#"
        carta = :fem "carta";
        mao = :fem "mao";
        test = "Compre {@o carta}, {@de mao}.";
    "#;

    // French
    let fr_source = r#"
        livre = :masc "livre";
        ami = :masc :vowel "ami";
        test = "Prends {@le livre}, {@de ami}.";
    "#;

    // Italian
    let it_source = r#"
        libro = :masc "libro";
        amico = :masc :vowel "amico";
        test = "Prendi {@il libro}, {@di amico}.";
    "#;

    let mut es_locale = Locale::builder().language("es").build();
    es_locale.load_translations_str("es", es_source).unwrap();

    let mut pt_locale = Locale::builder().language("pt").build();
    pt_locale.load_translations_str("pt", pt_source).unwrap();

    let mut fr_locale = Locale::builder().language("fr").build();
    fr_locale.load_translations_str("fr", fr_source).unwrap();

    let mut it_locale = Locale::builder().language("it").build();
    it_locale.load_translations_str("it", it_source).unwrap();

    assert_eq!(
        es_locale.get_phrase("test").unwrap().to_string(),
        "Roba la carta, compra una carta."
    );
    assert_eq!(
        pt_locale.get_phrase("test").unwrap().to_string(),
        "Compre a carta, da mao."
    );
    assert_eq!(
        fr_locale.get_phrase("test").unwrap().to_string(),
        "Prends le livre, de l'ami."
    );
    assert_eq!(
        it_locale.get_phrase("test").unwrap().to_string(),
        "Prendi il libro, dell'amico."
    );
}
