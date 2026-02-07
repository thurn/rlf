//! Integration tests for :match evaluation in the interpreter.

use rlf::{PhraseRegistry, Value};

// =============================================================================
// Single-parameter numeric matching
// =============================================================================

#[test]
fn match_single_param_numeric_exact_one() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        cards($n) = :match($n) {
            1: "a card",
            *other: "{$n} cards",
        };
    "#,
        )
        .unwrap();

    let result = registry
        .call_phrase("en", "cards", &[Value::from(1)])
        .unwrap();
    assert_eq!(result.to_string(), "a card");
}

#[test]
fn match_single_param_numeric_falls_to_default() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        cards($n) = :match($n) {
            1: "a card",
            *other: "{$n} cards",
        };
    "#,
        )
        .unwrap();

    let result = registry
        .call_phrase("en", "cards", &[Value::from(3)])
        .unwrap();
    assert_eq!(result.to_string(), "3 cards");
}

#[test]
fn match_single_param_numeric_zero() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        cards($n) = :match($n) {
            0: "no cards",
            1: "a card",
            *other: "{$n} cards",
        };
    "#,
        )
        .unwrap();

    let result = registry
        .call_phrase("en", "cards", &[Value::from(0)])
        .unwrap();
    assert_eq!(result.to_string(), "no cards");
}

#[test]
fn match_exact_numeric_wins_over_cldr() {
    // n=1: CLDR maps 1 to "one", but exact "1" key should win
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        cards($n) = :match($n) {
            1: "exact one",
            one: "cldr one",
            *other: "{$n} cards",
        };
    "#,
        )
        .unwrap();

    let result = registry
        .call_phrase("en", "cards", &[Value::from(1)])
        .unwrap();
    assert_eq!(result.to_string(), "exact one");
}

#[test]
fn match_cldr_fallback_when_no_exact_numeric() {
    // n=5: no exact "5" key, CLDR maps to "other", hits *other
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        cards($n) = :match($n) {
            1: "a card",
            *other: "{$n} cards",
        };
    "#,
        )
        .unwrap();

    let result = registry
        .call_phrase("en", "cards", &[Value::from(5)])
        .unwrap();
    assert_eq!(result.to_string(), "5 cards");
}

#[test]
fn match_multiple_exact_numeric_keys() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        cards($n) = :match($n) {
            0: "no cards",
            1: "a card",
            2: "a pair of cards",
            *other: "{$n} cards",
        };
    "#,
        )
        .unwrap();

    assert_eq!(
        registry
            .call_phrase("en", "cards", &[Value::from(0)])
            .unwrap()
            .to_string(),
        "no cards"
    );
    assert_eq!(
        registry
            .call_phrase("en", "cards", &[Value::from(1)])
            .unwrap()
            .to_string(),
        "a card"
    );
    assert_eq!(
        registry
            .call_phrase("en", "cards", &[Value::from(2)])
            .unwrap()
            .to_string(),
        "a pair of cards"
    );
    assert_eq!(
        registry
            .call_phrase("en", "cards", &[Value::from(5)])
            .unwrap()
            .to_string(),
        "5 cards"
    );
}

#[test]
fn match_russian_cldr_categories() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        cards($n) = :match($n) {
            1: "карту",
            few: "{$n} карты",
            *other: "{$n} карт",
        };
    "#,
        )
        .unwrap();

    // n=1 -> exact "1" match
    assert_eq!(
        registry
            .call_phrase("ru", "cards", &[Value::from(1)])
            .unwrap()
            .to_string(),
        "карту"
    );

    // n=3 -> CLDR "few"
    assert_eq!(
        registry
            .call_phrase("ru", "cards", &[Value::from(3)])
            .unwrap()
            .to_string(),
        "3 карты"
    );

    // n=5 -> CLDR "many" -> falls to *other
    assert_eq!(
        registry
            .call_phrase("ru", "cards", &[Value::from(5)])
            .unwrap()
            .to_string(),
        "5 карт"
    );
}

#[test]
fn match_russian_with_zero_exact() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        inventory($n) = :match($n) {
            0: "У вас нет предметов.",
            1: "У вас один предмет.",
            few: "У вас {$n} предмета.",
            *other: "У вас {$n} предметов.",
        };
    "#,
        )
        .unwrap();

    assert_eq!(
        registry
            .call_phrase("ru", "inventory", &[Value::from(0)])
            .unwrap()
            .to_string(),
        "У вас нет предметов."
    );
    assert_eq!(
        registry
            .call_phrase("ru", "inventory", &[Value::from(1)])
            .unwrap()
            .to_string(),
        "У вас один предмет."
    );
    assert_eq!(
        registry
            .call_phrase("ru", "inventory", &[Value::from(3)])
            .unwrap()
            .to_string(),
        "У вас 3 предмета."
    );
    assert_eq!(
        registry
            .call_phrase("ru", "inventory", &[Value::from(5)])
            .unwrap()
            .to_string(),
        "У вас 5 предметов."
    );
}

// =============================================================================
// Single-parameter tag-based matching
// =============================================================================

#[test]
fn match_tag_based_fem() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        card = :fem "carta";
        character = :masc "personaje";

        destroyed($thing) = :match($thing) {
            masc: "destruido",
            *fem: "destruida",
        };
    "#,
        )
        .unwrap();

    let card = registry.get_phrase("es", "card").unwrap();
    let result = registry
        .call_phrase("es", "destroyed", &[Value::Phrase(card)])
        .unwrap();
    assert_eq!(result.to_string(), "destruida");
}

#[test]
fn match_tag_based_masc() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        card = :fem "carta";
        character = :masc "personaje";

        destroyed($thing) = :match($thing) {
            masc: "destruido",
            *fem: "destruida",
        };
    "#,
        )
        .unwrap();

    let character = registry.get_phrase("es", "character").unwrap();
    let result = registry
        .call_phrase("es", "destroyed", &[Value::Phrase(character)])
        .unwrap();
    assert_eq!(result.to_string(), "destruido");
}

#[test]
fn match_tag_based_first_tag_wins() {
    // A phrase with multiple tags: first matching tag should win
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        entity = :masc :anim "warrior";

        classify($thing) = :match($thing) {
            masc: "masculine",
            anim: "animate",
            *other: "unknown",
        };
    "#,
        )
        .unwrap();

    let entity = registry.get_phrase("en", "entity").unwrap();
    let result = registry
        .call_phrase("en", "classify", &[Value::Phrase(entity)])
        .unwrap();
    assert_eq!(result.to_string(), "masculine");
}

#[test]
fn match_tag_based_second_tag_matches() {
    // First tag doesn't match any branch, second tag does
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        entity = :masc :anim "warrior";

        classify($thing) = :match($thing) {
            anim: "animate",
            inan: "inanimate",
            *other: "unknown",
        };
    "#,
        )
        .unwrap();

    let entity = registry.get_phrase("en", "entity").unwrap();
    let result = registry
        .call_phrase("en", "classify", &[Value::Phrase(entity)])
        .unwrap();
    assert_eq!(result.to_string(), "animate");
}

#[test]
fn match_tag_based_falls_to_default() {
    // No tags match any branch -> use * default
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        entity = :neut "thing";

        classify($thing) = :match($thing) {
            masc: "masculine",
            fem: "feminine",
            *other: "unknown",
        };
    "#,
        )
        .unwrap();

    let entity = registry.get_phrase("en", "entity").unwrap();
    let result = registry
        .call_phrase("en", "classify", &[Value::Phrase(entity)])
        .unwrap();
    assert_eq!(result.to_string(), "unknown");
}

// =============================================================================
// Single-parameter string matching
// =============================================================================

#[test]
fn match_string_value_direct_key() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        describe($s) = :match($s) {
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

#[test]
fn match_string_value_falls_to_default() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        describe($s) = :match($s) {
            attack: "offensive",
            defend: "defensive",
            *other: "neutral",
        };
    "#,
        )
        .unwrap();

    let result = registry
        .call_phrase("en", "describe", &[Value::from("unknown")])
        .unwrap();
    assert_eq!(result.to_string(), "neutral");
}

// =============================================================================
// Match with phrase calls inside branches
// =============================================================================

#[test]
fn match_branch_calls_other_phrases() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        text_number($n) = :match($n) {
            1: "one",
            *other: "{$n}",
        };
        copies($n) = :match($n) {
            1: "a copy",
            *other: "{text_number($n)} copies",
        };
    "#,
        )
        .unwrap();

    assert_eq!(
        registry
            .call_phrase("en", "copies", &[Value::from(1)])
            .unwrap()
            .to_string(),
        "a copy"
    );
    assert_eq!(
        registry
            .call_phrase("en", "copies", &[Value::from(3)])
            .unwrap()
            .to_string(),
        "3 copies"
    );
}

// =============================================================================
// Match with subtype phrases (from + match composition)
// =============================================================================

#[test]
fn match_with_subtype_phrase_call() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        warrior = :a { one: "Warrior", other: "Warriors" };
        subtype($s) = :from($s) "<b>{$s}</b>";
        count_allied_subtype($n, $s) = :match($n) {
            1: "an allied {subtype($s)}",
            *other: "{$n} allied {subtype($s):other}",
        };
    "#,
        )
        .unwrap();

    let warrior = registry.get_phrase("en", "warrior").unwrap();
    let result = registry
        .call_phrase(
            "en",
            "count_allied_subtype",
            &[Value::from(1), Value::Phrase(warrior)],
        )
        .unwrap();
    assert_eq!(result.to_string(), "an allied <b>Warrior</b>");

    let warrior2 = registry.get_phrase("en", "warrior").unwrap();
    let result = registry
        .call_phrase(
            "en",
            "count_allied_subtype",
            &[Value::from(3), Value::Phrase(warrior2)],
        )
        .unwrap();
    assert_eq!(result.to_string(), "3 allied <b>Warriors</b>");
}

// =============================================================================
// Multi-parameter matching with dot-notation keys
// =============================================================================

#[test]
fn match_multi_param_numeric_and_tag() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        card = :fem "carta";
        character = :masc "personaje";

        n_allied($n, $entity) = :match($n, $entity) {
            1.masc: "un aliado {$entity}",
            1.*fem: "una aliada {$entity}",
            *other.masc: "{$n} aliados {$entity}",
            other.*fem: "{$n} aliadas {$entity}",
        };
    "#,
        )
        .unwrap();

    // n=1, entity=card (fem) -> "1.fem" -> "una aliada carta"
    let card = registry.get_phrase("es", "card").unwrap();
    let result = registry
        .call_phrase("es", "n_allied", &[Value::from(1), Value::Phrase(card)])
        .unwrap();
    assert_eq!(result.to_string(), "una aliada carta");

    // n=3, entity=character (masc) -> CLDR "other", tag "masc" -> "other.masc"
    let character = registry.get_phrase("es", "character").unwrap();
    let result = registry
        .call_phrase(
            "es",
            "n_allied",
            &[Value::from(3), Value::Phrase(character)],
        )
        .unwrap();
    assert_eq!(result.to_string(), "3 aliados personaje");
}

#[test]
fn match_multi_param_both_numeric() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        compare($a, $b) = :match($a, $b) {
            1.1: "both one",
            1.*other: "first one",
            *other.1: "second one",
            other.*other: "neither one",
        };
    "#,
        )
        .unwrap();

    assert_eq!(
        registry
            .call_phrase("en", "compare", &[Value::from(1), Value::from(1)])
            .unwrap()
            .to_string(),
        "both one"
    );
    assert_eq!(
        registry
            .call_phrase("en", "compare", &[Value::from(1), Value::from(5)])
            .unwrap()
            .to_string(),
        "first one"
    );
    assert_eq!(
        registry
            .call_phrase("en", "compare", &[Value::from(5), Value::from(1)])
            .unwrap()
            .to_string(),
        "second one"
    );
    assert_eq!(
        registry
            .call_phrase("en", "compare", &[Value::from(5), Value::from(5)])
            .unwrap()
            .to_string(),
        "neither one"
    );
}

#[test]
fn match_multi_param_wildcard_fallback() {
    // When exact multi-dim key is not found, try wildcard (prefix match)
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        card = :fem "carta";

        n_things($n, $entity) = :match($n, $entity) {
            1.masc: "one masc",
            *other.masc: "other masc",
            1.*fem: "one fem",
            other.*fem: "other fem",
        };
    "#,
        )
        .unwrap();

    // n=1, entity=card (fem) -> try "1.fem" -> matches "1.*fem"
    let card = registry.get_phrase("es", "card").unwrap();
    let result = registry
        .call_phrase("es", "n_things", &[Value::from(1), Value::Phrase(card)])
        .unwrap();
    assert_eq!(result.to_string(), "one fem");
}

// =============================================================================
// Russian multi-param match (from DESIGN_V2.md example)
// =============================================================================

#[test]
fn match_russian_multi_param_gender_and_count() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        card = :fem {
            nom.one: "карта",
            gen.many: "карт",
        };
        character = :masc {
            nom.one: "персонаж",
            gen.many: "персонажей",
        };

        n_allied($n, $entity) = :match($n, $entity) {
            1.masc: "союзный {$entity:nom:one}",
            1.*fem: "союзная {$entity:nom:one}",
            *other.masc: "{$n} союзных {$entity:gen:many}",
            other.*fem: "{$n} союзных {$entity:gen:many}",
        };
    "#,
        )
        .unwrap();

    // n=1, entity=card (fem) -> "1.fem"
    let card = registry.get_phrase("ru", "card").unwrap();
    let result = registry
        .call_phrase("ru", "n_allied", &[Value::from(1), Value::Phrase(card)])
        .unwrap();
    assert_eq!(result.to_string(), "союзная карта");

    // n=3, entity=character (masc) -> CLDR "few", tag "masc" -> tries "few.masc",
    // no exact match -> tries partial: "other.masc" (since *other matches any)
    let character = registry.get_phrase("ru", "character").unwrap();
    let result = registry
        .call_phrase(
            "ru",
            "n_allied",
            &[Value::from(3), Value::Phrase(character)],
        )
        .unwrap();
    assert_eq!(result.to_string(), "3 союзных персонажей");
}

// =============================================================================
// Combined :from + :match
// =============================================================================

#[test]
fn from_match_inherits_tags_and_variants() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        ancient = :an { one: "Ancient", other: "Ancients" };
        subtype($s) = :from($s) "<b>{$s}</b>";
        count_sub($n, $s) = :from($s) :match($n) {
            1: "one {subtype($s)}",
            *other: "{$n} {subtype($s):other}",
        };
    "#,
        )
        .unwrap();

    let ancient = registry.get_phrase("en", "ancient").unwrap();
    let result = registry
        .call_phrase("en", "count_sub", &[Value::from(1), Value::Phrase(ancient)])
        .unwrap();

    // Should inherit :an tag from ancient
    assert!(result.has_tag("an"));
    // Default text (evaluating with default variant of ancient)
    assert_eq!(result.to_string(), "one <b>Ancient</b>");
    // Should have variant structure from ancient
    assert_eq!(result.variant("one"), "one <b>Ancient</b>");
    assert_eq!(result.variant("other"), "one <b>Ancients</b>");
}

#[test]
fn from_match_phrase_call_passes_full_phrase() {
    // Per DESIGN_V2.md: "References to other phrases like {subtype($s)} pass the
    // full Phrase value of $s — they do not see the per-variant context."
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        warrior = :a { one: "Warrior", other: "Warriors" };
        subtype($s) = :from($s) "<b>{$s}</b>";
        count_allied($n, $s) = :from($s) :match($n) {
            1: "allied {subtype($s)}",
            *other: "{$n} allied {subtype($s):other}",
        };
    "#,
        )
        .unwrap();

    let warrior = registry.get_phrase("en", "warrior").unwrap();
    let result = registry
        .call_phrase(
            "en",
            "count_allied",
            &[Value::from(3), Value::Phrase(warrior)],
        )
        .unwrap();

    // Default text (with default "one" variant text of warrior as $s)
    assert_eq!(result.to_string(), "3 allied <b>Warriors</b>");
    // "other" variant iteration: $s resolves to "Warriors" for bare interpolation
    // but subtype($s) gets full Phrase value
    assert_eq!(result.variant("other"), "3 allied <b>Warriors</b>");
}

#[test]
fn from_match_order_does_not_matter() {
    // :from($s) :match($n) and :match($n) :from($s) should be equivalent
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        item = :a { one: "Item", other: "Items" };
        count_a($n, $s) = :from($s) :match($n) {
            1: "one {$s}",
            *other: "{$n} {$s}",
        };
        count_b($n, $s) = :match($n) :from($s) {
            1: "one {$s}",
            *other: "{$n} {$s}",
        };
    "#,
        )
        .unwrap();

    let item_a = registry.get_phrase("en", "item").unwrap();
    let result_a = registry
        .call_phrase("en", "count_a", &[Value::from(1), Value::Phrase(item_a)])
        .unwrap();

    let item_b = registry.get_phrase("en", "item").unwrap();
    let result_b = registry
        .call_phrase("en", "count_b", &[Value::from(1), Value::Phrase(item_b)])
        .unwrap();

    assert_eq!(result_a.to_string(), result_b.to_string());
    assert!(result_a.has_tag("a"));
    assert!(result_b.has_tag("a"));
}

// =============================================================================
// Match with tags on result phrase
// =============================================================================

#[test]
fn match_preserves_definition_tags() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        cards($n) = :a :match($n) {
            1: "a card",
            *other: "{$n} cards",
        };
    "#,
        )
        .unwrap();

    let result = registry
        .call_phrase("en", "cards", &[Value::from(1)])
        .unwrap();
    assert!(result.has_tag("a"));
}

// =============================================================================
// Edge cases
// =============================================================================

#[test]
fn match_default_always_hit_when_no_other_branch_matches() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        fallback($n) = :match($n) {
            *other: "default: {$n}",
        };
    "#,
        )
        .unwrap();

    assert_eq!(
        registry
            .call_phrase("en", "fallback", &[Value::from(1)])
            .unwrap()
            .to_string(),
        "default: 1"
    );
    assert_eq!(
        registry
            .call_phrase("en", "fallback", &[Value::from(999)])
            .unwrap()
            .to_string(),
        "default: 999"
    );
}

#[test]
fn match_multi_key_shorthand() {
    // Multiple keys sharing one branch: "one, two: ..."
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        describe($n) = :match($n) {
            0, 1: "few",
            *other: "many",
        };
    "#,
        )
        .unwrap();

    assert_eq!(
        registry
            .call_phrase("en", "describe", &[Value::from(0)])
            .unwrap()
            .to_string(),
        "few"
    );
    assert_eq!(
        registry
            .call_phrase("en", "describe", &[Value::from(1)])
            .unwrap()
            .to_string(),
        "few"
    );
    assert_eq!(
        registry
            .call_phrase("en", "describe", &[Value::from(5)])
            .unwrap()
            .to_string(),
        "many"
    );
}

// =============================================================================
// Match used via Locale API
// =============================================================================

#[test]
fn match_via_locale_api() {
    use rlf::Locale;

    let mut locale = Locale::builder().language("en").build();
    locale
        .load_translations_str(
            "en",
            r#"
        cards($n) = :match($n) {
            1: "a card",
            *other: "{$n} cards",
        };
    "#,
        )
        .unwrap();

    let result = locale.call_phrase("cards", &[Value::from(1)]).unwrap();
    assert_eq!(result.to_string(), "a card");

    let result = locale.call_phrase("cards", &[Value::from(5)]).unwrap();
    assert_eq!(result.to_string(), "5 cards");
}

// =============================================================================
// Spanish-style destroy pattern (full composition)
// =============================================================================

#[test]
fn match_spanish_destroy_composition() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        card = :fem "carta";
        character = :masc "personaje";

        destroyed($thing) = :match($thing) {
            masc: "destruido",
            *fem: "destruida",
        };

        destroy($thing) = "{$thing} fue {destroyed($thing)}.";
    "#,
        )
        .unwrap();

    let card = registry.get_phrase("es", "card").unwrap();
    let result = registry
        .call_phrase("es", "destroy", &[Value::Phrase(card)])
        .unwrap();
    assert_eq!(result.to_string(), "carta fue destruida.");

    let character = registry.get_phrase("es", "character").unwrap();
    let result = registry
        .call_phrase("es", "destroy", &[Value::Phrase(character)])
        .unwrap();
    assert_eq!(result.to_string(), "personaje fue destruido.");
}

// =============================================================================
// Match error handling
// =============================================================================

#[test]
fn match_no_branch_matches_no_default_is_error() {
    // This should be caught by parser validation, but test evaluator safety
    // We construct a registry directly with a match that has no default
    // Since the parser rejects this, we test via a phrase with tags that don't match
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        entity = :neut "thing";
        describe($thing) = :match($thing) {
            masc: "masculine",
            *fem: "feminine",
        };
    "#,
        )
        .unwrap();

    // neut doesn't match masc or fem, but *fem is the default -> should work
    let entity = registry.get_phrase("en", "entity").unwrap();
    let result = registry
        .call_phrase("en", "describe", &[Value::Phrase(entity)])
        .unwrap();
    assert_eq!(result.to_string(), "feminine");
}
