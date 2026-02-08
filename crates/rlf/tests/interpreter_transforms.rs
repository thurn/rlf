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
        .load_phrases(r#"greeting($name) = "Hello, {@cap $name}!";"#)
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
        .load_phrases(r#"shout($text) = "{@upper $text}";"#)
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
        .load_phrases(r#"quiet($text) = "{@lower $text}";"#)
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
        .load_phrases(r#"empty_cap($text) = "{@cap $text}";"#)
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
        .load_phrases(r#"empty_upper($text) = "{@upper $text}";"#)
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
        .load_phrases(r#"empty_lower($text) = "{@lower $text}";"#)
        .unwrap();
    let result = registry
        .call_phrase("en", "empty_lower", &[Value::from("")])
        .unwrap();
    assert_eq!(result.to_string(), "");
}

// =============================================================================
// @cap with Markup Tags
// =============================================================================

#[test]
fn test_cap_skips_markup_tags() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"dissolve = "<color=#AA00FF><b>dissolve</b></color>";
            cap_dissolve = "{@cap dissolve}";"#,
        )
        .unwrap();
    let result = registry.get_phrase("en", "cap_dissolve").unwrap();
    assert_eq!(result.to_string(), "<color=#AA00FF><b>Dissolve</b></color>");
}

#[test]
fn test_cap_skips_single_markup_tag() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"bold_word = "<b>hello</b>";
            cap_bold = "{@cap bold_word}";"#,
        )
        .unwrap();
    let result = registry.get_phrase("en", "cap_bold").unwrap();
    assert_eq!(result.to_string(), "<b>Hello</b>");
}

#[test]
fn test_cap_no_markup_unchanged() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"plain = "hello";
            cap_plain = "{@cap plain}";"#,
        )
        .unwrap();
    let result = registry.get_phrase("en", "cap_plain").unwrap();
    assert_eq!(result.to_string(), "Hello");
}

#[test]
fn test_cap_markup_only() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"empty_markup = "<br/>";
            cap_empty_markup = "{@cap empty_markup}";"#,
        )
        .unwrap();
    let result = registry.get_phrase("en", "cap_empty_markup").unwrap();
    assert_eq!(result.to_string(), "<br/>");
}

// =============================================================================
// Unicode and Grapheme Handling
// =============================================================================

#[test]
fn test_cap_unicode_cyrillic() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(r#"cap_cyrillic($text) = "{@cap $text}";"#)
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
        .load_phrases(r#"cap_combining($text) = "{@cap $text}";"#)
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
        .load_phrases(r#"upper_greek($text) = "{@upper $text}";"#)
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
        .load_phrases(r#"lower_greek($text) = "{@lower $text}";"#)
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
        .load_phrases(r#"upper_tr($text) = "{@upper $text}";"#)
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
        .load_phrases(r#"lower_tr($text) = "{@lower $text}";"#)
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
        .load_phrases(r#"cap_tr($text) = "{@cap $text}";"#)
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
        .load_phrases(r#"upper_en($text) = "{@upper $text}";"#)
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
        .load_phrases(r#"chain($text) = "{@upper @cap $text}";"#)
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
        .load_phrases(r#"chain_cap_lower($text) = "{@cap @lower $text}";"#)
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
        .load_phrases(r#"chain_three($text) = "{@cap @lower @upper $text}";"#)
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
        .load_phrases(r#"unknown($text) = "{@nonexistent $text}";"#)
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
            display($n) = "The {@cap card:$n}.";
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
        .load_phrases(r#"sentence($name) = "Hello {@cap $name}, welcome to {@upper $name}!";"#)
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
        .eval_str("Hello {@cap $name}, see {@upper card}!", "en", params)
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
        .load_phrases(r#"cap_single($text) = "{@cap $text}";"#)
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
        .load_phrases(r#"upper_already($text) = "{@upper $text}";"#)
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
        .load_phrases(r#"lower_already($text) = "{@lower $text}";"#)
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
        .load_phrases(r#"cap_already($text) = "{@cap $text}";"#)
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
        .load_phrases(r#"mixed($text) = "{@upper $text}";"#)
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
        .load_phrases(r#"punct($text) = "{@cap $text}";"#)
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
        .load_phrases(r#"cap_ws($text) = "{@cap $text}";"#)
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
        .load_phrases(r#"upper_az($text) = "{@upper $text}";"#)
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
fn english_a_with_selector_preserves_tags() {
    // Tags should be preserved through variant selection so @a can still read them
    let source = r#"
        card = :a { one: "card", other: "cards" };
        draw_n($n) = "Draw {@a card:$n}.";
        draw_one = "Draw {@a card:one}.";
    "#;

    let mut locale = Locale::builder().language("en").build();
    locale.load_translations_str("en", source).unwrap();

    // :one selector → "card", @a reads :a tag → "a card"
    assert_eq!(
        locale.get_phrase("draw_one").unwrap().to_string(),
        "Draw a card."
    );

    // :n with n=1 → "card" (one), @a reads :a tag → "a card"
    assert_eq!(
        locale
            .call_phrase("draw_n", &[Value::from(1)])
            .unwrap()
            .to_string(),
        "Draw a card."
    );

    // :n with n=3 → "cards" (other), @a reads :a tag → "a cards"
    assert_eq!(
        locale
            .call_phrase("draw_n", &[Value::from(3)])
            .unwrap()
            .to_string(),
        "Draw a cards."
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
// English Plural Transform (@plural)
// =============================================================================

#[test]
fn english_plural_on_phrase_with_variants() {
    let phrase = Phrase::builder()
        .text("card".to_string())
        .variants(HashMap::from([
            (VariantKey::new("one"), "card".to_string()),
            (VariantKey::new("other"), "cards".to_string()),
        ]))
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::EnglishPlural;
    let result = transform.execute(&value, None, "en").unwrap();
    assert_eq!(result, "cards");
}

#[test]
fn english_plural_on_plain_string() {
    let value = Value::String("card".to_string());
    let transform = TransformKind::EnglishPlural;
    let result = transform.execute(&value, None, "en").unwrap();
    assert_eq!(result, "card");
}

#[test]
fn english_plural_registry_lookup() {
    let registry = TransformRegistry::new();
    let transform = registry.get("plural", "en");
    assert_eq!(transform, Some(TransformKind::EnglishPlural));
}

#[test]
fn english_plural_not_available_for_other_languages() {
    let registry = TransformRegistry::new();
    assert_eq!(registry.get("plural", "de"), None);
    assert_eq!(registry.get("plural", "fr"), None);
}

#[test]
fn english_plural_in_template() {
    let source = r#"
        card = { one: "card", other: "cards" };
        many_cards = "{@plural card}";
    "#;

    let mut locale = Locale::builder().language("en").build();
    locale.load_translations_str("en", source).unwrap();

    assert_eq!(
        locale.get_phrase("many_cards").unwrap().to_string(),
        "cards"
    );
}

#[test]
fn english_plural_with_phrase_call() {
    let source = r#"
        subtype = { one: "subtype", other: "subtypes" };
        label($s) = "{@plural subtype}";
    "#;

    let mut locale = Locale::builder().language("en").build();
    locale.load_translations_str("en", source).unwrap();

    assert_eq!(
        locale
            .call_phrase("label", &[Value::from("anything")])
            .unwrap()
            .to_string(),
        "subtypes"
    );
}

#[test]
fn english_plural_with_cap() {
    let source = r#"
        card = { one: "card", other: "cards" };
        many_cards_cap = "{@cap @plural card}";
    "#;

    let mut locale = Locale::builder().language("en").build();
    locale.load_translations_str("en", source).unwrap();

    // Right-to-left: @plural first ("cards"), then @cap ("Cards")
    assert_eq!(
        locale.get_phrase("many_cards_cap").unwrap().to_string(),
        "Cards"
    );
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
// German Plural Article Tests (@der with .other context)
// =============================================================================

#[test]
fn german_der_plural_nominative_masculine() {
    let phrase = Phrase::builder()
        .text("Charakter".to_string())
        .tags(vec![Tag::new("masc")])
        .build();
    let value = Value::Phrase(phrase);
    let context = Value::String("nom.other".to_string());
    let transform = TransformKind::GermanDer;
    let result = transform.execute(&value, Some(&context), "de").unwrap();
    assert_eq!(result, "die Charakter");
}

#[test]
fn german_der_plural_accusative_feminine() {
    let phrase = Phrase::builder()
        .text("Karte".to_string())
        .tags(vec![Tag::new("fem")])
        .build();
    let value = Value::Phrase(phrase);
    let context = Value::String("acc.other".to_string());
    let transform = TransformKind::GermanDer;
    let result = transform.execute(&value, Some(&context), "de").unwrap();
    assert_eq!(result, "die Karte");
}

#[test]
fn german_der_plural_dative_feminine() {
    let phrase = Phrase::builder()
        .text("Karte".to_string())
        .tags(vec![Tag::new("fem")])
        .build();
    let value = Value::Phrase(phrase);
    let context = Value::String("dat.other".to_string());
    let transform = TransformKind::GermanDer;
    let result = transform.execute(&value, Some(&context), "de").unwrap();
    assert_eq!(result, "den Karte");
}

#[test]
fn german_der_plural_genitive_neuter() {
    let phrase = Phrase::builder()
        .text("Ereignis".to_string())
        .tags(vec![Tag::new("neut")])
        .build();
    let value = Value::Phrase(phrase);
    let context = Value::String("gen.other".to_string());
    let transform = TransformKind::GermanDer;
    let result = transform.execute(&value, Some(&context), "de").unwrap();
    assert_eq!(result, "der Ereignis");
}

#[test]
fn german_der_plural_default_nominative() {
    // "other" without case prefix defaults to nominative plural
    let phrase = Phrase::builder()
        .text("Charakter".to_string())
        .tags(vec![Tag::new("masc")])
        .build();
    let value = Value::Phrase(phrase);
    let context = Value::String("other".to_string());
    let transform = TransformKind::GermanDer;
    let result = transform.execute(&value, Some(&context), "de").unwrap();
    assert_eq!(result, "die Charakter");
}

#[test]
fn german_der_singular_still_works_with_one() {
    // Explicit "nom.one" should produce the same as "nom" (singular)
    let phrase = Phrase::builder()
        .text("Charakter".to_string())
        .tags(vec![Tag::new("masc")])
        .build();
    let value = Value::Phrase(phrase);
    let context = Value::String("nom.one".to_string());
    let transform = TransformKind::GermanDer;
    let result = transform.execute(&value, Some(&context), "de").unwrap();
    assert_eq!(result, "der Charakter");
}

#[test]
fn german_der_plural_integration() {
    let source = r#"
        karte = :fem { one: "Karte", other: "Karten" };
        charakter = :masc { one: "Charakter", other: "Charaktere" };
        ereignis = :neut { one: "Ereignis", other: "Ereignisse" };
        the_cards = "{@der:nom.other karte}";
        destroy_chars = "Zerstöre {@der:acc.other charakter}.";
        with_events = "mit {@der:dat.other ereignis}";
        of_cards = "{@der:gen.other karte}";
    "#;

    let mut locale = Locale::builder().language("de").build();
    locale.load_translations_str("de", source).unwrap();

    assert_eq!(
        locale.get_phrase("the_cards").unwrap().to_string(),
        "die Karten"
    );
    assert_eq!(
        locale.get_phrase("destroy_chars").unwrap().to_string(),
        "Zerstöre die Charaktere."
    );
    assert_eq!(
        locale.get_phrase("with_events").unwrap().to_string(),
        "mit den Ereignisse"
    );
    assert_eq!(
        locale.get_phrase("of_cards").unwrap().to_string(),
        "der Karten"
    );
}

#[test]
fn german_der_plural_all_cases() {
    // Plural articles are gender-independent
    let source = r#"
        karte = :fem { one: "Karte", other: "Karten" };
        nom = "{@der:nom.other karte}";
        acc = "{@der:acc.other karte}";
        dat = "{@der:dat.other karte}";
        gen = "{@der:gen.other karte}";
    "#;

    let mut locale = Locale::builder().language("de").build();
    locale.load_translations_str("de", source).unwrap();

    assert_eq!(locale.get_phrase("nom").unwrap().to_string(), "die Karten");
    assert_eq!(locale.get_phrase("acc").unwrap().to_string(), "die Karten");
    assert_eq!(locale.get_phrase("dat").unwrap().to_string(), "den Karten");
    assert_eq!(locale.get_phrase("gen").unwrap().to_string(), "der Karten");
}

#[test]
fn german_der_numeric_plural_context() {
    // Numeric context: 1 = singular, anything else = plural
    let phrase = Phrase::builder()
        .text("Karte".to_string())
        .tags(vec![Tag::new("fem")])
        .build();
    let value = Value::Phrase(phrase);

    let transform = TransformKind::GermanDer;

    let context_one = Value::Number(1);
    let result = transform.execute(&value, Some(&context_one), "de").unwrap();
    assert_eq!(result, "die Karte"); // singular fem nom = "die"

    let context_many = Value::Number(3);
    let result = transform
        .execute(&value, Some(&context_many), "de")
        .unwrap();
    assert_eq!(result, "die Karte"); // plural nom = "die"
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
fn spanish_el_context_selects_variant_from_phrase() {
    // Test: @el:other should select both the plural article AND the :other variant of the phrase
    let source = r#"
        carta = :fem { one: "carta", other: "cartas" };
        return_all($t) = "devuelve {@el:other $t}";
    "#;

    let mut locale = Locale::builder().language("es").build();
    locale.load_translations_str("es", source).unwrap();

    let card = locale.get_phrase("carta").unwrap();
    let result = locale
        .call_phrase("return_all", &[Value::Phrase(card)])
        .unwrap();
    // @el:other should produce "las cartas", not "las carta"
    assert_eq!(result.to_string(), "devuelve las cartas");
}

#[test]
fn spanish_un_context_selects_variant_from_phrase() {
    // Test: @un:other should select both the plural article AND the :other variant of the phrase
    let source = r#"
        enemigo = :masc { one: "enemigo", other: "enemigos" };
        some_enemies($t) = "algunos {@un:other $t}";
    "#;

    let mut locale = Locale::builder().language("es").build();
    locale.load_translations_str("es", source).unwrap();

    let enemy = locale.get_phrase("enemigo").unwrap();
    let result = locale
        .call_phrase("some_enemies", &[Value::Phrase(enemy)])
        .unwrap();
    // @un:other should produce "unos enemigos", not "unos enemigo"
    assert_eq!(result.to_string(), "algunos unos enemigos");
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
    assert_eq!(result, "à la main"); // No contraction, accent grave on "à"
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
    assert_eq!(result, "à l'ami"); // à + l' (elided, accent grave on "à")
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
    // à + l' = à l'
    assert_eq!(
        locale.get_phrase("to_friend").unwrap().to_string(),
        "à l'ami"
    );
    // à + la = à la
    assert_eq!(
        locale.get_phrase("to_house").unwrap().to_string(),
        "à la maison"
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

// =============================================================================
// Greek Transform Tests (Phase 8)
// =============================================================================

#[test]
fn greek_o_masculine_nominative() {
    // Masculine nominative: ο
    let phrase = Phrase::builder()
        .text("φίλος".to_string())
        .tags(vec![Tag::new("masc")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::GreekO;
    let result = transform.execute(&value, None, "el").unwrap();
    assert_eq!(result, "ο φίλος");
}

#[test]
fn greek_o_masculine_accusative() {
    // Masculine accusative: τον
    let phrase = Phrase::builder()
        .text("φίλο".to_string())
        .tags(vec![Tag::new("masc")])
        .build();
    let value = Value::Phrase(phrase);
    let context = Value::String("acc".to_string());
    let transform = TransformKind::GreekO;
    let result = transform.execute(&value, Some(&context), "el").unwrap();
    assert_eq!(result, "τον φίλο");
}

#[test]
fn greek_o_masculine_genitive() {
    // Masculine genitive: του
    let phrase = Phrase::builder()
        .text("φίλου".to_string())
        .tags(vec![Tag::new("masc")])
        .build();
    let value = Value::Phrase(phrase);
    let context = Value::String("gen".to_string());
    let transform = TransformKind::GreekO;
    let result = transform.execute(&value, Some(&context), "el").unwrap();
    assert_eq!(result, "του φίλου");
}

#[test]
fn greek_o_feminine_nominative() {
    // Feminine nominative: η
    let phrase = Phrase::builder()
        .text("κάρτα".to_string())
        .tags(vec![Tag::new("fem")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::GreekO;
    let result = transform.execute(&value, None, "el").unwrap();
    assert_eq!(result, "η κάρτα");
}

#[test]
fn greek_o_feminine_accusative() {
    // Feminine accusative: την
    let phrase = Phrase::builder()
        .text("κάρτα".to_string())
        .tags(vec![Tag::new("fem")])
        .build();
    let value = Value::Phrase(phrase);
    let context = Value::String("acc".to_string());
    let transform = TransformKind::GreekO;
    let result = transform.execute(&value, Some(&context), "el").unwrap();
    assert_eq!(result, "την κάρτα");
}

#[test]
fn greek_o_feminine_genitive() {
    // Feminine genitive: της
    let phrase = Phrase::builder()
        .text("κάρτας".to_string())
        .tags(vec![Tag::new("fem")])
        .build();
    let value = Value::Phrase(phrase);
    let context = Value::String("gen".to_string());
    let transform = TransformKind::GreekO;
    let result = transform.execute(&value, Some(&context), "el").unwrap();
    assert_eq!(result, "της κάρτας");
}

#[test]
fn greek_o_neuter_nominative() {
    // Neuter nominative: το
    let phrase = Phrase::builder()
        .text("βιβλίο".to_string())
        .tags(vec![Tag::new("neut")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::GreekO;
    let result = transform.execute(&value, None, "el").unwrap();
    assert_eq!(result, "το βιβλίο");
}

#[test]
fn greek_o_neuter_accusative() {
    // Neuter accusative: το (same as nominative)
    let phrase = Phrase::builder()
        .text("βιβλίο".to_string())
        .tags(vec![Tag::new("neut")])
        .build();
    let value = Value::Phrase(phrase);
    let context = Value::String("acc".to_string());
    let transform = TransformKind::GreekO;
    let result = transform.execute(&value, Some(&context), "el").unwrap();
    assert_eq!(result, "το βιβλίο");
}

#[test]
fn greek_o_plural_masculine() {
    // Masculine plural nominative: οι
    let phrase = Phrase::builder()
        .text("φίλοι".to_string())
        .tags(vec![Tag::new("masc")])
        .build();
    let value = Value::Phrase(phrase);
    let context = Value::String("other".to_string());
    let transform = TransformKind::GreekO;
    let result = transform.execute(&value, Some(&context), "el").unwrap();
    assert_eq!(result, "οι φίλοι");
}

#[test]
fn greek_o_plural_feminine() {
    // Feminine plural nominative: οι (same as masculine!)
    let phrase = Phrase::builder()
        .text("κάρτες".to_string())
        .tags(vec![Tag::new("fem")])
        .build();
    let value = Value::Phrase(phrase);
    let context = Value::String("other".to_string());
    let transform = TransformKind::GreekO;
    let result = transform.execute(&value, Some(&context), "el").unwrap();
    assert_eq!(result, "οι κάρτες");
}

#[test]
fn greek_o_plural_neuter() {
    // Neuter plural nominative: τα
    let phrase = Phrase::builder()
        .text("βιβλία".to_string())
        .tags(vec![Tag::new("neut")])
        .build();
    let value = Value::Phrase(phrase);
    let context = Value::String("other".to_string());
    let transform = TransformKind::GreekO;
    let result = transform.execute(&value, Some(&context), "el").unwrap();
    assert_eq!(result, "τα βιβλία");
}

#[test]
fn greek_o_plural_genitive() {
    // Masculine plural genitive: των
    let phrase = Phrase::builder()
        .text("φίλων".to_string())
        .tags(vec![Tag::new("masc")])
        .build();
    let value = Value::Phrase(phrase);
    let context = Value::String("gen.other".to_string());
    let transform = TransformKind::GreekO;
    let result = transform.execute(&value, Some(&context), "el").unwrap();
    assert_eq!(result, "των φίλων");
}

#[test]
fn greek_o_plural_accusative() {
    // Masculine plural accusative: τους
    let phrase = Phrase::builder()
        .text("φίλους".to_string())
        .tags(vec![Tag::new("masc")])
        .build();
    let value = Value::Phrase(phrase);
    let context = Value::String("acc.other".to_string());
    let transform = TransformKind::GreekO;
    let result = transform.execute(&value, Some(&context), "el").unwrap();
    assert_eq!(result, "τους φίλους");
}

#[test]
fn greek_o_plural_dative() {
    // Feminine plural dative: ταις
    let phrase = Phrase::builder()
        .text("κάρταις".to_string())
        .tags(vec![Tag::new("fem")])
        .build();
    let value = Value::Phrase(phrase);
    let context = Value::String("dat.other".to_string());
    let transform = TransformKind::GreekO;
    let result = transform.execute(&value, Some(&context), "el").unwrap();
    assert_eq!(result, "ταις κάρταις");
}

#[test]
fn greek_o_singular_case_only_still_works() {
    // Backwards compatible: "gen" alone means genitive singular
    let phrase = Phrase::builder()
        .text("φίλου".to_string())
        .tags(vec![Tag::new("masc")])
        .build();
    let value = Value::Phrase(phrase);
    let context = Value::String("gen".to_string());
    let transform = TransformKind::GreekO;
    let result = transform.execute(&value, Some(&context), "el").unwrap();
    assert_eq!(result, "του φίλου");
}

#[test]
fn greek_o_plural_only_still_works() {
    // Backwards compatible: "other" alone means nominative plural
    let phrase = Phrase::builder()
        .text("φίλοι".to_string())
        .tags(vec![Tag::new("masc")])
        .build();
    let value = Value::Phrase(phrase);
    let context = Value::String("other".to_string());
    let transform = TransformKind::GreekO;
    let result = transform.execute(&value, Some(&context), "el").unwrap();
    assert_eq!(result, "οι φίλοι");
}

#[test]
fn greek_o_compound_nominative_singular() {
    // Explicit nominative singular via compound context
    let phrase = Phrase::builder()
        .text("φίλος".to_string())
        .tags(vec![Tag::new("masc")])
        .build();
    let value = Value::Phrase(phrase);
    let context = Value::String("nom.one".to_string());
    let transform = TransformKind::GreekO;
    let result = transform.execute(&value, Some(&context), "el").unwrap();
    assert_eq!(result, "ο φίλος");
}

#[test]
fn greek_o_compound_with_variants() {
    // Compound context selects correct variant AND article
    let phrase = Phrase::builder()
        .text("φίλος".to_string())
        .tags(vec![Tag::new("masc")])
        .variants(HashMap::from([(
            VariantKey::new("gen.other"),
            "φίλων".to_string(),
        )]))
        .build();
    let value = Value::Phrase(phrase);
    let context = Value::String("gen.other".to_string());
    let transform = TransformKind::GreekO;
    let result = transform.execute(&value, Some(&context), "el").unwrap();
    assert_eq!(result, "των φίλων");
}

#[test]
fn greek_o_neuter_plural_genitive() {
    // Neuter plural genitive: των (same as masc/fem)
    let phrase = Phrase::builder()
        .text("βιβλίων".to_string())
        .tags(vec![Tag::new("neut")])
        .build();
    let value = Value::Phrase(phrase);
    let context = Value::String("gen.other".to_string());
    let transform = TransformKind::GreekO;
    let result = transform.execute(&value, Some(&context), "el").unwrap();
    assert_eq!(result, "των βιβλίων");
}

#[test]
fn greek_o_feminine_plural_accusative() {
    // Feminine plural accusative: τις
    let phrase = Phrase::builder()
        .text("κάρτες".to_string())
        .tags(vec![Tag::new("fem")])
        .build();
    let value = Value::Phrase(phrase);
    let context = Value::String("acc.other".to_string());
    let transform = TransformKind::GreekO;
    let result = transform.execute(&value, Some(&context), "el").unwrap();
    assert_eq!(result, "τις κάρτες");
}

#[test]
fn greek_o_alias_i() {
    // @i resolves to @o for feminine
    let registry = TransformRegistry::new();
    assert_eq!(registry.get("i", "el"), Some(TransformKind::GreekO));
}

#[test]
fn greek_o_alias_to() {
    // @to resolves to @o for neuter
    let registry = TransformRegistry::new();
    assert_eq!(registry.get("to", "el"), Some(TransformKind::GreekO));
}

// =============================================================================
// Greek Indefinite Article Tests (@enas/@mia/@ena)
// =============================================================================

#[test]
fn greek_enas_masculine_nominative() {
    // Masculine nominative: ένας
    let phrase = Phrase::builder()
        .text("φίλος".to_string())
        .tags(vec![Tag::new("masc")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::GreekEnas;
    let result = transform.execute(&value, None, "el").unwrap();
    assert_eq!(result, "ένας φίλος");
}

#[test]
fn greek_enas_masculine_accusative() {
    // Masculine accusative: έναν
    let phrase = Phrase::builder()
        .text("φίλο".to_string())
        .tags(vec![Tag::new("masc")])
        .build();
    let value = Value::Phrase(phrase);
    let context = Value::String("acc".to_string());
    let transform = TransformKind::GreekEnas;
    let result = transform.execute(&value, Some(&context), "el").unwrap();
    assert_eq!(result, "έναν φίλο");
}

#[test]
fn greek_enas_masculine_genitive() {
    // Masculine genitive: ενός
    let phrase = Phrase::builder()
        .text("φίλου".to_string())
        .tags(vec![Tag::new("masc")])
        .build();
    let value = Value::Phrase(phrase);
    let context = Value::String("gen".to_string());
    let transform = TransformKind::GreekEnas;
    let result = transform.execute(&value, Some(&context), "el").unwrap();
    assert_eq!(result, "ενός φίλου");
}

#[test]
fn greek_enas_feminine_nominative() {
    // Feminine nominative: μία
    let phrase = Phrase::builder()
        .text("κάρτα".to_string())
        .tags(vec![Tag::new("fem")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::GreekEnas;
    let result = transform.execute(&value, None, "el").unwrap();
    assert_eq!(result, "μία κάρτα");
}

#[test]
fn greek_enas_feminine_genitive() {
    // Feminine genitive: μιας
    let phrase = Phrase::builder()
        .text("κάρτας".to_string())
        .tags(vec![Tag::new("fem")])
        .build();
    let value = Value::Phrase(phrase);
    let context = Value::String("gen".to_string());
    let transform = TransformKind::GreekEnas;
    let result = transform.execute(&value, Some(&context), "el").unwrap();
    assert_eq!(result, "μιας κάρτας");
}

#[test]
fn greek_enas_neuter_nominative() {
    // Neuter nominative: ένα
    let phrase = Phrase::builder()
        .text("βιβλίο".to_string())
        .tags(vec![Tag::new("neut")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::GreekEnas;
    let result = transform.execute(&value, None, "el").unwrap();
    assert_eq!(result, "ένα βιβλίο");
}

#[test]
fn greek_enas_alias_mia() {
    // @mia resolves to @enas
    let registry = TransformRegistry::new();
    assert_eq!(registry.get("mia", "el"), Some(TransformKind::GreekEnas));
}

#[test]
fn greek_enas_alias_ena() {
    // @ena resolves to @enas
    let registry = TransformRegistry::new();
    assert_eq!(registry.get("ena", "el"), Some(TransformKind::GreekEnas));
}

#[test]
fn greek_o_missing_gender_tag() {
    // No gender tag produces MissingTag error
    let phrase = Phrase::builder().text("πράγμα".to_string()).build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::GreekO;
    let result = transform.execute(&value, None, "el");
    assert!(matches!(result, Err(EvalError::MissingTag { .. })));
}

#[test]
fn greek_enas_missing_gender_tag() {
    // No gender tag produces MissingTag error
    let phrase = Phrase::builder().text("πράγμα".to_string()).build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::GreekEnas;
    let result = transform.execute(&value, None, "el");
    assert!(matches!(result, Err(EvalError::MissingTag { .. })));
}

// =============================================================================
// Greek Transform Integration Tests (Full Evaluation Path)
// =============================================================================

#[test]
fn greek_article_in_template() {
    let source = r#"
        karta = :fem "κάρτα";
        filos = :masc "φίλος";
        vivlio = :neut "βιβλίο";
        the_card = "{@o karta}";
        the_friend = "{@o filos}";
        the_book = "{@to vivlio}";
        a_card = "{@enas karta}";
        a_friend = "{@enas filos}";
    "#;

    let mut locale = Locale::builder().language("el").build();
    locale.load_translations_str("el", source).unwrap();

    assert_eq!(
        locale.get_phrase("the_card").unwrap().to_string(),
        "η κάρτα"
    );
    assert_eq!(
        locale.get_phrase("the_friend").unwrap().to_string(),
        "ο φίλος"
    );
    assert_eq!(
        locale.get_phrase("the_book").unwrap().to_string(),
        "το βιβλίο"
    );
    assert_eq!(
        locale.get_phrase("a_card").unwrap().to_string(),
        "μία κάρτα"
    );
    assert_eq!(
        locale.get_phrase("a_friend").unwrap().to_string(),
        "ένας φίλος"
    );
}

#[test]
fn greek_article_with_case_context() {
    // Test case selection via context
    let source = r#"
        filos = :masc "φίλο";
        karta = :fem "κάρτα";
        see_friend = "Βλέπω {@o:acc filos}.";
        see_card = "Βλέπω {@i:acc karta}.";
    "#;

    let mut locale = Locale::builder().language("el").build();
    locale.load_translations_str("el", source).unwrap();

    // Accusative forms: τον (masc), την (fem)
    assert_eq!(
        locale.get_phrase("see_friend").unwrap().to_string(),
        "Βλέπω τον φίλο."
    );
    assert_eq!(
        locale.get_phrase("see_card").unwrap().to_string(),
        "Βλέπω την κάρτα."
    );
}

// =============================================================================
// Romanian Transform Tests (Phase 8)
// =============================================================================

#[test]
fn romanian_def_masculine_singular() {
    // Masculine singular: -ul suffix
    let phrase = Phrase::builder()
        .text("prieten".to_string())
        .tags(vec![Tag::new("masc")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::RomanianDef;
    let result = transform.execute(&value, None, "ro").unwrap();
    assert_eq!(result, "prietenul");
}

#[test]
fn romanian_def_masculine_plural() {
    // Masculine plural: -ii suffix
    // Using "baieti" (boys - plural)
    // Note: Simple suffix append - in real Romanian, morphological changes occur
    let phrase = Phrase::builder()
        .text("baieti".to_string()) // "boys" (plural)
        .tags(vec![Tag::new("masc")])
        .build();
    let value = Value::Phrase(phrase);
    let context = Value::String("other".to_string());
    let transform = TransformKind::RomanianDef;
    let result = transform.execute(&value, Some(&context), "ro").unwrap();
    // Simple append: baieti + ii = baietiii (raw append, no morphological merge)
    assert_eq!(result, "baietiii");
}

#[test]
fn romanian_def_feminine_singular() {
    // Feminine singular: -a suffix
    let phrase = Phrase::builder()
        .text("carte".to_string())
        .tags(vec![Tag::new("fem")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::RomanianDef;
    let result = transform.execute(&value, None, "ro").unwrap();
    assert_eq!(result, "cartea");
}

#[test]
fn romanian_def_feminine_plural() {
    // Feminine plural: -le suffix
    let phrase = Phrase::builder()
        .text("carti".to_string())
        .tags(vec![Tag::new("fem")])
        .build();
    let value = Value::Phrase(phrase);
    let context = Value::String("other".to_string());
    let transform = TransformKind::RomanianDef;
    let result = transform.execute(&value, Some(&context), "ro").unwrap();
    assert_eq!(result, "cartile");
}

#[test]
fn romanian_def_neuter_singular() {
    // Neuter singular: -ul suffix (like masculine)
    // "drum" (road) -> "drumul" (the road)
    let phrase = Phrase::builder()
        .text("drum".to_string())
        .tags(vec![Tag::new("neut")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::RomanianDef;
    let result = transform.execute(&value, None, "ro").unwrap();
    assert_eq!(result, "drumul");
}

#[test]
fn romanian_def_neuter_plural() {
    // Neuter plural: -le suffix (like feminine)
    let phrase = Phrase::builder()
        .text("lucruri".to_string())
        .tags(vec![Tag::new("neut")])
        .build();
    let value = Value::Phrase(phrase);
    let context = Value::String("other".to_string());
    let transform = TransformKind::RomanianDef;
    let result = transform.execute(&value, Some(&context), "ro").unwrap();
    assert_eq!(result, "lucrurile");
}

#[test]
fn romanian_def_missing_gender_tag() {
    // No gender tag produces MissingTag error
    let phrase = Phrase::builder().text("ceva".to_string()).build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::RomanianDef;
    let result = transform.execute(&value, None, "ro");
    assert!(matches!(result, Err(EvalError::MissingTag { .. })));
}

#[test]
fn romanian_def_registry_lookup() {
    // @def resolves to RomanianDef for Romanian
    let registry = TransformRegistry::new();
    assert_eq!(registry.get("def", "ro"), Some(TransformKind::RomanianDef));
}

#[test]
fn romanian_def_not_available_for_other_languages() {
    // @def should not be available for other languages
    let registry = TransformRegistry::new();
    assert_eq!(registry.get("def", "en"), None);
    assert_eq!(registry.get("def", "el"), None);
}

// =============================================================================
// Romanian Transform Integration Tests (Full Evaluation Path)
// =============================================================================

#[test]
fn romanian_postposed_article_in_template() {
    let source = r#"
        carte = :fem "carte";
        prieten = :masc "prieten";
        drum = :neut "drum";
        the_book = "{@def carte}";
        the_friend = "{@def prieten}";
        the_road = "{@def drum}";
    "#;

    let mut locale = Locale::builder().language("ro").build();
    locale.load_translations_str("ro", source).unwrap();

    // Romanian appends article suffix
    assert_eq!(locale.get_phrase("the_book").unwrap().to_string(), "cartea");
    assert_eq!(
        locale.get_phrase("the_friend").unwrap().to_string(),
        "prietenul"
    );
    assert_eq!(locale.get_phrase("the_road").unwrap().to_string(), "drumul");
}

#[test]
fn romanian_postposed_article_with_plural() {
    // Test plural context
    // Note: Simple suffix append - in real Romanian, morphological changes occur
    let source = r#"
        flori = :fem "flori";
        baieti = :masc "baieti";
        drumuri = :neut "drumuri";
        the_flowers = "{@def:other flori}";
        the_boys = "{@def:other baieti}";
        the_roads = "{@def:other drumuri}";
    "#;

    let mut locale = Locale::builder().language("ro").build();
    locale.load_translations_str("ro", source).unwrap();

    // Plural suffixes: -le (fem), -ii (masc), -le (neut)
    // Simple append: flori + le = florile, baieti + ii = baietiii, drumuri + le = drumurile
    assert_eq!(
        locale.get_phrase("the_flowers").unwrap().to_string(),
        "florile"
    );
    assert_eq!(
        locale.get_phrase("the_boys").unwrap().to_string(),
        "baietiii"
    );
    assert_eq!(
        locale.get_phrase("the_roads").unwrap().to_string(),
        "drumurile"
    );
}

// =============================================================================
// Greek and Romanian Cross-Language Tests
// =============================================================================

#[test]
fn greek_transform_not_available_for_other_languages() {
    let registry = TransformRegistry::new();
    // Greek transforms should not be available for other languages
    assert_eq!(registry.get("o", "en"), None);
    assert_eq!(registry.get("enas", "en"), None);
    assert_eq!(registry.get("o", "de"), None);
    // But should be available for Greek
    assert_eq!(registry.get("o", "el"), Some(TransformKind::GreekO));
    assert_eq!(registry.get("enas", "el"), Some(TransformKind::GreekEnas));
}

// =============================================================================
// Arabic Transforms (Phase 8) - @al with sun/moon letter assimilation
// =============================================================================

#[test]
fn arabic_al_sun_letter() {
    // Sun letter: assimilation occurs, first consonant doubles with shadda
    let registry = TransformRegistry::new();
    let transform = registry.get("al", "ar").expect("Arabic @al should exist");
    assert_eq!(transform, TransformKind::ArabicAl);

    // Create a phrase with :sun tag
    let phrase = Phrase::builder()
        .text("شمس".to_string()) // shams (sun)
        .tags(vec![Tag::new("sun")])
        .build();
    let value = Value::Phrase(phrase);

    // Execute transform
    let result = transform.execute(&value, None, "ar").unwrap();

    // Should produce: ال + ش + shadda + مس
    // The shadda (U+0651) comes AFTER the first consonant
    let expected = "الش\u{0651}مس"; // al + sh + shadda + ms
    assert_eq!(result, expected);
}

#[test]
fn arabic_al_moon_letter() {
    // Moon letter: no assimilation, plain ال prefix
    let registry = TransformRegistry::new();
    let transform = registry.get("al", "ar").expect("Arabic @al should exist");

    // Create a phrase with :moon tag
    let phrase = Phrase::builder()
        .text("قمر".to_string()) // qamar (moon)
        .tags(vec![Tag::new("moon")])
        .build();
    let value = Value::Phrase(phrase);

    // Execute transform
    let result = transform.execute(&value, None, "ar").unwrap();

    // Should produce: ال + قمر (no assimilation)
    let expected = "القمر"; // al-qamar
    assert_eq!(result, expected);
}

#[test]
fn arabic_al_missing_tag() {
    // No :sun or :moon tag -> MissingTag error
    let registry = TransformRegistry::new();
    let transform = registry.get("al", "ar").expect("Arabic @al should exist");

    // Create a phrase without sun/moon tag
    let phrase = Phrase::builder()
        .text("كتاب".to_string()) // kitab (book)
        .build();
    let value = Value::Phrase(phrase);

    // Execute transform - should fail
    let result = transform.execute(&value, None, "ar");

    match result {
        Err(EvalError::MissingTag {
            transform,
            expected,
            ..
        }) => {
            assert_eq!(transform, "al");
            assert!(expected.contains(&"sun".to_string()));
            assert!(expected.contains(&"moon".to_string()));
        }
        _ => panic!("Expected MissingTag error"),
    }
}

#[test]
fn arabic_al_sun_shadda_position() {
    // Verify shadda comes AFTER consonant, not before (per RESEARCH.md pitfall)
    let registry = TransformRegistry::new();
    let transform = registry.get("al", "ar").unwrap();

    let phrase = Phrase::builder()
        .text("نور".to_string()) // noor (light) - starts with noon (sun letter)
        .tags(vec![Tag::new("sun")])
        .build();
    let value = Value::Phrase(phrase);

    let result = transform.execute(&value, None, "ar").unwrap();

    // Check byte-level: shadda (U+0651) should come after noon (U+0646), not before
    let bytes: Vec<char> = result.chars().collect();

    // ال = alef (U+0627) + lam (U+0644)
    // Then noon (U+0646), then shadda (U+0651), then rest
    assert_eq!(bytes[0], '\u{0627}'); // alef
    assert_eq!(bytes[1], '\u{0644}'); // lam
    assert_eq!(bytes[2], '\u{0646}'); // noon (first char of original text)
    assert_eq!(bytes[3], '\u{0651}'); // shadda AFTER noon
}

#[test]
fn arabic_al_output_bytes() {
    // Byte-level verification to avoid RTL text comparison issues
    let registry = TransformRegistry::new();
    let transform = registry.get("al", "ar").unwrap();

    // Test with simple Arabic text - taa (sun letter)
    let phrase = Phrase::builder()
        .text("ت".to_string()) // just the letter taa (U+062A)
        .tags(vec![Tag::new("sun")])
        .build();
    let value = Value::Phrase(phrase);

    let result = transform.execute(&value, None, "ar").unwrap();

    // Expected: alef + lam + taa + shadda
    let expected_chars: Vec<char> = vec![
        '\u{0627}', // ARABIC LETTER ALEF
        '\u{0644}', // ARABIC LETTER LAM
        '\u{062A}', // ARABIC LETTER TEH
        '\u{0651}', // ARABIC SHADDA
    ];

    let result_chars: Vec<char> = result.chars().collect();
    assert_eq!(result_chars, expected_chars);
}

// =============================================================================
// Persian Transforms (Phase 8) - @ezafe connector
// =============================================================================

#[test]
fn persian_ezafe_consonant() {
    // Word ends in consonant: use kasra (-e)
    let registry = TransformRegistry::new();
    let transform = registry
        .get("ezafe", "fa")
        .expect("Persian @ezafe should exist");
    assert_eq!(transform, TransformKind::PersianEzafe);

    // Create a phrase without :vowel tag (consonant-final)
    let phrase = Phrase::builder()
        .text("کتاب".to_string()) // ketab (book)
        .build();
    let value = Value::Phrase(phrase);

    // Execute transform
    let result = transform.execute(&value, None, "fa").unwrap();

    // Should produce: کتاب + kasra (U+0650)
    let expected = "کتاب\u{0650}"; // ketab + kasra
    assert_eq!(result, expected);
}

#[test]
fn persian_ezafe_vowel() {
    // Word ends in vowel: use -ye connector with ZWNJ
    let registry = TransformRegistry::new();
    let transform = registry.get("ezafe", "fa").unwrap();

    // Create a phrase with :vowel tag
    let phrase = Phrase::builder()
        .text("خانه".to_string()) // khane (house) - ends in silent h/vowel
        .tags(vec![Tag::new("vowel")])
        .build();
    let value = Value::Phrase(phrase);

    // Execute transform
    let result = transform.execute(&value, None, "fa").unwrap();

    // Should produce: خانه + ZWNJ + ye
    let expected = "خانه\u{200C}\u{06CC}"; // khane + ZWNJ + Persian ye
    assert_eq!(result, expected);
}

#[test]
fn persian_ezafe_kasra_unicode() {
    // Verify kasra is exactly U+0650
    let registry = TransformRegistry::new();
    let transform = registry.get("ezafe", "fa").unwrap();

    let phrase = Phrase::builder().text("x".to_string()).build();
    let value = Value::Phrase(phrase);

    let result = transform.execute(&value, None, "fa").unwrap();

    // Last character should be kasra
    let last_char = result.chars().last().unwrap();
    assert_eq!(last_char, '\u{0650}', "Kasra should be U+0650");
}

#[test]
fn persian_ezafe_zwnj_unicode() {
    // Verify ZWNJ is exactly U+200C
    let registry = TransformRegistry::new();
    let transform = registry.get("ezafe", "fa").unwrap();

    let phrase = Phrase::builder()
        .text("x".to_string())
        .tags(vec![Tag::new("vowel")])
        .build();
    let value = Value::Phrase(phrase);

    let result = transform.execute(&value, None, "fa").unwrap();

    // Second to last character should be ZWNJ
    let chars: Vec<char> = result.chars().collect();
    assert_eq!(chars[1], '\u{200C}', "ZWNJ should be U+200C");
}

#[test]
fn persian_ezafe_output_bytes() {
    // Byte-level verification for vowel case
    let registry = TransformRegistry::new();
    let transform = registry.get("ezafe", "fa").unwrap();

    // Simple test: single character + ezafe
    let phrase = Phrase::builder()
        .text("ا".to_string()) // alef
        .tags(vec![Tag::new("vowel")])
        .build();
    let value = Value::Phrase(phrase);

    let result = transform.execute(&value, None, "fa").unwrap();

    let expected_chars: Vec<char> = vec![
        '\u{0627}', // ARABIC LETTER ALEF
        '\u{200C}', // ZERO WIDTH NON-JOINER
        '\u{06CC}', // ARABIC LETTER FARSI YEH
    ];

    let result_chars: Vec<char> = result.chars().collect();
    assert_eq!(result_chars, expected_chars);
}

// =============================================================================
// Arabic and Persian Integration Tests
// =============================================================================

#[test]
fn arabic_definite_article_in_phrase() {
    // Test @al in full phrase evaluation using Locale
    let source = r#"
        shams = :sun "شمس";
        qamar = :moon "قمر";
        sun_sentence = "This is {@al shams}.";
        moon_sentence = "This is {@al qamar}.";
    "#;

    let mut locale = Locale::builder().language("ar").build();
    locale.load_translations_str("ar", source).unwrap();

    // Sun letter with assimilation
    assert_eq!(
        locale.get_phrase("sun_sentence").unwrap().to_string(),
        "This is الش\u{0651}مس." // ash-shams with shadda
    );

    // Moon letter without assimilation
    assert_eq!(
        locale.get_phrase("moon_sentence").unwrap().to_string(),
        "This is القمر." // al-qamar
    );
}

#[test]
fn persian_ezafe_in_phrase() {
    // Test @ezafe in full phrase evaluation using Locale
    let source = r#"
        ketab = "کتاب";
        khane = :vowel "خانه";
        book_of = "The {@ezafe ketab} man.";
        house_of = "The {@ezafe khane} friend.";
    "#;

    let mut locale = Locale::builder().language("fa").build();
    locale.load_translations_str("fa", source).unwrap();

    // Consonant-final with kasra
    assert_eq!(
        locale.get_phrase("book_of").unwrap().to_string(),
        "The کتاب\u{0650} man." // ketab-e
    );

    // Vowel-final with ZWNJ + ye
    assert_eq!(
        locale.get_phrase("house_of").unwrap().to_string(),
        "The خانه\u{200C}\u{06CC} friend." // khane-ye
    );
}

#[test]
fn arabic_transform_not_available_for_other_languages() {
    let registry = TransformRegistry::new();
    // Arabic transforms should not be available for other languages
    assert_eq!(registry.get("al", "en"), None);
    assert_eq!(registry.get("al", "fa"), None); // Not Persian either
    // But should be available for Arabic
    assert_eq!(registry.get("al", "ar"), Some(TransformKind::ArabicAl));
}

#[test]
fn persian_transform_not_available_for_other_languages() {
    let registry = TransformRegistry::new();
    // Persian transforms should not be available for other languages
    assert_eq!(registry.get("ezafe", "en"), None);
    assert_eq!(registry.get("ezafe", "ar"), None); // Not Arabic either
    // But should be available for Persian
    assert_eq!(
        registry.get("ezafe", "fa"),
        Some(TransformKind::PersianEzafe)
    );
}

// =============================================================================
// CJK Count Transforms (Phase 9)
// =============================================================================

// -----------------------------------------------------------------------------
// Chinese @count Transform Tests
// -----------------------------------------------------------------------------

#[test]
fn chinese_count_zhang() {
    // :zhang "牌" with context 3 -> "3张牌"
    let phrase = Phrase::builder()
        .text("牌".to_string())
        .tags(vec![Tag::new("zhang")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::ChineseCount;
    let context = Value::Number(3);
    let result = transform.execute(&value, Some(&context), "zh").unwrap();
    assert_eq!(result, "3张牌");
}

#[test]
fn chinese_count_ge() {
    // :ge "角色" with context 2 -> "2个角色"
    let phrase = Phrase::builder()
        .text("角色".to_string())
        .tags(vec![Tag::new("ge")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::ChineseCount;
    let context = Value::Number(2);
    let result = transform.execute(&value, Some(&context), "zh").unwrap();
    assert_eq!(result, "2个角色");
}

#[test]
fn chinese_count_ming() {
    // :ming "玩家" with context 1 -> "1名玩家"
    let phrase = Phrase::builder()
        .text("玩家".to_string())
        .tags(vec![Tag::new("ming")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::ChineseCount;
    let context = Value::Number(1);
    let result = transform.execute(&value, Some(&context), "zh").unwrap();
    assert_eq!(result, "1名玩家");
}

#[test]
fn chinese_count_wei() {
    // :wei "客人" with context 5 -> "5位客人"
    let phrase = Phrase::builder()
        .text("客人".to_string())
        .tags(vec![Tag::new("wei")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::ChineseCount;
    let context = Value::Number(5);
    let result = transform.execute(&value, Some(&context), "zh").unwrap();
    assert_eq!(result, "5位客人");
}

#[test]
fn chinese_count_ben() {
    // :ben "书" with context 4 -> "4本书"
    let phrase = Phrase::builder()
        .text("书".to_string())
        .tags(vec![Tag::new("ben")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::ChineseCount;
    let context = Value::Number(4);
    let result = transform.execute(&value, Some(&context), "zh").unwrap();
    assert_eq!(result, "4本书");
}

#[test]
fn chinese_count_missing_tag() {
    // Phrase without classifier tag returns MissingTag error
    let phrase = Phrase::builder().text("东西".to_string()).build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::ChineseCount;
    let context = Value::Number(3);
    let result = transform.execute(&value, Some(&context), "zh");
    assert!(matches!(result, Err(EvalError::MissingTag { .. })));
}

#[test]
fn chinese_count_default_to_one() {
    // Without context, default to count=1
    let phrase = Phrase::builder()
        .text("牌".to_string())
        .tags(vec![Tag::new("zhang")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::ChineseCount;
    let result = transform.execute(&value, None, "zh").unwrap();
    assert_eq!(result, "1张牌");
}

// -----------------------------------------------------------------------------
// Japanese @count Transform Tests
// -----------------------------------------------------------------------------

#[test]
fn japanese_count_mai() {
    // :mai "カード" with context 3 -> "3枚カード"
    let phrase = Phrase::builder()
        .text("カード".to_string())
        .tags(vec![Tag::new("mai")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::JapaneseCount;
    let context = Value::Number(3);
    let result = transform.execute(&value, Some(&context), "ja").unwrap();
    assert_eq!(result, "3枚カード");
}

#[test]
fn japanese_count_nin() {
    // :nin "キャラクター" with context 2 -> "2人キャラクター"
    let phrase = Phrase::builder()
        .text("キャラクター".to_string())
        .tags(vec![Tag::new("nin")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::JapaneseCount;
    let context = Value::Number(2);
    let result = transform.execute(&value, Some(&context), "ja").unwrap();
    assert_eq!(result, "2人キャラクター");
}

#[test]
fn japanese_count_hiki() {
    // :hiki "猫" with context 3 -> "3匹猫"
    let phrase = Phrase::builder()
        .text("猫".to_string())
        .tags(vec![Tag::new("hiki")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::JapaneseCount;
    let context = Value::Number(3);
    let result = transform.execute(&value, Some(&context), "ja").unwrap();
    assert_eq!(result, "3匹猫");
}

#[test]
fn japanese_count_hon() {
    // :hon "ペン" with context 2 -> "2本ペン"
    let phrase = Phrase::builder()
        .text("ペン".to_string())
        .tags(vec![Tag::new("hon")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::JapaneseCount;
    let context = Value::Number(2);
    let result = transform.execute(&value, Some(&context), "ja").unwrap();
    assert_eq!(result, "2本ペン");
}

#[test]
fn japanese_count_satsu() {
    // :satsu "本" with context 5 -> "5冊本"
    let phrase = Phrase::builder()
        .text("本".to_string())
        .tags(vec![Tag::new("satsu")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::JapaneseCount;
    let context = Value::Number(5);
    let result = transform.execute(&value, Some(&context), "ja").unwrap();
    assert_eq!(result, "5冊本");
}

#[test]
fn japanese_count_missing_tag() {
    // Phrase without counter tag returns MissingTag error
    let phrase = Phrase::builder().text("物".to_string()).build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::JapaneseCount;
    let context = Value::Number(3);
    let result = transform.execute(&value, Some(&context), "ja");
    assert!(matches!(result, Err(EvalError::MissingTag { .. })));
}

// -----------------------------------------------------------------------------
// Korean @count Transform Tests
// -----------------------------------------------------------------------------

#[test]
fn korean_count_jang() {
    // :jang "카드" with context 3 -> "3장카드"
    let phrase = Phrase::builder()
        .text("카드".to_string())
        .tags(vec![Tag::new("jang")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::KoreanCount;
    let context = Value::Number(3);
    let result = transform.execute(&value, Some(&context), "ko").unwrap();
    assert_eq!(result, "3장카드");
}

#[test]
fn korean_count_myeong() {
    // :myeong "캐릭터" with context 2 -> "2명캐릭터"
    let phrase = Phrase::builder()
        .text("캐릭터".to_string())
        .tags(vec![Tag::new("myeong")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::KoreanCount;
    let context = Value::Number(2);
    let result = transform.execute(&value, Some(&context), "ko").unwrap();
    assert_eq!(result, "2명캐릭터");
}

#[test]
fn korean_count_mari() {
    // :mari "고양이" with context 3 -> "3마리고양이"
    let phrase = Phrase::builder()
        .text("고양이".to_string())
        .tags(vec![Tag::new("mari")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::KoreanCount;
    let context = Value::Number(3);
    let result = transform.execute(&value, Some(&context), "ko").unwrap();
    assert_eq!(result, "3마리고양이");
}

#[test]
fn korean_count_gae() {
    // :gae "사과" with context 5 -> "5개사과"
    let phrase = Phrase::builder()
        .text("사과".to_string())
        .tags(vec![Tag::new("gae")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::KoreanCount;
    let context = Value::Number(5);
    let result = transform.execute(&value, Some(&context), "ko").unwrap();
    assert_eq!(result, "5개사과");
}

#[test]
fn korean_count_gwon() {
    // :gwon "책" with context 4 -> "4권책"
    let phrase = Phrase::builder()
        .text("책".to_string())
        .tags(vec![Tag::new("gwon")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::KoreanCount;
    let context = Value::Number(4);
    let result = transform.execute(&value, Some(&context), "ko").unwrap();
    assert_eq!(result, "4권책");
}

#[test]
fn korean_count_missing_tag() {
    // Phrase without counter tag returns MissingTag error
    let phrase = Phrase::builder().text("것".to_string()).build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::KoreanCount;
    let context = Value::Number(3);
    let result = transform.execute(&value, Some(&context), "ko");
    assert!(matches!(result, Err(EvalError::MissingTag { .. })));
}

// -----------------------------------------------------------------------------
// CJK Transform Registry Tests
// -----------------------------------------------------------------------------

#[test]
fn cjk_count_transform_registry_lookup() {
    let registry = TransformRegistry::new();
    assert_eq!(
        registry.get("count", "zh"),
        Some(TransformKind::ChineseCount)
    );
    assert_eq!(
        registry.get("count", "ja"),
        Some(TransformKind::JapaneseCount)
    );
    assert_eq!(
        registry.get("count", "ko"),
        Some(TransformKind::KoreanCount)
    );
}

#[test]
fn cjk_count_not_available_for_other_languages() {
    let registry = TransformRegistry::new();
    // CJK count transforms should not be available for non-CJK languages
    assert_eq!(registry.get("count", "en"), None);
    assert_eq!(registry.get("count", "de"), None);
    assert_eq!(registry.get("count", "es"), None);
}

// -----------------------------------------------------------------------------
// CJK Count String Context Tests
// -----------------------------------------------------------------------------

#[test]
fn chinese_count_string_context() {
    // String context should be parsed as number
    let phrase = Phrase::builder()
        .text("牌".to_string())
        .tags(vec![Tag::new("zhang")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::ChineseCount;
    let context = Value::String("5".to_string());
    let result = transform.execute(&value, Some(&context), "zh").unwrap();
    assert_eq!(result, "5张牌");
}

#[test]
fn japanese_count_string_context() {
    // String context should be parsed as number
    let phrase = Phrase::builder()
        .text("カード".to_string())
        .tags(vec![Tag::new("mai")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::JapaneseCount;
    let context = Value::String("7".to_string());
    let result = transform.execute(&value, Some(&context), "ja").unwrap();
    assert_eq!(result, "7枚カード");
}

#[test]
fn korean_count_string_context() {
    // String context should be parsed as number
    let phrase = Phrase::builder()
        .text("카드".to_string())
        .tags(vec![Tag::new("jang")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::KoreanCount;
    let context = Value::String("10".to_string());
    let result = transform.execute(&value, Some(&context), "ko").unwrap();
    assert_eq!(result, "10장카드");
}

// =============================================================================
// Southeast Asian Transforms (Phase 9)
// =============================================================================

// -----------------------------------------------------------------------------
// Vietnamese @count Transform Tests
// -----------------------------------------------------------------------------

#[test]
fn vietnamese_count_cai() {
    // :cai "ban" (table) with context 3 -> "3 cái ban"
    let phrase = Phrase::builder()
        .text("ban".to_string())
        .tags(vec![Tag::new("cai")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::VietnameseCount;
    let context = Value::Number(3);
    let result = transform.execute(&value, Some(&context), "vi").unwrap();
    assert_eq!(result, "3 cái ban");
}

#[test]
fn vietnamese_count_con() {
    // :con "meo" (cat) with context 2 -> "2 con meo"
    let phrase = Phrase::builder()
        .text("meo".to_string())
        .tags(vec![Tag::new("con")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::VietnameseCount;
    let context = Value::Number(2);
    let result = transform.execute(&value, Some(&context), "vi").unwrap();
    assert_eq!(result, "2 con meo");
}

#[test]
fn vietnamese_count_nguoi() {
    // :nguoi "ban" (friend) with context 5 -> "5 người ban"
    let phrase = Phrase::builder()
        .text("ban".to_string())
        .tags(vec![Tag::new("nguoi")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::VietnameseCount;
    let context = Value::Number(5);
    let result = transform.execute(&value, Some(&context), "vi").unwrap();
    assert_eq!(result, "5 người ban");
}

#[test]
fn vietnamese_count_chiec() {
    // :chiec "xe" (vehicle) with context 4 -> "4 chiếc xe"
    let phrase = Phrase::builder()
        .text("xe".to_string())
        .tags(vec![Tag::new("chiec")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::VietnameseCount;
    let context = Value::Number(4);
    let result = transform.execute(&value, Some(&context), "vi").unwrap();
    assert_eq!(result, "4 chiếc xe");
}

#[test]
fn vietnamese_count_to() {
    // :to "giay" (paper) with context 6 -> "6 tờ giay"
    let phrase = Phrase::builder()
        .text("giay".to_string())
        .tags(vec![Tag::new("to")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::VietnameseCount;
    let context = Value::Number(6);
    let result = transform.execute(&value, Some(&context), "vi").unwrap();
    assert_eq!(result, "6 tờ giay");
}

#[test]
fn vietnamese_count_missing_tag() {
    // Phrase without classifier tag returns MissingTag error
    let phrase = Phrase::builder().text("vat".to_string()).build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::VietnameseCount;
    let context = Value::Number(3);
    let result = transform.execute(&value, Some(&context), "vi");
    assert!(matches!(result, Err(EvalError::MissingTag { .. })));
}

// -----------------------------------------------------------------------------
// Thai @count Transform Tests
// -----------------------------------------------------------------------------

#[test]
fn thai_count_bai() {
    // :bai "การ์ด" (card) with context 3 -> "3ใบการ์ด"
    let phrase = Phrase::builder()
        .text("การ์ด".to_string())
        .tags(vec![Tag::new("bai")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::ThaiCount;
    let context = Value::Number(3);
    let result = transform.execute(&value, Some(&context), "th").unwrap();
    assert_eq!(result, "3ใบการ์ด");
}

#[test]
fn thai_count_khon() {
    // :khon "ผู้เล่น" (player) with context 2 -> "2คนผู้เล่น"
    let phrase = Phrase::builder()
        .text("ผู้เล่น".to_string())
        .tags(vec![Tag::new("khon")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::ThaiCount;
    let context = Value::Number(2);
    let result = transform.execute(&value, Some(&context), "th").unwrap();
    assert_eq!(result, "2คนผู้เล่น");
}

#[test]
fn thai_count_tua() {
    // :tua "แมว" (cat) with context 4 -> "4ตัวแมว"
    let phrase = Phrase::builder()
        .text("แมว".to_string())
        .tags(vec![Tag::new("tua")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::ThaiCount;
    let context = Value::Number(4);
    let result = transform.execute(&value, Some(&context), "th").unwrap();
    assert_eq!(result, "4ตัวแมว");
}

#[test]
fn thai_count_missing_tag() {
    // Phrase without classifier tag returns MissingTag error
    let phrase = Phrase::builder().text("สิ่งของ".to_string()).build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::ThaiCount;
    let context = Value::Number(3);
    let result = transform.execute(&value, Some(&context), "th");
    assert!(matches!(result, Err(EvalError::MissingTag { .. })));
}

// -----------------------------------------------------------------------------
// Bengali @count Transform Tests
// -----------------------------------------------------------------------------

#[test]
fn bengali_count_ta() {
    // :ta "বই" (book) with context 3 -> "3টা বই"
    let phrase = Phrase::builder()
        .text("বই".to_string())
        .tags(vec![Tag::new("ta")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::BengaliCount;
    let context = Value::Number(3);
    let result = transform.execute(&value, Some(&context), "bn").unwrap();
    assert_eq!(result, "3টা বই");
}

#[test]
fn bengali_count_jon() {
    // :jon "খেলোয়াড়" (player) with context 2 -> "2জন খেলোয়াড়"
    let phrase = Phrase::builder()
        .text("খেলোয়াড়".to_string())
        .tags(vec![Tag::new("jon")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::BengaliCount;
    let context = Value::Number(2);
    let result = transform.execute(&value, Some(&context), "bn").unwrap();
    assert_eq!(result, "2জন খেলোয়াড়");
}

#[test]
fn bengali_count_ti() {
    // :ti "কলম" (pen) with context 5 -> "5টি কলম" (formal)
    let phrase = Phrase::builder()
        .text("কলম".to_string())
        .tags(vec![Tag::new("ti")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::BengaliCount;
    let context = Value::Number(5);
    let result = transform.execute(&value, Some(&context), "bn").unwrap();
    assert_eq!(result, "5টি কলম");
}

#[test]
fn bengali_count_missing_tag() {
    // Phrase without classifier tag returns MissingTag error
    let phrase = Phrase::builder().text("জিনিস".to_string()).build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::BengaliCount;
    let context = Value::Number(3);
    let result = transform.execute(&value, Some(&context), "bn");
    assert!(matches!(result, Err(EvalError::MissingTag { .. })));
}

// -----------------------------------------------------------------------------
// Indonesian @plural Transform Tests
// -----------------------------------------------------------------------------

#[test]
fn indonesian_plural_basic() {
    // "kartu" -> "kartu-kartu"
    let phrase = Phrase::builder().text("kartu".to_string()).build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::IndonesianPlural;
    let result = transform.execute(&value, None, "id").unwrap();
    assert_eq!(result, "kartu-kartu");
}

#[test]
fn indonesian_plural_buku() {
    // "buku" (book) -> "buku-buku"
    let phrase = Phrase::builder().text("buku".to_string()).build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::IndonesianPlural;
    let result = transform.execute(&value, None, "id").unwrap();
    assert_eq!(result, "buku-buku");
}

#[test]
fn indonesian_plural_empty() {
    // "" -> "-" (edge case)
    let phrase = Phrase::builder().text("".to_string()).build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::IndonesianPlural;
    let result = transform.execute(&value, None, "id").unwrap();
    assert_eq!(result, "-");
}

#[test]
fn indonesian_plural_orang() {
    // "orang" (person) -> "orang-orang"
    let phrase = Phrase::builder().text("orang".to_string()).build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::IndonesianPlural;
    let result = transform.execute(&value, None, "id").unwrap();
    assert_eq!(result, "orang-orang");
}

// -----------------------------------------------------------------------------
// SEA Transform Registry Tests
// -----------------------------------------------------------------------------

#[test]
fn sea_transforms_registered() {
    let registry = TransformRegistry::new();

    // Vietnamese @count
    assert!(registry.get("count", "vi").is_some());
    assert_eq!(
        registry.get("count", "vi"),
        Some(TransformKind::VietnameseCount)
    );

    // Thai @count
    assert!(registry.get("count", "th").is_some());
    assert_eq!(registry.get("count", "th"), Some(TransformKind::ThaiCount));

    // Bengali @count
    assert!(registry.get("count", "bn").is_some());
    assert_eq!(
        registry.get("count", "bn"),
        Some(TransformKind::BengaliCount)
    );

    // Indonesian @plural
    assert!(registry.get("plural", "id").is_some());
    assert_eq!(
        registry.get("plural", "id"),
        Some(TransformKind::IndonesianPlural)
    );
}

#[test]
fn sea_count_default_to_one() {
    // Without context, default to count=1
    let phrase = Phrase::builder()
        .text("ban".to_string())
        .tags(vec![Tag::new("cai")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::VietnameseCount;
    let result = transform.execute(&value, None, "vi").unwrap();
    assert_eq!(result, "1 cái ban");
}

// =============================================================================
// Korean @particle Transform Tests (Phase 9)
// =============================================================================

// -----------------------------------------------------------------------------
// Korean @particle - Vowel-final words
// -----------------------------------------------------------------------------

#[test]
fn korean_particle_subj_vowel() {
    // "사과" (apple, ends in 과 which has no jongseong) + :subj -> "가"
    let phrase = Phrase::builder().text("사과".to_string()).build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::KoreanParticle;
    let context = Value::String("subj".to_string());
    let result = transform.execute(&value, Some(&context), "ko").unwrap();
    assert_eq!(result, "가");
}

#[test]
fn korean_particle_obj_vowel() {
    // "사과" + :obj -> "를"
    let phrase = Phrase::builder().text("사과".to_string()).build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::KoreanParticle;
    let context = Value::String("obj".to_string());
    let result = transform.execute(&value, Some(&context), "ko").unwrap();
    assert_eq!(result, "를");
}

#[test]
fn korean_particle_topic_vowel() {
    // "사과" + :topic -> "는"
    let phrase = Phrase::builder().text("사과".to_string()).build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::KoreanParticle;
    let context = Value::String("topic".to_string());
    let result = transform.execute(&value, Some(&context), "ko").unwrap();
    assert_eq!(result, "는");
}

// -----------------------------------------------------------------------------
// Korean @particle - Consonant-final words
// -----------------------------------------------------------------------------

#[test]
fn korean_particle_subj_consonant() {
    // "책" (book, ends in 책 which has jongseong ㄱ) + :subj -> "이"
    let phrase = Phrase::builder().text("책".to_string()).build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::KoreanParticle;
    let context = Value::String("subj".to_string());
    let result = transform.execute(&value, Some(&context), "ko").unwrap();
    assert_eq!(result, "이");
}

#[test]
fn korean_particle_obj_consonant() {
    // "책" + :obj -> "을"
    let phrase = Phrase::builder().text("책".to_string()).build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::KoreanParticle;
    let context = Value::String("obj".to_string());
    let result = transform.execute(&value, Some(&context), "ko").unwrap();
    assert_eq!(result, "을");
}

#[test]
fn korean_particle_topic_consonant() {
    // "책" + :topic -> "은"
    let phrase = Phrase::builder().text("책".to_string()).build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::KoreanParticle;
    let context = Value::String("topic".to_string());
    let result = transform.execute(&value, Some(&context), "ko").unwrap();
    assert_eq!(result, "은");
}

// -----------------------------------------------------------------------------
// Korean @particle - Edge cases
// -----------------------------------------------------------------------------

#[test]
fn korean_particle_english_text() {
    // "card" (non-Hangul) + :subj -> "가" (treated as vowel-ending)
    let phrase = Phrase::builder().text("card".to_string()).build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::KoreanParticle;
    let context = Value::String("subj".to_string());
    let result = transform.execute(&value, Some(&context), "ko").unwrap();
    assert_eq!(result, "가");
}

#[test]
fn korean_particle_default() {
    // No context -> defaults to Subject particle
    // "책" (consonant-final) -> "이"
    let phrase = Phrase::builder().text("책".to_string()).build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::KoreanParticle;
    let result = transform.execute(&value, None, "ko").unwrap();
    assert_eq!(result, "이");
}

// -----------------------------------------------------------------------------
// Korean @particle Registry Test
// -----------------------------------------------------------------------------

#[test]
fn korean_particle_registered() {
    let registry = TransformRegistry::new();
    assert!(registry.get("particle", "ko").is_some());
    assert_eq!(
        registry.get("particle", "ko"),
        Some(TransformKind::KoreanParticle)
    );
}

// =============================================================================
// Japanese @particle Transform Tests
// =============================================================================

// -----------------------------------------------------------------------------
// Japanese @particle Particle Types
// -----------------------------------------------------------------------------

#[test]
fn japanese_particle_subj() {
    // "カード" + :subj -> "が"
    let phrase = Phrase::builder().text("カード".to_string()).build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::JapaneseParticle;
    let context = Value::String("subj".to_string());
    let result = transform.execute(&value, Some(&context), "ja").unwrap();
    assert_eq!(result, "が");
}

#[test]
fn japanese_particle_obj() {
    // "カード" + :obj -> "を"
    let phrase = Phrase::builder().text("カード".to_string()).build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::JapaneseParticle;
    let context = Value::String("obj".to_string());
    let result = transform.execute(&value, Some(&context), "ja").unwrap();
    assert_eq!(result, "を");
}

#[test]
fn japanese_particle_topic() {
    // "カード" + :topic -> "は"
    let phrase = Phrase::builder().text("カード".to_string()).build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::JapaneseParticle;
    let context = Value::String("topic".to_string());
    let result = transform.execute(&value, Some(&context), "ja").unwrap();
    assert_eq!(result, "は");
}

#[test]
fn japanese_particle_loc() {
    // "東京" + :loc -> "に"
    let phrase = Phrase::builder().text("東京".to_string()).build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::JapaneseParticle;
    let context = Value::String("loc".to_string());
    let result = transform.execute(&value, Some(&context), "ja").unwrap();
    assert_eq!(result, "に");
}

#[test]
fn japanese_particle_place() {
    // "学校" + :place -> "で"
    let phrase = Phrase::builder().text("学校".to_string()).build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::JapaneseParticle;
    let context = Value::String("place".to_string());
    let result = transform.execute(&value, Some(&context), "ja").unwrap();
    assert_eq!(result, "で");
}

#[test]
fn japanese_particle_dir() {
    // "東" + :dir -> "へ"
    let phrase = Phrase::builder().text("東".to_string()).build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::JapaneseParticle;
    let context = Value::String("dir".to_string());
    let result = transform.execute(&value, Some(&context), "ja").unwrap();
    assert_eq!(result, "へ");
}

#[test]
fn japanese_particle_from() {
    // "駅" + :from -> "から"
    let phrase = Phrase::builder().text("駅".to_string()).build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::JapaneseParticle;
    let context = Value::String("from".to_string());
    let result = transform.execute(&value, Some(&context), "ja").unwrap();
    assert_eq!(result, "から");
}

#[test]
fn japanese_particle_until() {
    // "家" + :until -> "まで"
    let phrase = Phrase::builder().text("家".to_string()).build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::JapaneseParticle;
    let context = Value::String("until".to_string());
    let result = transform.execute(&value, Some(&context), "ja").unwrap();
    assert_eq!(result, "まで");
}

// -----------------------------------------------------------------------------
// Japanese @particle Default and Edge Cases
// -----------------------------------------------------------------------------

#[test]
fn japanese_particle_default() {
    // No context -> defaults to Subject particle が
    let phrase = Phrase::builder().text("猫".to_string()).build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::JapaneseParticle;
    let result = transform.execute(&value, None, "ja").unwrap();
    assert_eq!(result, "が");
}

#[test]
fn japanese_particle_unknown_context() {
    // Unknown context string -> defaults to Subject particle が
    let phrase = Phrase::builder().text("猫".to_string()).build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::JapaneseParticle;
    let context = Value::String("unknown".to_string());
    let result = transform.execute(&value, Some(&context), "ja").unwrap();
    assert_eq!(result, "が");
}

#[test]
fn japanese_particle_english_text() {
    // English text works the same since Japanese particles are phonology-independent
    let phrase = Phrase::builder().text("card".to_string()).build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::JapaneseParticle;
    let context = Value::String("obj".to_string());
    let result = transform.execute(&value, Some(&context), "ja").unwrap();
    assert_eq!(result, "を");
}

// -----------------------------------------------------------------------------
// Japanese @particle Registry Test
// -----------------------------------------------------------------------------

#[test]
fn japanese_particle_registered() {
    let registry = TransformRegistry::new();
    assert!(registry.get("particle", "ja").is_some());
    assert_eq!(
        registry.get("particle", "ja"),
        Some(TransformKind::JapaneseParticle)
    );
}

// =============================================================================
// Turkish @inflect Transform Tests (Phase 9)
// =============================================================================

// -----------------------------------------------------------------------------
// Turkish @inflect - Basic suffixes
// -----------------------------------------------------------------------------

#[test]
fn turkish_inflect_plural_front() {
    // :front "ev" (house) + :pl -> "evler"
    let phrase = Phrase::builder()
        .text("ev".to_string())
        .tags(vec![Tag::new("front")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::TurkishInflect;
    let context = Value::String("pl".to_string());
    let result = transform.execute(&value, Some(&context), "tr").unwrap();
    assert_eq!(result, "evler");
}

#[test]
fn turkish_inflect_plural_back() {
    // :back "at" (horse) + :pl -> "atlar"
    let phrase = Phrase::builder()
        .text("at".to_string())
        .tags(vec![Tag::new("back")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::TurkishInflect;
    let context = Value::String("pl".to_string());
    let result = transform.execute(&value, Some(&context), "tr").unwrap();
    assert_eq!(result, "atlar");
}

#[test]
fn turkish_inflect_dative_front() {
    // :front "ev" + :dat -> "eve"
    let phrase = Phrase::builder()
        .text("ev".to_string())
        .tags(vec![Tag::new("front")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::TurkishInflect;
    let context = Value::String("dat".to_string());
    let result = transform.execute(&value, Some(&context), "tr").unwrap();
    assert_eq!(result, "eve");
}

#[test]
fn turkish_inflect_dative_back() {
    // :back "at" + :dat -> "ata"
    let phrase = Phrase::builder()
        .text("at".to_string())
        .tags(vec![Tag::new("back")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::TurkishInflect;
    let context = Value::String("dat".to_string());
    let result = transform.execute(&value, Some(&context), "tr").unwrap();
    assert_eq!(result, "ata");
}

// -----------------------------------------------------------------------------
// Turkish @inflect - Suffix chains
// -----------------------------------------------------------------------------

#[test]
fn turkish_inflect_pl_dat() {
    // :front "ev" + :pl.dat -> "evlere"
    let phrase = Phrase::builder()
        .text("ev".to_string())
        .tags(vec![Tag::new("front")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::TurkishInflect;
    let context = Value::String("pl.dat".to_string());
    let result = transform.execute(&value, Some(&context), "tr").unwrap();
    assert_eq!(result, "evlere");
}

#[test]
fn turkish_inflect_abl_front() {
    // :front "ev" + :abl -> "evden"
    let phrase = Phrase::builder()
        .text("ev".to_string())
        .tags(vec![Tag::new("front")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::TurkishInflect;
    let context = Value::String("abl".to_string());
    let result = transform.execute(&value, Some(&context), "tr").unwrap();
    assert_eq!(result, "evden");
}

#[test]
fn turkish_inflect_abl_back() {
    // :back "at" + :abl -> "atdan"
    let phrase = Phrase::builder()
        .text("at".to_string())
        .tags(vec![Tag::new("back")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::TurkishInflect;
    let context = Value::String("abl".to_string());
    let result = transform.execute(&value, Some(&context), "tr").unwrap();
    assert_eq!(result, "atdan");
}

#[test]
fn turkish_inflect_loc_front() {
    // :front "ev" + :loc -> "evde"
    let phrase = Phrase::builder()
        .text("ev".to_string())
        .tags(vec![Tag::new("front")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::TurkishInflect;
    let context = Value::String("loc".to_string());
    let result = transform.execute(&value, Some(&context), "tr").unwrap();
    assert_eq!(result, "evde");
}

#[test]
fn turkish_inflect_loc_back() {
    // :back "at" + :loc -> "atda"
    let phrase = Phrase::builder()
        .text("at".to_string())
        .tags(vec![Tag::new("back")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::TurkishInflect;
    let context = Value::String("loc".to_string());
    let result = transform.execute(&value, Some(&context), "tr").unwrap();
    assert_eq!(result, "atda");
}

// -----------------------------------------------------------------------------
// Turkish @inflect - Error cases
// -----------------------------------------------------------------------------

#[test]
fn turkish_inflect_missing_harmony() {
    // Phrase without :front/:back returns MissingTag error
    let phrase = Phrase::builder().text("ev".to_string()).build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::TurkishInflect;
    let context = Value::String("pl".to_string());
    let result = transform.execute(&value, Some(&context), "tr");
    assert!(matches!(result, Err(EvalError::MissingTag { .. })));
}

#[test]
fn turkish_inflect_no_context() {
    // No context -> no suffixes applied, returns original word
    let phrase = Phrase::builder()
        .text("ev".to_string())
        .tags(vec![Tag::new("front")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::TurkishInflect;
    let result = transform.execute(&value, None, "tr").unwrap();
    assert_eq!(result, "ev");
}

// -----------------------------------------------------------------------------
// Turkish @inflect Registry Test
// -----------------------------------------------------------------------------

#[test]
fn turkish_inflect_registered() {
    let registry = TransformRegistry::new();
    assert!(registry.get("inflect", "tr").is_some());
    assert_eq!(
        registry.get("inflect", "tr"),
        Some(TransformKind::TurkishInflect)
    );
}

// -----------------------------------------------------------------------------
// Turkish @inflect - Nominative case (no suffix)
// -----------------------------------------------------------------------------

#[test]
fn turkish_inflect_nom_front() {
    // :front "ev" + :nom -> "ev" (nominative adds no suffix)
    let phrase = Phrase::builder()
        .text("ev".to_string())
        .tags(vec![Tag::new("front")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::TurkishInflect;
    let context = Value::String("nom".to_string());
    let result = transform.execute(&value, Some(&context), "tr").unwrap();
    assert_eq!(result, "ev");
}

#[test]
fn turkish_inflect_nom_back() {
    // :back "at" + :nom -> "at" (nominative adds no suffix)
    let phrase = Phrase::builder()
        .text("at".to_string())
        .tags(vec![Tag::new("back")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::TurkishInflect;
    let context = Value::String("nom".to_string());
    let result = transform.execute(&value, Some(&context), "tr").unwrap();
    assert_eq!(result, "at");
}

// -----------------------------------------------------------------------------
// Turkish @inflect - Accusative case
// -----------------------------------------------------------------------------

#[test]
fn turkish_inflect_acc_front() {
    // :front "ev" + :acc -> "evi"
    let phrase = Phrase::builder()
        .text("ev".to_string())
        .tags(vec![Tag::new("front")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::TurkishInflect;
    let context = Value::String("acc".to_string());
    let result = transform.execute(&value, Some(&context), "tr").unwrap();
    assert_eq!(result, "evi");
}

#[test]
fn turkish_inflect_acc_back() {
    // :back "at" + :acc -> "atı"
    let phrase = Phrase::builder()
        .text("at".to_string())
        .tags(vec![Tag::new("back")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::TurkishInflect;
    let context = Value::String("acc".to_string());
    let result = transform.execute(&value, Some(&context), "tr").unwrap();
    assert_eq!(result, "at\u{0131}");
}

// -----------------------------------------------------------------------------
// Turkish @inflect - Genitive case
// -----------------------------------------------------------------------------

#[test]
fn turkish_inflect_gen_front() {
    // :front "ev" + :gen -> "evin"
    let phrase = Phrase::builder()
        .text("ev".to_string())
        .tags(vec![Tag::new("front")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::TurkishInflect;
    let context = Value::String("gen".to_string());
    let result = transform.execute(&value, Some(&context), "tr").unwrap();
    assert_eq!(result, "evin");
}

#[test]
fn turkish_inflect_gen_back() {
    // :back "at" + :gen -> "atın"
    let phrase = Phrase::builder()
        .text("at".to_string())
        .tags(vec![Tag::new("back")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::TurkishInflect;
    let context = Value::String("gen".to_string());
    let result = transform.execute(&value, Some(&context), "tr").unwrap();
    assert_eq!(result, "at\u{0131}n");
}

// -----------------------------------------------------------------------------
// Turkish @inflect - Possessive suffixes
// -----------------------------------------------------------------------------

#[test]
fn turkish_inflect_poss1sg_front() {
    // :front "ev" + :poss1sg -> "evim" (my house)
    let phrase = Phrase::builder()
        .text("ev".to_string())
        .tags(vec![Tag::new("front")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::TurkishInflect;
    let context = Value::String("poss1sg".to_string());
    let result = transform.execute(&value, Some(&context), "tr").unwrap();
    assert_eq!(result, "evim");
}

#[test]
fn turkish_inflect_poss1sg_back() {
    // :back "at" + :poss1sg -> "atım" (my horse)
    let phrase = Phrase::builder()
        .text("at".to_string())
        .tags(vec![Tag::new("back")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::TurkishInflect;
    let context = Value::String("poss1sg".to_string());
    let result = transform.execute(&value, Some(&context), "tr").unwrap();
    assert_eq!(result, "at\u{0131}m");
}

#[test]
fn turkish_inflect_poss2sg_front() {
    // :front "ev" + :poss2sg -> "evin" (your house)
    let phrase = Phrase::builder()
        .text("ev".to_string())
        .tags(vec![Tag::new("front")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::TurkishInflect;
    let context = Value::String("poss2sg".to_string());
    let result = transform.execute(&value, Some(&context), "tr").unwrap();
    assert_eq!(result, "evin");
}

#[test]
fn turkish_inflect_poss2sg_back() {
    // :back "at" + :poss2sg -> "atın" (your horse)
    let phrase = Phrase::builder()
        .text("at".to_string())
        .tags(vec![Tag::new("back")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::TurkishInflect;
    let context = Value::String("poss2sg".to_string());
    let result = transform.execute(&value, Some(&context), "tr").unwrap();
    assert_eq!(result, "at\u{0131}n");
}

#[test]
fn turkish_inflect_poss3sg_front() {
    // :front "ev" + :poss3sg -> "evi" (his/her house)
    let phrase = Phrase::builder()
        .text("ev".to_string())
        .tags(vec![Tag::new("front")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::TurkishInflect;
    let context = Value::String("poss3sg".to_string());
    let result = transform.execute(&value, Some(&context), "tr").unwrap();
    assert_eq!(result, "evi");
}

#[test]
fn turkish_inflect_poss3sg_back() {
    // :back "at" + :poss3sg -> "atı" (his/her horse)
    let phrase = Phrase::builder()
        .text("at".to_string())
        .tags(vec![Tag::new("back")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::TurkishInflect;
    let context = Value::String("poss3sg".to_string());
    let result = transform.execute(&value, Some(&context), "tr").unwrap();
    assert_eq!(result, "at\u{0131}");
}

#[test]
fn turkish_inflect_poss1pl_front() {
    // :front "ev" + :poss1pl -> "evimiz" (our house)
    let phrase = Phrase::builder()
        .text("ev".to_string())
        .tags(vec![Tag::new("front")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::TurkishInflect;
    let context = Value::String("poss1pl".to_string());
    let result = transform.execute(&value, Some(&context), "tr").unwrap();
    assert_eq!(result, "evimiz");
}

#[test]
fn turkish_inflect_poss1pl_back() {
    // :back "at" + :poss1pl -> "atımız" (our horse)
    let phrase = Phrase::builder()
        .text("at".to_string())
        .tags(vec![Tag::new("back")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::TurkishInflect;
    let context = Value::String("poss1pl".to_string());
    let result = transform.execute(&value, Some(&context), "tr").unwrap();
    assert_eq!(result, "at\u{0131}m\u{0131}z");
}

#[test]
fn turkish_inflect_poss2pl_front() {
    // :front "ev" + :poss2pl -> "eviniz" (your (pl.) house)
    let phrase = Phrase::builder()
        .text("ev".to_string())
        .tags(vec![Tag::new("front")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::TurkishInflect;
    let context = Value::String("poss2pl".to_string());
    let result = transform.execute(&value, Some(&context), "tr").unwrap();
    assert_eq!(result, "eviniz");
}

#[test]
fn turkish_inflect_poss2pl_back() {
    // :back "at" + :poss2pl -> "atınız" (your (pl.) horse)
    let phrase = Phrase::builder()
        .text("at".to_string())
        .tags(vec![Tag::new("back")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::TurkishInflect;
    let context = Value::String("poss2pl".to_string());
    let result = transform.execute(&value, Some(&context), "tr").unwrap();
    assert_eq!(result, "at\u{0131}n\u{0131}z");
}

#[test]
fn turkish_inflect_poss3pl_front() {
    // :front "ev" + :poss3pl -> "evleri" (their house)
    let phrase = Phrase::builder()
        .text("ev".to_string())
        .tags(vec![Tag::new("front")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::TurkishInflect;
    let context = Value::String("poss3pl".to_string());
    let result = transform.execute(&value, Some(&context), "tr").unwrap();
    assert_eq!(result, "evleri");
}

#[test]
fn turkish_inflect_poss3pl_back() {
    // :back "at" + :poss3pl -> "atları" (their horse)
    let phrase = Phrase::builder()
        .text("at".to_string())
        .tags(vec![Tag::new("back")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::TurkishInflect;
    let context = Value::String("poss3pl".to_string());
    let result = transform.execute(&value, Some(&context), "tr").unwrap();
    assert_eq!(result, "atlar\u{0131}");
}

// -----------------------------------------------------------------------------
// Turkish @inflect - Complex suffix chains from documentation
// -----------------------------------------------------------------------------

#[test]
fn turkish_inflect_pl_poss1sg_abl_front() {
    // :front "ev" + :pl.poss1sg.abl -> "evlerimden" (from my houses)
    // This is the example from APPENDIX_STDLIB.md
    let phrase = Phrase::builder()
        .text("ev".to_string())
        .tags(vec![Tag::new("front")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::TurkishInflect;
    let context = Value::String("pl.poss1sg.abl".to_string());
    let result = transform.execute(&value, Some(&context), "tr").unwrap();
    assert_eq!(result, "evlerimden");
}

#[test]
fn turkish_inflect_pl_poss1sg_abl_back() {
    // :back "at" + :pl.poss1sg.abl -> "atlarımdan" (from my horses)
    let phrase = Phrase::builder()
        .text("at".to_string())
        .tags(vec![Tag::new("back")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::TurkishInflect;
    let context = Value::String("pl.poss1sg.abl".to_string());
    let result = transform.execute(&value, Some(&context), "tr").unwrap();
    assert_eq!(result, "atlar\u{0131}mdan");
}

#[test]
fn turkish_inflect_poss1sg_gen_front() {
    // :front "ev" + :poss1sg.gen -> "evimin" (of my house)
    let phrase = Phrase::builder()
        .text("ev".to_string())
        .tags(vec![Tag::new("front")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::TurkishInflect;
    let context = Value::String("poss1sg.gen".to_string());
    let result = transform.execute(&value, Some(&context), "tr").unwrap();
    assert_eq!(result, "evimin");
}

#[test]
fn turkish_inflect_pl_acc_front() {
    // :front "ev" + :pl.acc -> "evleri"
    let phrase = Phrase::builder()
        .text("ev".to_string())
        .tags(vec![Tag::new("front")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::TurkishInflect;
    let context = Value::String("pl.acc".to_string());
    let result = transform.execute(&value, Some(&context), "tr").unwrap();
    assert_eq!(result, "evleri");
}

#[test]
fn turkish_inflect_pl_gen_back() {
    // :back "at" + :pl.gen -> "atların"
    let phrase = Phrase::builder()
        .text("at".to_string())
        .tags(vec![Tag::new("back")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::TurkishInflect;
    let context = Value::String("pl.gen".to_string());
    let result = transform.execute(&value, Some(&context), "tr").unwrap();
    assert_eq!(result, "atlar\u{0131}n");
}

#[test]
fn turkish_inflect_poss2pl_loc_front() {
    // :front "göz" + :poss2pl.loc -> "gözinizde" (in your (pl.) eye)
    let phrase = Phrase::builder()
        .text("g\u{00f6}z".to_string())
        .tags(vec![Tag::new("front")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::TurkishInflect;
    let context = Value::String("poss2pl.loc".to_string());
    let result = transform.execute(&value, Some(&context), "tr").unwrap();
    assert_eq!(result, "g\u{00f6}zinizde");
}

// -----------------------------------------------------------------------------
// Turkish @inflect - Unknown suffix names ignored
// -----------------------------------------------------------------------------

#[test]
fn turkish_inflect_unknown_suffix_ignored() {
    // Unknown suffixes in chain are silently ignored
    let phrase = Phrase::builder()
        .text("ev".to_string())
        .tags(vec![Tag::new("front")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::TurkishInflect;
    let context = Value::String("pl.invalid.dat".to_string());
    let result = transform.execute(&value, Some(&context), "tr").unwrap();
    assert_eq!(result, "evlere");
}

// =============================================================================
// Finnish @inflect Transform Tests
// =============================================================================

// -----------------------------------------------------------------------------
// Finnish @inflect - Registry
// -----------------------------------------------------------------------------

#[test]
fn finnish_inflect_registered() {
    let registry = TransformRegistry::new();
    assert!(registry.get("inflect", "fi").is_some());
    assert_eq!(
        registry.get("inflect", "fi"),
        Some(TransformKind::FinnishInflect)
    );
}

// -----------------------------------------------------------------------------
// Finnish @inflect - Nominative (no suffix)
// -----------------------------------------------------------------------------

#[test]
fn finnish_inflect_nom_front() {
    // :front "talo" + :nom -> "talo" (nominative adds no suffix)
    // Note: "talo" is actually back-vowel, but we test tag-driven behavior
    let phrase = Phrase::builder()
        .text("pöytä".to_string())
        .tags(vec![Tag::new("front")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::FinnishInflect;
    let context = Value::String("nom".to_string());
    let result = transform.execute(&value, Some(&context), "fi").unwrap();
    assert_eq!(result, "pöytä");
}

#[test]
fn finnish_inflect_nom_back() {
    // :back "talo" + :nom -> "talo"
    let phrase = Phrase::builder()
        .text("talo".to_string())
        .tags(vec![Tag::new("back")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::FinnishInflect;
    let context = Value::String("nom".to_string());
    let result = transform.execute(&value, Some(&context), "fi").unwrap();
    assert_eq!(result, "talo");
}

// -----------------------------------------------------------------------------
// Finnish @inflect - Genitive (-n)
// -----------------------------------------------------------------------------

#[test]
fn finnish_inflect_gen_front() {
    // :front "pöytä" + :gen -> "pöytän"
    let phrase = Phrase::builder()
        .text("pöytä".to_string())
        .tags(vec![Tag::new("front")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::FinnishInflect;
    let context = Value::String("gen".to_string());
    let result = transform.execute(&value, Some(&context), "fi").unwrap();
    assert_eq!(result, "pöytän");
}

#[test]
fn finnish_inflect_gen_back() {
    // :back "talo" + :gen -> "talon"
    let phrase = Phrase::builder()
        .text("talo".to_string())
        .tags(vec![Tag::new("back")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::FinnishInflect;
    let context = Value::String("gen".to_string());
    let result = transform.execute(&value, Some(&context), "fi").unwrap();
    assert_eq!(result, "talon");
}

// -----------------------------------------------------------------------------
// Finnish @inflect - Partitive (-a/-ä)
// -----------------------------------------------------------------------------

#[test]
fn finnish_inflect_par_front() {
    // :front "pöytä" + :par -> "pöytää" (partitive with front harmony)
    let phrase = Phrase::builder()
        .text("pöytä".to_string())
        .tags(vec![Tag::new("front")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::FinnishInflect;
    let context = Value::String("par".to_string());
    let result = transform.execute(&value, Some(&context), "fi").unwrap();
    assert_eq!(result, "pöytää");
}

#[test]
fn finnish_inflect_par_back() {
    // :back "talo" + :par -> "taloa"
    let phrase = Phrase::builder()
        .text("talo".to_string())
        .tags(vec![Tag::new("back")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::FinnishInflect;
    let context = Value::String("par".to_string());
    let result = transform.execute(&value, Some(&context), "fi").unwrap();
    assert_eq!(result, "taloa");
}

// -----------------------------------------------------------------------------
// Finnish @inflect - Inessive (-ssa/-ssä)
// -----------------------------------------------------------------------------

#[test]
fn finnish_inflect_ine_front() {
    // :front "pöytä" + :ine -> "pöytässä"
    let phrase = Phrase::builder()
        .text("pöytä".to_string())
        .tags(vec![Tag::new("front")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::FinnishInflect;
    let context = Value::String("ine".to_string());
    let result = transform.execute(&value, Some(&context), "fi").unwrap();
    assert_eq!(result, "pöytässä");
}

#[test]
fn finnish_inflect_ine_back() {
    // :back "talo" + :ine -> "talossa"
    let phrase = Phrase::builder()
        .text("talo".to_string())
        .tags(vec![Tag::new("back")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::FinnishInflect;
    let context = Value::String("ine".to_string());
    let result = transform.execute(&value, Some(&context), "fi").unwrap();
    assert_eq!(result, "talossa");
}

// -----------------------------------------------------------------------------
// Finnish @inflect - Elative (-sta/-stä)
// -----------------------------------------------------------------------------

#[test]
fn finnish_inflect_ela_front() {
    // :front "pöytä" + :ela -> "pöytästä"
    let phrase = Phrase::builder()
        .text("pöytä".to_string())
        .tags(vec![Tag::new("front")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::FinnishInflect;
    let context = Value::String("ela".to_string());
    let result = transform.execute(&value, Some(&context), "fi").unwrap();
    assert_eq!(result, "pöytästä");
}

#[test]
fn finnish_inflect_ela_back() {
    // :back "talo" + :ela -> "talosta"
    let phrase = Phrase::builder()
        .text("talo".to_string())
        .tags(vec![Tag::new("back")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::FinnishInflect;
    let context = Value::String("ela".to_string());
    let result = transform.execute(&value, Some(&context), "fi").unwrap();
    assert_eq!(result, "talosta");
}

// -----------------------------------------------------------------------------
// Finnish @inflect - Illative (vowel lengthening + -n)
// -----------------------------------------------------------------------------

#[test]
fn finnish_inflect_ill_back() {
    // :back "talo" + :ill -> "taloon" (last vowel duplicated + n)
    let phrase = Phrase::builder()
        .text("talo".to_string())
        .tags(vec![Tag::new("back")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::FinnishInflect;
    let context = Value::String("ill".to_string());
    let result = transform.execute(&value, Some(&context), "fi").unwrap();
    assert_eq!(result, "taloon");
}

#[test]
fn finnish_inflect_ill_front() {
    // :front "pöytä" + :ill -> "pöytään"
    let phrase = Phrase::builder()
        .text("pöytä".to_string())
        .tags(vec![Tag::new("front")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::FinnishInflect;
    let context = Value::String("ill".to_string());
    let result = transform.execute(&value, Some(&context), "fi").unwrap();
    assert_eq!(result, "pöytään");
}

// -----------------------------------------------------------------------------
// Finnish @inflect - Adessive (-lla/-llä)
// -----------------------------------------------------------------------------

#[test]
fn finnish_inflect_ade_front() {
    // :front "pöytä" + :ade -> "pöytällä"
    let phrase = Phrase::builder()
        .text("pöytä".to_string())
        .tags(vec![Tag::new("front")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::FinnishInflect;
    let context = Value::String("ade".to_string());
    let result = transform.execute(&value, Some(&context), "fi").unwrap();
    assert_eq!(result, "pöytällä");
}

#[test]
fn finnish_inflect_ade_back() {
    // :back "talo" + :ade -> "talolla"
    let phrase = Phrase::builder()
        .text("talo".to_string())
        .tags(vec![Tag::new("back")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::FinnishInflect;
    let context = Value::String("ade".to_string());
    let result = transform.execute(&value, Some(&context), "fi").unwrap();
    assert_eq!(result, "talolla");
}

// -----------------------------------------------------------------------------
// Finnish @inflect - Ablative (-lta/-ltä)
// -----------------------------------------------------------------------------

#[test]
fn finnish_inflect_abl_front() {
    // :front "pöytä" + :abl -> "pöytältä"
    let phrase = Phrase::builder()
        .text("pöytä".to_string())
        .tags(vec![Tag::new("front")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::FinnishInflect;
    let context = Value::String("abl".to_string());
    let result = transform.execute(&value, Some(&context), "fi").unwrap();
    assert_eq!(result, "pöytältä");
}

#[test]
fn finnish_inflect_abl_back() {
    // :back "talo" + :abl -> "talolta"
    let phrase = Phrase::builder()
        .text("talo".to_string())
        .tags(vec![Tag::new("back")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::FinnishInflect;
    let context = Value::String("abl".to_string());
    let result = transform.execute(&value, Some(&context), "fi").unwrap();
    assert_eq!(result, "talolta");
}

// -----------------------------------------------------------------------------
// Finnish @inflect - Allative (-lle)
// -----------------------------------------------------------------------------

#[test]
fn finnish_inflect_all_front() {
    // :front "pöytä" + :all -> "pöytälle"
    let phrase = Phrase::builder()
        .text("pöytä".to_string())
        .tags(vec![Tag::new("front")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::FinnishInflect;
    let context = Value::String("all".to_string());
    let result = transform.execute(&value, Some(&context), "fi").unwrap();
    assert_eq!(result, "pöytälle");
}

#[test]
fn finnish_inflect_all_back() {
    // :back "talo" + :all -> "talolle"
    let phrase = Phrase::builder()
        .text("talo".to_string())
        .tags(vec![Tag::new("back")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::FinnishInflect;
    let context = Value::String("all".to_string());
    let result = transform.execute(&value, Some(&context), "fi").unwrap();
    assert_eq!(result, "talolle");
}

// -----------------------------------------------------------------------------
// Finnish @inflect - Essive (-na/-nä)
// -----------------------------------------------------------------------------

#[test]
fn finnish_inflect_ess_front() {
    // :front "pöytä" + :ess -> "pöytänä"
    let phrase = Phrase::builder()
        .text("pöytä".to_string())
        .tags(vec![Tag::new("front")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::FinnishInflect;
    let context = Value::String("ess".to_string());
    let result = transform.execute(&value, Some(&context), "fi").unwrap();
    assert_eq!(result, "pöytänä");
}

#[test]
fn finnish_inflect_ess_back() {
    // :back "talo" + :ess -> "talona"
    let phrase = Phrase::builder()
        .text("talo".to_string())
        .tags(vec![Tag::new("back")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::FinnishInflect;
    let context = Value::String("ess".to_string());
    let result = transform.execute(&value, Some(&context), "fi").unwrap();
    assert_eq!(result, "talona");
}

// -----------------------------------------------------------------------------
// Finnish @inflect - Translative (-ksi)
// -----------------------------------------------------------------------------

#[test]
fn finnish_inflect_tra_front() {
    // :front "pöytä" + :tra -> "pöytäksi"
    let phrase = Phrase::builder()
        .text("pöytä".to_string())
        .tags(vec![Tag::new("front")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::FinnishInflect;
    let context = Value::String("tra".to_string());
    let result = transform.execute(&value, Some(&context), "fi").unwrap();
    assert_eq!(result, "pöytäksi");
}

#[test]
fn finnish_inflect_tra_back() {
    // :back "talo" + :tra -> "taloksi"
    let phrase = Phrase::builder()
        .text("talo".to_string())
        .tags(vec![Tag::new("back")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::FinnishInflect;
    let context = Value::String("tra".to_string());
    let result = transform.execute(&value, Some(&context), "fi").unwrap();
    assert_eq!(result, "taloksi");
}

// -----------------------------------------------------------------------------
// Finnish @inflect - Accusative (-n)
// -----------------------------------------------------------------------------

#[test]
fn finnish_inflect_acc_back() {
    // :back "talo" + :acc -> "talon"
    let phrase = Phrase::builder()
        .text("talo".to_string())
        .tags(vec![Tag::new("back")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::FinnishInflect;
    let context = Value::String("acc".to_string());
    let result = transform.execute(&value, Some(&context), "fi").unwrap();
    assert_eq!(result, "talon");
}

// -----------------------------------------------------------------------------
// Finnish @inflect - Plural marker
// -----------------------------------------------------------------------------

#[test]
fn finnish_inflect_pl_back() {
    // :back "talo" + :pl -> "talot" (nominative plural)
    let phrase = Phrase::builder()
        .text("talo".to_string())
        .tags(vec![Tag::new("back")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::FinnishInflect;
    let context = Value::String("pl".to_string());
    let result = transform.execute(&value, Some(&context), "fi").unwrap();
    assert_eq!(result, "talot");
}

#[test]
fn finnish_inflect_pl_front() {
    // :front "pöytä" + :pl -> "pöytät"
    let phrase = Phrase::builder()
        .text("pöytä".to_string())
        .tags(vec![Tag::new("front")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::FinnishInflect;
    let context = Value::String("pl".to_string());
    let result = transform.execute(&value, Some(&context), "fi").unwrap();
    assert_eq!(result, "pöytät");
}

// -----------------------------------------------------------------------------
// Finnish @inflect - Possessive suffixes
// -----------------------------------------------------------------------------

#[test]
fn finnish_inflect_poss1sg() {
    // :back "talo" + :poss1sg -> "taloni"
    let phrase = Phrase::builder()
        .text("talo".to_string())
        .tags(vec![Tag::new("back")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::FinnishInflect;
    let context = Value::String("poss1sg".to_string());
    let result = transform.execute(&value, Some(&context), "fi").unwrap();
    assert_eq!(result, "taloni");
}

#[test]
fn finnish_inflect_poss2sg() {
    // :back "talo" + :poss2sg -> "talosi"
    let phrase = Phrase::builder()
        .text("talo".to_string())
        .tags(vec![Tag::new("back")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::FinnishInflect;
    let context = Value::String("poss2sg".to_string());
    let result = transform.execute(&value, Some(&context), "fi").unwrap();
    assert_eq!(result, "talosi");
}

#[test]
fn finnish_inflect_poss3sg_back() {
    // :back "talo" + :poss3sg -> "talonsa"
    let phrase = Phrase::builder()
        .text("talo".to_string())
        .tags(vec![Tag::new("back")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::FinnishInflect;
    let context = Value::String("poss3sg".to_string());
    let result = transform.execute(&value, Some(&context), "fi").unwrap();
    assert_eq!(result, "talonsa");
}

#[test]
fn finnish_inflect_poss3sg_front() {
    // :front "pöytä" + :poss3sg -> "pöytänsä"
    let phrase = Phrase::builder()
        .text("pöytä".to_string())
        .tags(vec![Tag::new("front")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::FinnishInflect;
    let context = Value::String("poss3sg".to_string());
    let result = transform.execute(&value, Some(&context), "fi").unwrap();
    assert_eq!(result, "pöytänsä");
}

#[test]
fn finnish_inflect_poss1pl() {
    // :back "talo" + :poss1pl -> "talomme"
    let phrase = Phrase::builder()
        .text("talo".to_string())
        .tags(vec![Tag::new("back")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::FinnishInflect;
    let context = Value::String("poss1pl".to_string());
    let result = transform.execute(&value, Some(&context), "fi").unwrap();
    assert_eq!(result, "talomme");
}

#[test]
fn finnish_inflect_poss2pl() {
    // :back "talo" + :poss2pl -> "talonne"
    let phrase = Phrase::builder()
        .text("talo".to_string())
        .tags(vec![Tag::new("back")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::FinnishInflect;
    let context = Value::String("poss2pl".to_string());
    let result = transform.execute(&value, Some(&context), "fi").unwrap();
    assert_eq!(result, "talonne");
}

#[test]
fn finnish_inflect_poss3pl_back() {
    // :back "talo" + :poss3pl -> "talonsa"
    let phrase = Phrase::builder()
        .text("talo".to_string())
        .tags(vec![Tag::new("back")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::FinnishInflect;
    let context = Value::String("poss3pl".to_string());
    let result = transform.execute(&value, Some(&context), "fi").unwrap();
    assert_eq!(result, "talonsa");
}

#[test]
fn finnish_inflect_poss3pl_front() {
    // :front "pöytä" + :poss3pl -> "pöytänsä"
    let phrase = Phrase::builder()
        .text("pöytä".to_string())
        .tags(vec![Tag::new("front")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::FinnishInflect;
    let context = Value::String("poss3pl".to_string());
    let result = transform.execute(&value, Some(&context), "fi").unwrap();
    assert_eq!(result, "pöytänsä");
}

// -----------------------------------------------------------------------------
// Finnish @inflect - Suffix chains
// -----------------------------------------------------------------------------

#[test]
fn finnish_inflect_pl_ine_back() {
    // :back "talo" + :pl.ine -> "talotssa" (plural + inessive)
    let phrase = Phrase::builder()
        .text("talo".to_string())
        .tags(vec![Tag::new("back")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::FinnishInflect;
    let context = Value::String("pl.ine".to_string());
    let result = transform.execute(&value, Some(&context), "fi").unwrap();
    assert_eq!(result, "talotssa");
}

#[test]
fn finnish_inflect_gen_poss1sg() {
    // :back "talo" + :gen.poss1sg -> "talonni"
    let phrase = Phrase::builder()
        .text("talo".to_string())
        .tags(vec![Tag::new("back")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::FinnishInflect;
    let context = Value::String("gen.poss1sg".to_string());
    let result = transform.execute(&value, Some(&context), "fi").unwrap();
    assert_eq!(result, "talonni");
}

#[test]
fn finnish_inflect_ela_poss3sg_back() {
    // :back "talo" + :ela.poss3sg -> "talostansa"
    let phrase = Phrase::builder()
        .text("talo".to_string())
        .tags(vec![Tag::new("back")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::FinnishInflect;
    let context = Value::String("ela.poss3sg".to_string());
    let result = transform.execute(&value, Some(&context), "fi").unwrap();
    assert_eq!(result, "talostansa");
}

#[test]
fn finnish_inflect_ela_poss3sg_front() {
    // :front "pöytä" + :ela.poss3sg -> "pöytästänsä"
    let phrase = Phrase::builder()
        .text("pöytä".to_string())
        .tags(vec![Tag::new("front")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::FinnishInflect;
    let context = Value::String("ela.poss3sg".to_string());
    let result = transform.execute(&value, Some(&context), "fi").unwrap();
    assert_eq!(result, "pöytästänsä");
}

#[test]
fn finnish_inflect_ill_poss1sg_back() {
    // :back "talo" + :ill.poss1sg -> "taloonni" (illative then possessive)
    let phrase = Phrase::builder()
        .text("talo".to_string())
        .tags(vec![Tag::new("back")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::FinnishInflect;
    let context = Value::String("ill.poss1sg".to_string());
    let result = transform.execute(&value, Some(&context), "fi").unwrap();
    assert_eq!(result, "taloonni");
}

// -----------------------------------------------------------------------------
// Finnish @inflect - Error cases
// -----------------------------------------------------------------------------

#[test]
fn finnish_inflect_missing_harmony() {
    // Phrase without :front/:back returns MissingTag error
    let phrase = Phrase::builder().text("talo".to_string()).build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::FinnishInflect;
    let context = Value::String("gen".to_string());
    let result = transform.execute(&value, Some(&context), "fi");
    assert!(matches!(result, Err(EvalError::MissingTag { .. })));
}

#[test]
fn finnish_inflect_no_context() {
    // No context -> no suffixes, returns original word
    let phrase = Phrase::builder()
        .text("talo".to_string())
        .tags(vec![Tag::new("back")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::FinnishInflect;
    let result = transform.execute(&value, None, "fi").unwrap();
    assert_eq!(result, "talo");
}

#[test]
fn finnish_inflect_unknown_suffix_ignored() {
    // Unknown suffixes in chain are silently ignored
    let phrase = Phrase::builder()
        .text("talo".to_string())
        .tags(vec![Tag::new("back")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::FinnishInflect;
    let context = Value::String("gen.invalid.ine".to_string());
    let result = transform.execute(&value, Some(&context), "fi").unwrap();
    assert_eq!(result, "talonssa");
}

// -----------------------------------------------------------------------------
// Finnish @inflect - Not registered for other languages
// -----------------------------------------------------------------------------

#[test]
fn finnish_inflect_not_registered_for_english() {
    let registry = TransformRegistry::new();
    assert!(registry.get("inflect", "en").is_none());
}

// -----------------------------------------------------------------------------
// Finnish @inflect - Illative with consonant ending
// -----------------------------------------------------------------------------

#[test]
fn finnish_inflect_ill_consonant_ending() {
    // Word ending in consonant: append last vowel + n
    let phrase = Phrase::builder()
        .text("maan".to_string())
        .tags(vec![Tag::new("back")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::FinnishInflect;
    let context = Value::String("ill".to_string());
    let result = transform.execute(&value, Some(&context), "fi").unwrap();
    assert_eq!(result, "maanan");
}

// =============================================================================
// Hungarian @inflect Transform
// =============================================================================

// -----------------------------------------------------------------------------
// Hungarian @inflect - Registry
// -----------------------------------------------------------------------------

#[test]
fn hungarian_inflect_registered() {
    let registry = TransformRegistry::new();
    assert!(registry.get("inflect", "hu").is_some());
    assert_eq!(
        registry.get("inflect", "hu"),
        Some(TransformKind::HungarianInflect)
    );
}

// -----------------------------------------------------------------------------
// Hungarian @inflect - Nominative (no suffix)
// -----------------------------------------------------------------------------

#[test]
fn hungarian_inflect_nom_back() {
    // :back "ház" + :nom -> "ház"
    let phrase = Phrase::builder()
        .text("ház".to_string())
        .tags(vec![Tag::new("back")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::HungarianInflect;
    let context = Value::String("nom".to_string());
    let result = transform.execute(&value, Some(&context), "hu").unwrap();
    assert_eq!(result, "ház");
}

#[test]
fn hungarian_inflect_nom_front() {
    // :front "kert" + :nom -> "kert"
    let phrase = Phrase::builder()
        .text("kert".to_string())
        .tags(vec![Tag::new("front")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::HungarianInflect;
    let context = Value::String("nom".to_string());
    let result = transform.execute(&value, Some(&context), "hu").unwrap();
    assert_eq!(result, "kert");
}

#[test]
fn hungarian_inflect_nom_round() {
    // :round "tükör" + :nom -> "tükör"
    let phrase = Phrase::builder()
        .text("tükör".to_string())
        .tags(vec![Tag::new("round")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::HungarianInflect;
    let context = Value::String("nom".to_string());
    let result = transform.execute(&value, Some(&context), "hu").unwrap();
    assert_eq!(result, "tükör");
}

// -----------------------------------------------------------------------------
// Hungarian @inflect - Plural (-ok/-ek/-ök)
// -----------------------------------------------------------------------------

#[test]
fn hungarian_inflect_pl_back() {
    // :back "ház" + :pl -> "házok"
    let phrase = Phrase::builder()
        .text("ház".to_string())
        .tags(vec![Tag::new("back")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::HungarianInflect;
    let context = Value::String("pl".to_string());
    let result = transform.execute(&value, Some(&context), "hu").unwrap();
    assert_eq!(result, "házok");
}

#[test]
fn hungarian_inflect_pl_front() {
    // :front "kert" + :pl -> "kertek"
    let phrase = Phrase::builder()
        .text("kert".to_string())
        .tags(vec![Tag::new("front")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::HungarianInflect;
    let context = Value::String("pl".to_string());
    let result = transform.execute(&value, Some(&context), "hu").unwrap();
    assert_eq!(result, "kertek");
}

#[test]
fn hungarian_inflect_pl_round() {
    // :round "tükör" + :pl -> "tükörök"
    let phrase = Phrase::builder()
        .text("tükör".to_string())
        .tags(vec![Tag::new("round")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::HungarianInflect;
    let context = Value::String("pl".to_string());
    let result = transform.execute(&value, Some(&context), "hu").unwrap();
    assert_eq!(result, "tükörök");
}

// -----------------------------------------------------------------------------
// Hungarian @inflect - Accusative (-ot/-et/-öt)
// -----------------------------------------------------------------------------

#[test]
fn hungarian_inflect_acc_back() {
    // :back "ház" + :acc -> "házot"
    let phrase = Phrase::builder()
        .text("ház".to_string())
        .tags(vec![Tag::new("back")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::HungarianInflect;
    let context = Value::String("acc".to_string());
    let result = transform.execute(&value, Some(&context), "hu").unwrap();
    assert_eq!(result, "házot");
}

#[test]
fn hungarian_inflect_acc_front() {
    // :front "kert" + :acc -> "kertet"
    let phrase = Phrase::builder()
        .text("kert".to_string())
        .tags(vec![Tag::new("front")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::HungarianInflect;
    let context = Value::String("acc".to_string());
    let result = transform.execute(&value, Some(&context), "hu").unwrap();
    assert_eq!(result, "kertet");
}

#[test]
fn hungarian_inflect_acc_round() {
    // :round "tükör" + :acc -> "tüköröt"
    let phrase = Phrase::builder()
        .text("tükör".to_string())
        .tags(vec![Tag::new("round")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::HungarianInflect;
    let context = Value::String("acc".to_string());
    let result = transform.execute(&value, Some(&context), "hu").unwrap();
    assert_eq!(result, "tüköröt");
}

// -----------------------------------------------------------------------------
// Hungarian @inflect - Dative (-nak/-nek)
// -----------------------------------------------------------------------------

#[test]
fn hungarian_inflect_dat_back() {
    // :back "ház" + :dat -> "háznak"
    let phrase = Phrase::builder()
        .text("ház".to_string())
        .tags(vec![Tag::new("back")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::HungarianInflect;
    let context = Value::String("dat".to_string());
    let result = transform.execute(&value, Some(&context), "hu").unwrap();
    assert_eq!(result, "háznak");
}

#[test]
fn hungarian_inflect_dat_front() {
    // :front "kert" + :dat -> "kertnek"
    let phrase = Phrase::builder()
        .text("kert".to_string())
        .tags(vec![Tag::new("front")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::HungarianInflect;
    let context = Value::String("dat".to_string());
    let result = transform.execute(&value, Some(&context), "hu").unwrap();
    assert_eq!(result, "kertnek");
}

// -----------------------------------------------------------------------------
// Hungarian @inflect - Inessive (-ban/-ben)
// -----------------------------------------------------------------------------

#[test]
fn hungarian_inflect_ine_back() {
    // :back "ház" + :ine -> "házban"
    let phrase = Phrase::builder()
        .text("ház".to_string())
        .tags(vec![Tag::new("back")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::HungarianInflect;
    let context = Value::String("ine".to_string());
    let result = transform.execute(&value, Some(&context), "hu").unwrap();
    assert_eq!(result, "házban");
}

#[test]
fn hungarian_inflect_ine_front() {
    // :front "kert" + :ine -> "kertben"
    let phrase = Phrase::builder()
        .text("kert".to_string())
        .tags(vec![Tag::new("front")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::HungarianInflect;
    let context = Value::String("ine".to_string());
    let result = transform.execute(&value, Some(&context), "hu").unwrap();
    assert_eq!(result, "kertben");
}

// -----------------------------------------------------------------------------
// Hungarian @inflect - Illative (-ba/-be)
// -----------------------------------------------------------------------------

#[test]
fn hungarian_inflect_ill_back() {
    // :back "ház" + :ill -> "házba"
    let phrase = Phrase::builder()
        .text("ház".to_string())
        .tags(vec![Tag::new("back")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::HungarianInflect;
    let context = Value::String("ill".to_string());
    let result = transform.execute(&value, Some(&context), "hu").unwrap();
    assert_eq!(result, "házba");
}

#[test]
fn hungarian_inflect_ill_front() {
    // :front "kert" + :ill -> "kertbe"
    let phrase = Phrase::builder()
        .text("kert".to_string())
        .tags(vec![Tag::new("front")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::HungarianInflect;
    let context = Value::String("ill".to_string());
    let result = transform.execute(&value, Some(&context), "hu").unwrap();
    assert_eq!(result, "kertbe");
}

// -----------------------------------------------------------------------------
// Hungarian @inflect - Elative (-ból/-ből)
// -----------------------------------------------------------------------------

#[test]
fn hungarian_inflect_ela_back() {
    // :back "ház" + :ela -> "házból"
    let phrase = Phrase::builder()
        .text("ház".to_string())
        .tags(vec![Tag::new("back")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::HungarianInflect;
    let context = Value::String("ela".to_string());
    let result = transform.execute(&value, Some(&context), "hu").unwrap();
    assert_eq!(result, "házból");
}

#[test]
fn hungarian_inflect_ela_front() {
    // :front "kert" + :ela -> "kertből"
    let phrase = Phrase::builder()
        .text("kert".to_string())
        .tags(vec![Tag::new("front")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::HungarianInflect;
    let context = Value::String("ela".to_string());
    let result = transform.execute(&value, Some(&context), "hu").unwrap();
    assert_eq!(result, "kertből");
}

// -----------------------------------------------------------------------------
// Hungarian @inflect - Superessive (-on/-en/-ön)
// -----------------------------------------------------------------------------

#[test]
fn hungarian_inflect_sup_back() {
    // :back "ház" + :sup -> "házon"
    let phrase = Phrase::builder()
        .text("ház".to_string())
        .tags(vec![Tag::new("back")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::HungarianInflect;
    let context = Value::String("sup".to_string());
    let result = transform.execute(&value, Some(&context), "hu").unwrap();
    assert_eq!(result, "házon");
}

#[test]
fn hungarian_inflect_sup_front() {
    // :front "kert" + :sup -> "kerten"
    let phrase = Phrase::builder()
        .text("kert".to_string())
        .tags(vec![Tag::new("front")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::HungarianInflect;
    let context = Value::String("sup".to_string());
    let result = transform.execute(&value, Some(&context), "hu").unwrap();
    assert_eq!(result, "kerten");
}

#[test]
fn hungarian_inflect_sup_round() {
    // :round "tükör" + :sup -> "tükörön"
    let phrase = Phrase::builder()
        .text("tükör".to_string())
        .tags(vec![Tag::new("round")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::HungarianInflect;
    let context = Value::String("sup".to_string());
    let result = transform.execute(&value, Some(&context), "hu").unwrap();
    assert_eq!(result, "tükörön");
}

// -----------------------------------------------------------------------------
// Hungarian @inflect - Sublative (-ra/-re)
// -----------------------------------------------------------------------------

#[test]
fn hungarian_inflect_sub_back() {
    // :back "ház" + :sub -> "házra"
    let phrase = Phrase::builder()
        .text("ház".to_string())
        .tags(vec![Tag::new("back")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::HungarianInflect;
    let context = Value::String("sub".to_string());
    let result = transform.execute(&value, Some(&context), "hu").unwrap();
    assert_eq!(result, "házra");
}

#[test]
fn hungarian_inflect_sub_front() {
    // :front "kert" + :sub -> "kertre"
    let phrase = Phrase::builder()
        .text("kert".to_string())
        .tags(vec![Tag::new("front")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::HungarianInflect;
    let context = Value::String("sub".to_string());
    let result = transform.execute(&value, Some(&context), "hu").unwrap();
    assert_eq!(result, "kertre");
}

// -----------------------------------------------------------------------------
// Hungarian @inflect - Delative (-ról/-ről)
// -----------------------------------------------------------------------------

#[test]
fn hungarian_inflect_del_back() {
    // :back "ház" + :del -> "házról"
    let phrase = Phrase::builder()
        .text("ház".to_string())
        .tags(vec![Tag::new("back")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::HungarianInflect;
    let context = Value::String("del".to_string());
    let result = transform.execute(&value, Some(&context), "hu").unwrap();
    assert_eq!(result, "házról");
}

#[test]
fn hungarian_inflect_del_front() {
    // :front "kert" + :del -> "kertről"
    let phrase = Phrase::builder()
        .text("kert".to_string())
        .tags(vec![Tag::new("front")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::HungarianInflect;
    let context = Value::String("del".to_string());
    let result = transform.execute(&value, Some(&context), "hu").unwrap();
    assert_eq!(result, "kertről");
}

// -----------------------------------------------------------------------------
// Hungarian @inflect - Adessive (-nál/-nél)
// -----------------------------------------------------------------------------

#[test]
fn hungarian_inflect_ade_back() {
    // :back "ház" + :ade -> "háznál"
    let phrase = Phrase::builder()
        .text("ház".to_string())
        .tags(vec![Tag::new("back")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::HungarianInflect;
    let context = Value::String("ade".to_string());
    let result = transform.execute(&value, Some(&context), "hu").unwrap();
    assert_eq!(result, "háznál");
}

#[test]
fn hungarian_inflect_ade_front() {
    // :front "kert" + :ade -> "kertnél"
    let phrase = Phrase::builder()
        .text("kert".to_string())
        .tags(vec![Tag::new("front")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::HungarianInflect;
    let context = Value::String("ade".to_string());
    let result = transform.execute(&value, Some(&context), "hu").unwrap();
    assert_eq!(result, "kertnél");
}

// -----------------------------------------------------------------------------
// Hungarian @inflect - Ablative (-tól/-től)
// -----------------------------------------------------------------------------

#[test]
fn hungarian_inflect_abl_back() {
    // :back "ház" + :abl -> "háztól"
    let phrase = Phrase::builder()
        .text("ház".to_string())
        .tags(vec![Tag::new("back")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::HungarianInflect;
    let context = Value::String("abl".to_string());
    let result = transform.execute(&value, Some(&context), "hu").unwrap();
    assert_eq!(result, "háztól");
}

#[test]
fn hungarian_inflect_abl_front() {
    // :front "kert" + :abl -> "kerttől"
    let phrase = Phrase::builder()
        .text("kert".to_string())
        .tags(vec![Tag::new("front")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::HungarianInflect;
    let context = Value::String("abl".to_string());
    let result = transform.execute(&value, Some(&context), "hu").unwrap();
    assert_eq!(result, "kerttől");
}

// -----------------------------------------------------------------------------
// Hungarian @inflect - Allative (-hoz/-hez/-höz) — 3-way harmony
// -----------------------------------------------------------------------------

#[test]
fn hungarian_inflect_all_back() {
    // :back "ház" + :all -> "házhoz"
    let phrase = Phrase::builder()
        .text("ház".to_string())
        .tags(vec![Tag::new("back")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::HungarianInflect;
    let context = Value::String("all".to_string());
    let result = transform.execute(&value, Some(&context), "hu").unwrap();
    assert_eq!(result, "házhoz");
}

#[test]
fn hungarian_inflect_all_front() {
    // :front "kert" + :all -> "kerthez"
    let phrase = Phrase::builder()
        .text("kert".to_string())
        .tags(vec![Tag::new("front")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::HungarianInflect;
    let context = Value::String("all".to_string());
    let result = transform.execute(&value, Some(&context), "hu").unwrap();
    assert_eq!(result, "kerthez");
}

#[test]
fn hungarian_inflect_all_round() {
    // :round "tükör" + :all -> "tükörhöz"
    let phrase = Phrase::builder()
        .text("tükör".to_string())
        .tags(vec![Tag::new("round")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::HungarianInflect;
    let context = Value::String("all".to_string());
    let result = transform.execute(&value, Some(&context), "hu").unwrap();
    assert_eq!(result, "tükörhöz");
}

// -----------------------------------------------------------------------------
// Hungarian @inflect - Instrumental (-val/-vel)
// -----------------------------------------------------------------------------

#[test]
fn hungarian_inflect_ins_back() {
    // :back "ház" + :ins -> "házval"
    let phrase = Phrase::builder()
        .text("ház".to_string())
        .tags(vec![Tag::new("back")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::HungarianInflect;
    let context = Value::String("ins".to_string());
    let result = transform.execute(&value, Some(&context), "hu").unwrap();
    assert_eq!(result, "házval");
}

#[test]
fn hungarian_inflect_ins_front() {
    // :front "kert" + :ins -> "kertvel"
    let phrase = Phrase::builder()
        .text("kert".to_string())
        .tags(vec![Tag::new("front")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::HungarianInflect;
    let context = Value::String("ins".to_string());
    let result = transform.execute(&value, Some(&context), "hu").unwrap();
    assert_eq!(result, "kertvel");
}

// -----------------------------------------------------------------------------
// Hungarian @inflect - Translative (-vá/-vé)
// -----------------------------------------------------------------------------

#[test]
fn hungarian_inflect_tra_back() {
    // :back "ház" + :tra -> "házvá"
    let phrase = Phrase::builder()
        .text("ház".to_string())
        .tags(vec![Tag::new("back")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::HungarianInflect;
    let context = Value::String("tra".to_string());
    let result = transform.execute(&value, Some(&context), "hu").unwrap();
    assert_eq!(result, "házvá");
}

#[test]
fn hungarian_inflect_tra_front() {
    // :front "kert" + :tra -> "kertvé"
    let phrase = Phrase::builder()
        .text("kert".to_string())
        .tags(vec![Tag::new("front")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::HungarianInflect;
    let context = Value::String("tra".to_string());
    let result = transform.execute(&value, Some(&context), "hu").unwrap();
    assert_eq!(result, "kertvé");
}

// -----------------------------------------------------------------------------
// Hungarian @inflect - Invariant suffixes
// -----------------------------------------------------------------------------

#[test]
fn hungarian_inflect_cau_back() {
    // :back "ház" + :cau -> "házért" (invariant suffix)
    let phrase = Phrase::builder()
        .text("ház".to_string())
        .tags(vec![Tag::new("back")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::HungarianInflect;
    let context = Value::String("cau".to_string());
    let result = transform.execute(&value, Some(&context), "hu").unwrap();
    assert_eq!(result, "házért");
}

#[test]
fn hungarian_inflect_cau_front() {
    // :front "kert" + :cau -> "kertért" (invariant suffix)
    let phrase = Phrase::builder()
        .text("kert".to_string())
        .tags(vec![Tag::new("front")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::HungarianInflect;
    let context = Value::String("cau".to_string());
    let result = transform.execute(&value, Some(&context), "hu").unwrap();
    assert_eq!(result, "kertért");
}

#[test]
fn hungarian_inflect_ter_back() {
    // :back "ház" + :ter -> "házig" (invariant suffix)
    let phrase = Phrase::builder()
        .text("ház".to_string())
        .tags(vec![Tag::new("back")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::HungarianInflect;
    let context = Value::String("ter".to_string());
    let result = transform.execute(&value, Some(&context), "hu").unwrap();
    assert_eq!(result, "házig");
}

#[test]
fn hungarian_inflect_ess_back() {
    // :back "ház" + :ess -> "házként" (invariant suffix)
    let phrase = Phrase::builder()
        .text("ház".to_string())
        .tags(vec![Tag::new("back")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::HungarianInflect;
    let context = Value::String("ess".to_string());
    let result = transform.execute(&value, Some(&context), "hu").unwrap();
    assert_eq!(result, "házként");
}

// -----------------------------------------------------------------------------
// Hungarian @inflect - Possessives
// -----------------------------------------------------------------------------

#[test]
fn hungarian_inflect_poss1sg_back() {
    // :back "ház" + :poss1sg -> "házom"
    let phrase = Phrase::builder()
        .text("ház".to_string())
        .tags(vec![Tag::new("back")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::HungarianInflect;
    let context = Value::String("poss1sg".to_string());
    let result = transform.execute(&value, Some(&context), "hu").unwrap();
    assert_eq!(result, "házom");
}

#[test]
fn hungarian_inflect_poss1sg_front() {
    // :front "kert" + :poss1sg -> "kertem"
    let phrase = Phrase::builder()
        .text("kert".to_string())
        .tags(vec![Tag::new("front")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::HungarianInflect;
    let context = Value::String("poss1sg".to_string());
    let result = transform.execute(&value, Some(&context), "hu").unwrap();
    assert_eq!(result, "kertem");
}

#[test]
fn hungarian_inflect_poss1sg_round() {
    // :round "tükör" + :poss1sg -> "tükörröm" (simplified, using -öm suffix)
    let phrase = Phrase::builder()
        .text("tükör".to_string())
        .tags(vec![Tag::new("round")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::HungarianInflect;
    let context = Value::String("poss1sg".to_string());
    let result = transform.execute(&value, Some(&context), "hu").unwrap();
    assert_eq!(result, "tükör\u{00f6}m");
}

#[test]
fn hungarian_inflect_poss2sg_back() {
    // :back "ház" + :poss2sg -> "házod"
    let phrase = Phrase::builder()
        .text("ház".to_string())
        .tags(vec![Tag::new("back")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::HungarianInflect;
    let context = Value::String("poss2sg".to_string());
    let result = transform.execute(&value, Some(&context), "hu").unwrap();
    assert_eq!(result, "házod");
}

#[test]
fn hungarian_inflect_poss3sg_back() {
    // :back "ház" + :poss3sg -> "háza"
    let phrase = Phrase::builder()
        .text("ház".to_string())
        .tags(vec![Tag::new("back")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::HungarianInflect;
    let context = Value::String("poss3sg".to_string());
    let result = transform.execute(&value, Some(&context), "hu").unwrap();
    assert_eq!(result, "háza");
}

#[test]
fn hungarian_inflect_poss3sg_front() {
    // :front "kert" + :poss3sg -> "kerte"
    let phrase = Phrase::builder()
        .text("kert".to_string())
        .tags(vec![Tag::new("front")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::HungarianInflect;
    let context = Value::String("poss3sg".to_string());
    let result = transform.execute(&value, Some(&context), "hu").unwrap();
    assert_eq!(result, "kerte");
}

#[test]
fn hungarian_inflect_poss1pl_back() {
    // :back "ház" + :poss1pl -> "házunk"
    let phrase = Phrase::builder()
        .text("ház".to_string())
        .tags(vec![Tag::new("back")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::HungarianInflect;
    let context = Value::String("poss1pl".to_string());
    let result = transform.execute(&value, Some(&context), "hu").unwrap();
    assert_eq!(result, "házunk");
}

#[test]
fn hungarian_inflect_poss1pl_front() {
    // :front "kert" + :poss1pl -> "kertünk"
    let phrase = Phrase::builder()
        .text("kert".to_string())
        .tags(vec![Tag::new("front")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::HungarianInflect;
    let context = Value::String("poss1pl".to_string());
    let result = transform.execute(&value, Some(&context), "hu").unwrap();
    assert_eq!(result, "kertünk");
}

#[test]
fn hungarian_inflect_poss2pl_back() {
    // :back "ház" + :poss2pl -> "háztok"
    let phrase = Phrase::builder()
        .text("ház".to_string())
        .tags(vec![Tag::new("back")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::HungarianInflect;
    let context = Value::String("poss2pl".to_string());
    let result = transform.execute(&value, Some(&context), "hu").unwrap();
    assert_eq!(result, "háztok");
}

#[test]
fn hungarian_inflect_poss2pl_front() {
    // :front "kert" + :poss2pl -> "kerttek"
    let phrase = Phrase::builder()
        .text("kert".to_string())
        .tags(vec![Tag::new("front")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::HungarianInflect;
    let context = Value::String("poss2pl".to_string());
    let result = transform.execute(&value, Some(&context), "hu").unwrap();
    assert_eq!(result, "kerttek");
}

#[test]
fn hungarian_inflect_poss2pl_round() {
    // :round "tükör" + :poss2pl -> "tükörtök"
    let phrase = Phrase::builder()
        .text("tükör".to_string())
        .tags(vec![Tag::new("round")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::HungarianInflect;
    let context = Value::String("poss2pl".to_string());
    let result = transform.execute(&value, Some(&context), "hu").unwrap();
    assert_eq!(result, "tükörtök");
}

#[test]
fn hungarian_inflect_poss3pl_back() {
    // :back "ház" + :poss3pl -> "házuk"
    let phrase = Phrase::builder()
        .text("ház".to_string())
        .tags(vec![Tag::new("back")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::HungarianInflect;
    let context = Value::String("poss3pl".to_string());
    let result = transform.execute(&value, Some(&context), "hu").unwrap();
    assert_eq!(result, "házuk");
}

#[test]
fn hungarian_inflect_poss3pl_front() {
    // :front "kert" + :poss3pl -> "kertük"
    let phrase = Phrase::builder()
        .text("kert".to_string())
        .tags(vec![Tag::new("front")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::HungarianInflect;
    let context = Value::String("poss3pl".to_string());
    let result = transform.execute(&value, Some(&context), "hu").unwrap();
    assert_eq!(result, "kertük");
}

// -----------------------------------------------------------------------------
// Hungarian @inflect - Suffix chains
// -----------------------------------------------------------------------------

#[test]
fn hungarian_inflect_pl_dat_back() {
    // :back "ház" + :pl.dat -> "házoknak"
    let phrase = Phrase::builder()
        .text("ház".to_string())
        .tags(vec![Tag::new("back")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::HungarianInflect;
    let context = Value::String("pl.dat".to_string());
    let result = transform.execute(&value, Some(&context), "hu").unwrap();
    assert_eq!(result, "házoknak");
}

#[test]
fn hungarian_inflect_pl_ine_front() {
    // :front "kert" + :pl.ine -> "kertekben"
    let phrase = Phrase::builder()
        .text("kert".to_string())
        .tags(vec![Tag::new("front")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::HungarianInflect;
    let context = Value::String("pl.ine".to_string());
    let result = transform.execute(&value, Some(&context), "hu").unwrap();
    assert_eq!(result, "kertekben");
}

#[test]
fn hungarian_inflect_poss1sg_dat_back() {
    // :back "ház" + :poss1sg.dat -> "házomnak"
    let phrase = Phrase::builder()
        .text("ház".to_string())
        .tags(vec![Tag::new("back")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::HungarianInflect;
    let context = Value::String("poss1sg.dat".to_string());
    let result = transform.execute(&value, Some(&context), "hu").unwrap();
    assert_eq!(result, "házomnak");
}

#[test]
fn hungarian_inflect_pl_abl_round() {
    // :round "tükör" + :pl.abl -> "tükörök" + "től" -> "tüköröktől"
    let phrase = Phrase::builder()
        .text("tükör".to_string())
        .tags(vec![Tag::new("round")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::HungarianInflect;
    let context = Value::String("pl.abl".to_string());
    let result = transform.execute(&value, Some(&context), "hu").unwrap();
    assert_eq!(result, "tüköröktől");
}

// -----------------------------------------------------------------------------
// Hungarian @inflect - Error cases
// -----------------------------------------------------------------------------

#[test]
fn hungarian_inflect_missing_harmony() {
    // No harmony tag -> error
    let phrase = Phrase::builder().text("ház".to_string()).build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::HungarianInflect;
    let context = Value::String("dat".to_string());
    let result = transform.execute(&value, Some(&context), "hu");
    assert!(result.is_err());
}

#[test]
fn hungarian_inflect_no_context() {
    // No context -> no suffixes added, word returned as-is
    let phrase = Phrase::builder()
        .text("ház".to_string())
        .tags(vec![Tag::new("back")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::HungarianInflect;
    let result = transform.execute(&value, None, "hu").unwrap();
    assert_eq!(result, "ház");
}

#[test]
fn hungarian_inflect_unknown_suffix_ignored() {
    // Unknown suffix parts are ignored
    let phrase = Phrase::builder()
        .text("ház".to_string())
        .tags(vec![Tag::new("back")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::HungarianInflect;
    let context = Value::String("dat.xyz".to_string());
    let result = transform.execute(&value, Some(&context), "hu").unwrap();
    assert_eq!(result, "háznak");
}

#[test]
fn hungarian_inflect_not_registered_for_english() {
    let registry = TransformRegistry::new();
    assert!(registry.get("inflect", "en").is_none());
}

// =============================================================================
// Hindi Transform Tests
// =============================================================================

#[test]
fn hindi_ka_masculine_singular() {
    let phrase = Phrase::builder()
        .text("कार्ड".to_string())
        .tags(vec![Tag::new("masc")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::HindiKa;
    let result = transform.execute(&value, None, "hi").unwrap();
    assert_eq!(result, "कार्ड का");
}

#[test]
fn hindi_ka_masculine_plural() {
    let phrase = Phrase::builder()
        .text("कार्ड".to_string())
        .tags(vec![Tag::new("masc")])
        .build();
    let value = Value::Phrase(phrase);
    let context = Value::String("other".to_string());
    let transform = TransformKind::HindiKa;
    let result = transform.execute(&value, Some(&context), "hi").unwrap();
    assert_eq!(result, "कार्ड के");
}

#[test]
fn hindi_ka_feminine_singular() {
    let phrase = Phrase::builder()
        .text("घटना".to_string())
        .tags(vec![Tag::new("fem")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::HindiKa;
    let result = transform.execute(&value, None, "hi").unwrap();
    assert_eq!(result, "घटना की");
}

#[test]
fn hindi_ka_feminine_plural() {
    let phrase = Phrase::builder()
        .text("घटनाओं".to_string())
        .tags(vec![Tag::new("fem")])
        .build();
    let value = Value::Phrase(phrase);
    let context = Value::String("other".to_string());
    let transform = TransformKind::HindiKa;
    let result = transform.execute(&value, Some(&context), "hi").unwrap();
    assert_eq!(result, "घटनाओं की");
}

#[test]
fn hindi_ka_missing_gender() {
    let phrase = Phrase::builder().text("चीज़".to_string()).build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::HindiKa;
    let result = transform.execute(&value, None, "hi");
    assert!(matches!(result, Err(EvalError::MissingTag { .. })));
}

#[test]
fn hindi_ka_numeric_context() {
    let phrase = Phrase::builder()
        .text("कार्ड".to_string())
        .tags(vec![Tag::new("masc")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::HindiKa;

    let ctx_one = Value::Number(1);
    let result_one = transform.execute(&value, Some(&ctx_one), "hi").unwrap();
    assert_eq!(result_one, "कार्ड का");

    let ctx_many = Value::Number(5);
    let result_many = transform.execute(&value, Some(&ctx_many), "hi").unwrap();
    assert_eq!(result_many, "कार्ड के");
}

#[test]
fn hindi_ko_transform() {
    let phrase = Phrase::builder()
        .text("कार्ड".to_string())
        .tags(vec![Tag::new("masc")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::HindiKo;
    let result = transform.execute(&value, None, "hi").unwrap();
    assert_eq!(result, "कार्ड को");
}

#[test]
fn hindi_se_transform() {
    let phrase = Phrase::builder()
        .text("कार्ड".to_string())
        .tags(vec![Tag::new("masc")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::HindiSe;
    let result = transform.execute(&value, None, "hi").unwrap();
    assert_eq!(result, "कार्ड से");
}

#[test]
fn hindi_me_transform() {
    let phrase = Phrase::builder()
        .text("हाथ".to_string())
        .tags(vec![Tag::new("masc")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::HindiMe;
    let result = transform.execute(&value, None, "hi").unwrap();
    assert_eq!(result, "हाथ में");
}

#[test]
fn hindi_par_transform() {
    let phrase = Phrase::builder()
        .text("मेज़".to_string())
        .tags(vec![Tag::new("fem")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::HindiPar;
    let result = transform.execute(&value, None, "hi").unwrap();
    assert_eq!(result, "मेज़ पर");
}

#[test]
fn hindi_ne_transform() {
    let phrase = Phrase::builder()
        .text("खिलाड़ी".to_string())
        .tags(vec![Tag::new("masc")])
        .build();
    let value = Value::Phrase(phrase);
    let transform = TransformKind::HindiNe;
    let result = transform.execute(&value, None, "hi").unwrap();
    assert_eq!(result, "खिलाड़ी ने");
}

#[test]
fn hindi_transform_aliases() {
    let registry = TransformRegistry::new();
    assert_eq!(registry.get("ka", "hi"), Some(TransformKind::HindiKa));
    assert_eq!(registry.get("ki", "hi"), Some(TransformKind::HindiKa));
    assert_eq!(registry.get("ke", "hi"), Some(TransformKind::HindiKa));
    assert_eq!(registry.get("ko", "hi"), Some(TransformKind::HindiKo));
    assert_eq!(registry.get("se", "hi"), Some(TransformKind::HindiSe));
    assert_eq!(registry.get("me", "hi"), Some(TransformKind::HindiMe));
    assert_eq!(registry.get("par", "hi"), Some(TransformKind::HindiPar));
    assert_eq!(registry.get("ne", "hi"), Some(TransformKind::HindiNe));
}

#[test]
fn hindi_transforms_not_registered_for_english() {
    let registry = TransformRegistry::new();
    assert!(registry.get("ka", "en").is_none());
    assert!(registry.get("ko", "en").is_none());
    assert!(registry.get("se", "en").is_none());
    assert!(registry.get("me", "en").is_none());
    assert!(registry.get("par", "en").is_none());
    assert!(registry.get("ne", "en").is_none());
}

#[test]
fn hindi_ka_in_template() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
            card = :masc "कार्ड";
            card_of = "{@ka card}";
            "#,
        )
        .unwrap();
    let result = registry.call_phrase("hi", "card_of", &[]).unwrap();
    assert_eq!(result.to_string(), "कार्ड का");
}

#[test]
fn hindi_ko_in_template() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
            card = :masc "कार्ड";
            give_card = "{@ko card} दो";
            "#,
        )
        .unwrap();
    let result = registry.call_phrase("hi", "give_card", &[]).unwrap();
    assert_eq!(result.to_string(), "कार्ड को दो");
}

#[test]
fn hindi_se_in_template() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
            card = :masc "कार्ड";
            from_card = "{@se card} लो";
            "#,
        )
        .unwrap();
    let result = registry.call_phrase("hi", "from_card", &[]).unwrap();
    assert_eq!(result.to_string(), "कार्ड से लो");
}

#[test]
fn hindi_me_in_template() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
            hand = :masc "हाथ";
            in_hand = "{@me hand}";
            "#,
        )
        .unwrap();
    let result = registry.call_phrase("hi", "in_hand", &[]).unwrap();
    assert_eq!(result.to_string(), "हाथ में");
}

#[test]
fn hindi_ka_with_plural_context() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
            card = :masc "कार्ड";
            cards_of = "{@ka:other card}";
            "#,
        )
        .unwrap();
    let result = registry.call_phrase("hi", "cards_of", &[]).unwrap();
    assert_eq!(result.to_string(), "कार्ड के");
}

#[test]
fn hindi_ki_alias_in_template() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
            event = :fem "घटना";
            event_of = "{@ki event}";
            "#,
        )
        .unwrap();
    let result = registry.call_phrase("hi", "event_of", &[]).unwrap();
    assert_eq!(result.to_string(), "घटना की");
}

#[test]
fn hindi_ne_in_template() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
            player = :masc "खिलाड़ी";
            player_did = "{@ne player} किया";
            "#,
        )
        .unwrap();
    let result = registry.call_phrase("hi", "player_did", &[]).unwrap();
    assert_eq!(result.to_string(), "खिलाड़ी ने किया");
}

#[test]
fn hindi_par_in_template() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
            table = :fem "मेज़";
            on_table = "{@par table}";
            "#,
        )
        .unwrap();
    let result = registry.call_phrase("hi", "on_table", &[]).unwrap();
    assert_eq!(result.to_string(), "मेज़ पर");
}

#[test]
fn hindi_invariant_postpositions_ignore_gender() {
    // ko, se, me, par, ne do not change form based on gender
    let masc_phrase = Phrase::builder()
        .text("कार्ड".to_string())
        .tags(vec![Tag::new("masc")])
        .build();
    let fem_phrase = Phrase::builder()
        .text("घटना".to_string())
        .tags(vec![Tag::new("fem")])
        .build();
    let masc = Value::Phrase(masc_phrase);
    let fem = Value::Phrase(fem_phrase);

    let ko = TransformKind::HindiKo;
    assert_eq!(ko.execute(&masc, None, "hi").unwrap(), "कार्ड को");
    assert_eq!(ko.execute(&fem, None, "hi").unwrap(), "घटना को");

    let se = TransformKind::HindiSe;
    assert_eq!(se.execute(&masc, None, "hi").unwrap(), "कार्ड से");
    assert_eq!(se.execute(&fem, None, "hi").unwrap(), "घटना से");
}

#[test]
fn hindi_ko_with_string_value() {
    // Invariant postpositions work on plain strings too
    let value = Value::String("राम".to_string());
    let transform = TransformKind::HindiKo;
    let result = transform.execute(&value, None, "hi").unwrap();
    assert_eq!(result, "राम को");
}

// =============================================================================
// Dynamic Transform Context
// =============================================================================

#[test]
fn dynamic_context_chinese_count() {
    // @count($n) syntax: dynamic context from parameter
    let source = r#"
        card = :zhang "牌";
        draw($n) = "抽{@count($n) card}";
    "#;

    let mut locale = Locale::builder().language("zh").build();
    locale.load_translations_str("zh", source).unwrap();

    let result = locale.call_phrase("draw", &[Value::from(3)]).unwrap();
    assert_eq!(result.to_string(), "抽3张牌");
}

#[test]
fn dynamic_context_german_case() {
    // Static context still works: @der:acc
    let source = r#"
        karte = :fem "Karte";
        destroy = "Zerstöre {@der:acc karte}.";
    "#;

    let mut locale = Locale::builder().language("de").build();
    locale.load_translations_str("de", source).unwrap();

    assert_eq!(
        locale.get_phrase("destroy").unwrap().to_string(),
        "Zerstöre die Karte."
    );
}

#[test]
fn dynamic_context_resolves_number() {
    // Dynamic context passes numeric value to transform
    let source = r#"
        card = :zhang "牌";
        draw($n) = "抽{@count($n) card}";
    "#;

    let mut locale = Locale::builder().language("zh").build();
    locale.load_translations_str("zh", source).unwrap();

    // Test with different numbers
    let one = locale.call_phrase("draw", &[Value::from(1)]).unwrap();
    assert_eq!(one.to_string(), "抽1张牌");

    let five = locale.call_phrase("draw", &[Value::from(5)]).unwrap();
    assert_eq!(five.to_string(), "抽5张牌");
}

// =============================================================================
// Auto-capitalization with explicit transforms
// =============================================================================

#[test]
fn auto_cap_with_a_transform_phrase_call() {
    // Test that {@a Subtype($t)} works the same as {@cap @a subtype($t)}
    // Auto-cap from uppercase first letter should be outermost (leftmost) transform
    let source = r#"
        warrior = :a { one: "warrior", other: "warriors" };
        subtype($s) = :from($s) "<b>{$s}</b>";
        dissolve_explicit($s) = "Dissolve {@cap @a subtype($s)}.";
        dissolve_auto($s) = "Dissolve {@a Subtype($s)}.";
    "#;

    let mut locale = Locale::builder().language("en").build();
    locale.load_translations_str("en", source).unwrap();

    let warrior = locale.get_phrase("warrior").unwrap();

    // Explicit: {@cap @a subtype($s)} -> @a first ("a <b>warrior</b>"), then @cap ("A <b>warrior</b>")
    let explicit = locale
        .call_phrase("dissolve_explicit", &[Value::Phrase(warrior.clone())])
        .unwrap();
    assert_eq!(explicit.to_string(), "Dissolve A <b>warrior</b>.");

    // Auto-cap: {@a Subtype($s)} should produce the same result
    let auto = locale
        .call_phrase("dissolve_auto", &[Value::Phrase(warrior)])
        .unwrap();
    assert_eq!(auto.to_string(), "Dissolve A <b>warrior</b>.");
}

#[test]
fn auto_cap_with_a_transform_an_tag() {
    // Test {@a Subtype($t)} with a term that has the :an tag
    let source = r#"
        ancient = :an { one: "ancient", other: "ancients" };
        subtype($s) = :from($s) "<b>{$s}</b>";
        dissolve_explicit($s) = "Dissolve {@cap @a subtype($s)}.";
        dissolve_auto($s) = "Dissolve {@a Subtype($s)}.";
    "#;

    let mut locale = Locale::builder().language("en").build();
    locale.load_translations_str("en", source).unwrap();

    let ancient = locale.get_phrase("ancient").unwrap();

    // Explicit: {@cap @a subtype($s)} -> @a first ("an <b>ancient</b>"), then @cap ("An <b>ancient</b>")
    let explicit = locale
        .call_phrase("dissolve_explicit", &[Value::Phrase(ancient.clone())])
        .unwrap();
    assert_eq!(explicit.to_string(), "Dissolve An <b>ancient</b>.");

    // Auto-cap: {@a Subtype($s)} should produce the same result
    let auto = locale
        .call_phrase("dissolve_auto", &[Value::Phrase(ancient)])
        .unwrap();
    assert_eq!(auto.to_string(), "Dissolve An <b>ancient</b>.");
}

#[test]
fn auto_cap_bare_term_still_works() {
    // Existing behavior: {Card} is equivalent to {@cap card}
    let source = r#"
        card = :a "card";
        cap_card = "{Card}";
        explicit_cap_card = "{@cap card}";
    "#;

    let mut locale = Locale::builder().language("en").build();
    locale.load_translations_str("en", source).unwrap();

    assert_eq!(locale.get_phrase("cap_card").unwrap().to_string(), "Card");
    assert_eq!(
        locale.get_phrase("explicit_cap_card").unwrap().to_string(),
        "Card"
    );
}

#[test]
fn auto_cap_with_multiple_explicit_transforms() {
    // Test auto-cap combined with multiple explicit transforms
    // {@upper @a Subtype($s)} should be equivalent to {@cap @upper @a subtype($s)}
    // Wait, that doesn't make sense. Let's test with just @a.
    // The key insight: auto-cap inserts @cap as leftmost (outermost).
    // So {@a Subtype($s)} -> transforms: [@cap, @a] -> right-to-left: @a then @cap
    let source = r#"
        card = :a "card";
        a_card_auto = "{@a Card}";
        a_card_explicit = "{@cap @a card}";
    "#;

    let mut locale = Locale::builder().language("en").build();
    locale.load_translations_str("en", source).unwrap();

    // Both should produce "A card"
    assert_eq!(
        locale.get_phrase("a_card_auto").unwrap().to_string(),
        "A card"
    );
    assert_eq!(
        locale.get_phrase("a_card_explicit").unwrap().to_string(),
        "A card"
    );
}

#[test]
fn auto_cap_with_the_transform() {
    // Test {@the Card} is same as {@cap @the card}
    let source = r#"
        card = :a "card";
        the_card_auto = "{@the Card}";
        the_card_explicit = "{@cap @the card}";
    "#;

    let mut locale = Locale::builder().language("en").build();
    locale.load_translations_str("en", source).unwrap();

    assert_eq!(
        locale.get_phrase("the_card_auto").unwrap().to_string(),
        "The card"
    );
    assert_eq!(
        locale.get_phrase("the_card_explicit").unwrap().to_string(),
        "The card"
    );
}

#[test]
fn explicit_cap_a_still_works() {
    // Verify existing explicit {@cap @a ...} behavior is preserved
    let source = r#"
        card = :a "card";
        event = :an "event";
        cap_a_card = "{@cap @a card}";
        cap_a_event = "{@cap @a event}";
    "#;

    let mut locale = Locale::builder().language("en").build();
    locale.load_translations_str("en", source).unwrap();

    assert_eq!(
        locale.get_phrase("cap_a_card").unwrap().to_string(),
        "A card"
    );
    assert_eq!(
        locale.get_phrase("cap_a_event").unwrap().to_string(),
        "An event"
    );
}
