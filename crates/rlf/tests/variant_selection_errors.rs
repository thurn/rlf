//! Tests for String/Number variant selection error paths.
//!
//! Covers error scenarios when String and Number values are used as selectors
//! on terms with variants, in :match blocks, and in contexts where variant
//! lookup fails.

use rlf::interpreter::EvalError;
use rlf::{Phrase, PhraseRegistry, Value};

// =============================================================================
// String Selector on Term Variants — MissingVariant
// =============================================================================

#[test]
fn string_selector_no_matching_variant_key() {
    // String parameter selects a variant key that doesn't exist
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        card = { one: "card", other: "cards" };
        pick($s) = "{card:$s}";
    "#,
        )
        .unwrap();

    let err = registry
        .call_phrase("en", "pick", &[Value::from("nonexistent")])
        .unwrap_err();
    assert!(
        matches!(err, EvalError::MissingVariant { ref key, .. } if key == "nonexistent"),
        "expected MissingVariant with key 'nonexistent', got: {err:?}"
    );
}

#[test]
fn string_selector_matching_variant_key_succeeds() {
    // String parameter with a value that matches a variant key should succeed
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        card = { one: "card", other: "cards" };
        pick($s) = "{card:$s}";
    "#,
        )
        .unwrap();

    let result = registry
        .call_phrase("en", "pick", &[Value::from("other")])
        .unwrap();
    assert_eq!(result.to_string(), "cards");
}

#[test]
fn string_numeric_selector_uses_plural_category() {
    // String "5" should be parsed as number and resolved to plural category "other"
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        card = { one: "card", other: "cards" };
        pick($s) = "{card:$s}";
    "#,
        )
        .unwrap();

    let result = registry
        .call_phrase("en", "pick", &[Value::from("5")])
        .unwrap();
    assert_eq!(result.to_string(), "cards");
}

#[test]
fn string_numeric_one_selector_uses_plural_category() {
    // String "1" should parse as number and resolve to plural category "one"
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        card = { one: "card", other: "cards" };
        pick($s) = "{card:$s}";
    "#,
        )
        .unwrap();

    let result = registry
        .call_phrase("en", "pick", &[Value::from("1")])
        .unwrap();
    assert_eq!(result.to_string(), "card");
}

#[test]
fn string_selector_empty_string_no_variant_match() {
    // Empty string won't match any variant key
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        card = { one: "card", other: "cards" };
        pick($s) = "{card:$s}";
    "#,
        )
        .unwrap();

    let err = registry
        .call_phrase("en", "pick", &[Value::from("")])
        .unwrap_err();
    assert!(
        matches!(err, EvalError::MissingVariant { .. }),
        "expected MissingVariant, got: {err:?}"
    );
}

#[test]
fn string_selector_case_sensitive() {
    // Variant keys are case-sensitive: "One" won't match "one"
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        card = { one: "card", other: "cards" };
        pick($s) = "{card:$s}";
    "#,
        )
        .unwrap();

    let err = registry
        .call_phrase("en", "pick", &[Value::from("One")])
        .unwrap_err();
    assert!(
        matches!(err, EvalError::MissingVariant { ref key, .. } if key == "One"),
        "expected MissingVariant with key 'One', got: {err:?}"
    );
}

#[test]
fn string_selector_suggestions_in_error() {
    // String selector that's close to a real key should produce suggestions
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        card = { one: "card", other: "cards" };
        pick($s) = "{card:$s}";
    "#,
        )
        .unwrap();

    let err = registry
        .call_phrase("en", "pick", &[Value::from("oter")])
        .unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("did you mean: other"),
        "error should suggest 'other': {msg}"
    );
}

// =============================================================================
// Number Selector on Term Variants — MissingVariant
// =============================================================================

#[test]
fn number_selector_missing_plural_category() {
    // Number resolves to plural category that doesn't exist in variants
    // "few" is needed for Russian but not provided
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        card = { one: "карта", many: "карт" };
        pick($n) = "{card:$n}";
    "#,
        )
        .unwrap();

    // n=2 in Russian -> "few" category, which is missing
    let err = registry
        .call_phrase("ru", "pick", &[Value::from(2)])
        .unwrap_err();
    assert!(
        matches!(err, EvalError::MissingVariant { ref key, .. } if key == "few"),
        "expected MissingVariant with key 'few', got: {err:?}"
    );
}

#[test]
fn number_selector_missing_other_category() {
    // Number resolves to "other" category which is missing
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        card = { one: "card" };
        pick($n) = "{card:$n}";
    "#,
        )
        .unwrap();

    // n=5 in English -> "other" category, which is missing
    let err = registry
        .call_phrase("en", "pick", &[Value::from(5)])
        .unwrap_err();
    assert!(
        matches!(err, EvalError::MissingVariant { ref key, .. } if key == "other"),
        "expected MissingVariant with key 'other', got: {err:?}"
    );
}

#[test]
fn number_one_selects_one_variant() {
    // Number 1 should resolve to "one" category in English
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        card = { one: "card", other: "cards" };
        pick($n) = "{card:$n}";
    "#,
        )
        .unwrap();

    let result = registry
        .call_phrase("en", "pick", &[Value::from(1)])
        .unwrap();
    assert_eq!(result.to_string(), "card");
}

#[test]
fn number_selector_error_lists_available_variants() {
    // Error message should list available variant keys
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        card = { one: "card" };
        pick($n) = "{card:$n}";
    "#,
        )
        .unwrap();

    let err = registry
        .call_phrase("en", "pick", &[Value::from(5)])
        .unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("one"),
        "error should list available variant 'one': {msg}"
    );
}

// =============================================================================
// Float Selector on Term Variants
// =============================================================================

#[test]
fn float_selector_missing_plural_category() {
    // Float value is converted to integer for plural category
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        card = { one: "card" };
        pick($n) = "{card:$n}";
    "#,
        )
        .unwrap();

    // 3.5 -> truncated to 3 -> "other" in English, which is missing
    let err = registry
        .call_phrase("en", "pick", &[Value::from(3.5f64)])
        .unwrap_err();
    assert!(
        matches!(err, EvalError::MissingVariant { ref key, .. } if key == "other"),
        "expected MissingVariant with key 'other', got: {err:?}"
    );
}

#[test]
fn float_one_selects_one_variant() {
    // Float 1.0 -> truncated to 1 -> "one" category in English
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        card = { one: "card", other: "cards" };
        pick($n) = "{card:$n}";
    "#,
        )
        .unwrap();

    let result = registry
        .call_phrase("en", "pick", &[Value::from(1.0f64)])
        .unwrap();
    assert_eq!(result.to_string(), "card");
}

// =============================================================================
// Phrase Without Tags Used as Selector — MissingTag
// =============================================================================

#[test]
fn phrase_without_tags_in_selector_is_missing_variant() {
    // Static selector "thing" used as literal variant key -> MissingVariant
    // (Static selectors are used as literal keys, not resolved through the registry)
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        destroyed = { masc: "destruido", fem: "destruida" };
        thing = "cosa";
        result = "{destroyed:thing}";
    "#,
        )
        .unwrap();

    // "thing" is a literal variant key, not a phrase lookup -> MissingVariant
    let err = registry.get_phrase("es", "result").unwrap_err();
    assert!(
        matches!(err, EvalError::MissingVariant { ref key, .. } if key == "thing"),
        "expected MissingVariant with key 'thing', got: {err:?}"
    );
}

#[test]
fn phrase_without_tags_as_param_selector_is_error() {
    // Phrase parameter with no tags used as selector should error
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        adj = { masc: "bueno", fem: "buena" };
        describe($thing) = "{adj:$thing} {$thing}";
    "#,
        )
        .unwrap();

    let tagless_phrase = Phrase::builder().text("mesa".to_string()).build();
    let err = registry
        .call_phrase("es", "describe", &[Value::Phrase(tagless_phrase)])
        .unwrap_err();
    assert!(
        matches!(err, EvalError::MissingTag { .. }),
        "expected MissingTag, got: {err:?}"
    );
}

// =============================================================================
// :match Block Errors with String Values
// =============================================================================

#[test]
fn match_string_no_branch_matches_uses_default() {
    // String value doesn't match any explicit branch, but * default exists
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        classify($s) = :match($s) {
            attack: "offensive",
            defend: "defensive",
            *other: "unknown",
        };
    "#,
        )
        .unwrap();

    let result = registry
        .call_phrase("en", "classify", &[Value::from("move")])
        .unwrap();
    assert_eq!(result.to_string(), "unknown");
}

#[test]
fn match_string_numeric_value_resolves_to_plural() {
    // String "1" in :match should be parsed as number -> exact "1" match or CLDR "one"
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        count_things($s) = :match($s) {
            1: "exactly one",
            *other: "many",
        };
    "#,
        )
        .unwrap();

    let result = registry
        .call_phrase("en", "count_things", &[Value::from("1")])
        .unwrap();
    assert_eq!(result.to_string(), "exactly one");
}

#[test]
fn match_string_numeric_value_plural_category() {
    // String "5" in :match should parse as number -> CLDR "other"
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        count_things($s) = :match($s) {
            1: "exactly one",
            *other: "many",
        };
    "#,
        )
        .unwrap();

    let result = registry
        .call_phrase("en", "count_things", &[Value::from("5")])
        .unwrap();
    assert_eq!(result.to_string(), "many");
}

#[test]
fn match_string_literal_match() {
    // Non-numeric string in :match should match branch key literally
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        describe($action) = :match($action) {
            attack: "offensive",
            defend: "defensive",
            *other: "neutral",
        };
    "#,
        )
        .unwrap();

    let result = registry
        .call_phrase("en", "describe", &[Value::from("attack")])
        .unwrap();
    assert_eq!(result.to_string(), "offensive");
}

// =============================================================================
// :match Block Errors with Number Values
// =============================================================================

#[test]
fn match_number_exact_match_preferred_over_plural() {
    // Exact numeric key should be preferred over CLDR plural category
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        describe($n) = :match($n) {
            0: "none",
            1: "a single one",
            *other: "{$n} things",
        };
    "#,
        )
        .unwrap();

    let result = registry
        .call_phrase("en", "describe", &[Value::from(0)])
        .unwrap();
    assert_eq!(result.to_string(), "none");

    let result = registry
        .call_phrase("en", "describe", &[Value::from(1)])
        .unwrap();
    assert_eq!(result.to_string(), "a single one");

    let result = registry
        .call_phrase("en", "describe", &[Value::from(42)])
        .unwrap();
    assert_eq!(result.to_string(), "42 things");
}

#[test]
fn match_number_falls_through_to_plural_category() {
    // No exact numeric match -> use CLDR plural category
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        count($n) = :match($n) {
            one: "a thing",
            *other: "{$n} things",
        };
    "#,
        )
        .unwrap();

    let result = registry
        .call_phrase("en", "count", &[Value::from(1)])
        .unwrap();
    assert_eq!(result.to_string(), "a thing");

    let result = registry
        .call_phrase("en", "count", &[Value::from(99)])
        .unwrap();
    assert_eq!(result.to_string(), "99 things");
}

// =============================================================================
// :match Block with Float Values
// =============================================================================

#[test]
fn match_float_uses_plural_category() {
    // Float values use CLDR plural category (truncated to integer)
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        count($n) = :match($n) {
            one: "about one",
            *other: "about {$n}",
        };
    "#,
        )
        .unwrap();

    let result = registry
        .call_phrase("en", "count", &[Value::from(1.7f64)])
        .unwrap();
    // 1.7 truncated to 1 -> "one" in English
    assert_eq!(result.to_string(), "about one");

    let result = registry
        .call_phrase("en", "count", &[Value::from(3.5f64)])
        .unwrap();
    assert_eq!(result.to_string(), "about 3.5");
}

// =============================================================================
// Multi-dimensional Selection Errors
// =============================================================================

#[test]
fn multi_dim_string_number_selector_missing_variant() {
    // Multi-dimensional selection: first dim string (literal), second dim number
    // When the combined key doesn't exist -> MissingVariant
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        card = {
            nom.one: "карта",
            nom.other: "карт",
        };
        pick($n) = "{card:acc:$n}";
    "#,
        )
        .unwrap();

    let err = registry
        .call_phrase("ru", "pick", &[Value::from(1)])
        .unwrap_err();
    assert!(
        matches!(err, EvalError::MissingVariant { .. }),
        "expected MissingVariant, got: {err:?}"
    );
}

#[test]
fn multi_dim_string_param_number_param_error() {
    // Both dimensions from parameters: string + number
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        card = {
            nom.one: "карта",
            nom.few: "карты",
            acc.one: "карту",
        };
        pick($case, $n) = "{card:$case:$n}";
    "#,
        )
        .unwrap();

    // acc.few doesn't exist -> error
    let err = registry
        .call_phrase("ru", "pick", &[Value::from("acc"), Value::from(2)])
        .unwrap_err();
    assert!(
        matches!(err, EvalError::MissingVariant { .. }),
        "expected MissingVariant, got: {err:?}"
    );
}

#[test]
fn multi_dim_string_param_number_param_success() {
    // Both dimensions from parameters: string + number, matching variant exists
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        card = {
            nom.one: "карта",
            nom.few: "карты",
            acc.one: "карту",
            acc.few: "карты",
        };
        pick($case, $n) = "{card:$case:$n}";
    "#,
        )
        .unwrap();

    let result = registry
        .call_phrase("ru", "pick", &[Value::from("nom"), Value::from(1)])
        .unwrap();
    assert_eq!(result.to_string(), "карта");

    let result = registry
        .call_phrase("ru", "pick", &[Value::from("acc"), Value::from(1)])
        .unwrap();
    assert_eq!(result.to_string(), "карту");
}

// =============================================================================
// MissingVariant Error Message Format
// =============================================================================

#[test]
fn missing_variant_error_contains_phrase_text() {
    let err = EvalError::MissingVariant {
        phrase: "card".to_string(),
        key: "triple".to_string(),
        available: vec!["one".to_string(), "other".to_string()],
        suggestions: vec![],
    };
    let msg = err.to_string();
    assert!(
        msg.contains("card"),
        "error should contain phrase name: {msg}"
    );
    assert!(
        msg.contains("triple"),
        "error should contain missing key: {msg}"
    );
    assert!(
        msg.contains("one"),
        "error should list available keys: {msg}"
    );
    assert!(
        msg.contains("other"),
        "error should list available keys: {msg}"
    );
}

#[test]
fn missing_variant_error_with_suggestions() {
    let err = EvalError::MissingVariant {
        phrase: "card".to_string(),
        key: "oen".to_string(),
        available: vec!["one".to_string(), "other".to_string()],
        suggestions: vec!["one".to_string()],
    };
    let msg = err.to_string();
    assert!(
        msg.contains("did you mean: one?"),
        "error should include suggestion: {msg}"
    );
}

#[test]
fn missing_variant_error_without_suggestions() {
    let err = EvalError::MissingVariant {
        phrase: "card".to_string(),
        key: "xyz".to_string(),
        available: vec!["one".to_string(), "other".to_string()],
        suggestions: vec![],
    };
    let msg = err.to_string();
    assert!(
        !msg.contains("did you mean"),
        "error should not include suggestions: {msg}"
    );
}

// =============================================================================
// String Value as Selector via eval_str
// =============================================================================

#[test]
fn eval_str_string_param_selects_variant() {
    let mut locale = rlf::Locale::new();
    locale
        .load_translations_str(
            "en",
            r#"
        card = { one: "card", other: "cards" };
    "#,
        )
        .unwrap();

    let result = locale
        .eval_str(
            "{card:$v}",
            [("v".to_string(), Value::from("one"))]
                .into_iter()
                .collect(),
        )
        .unwrap();
    assert_eq!(result.to_string(), "card");
}

#[test]
fn eval_str_string_param_no_match_is_error() {
    let mut locale = rlf::Locale::new();
    locale
        .load_translations_str(
            "en",
            r#"
        card = { one: "card", other: "cards" };
    "#,
        )
        .unwrap();

    let err = locale
        .eval_str(
            "{card:$v}",
            [("v".to_string(), Value::from("triple"))]
                .into_iter()
                .collect(),
        )
        .unwrap_err();
    assert!(
        matches!(err, EvalError::MissingVariant { .. }),
        "expected MissingVariant, got: {err:?}"
    );
}

#[test]
fn eval_str_number_param_selects_plural_variant() {
    let mut locale = rlf::Locale::new();
    locale
        .load_translations_str(
            "en",
            r#"
        card = { one: "card", other: "cards" };
    "#,
        )
        .unwrap();

    let result = locale
        .eval_str(
            "{card:$n}",
            [("n".to_string(), Value::from(1))].into_iter().collect(),
        )
        .unwrap();
    assert_eq!(result.to_string(), "card");

    let result = locale
        .eval_str(
            "{card:$n}",
            [("n".to_string(), Value::from(5))].into_iter().collect(),
        )
        .unwrap();
    assert_eq!(result.to_string(), "cards");
}

#[test]
fn eval_str_number_param_missing_plural_category() {
    let mut locale = rlf::Locale::new();
    locale
        .load_translations_str(
            "en",
            r#"
        card = { one: "card" };
    "#,
        )
        .unwrap();

    // n=5 -> "other" in English, but only "one" exists
    let err = locale
        .eval_str(
            "{card:$n}",
            [("n".to_string(), Value::from(5))].into_iter().collect(),
        )
        .unwrap_err();
    assert!(
        matches!(err, EvalError::MissingVariant { ref key, .. } if key == "other"),
        "expected MissingVariant with key 'other', got: {err:?}"
    );
}

// =============================================================================
// Phrase Call Results Used in Selection — Errors
// =============================================================================

#[test]
fn phrase_call_result_missing_variant_on_select() {
    // Phrase call returns a phrase, then :variant selection fails
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        ancient = :an { one: "Ancient", other: "Ancients" };
        subtype($s) = :from($s) "<b>{$s}</b>";
        get_form($s) = "{subtype($s):triple}";
    "#,
        )
        .unwrap();

    let ancient = registry.get_phrase("en", "ancient").unwrap();
    let err = registry
        .call_phrase("en", "get_form", &[Value::Phrase(ancient)])
        .unwrap_err();
    assert!(
        matches!(err, EvalError::MissingVariant { ref key, .. } if key == "triple"),
        "expected MissingVariant with key 'triple', got: {err:?}"
    );
}

// =============================================================================
// Phrase with Tags — Tag-based Selection Errors
// =============================================================================

#[test]
fn tag_selection_no_matching_variant_for_tag() {
    // Phrase has a tag that doesn't match any variant key
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        adj = { masc: "bueno", fem: "buena" };
        thing = :neut "cosa";
        result($t) = "{adj:$t}";
    "#,
        )
        .unwrap();

    let thing = registry.get_phrase("es", "thing").unwrap();
    let err = registry
        .call_phrase("es", "result", &[Value::Phrase(thing)])
        .unwrap_err();
    assert!(
        matches!(err, EvalError::MissingVariant { ref key, .. } if key == "neut"),
        "expected MissingVariant with key 'neut', got: {err:?}"
    );
}

#[test]
fn tag_selection_multiple_tags_first_matching_wins() {
    // Phrase has multiple tags; first matching one should be used
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        adj = { masc: "bueno", anim: "vivo" };
        thing = :masc :anim "gato";
        result($t) = "{adj:$t}";
    "#,
        )
        .unwrap();

    let thing = registry.get_phrase("es", "thing").unwrap();
    let result = registry
        .call_phrase("es", "result", &[Value::Phrase(thing)])
        .unwrap();
    assert_eq!(result.to_string(), "bueno");
}

#[test]
fn tag_selection_second_tag_matches() {
    // First tag doesn't match any variant, but second tag does
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        adj = { anim: "vivo", inanim: "inerte" };
        thing = :masc :anim "gato";
        result($t) = "{adj:$t}";
    "#,
        )
        .unwrap();

    let thing = registry.get_phrase("es", "thing").unwrap();
    let result = registry
        .call_phrase("es", "result", &[Value::Phrase(thing)])
        .unwrap();
    assert_eq!(result.to_string(), "vivo");
}

// =============================================================================
// Russian Plural Selection Errors
// =============================================================================

#[test]
fn russian_number_selector_missing_few_category() {
    // Russian has one/few/many/other; test error when "few" is missing
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        card = { one: "карта", many: "карт", other: "карт" };
        pick($n) = "{card:$n}";
    "#,
        )
        .unwrap();

    // n=3 in Russian -> "few", which is not defined
    let err = registry
        .call_phrase("ru", "pick", &[Value::from(3)])
        .unwrap_err();
    assert!(
        matches!(err, EvalError::MissingVariant { ref key, .. } if key == "few"),
        "expected MissingVariant with key 'few', got: {err:?}"
    );
}

#[test]
fn russian_string_numeric_selector_uses_russian_plurals() {
    // String "21" in Russian -> number 21 -> "one" category
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        card = { one: "карта", few: "карты", many: "карт", other: "карты" };
        pick($n) = "{card:$n}";
    "#,
        )
        .unwrap();

    let result = registry
        .call_phrase("ru", "pick", &[Value::from("21")])
        .unwrap();
    assert_eq!(result.to_string(), "карта");
}

// =============================================================================
// Variant Fallback with Missing Keys
// =============================================================================

#[test]
fn multidim_variant_fallback_on_missing_full_key() {
    // Multi-dimensional: nom.one doesn't exist, falls back to nom
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        card = {
            nom: "карта-ном",
            acc.one: "карту",
            acc.other: "карт",
        };
        pick($n) = "{card:nom:$n}";
    "#,
        )
        .unwrap();

    // nom.one -> falls back to nom
    let result = registry
        .call_phrase("en", "pick", &[Value::from(1)])
        .unwrap();
    assert_eq!(result.to_string(), "карта-ном");
}

#[test]
fn multidim_variant_no_fallback_available() {
    // Multi-dimensional: gen.one doesn't exist, gen doesn't exist either
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        card = {
            nom.one: "карта",
            nom.other: "карт",
        };
        pick($n) = "{card:gen:$n}";
    "#,
        )
        .unwrap();

    let err = registry
        .call_phrase("en", "pick", &[Value::from(1)])
        .unwrap_err();
    assert!(
        matches!(err, EvalError::MissingVariant { .. }),
        "expected MissingVariant, got: {err:?}"
    );
}
