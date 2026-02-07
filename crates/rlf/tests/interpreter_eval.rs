//! Integration tests for interpreter evaluation.

use rlf::interpreter::EvalError;
use rlf::{Locale, PhraseId, PhraseRegistry, Value};
use std::collections::HashMap;

// =============================================================================
// Basic Template Evaluation
// =============================================================================

#[test]
fn eval_literal_only() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(r#"hello = "Hello, world!";"#)
        .unwrap();
    let result = registry.get_phrase("en", "hello").unwrap();
    assert_eq!(result.to_string(), "Hello, world!");
}

#[test]
fn eval_with_parameter() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(r#"greet($name) = "Hello, {$name}!";"#)
        .unwrap();
    let result = registry
        .call_phrase("en", "greet", &[Value::from("World")])
        .unwrap();
    assert_eq!(result.to_string(), "Hello, World!");
}

#[test]
fn eval_with_number_parameter() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(r#"count($n) = "Count: {$n}";"#)
        .unwrap();
    let result = registry
        .call_phrase("en", "count", &[Value::from(42)])
        .unwrap();
    assert_eq!(result.to_string(), "Count: 42");
}

// =============================================================================
// Variant Selection
// =============================================================================

#[test]
fn eval_literal_variant_selector() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        card = { one: "card", other: "cards" };
        all_cards = "All {card:other}.";
    "#,
        )
        .unwrap();
    let result = registry.get_phrase("en", "all_cards").unwrap();
    assert_eq!(result.to_string(), "All cards.");
}

#[test]
fn eval_numeric_variant_selector_english() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        card = { one: "card", other: "cards" };
        draw($n) = "Draw {$n} {card:$n}.";
    "#,
        )
        .unwrap();

    let one = registry
        .call_phrase("en", "draw", &[Value::from(1)])
        .unwrap();
    assert_eq!(one.to_string(), "Draw 1 card.");

    let five = registry
        .call_phrase("en", "draw", &[Value::from(5)])
        .unwrap();
    assert_eq!(five.to_string(), "Draw 5 cards.");
}

#[test]
fn eval_numeric_variant_selector_russian() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        card = { one: "карта", few: "карты", many: "карт", other: "карты" };
        draw($n) = "Возьмите {$n} {card:$n}.";
    "#,
        )
        .unwrap();

    let one = registry
        .call_phrase("ru", "draw", &[Value::from(1)])
        .unwrap();
    assert_eq!(one.to_string(), "Возьмите 1 карта.");

    let two = registry
        .call_phrase("ru", "draw", &[Value::from(2)])
        .unwrap();
    assert_eq!(two.to_string(), "Возьмите 2 карты.");

    let five = registry
        .call_phrase("ru", "draw", &[Value::from(5)])
        .unwrap();
    assert_eq!(five.to_string(), "Возьмите 5 карт.");

    let twenty_one = registry
        .call_phrase("ru", "draw", &[Value::from(21)])
        .unwrap();
    assert_eq!(twenty_one.to_string(), "Возьмите 21 карта.");
}

// =============================================================================
// Multi-dimensional Variants
// =============================================================================

#[test]
fn eval_multidimensional_variant() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        card = {
            nom.one: "карта",
            nom.few: "карты",
            nom.many: "карт",
            acc.one: "карту",
            acc.few: "карты",
            acc.many: "карт"
        };
        draw($n) = "Возьмите {card:acc:$n}.";
    "#,
        )
        .unwrap();

    let one = registry
        .call_phrase("ru", "draw", &[Value::from(1)])
        .unwrap();
    assert_eq!(one.to_string(), "Возьмите карту.");

    let five = registry
        .call_phrase("ru", "draw", &[Value::from(5)])
        .unwrap();
    assert_eq!(five.to_string(), "Возьмите карт.");
}

#[test]
fn eval_variant_fallback() {
    let mut registry = PhraseRegistry::new();
    // "nom" is fallback for nom.one (accessed as :nom:one), nom.few, etc.
    // Selectors use chained colons, variant keys use dots
    registry
        .load_phrases(
            r#"
        card = {
            nom: "card-nom",
            nom.other: "cards-nom-other",
            acc: "card-acc",
        };
        test_nom_one = "{card:nom:one}";
        test_nom_other = "{card:nom:other}";
        test_acc_one = "{card:acc:one}";
    "#,
        )
        .unwrap();

    // nom.one -> fallback to nom
    let nom_one = registry.get_phrase("en", "test_nom_one").unwrap();
    assert_eq!(nom_one.to_string(), "card-nom");

    // nom.other -> exact match
    let nom_other = registry.get_phrase("en", "test_nom_other").unwrap();
    assert_eq!(nom_other.to_string(), "cards-nom-other");

    // acc.one -> fallback to acc
    let acc_one = registry.get_phrase("en", "test_acc_one").unwrap();
    assert_eq!(acc_one.to_string(), "card-acc");
}

// =============================================================================
// Tag-based Selection
// =============================================================================

#[test]
fn eval_tag_based_selection() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        destroyed = { masc: "destruido", fem: "destruida" };
        card = :fem "carta";
        enemy = :masc "enemigo";
        destroy($thing) = "{$thing} fue {destroyed:$thing}.";
    "#,
        )
        .unwrap();

    // card has :fem tag -> selects "destruida"
    let card = registry.get_phrase("es", "card").unwrap();
    let card_destroyed = registry
        .call_phrase("es", "destroy", &[Value::Phrase(card)])
        .unwrap();
    assert_eq!(card_destroyed.to_string(), "carta fue destruida.");

    // enemy has :masc tag -> selects "destruido"
    let enemy = registry.get_phrase("es", "enemy").unwrap();
    let enemy_destroyed = registry
        .call_phrase("es", "destroy", &[Value::Phrase(enemy)])
        .unwrap();
    assert_eq!(enemy_destroyed.to_string(), "enemigo fue destruido.");
}

// =============================================================================
// Phrase Calls with Arguments
// =============================================================================

#[test]
fn eval_phrase_call_in_template() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        card = { one: "card", other: "cards" };
        draw($n) = "Draw {$n} {card:$n}.";
        draw_and_play($n) = "{draw($n)} Then play one.";
    "#,
        )
        .unwrap();

    let result = registry
        .call_phrase("en", "draw_and_play", &[Value::from(3)])
        .unwrap();
    assert_eq!(result.to_string(), "Draw 3 cards. Then play one.");
}

// =============================================================================
// Phrase as Return Value
// =============================================================================

#[test]
fn get_phrase_returns_phrase_with_variants() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        card = { one: "card", other: "cards" };
    "#,
        )
        .unwrap();

    let phrase = registry.get_phrase("en", "card").unwrap();
    assert_eq!(phrase.to_string(), "card"); // default is first variant
    assert_eq!(phrase.variant("one"), "card");
    assert_eq!(phrase.variant("other"), "cards");
}

#[test]
fn get_phrase_with_tags() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        card = :a "card";
        event = :an "event";
    "#,
        )
        .unwrap();

    let card = registry.get_phrase("en", "card").unwrap();
    assert!(card.has_tag("a"));
    assert!(!card.has_tag("an"));

    let event = registry.get_phrase("en", "event").unwrap();
    assert!(event.has_tag("an"));
}

// =============================================================================
// eval_str
// =============================================================================

#[test]
fn eval_str_basic() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        card = { one: "card", other: "cards" };
    "#,
        )
        .unwrap();

    let params: HashMap<String, Value> = [("n".to_string(), Value::from(3))].into_iter().collect();
    let result = registry
        .eval_str("Draw {$n} {card:$n}.", "en", params)
        .unwrap();
    assert_eq!(result.to_string(), "Draw 3 cards.");
}

// =============================================================================
// PhraseId Resolution
// =============================================================================

#[test]
fn phrase_id_resolve() {
    let mut registry = PhraseRegistry::new();
    registry.load_phrases(r#"hello = "Hello!";"#).unwrap();

    let id = PhraseId::from_name("hello");
    let phrase = id.resolve_with_registry(&registry, "en").unwrap();
    assert_eq!(phrase.to_string(), "Hello!");
}

#[test]
fn phrase_id_call() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(r#"greet($name) = "Hello, {$name}!";"#)
        .unwrap();

    let id = PhraseId::from_name("greet");
    let phrase = id
        .call_with_registry(&registry, "en", &[Value::from("World")])
        .unwrap();
    assert_eq!(phrase.to_string(), "Hello, World!");
}

// =============================================================================
// Error Cases
// =============================================================================

#[test]
fn error_phrase_not_found() {
    let registry = PhraseRegistry::new();
    let err = registry.get_phrase("en", "missing").unwrap_err();
    assert!(matches!(err, EvalError::PhraseNotFound { name } if name == "missing"));
}

#[test]
fn error_argument_count_too_few() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(r#"greet($name) = "Hello, {$name}!";"#)
        .unwrap();

    let err = registry.call_phrase("en", "greet", &[]).unwrap_err();
    assert!(matches!(
        err,
        EvalError::ArgumentCount {
            expected: 1,
            got: 0,
            ..
        }
    ));
}

#[test]
fn error_argument_count_too_many() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(r#"greet($name) = "Hello, {$name}!";"#)
        .unwrap();

    let err = registry
        .call_phrase("en", "greet", &[Value::from("a"), Value::from("b")])
        .unwrap_err();
    assert!(matches!(
        err,
        EvalError::ArgumentCount {
            expected: 1,
            got: 2,
            ..
        }
    ));
}

#[test]
fn error_missing_variant() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        card = { one: "card", other: "cards" };
        bad = "{card:accusative}";
    "#,
        )
        .unwrap();

    let err = registry.get_phrase("en", "bad").unwrap_err();
    assert!(matches!(
        err,
        EvalError::MissingVariant { key, .. } if key == "accusative"
    ));
}

#[test]
fn error_cyclic_reference() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        a = "see {b}";
        b = "see {c}";
        c = "see {a}";
    "#,
        )
        .unwrap();

    let err = registry.get_phrase("en", "a").unwrap_err();
    assert!(
        matches!(err, EvalError::CyclicReference { chain } if chain.contains(&"a".to_string()))
    );
}

#[test]
fn error_max_depth() {
    // Create a deep chain that doesn't cycle but exceeds depth
    let mut content = String::new();
    for i in 0..70 {
        content.push_str(&format!("p{} = \"{{p{}}}\";\n", i, i + 1));
    }
    content.push_str("p70 = \"end\";\n");

    let mut registry = PhraseRegistry::new();
    registry.load_phrases(&content).unwrap();

    let err = registry.get_phrase("en", "p0").unwrap_err();
    assert!(matches!(err, EvalError::MaxDepthExceeded));
}

// =============================================================================
// Metadata Inheritance (:from)
// =============================================================================

#[test]
fn eval_from_modifier_inherits_tags() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        ancient = :an { one: "Ancient", other: "Ancients" };
        subtype($s) = :from($s) "<b>{$s}</b>";
    "#,
        )
        .unwrap();

    let ancient = registry.get_phrase("en", "ancient").unwrap();
    let subtype = registry
        .call_phrase("en", "subtype", &[Value::Phrase(ancient)])
        .unwrap();

    // Should inherit :an tag from ancient
    assert!(subtype.has_tag("an"));
    // Should have variants from evaluating template with each variant
    assert_eq!(subtype.variant("one"), "<b>Ancient</b>");
    assert_eq!(subtype.variant("other"), "<b>Ancients</b>");
}

// =============================================================================
// Escape Sequences
// =============================================================================

#[test]
fn eval_escape_sequences() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        braces = "Use {{$name}} for interpolation.";
        at_sign = "Use @ for transforms.";
        colon = "Ratio 1:2.";
    "#,
        )
        .unwrap();

    assert_eq!(
        registry.get_phrase("en", "braces").unwrap().to_string(),
        "Use {$name} for interpolation."
    );
    assert_eq!(
        registry.get_phrase("en", "at_sign").unwrap().to_string(),
        "Use @ for transforms."
    );
    assert_eq!(
        registry.get_phrase("en", "colon").unwrap().to_string(),
        "Ratio 1:2."
    );
}

// =============================================================================
// Multiple Parameters
// =============================================================================

#[test]
fn eval_multiple_parameters() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(r#"full_greeting($name, $title) = "Hello, {$title} {$name}!";"#)
        .unwrap();

    let result = registry
        .call_phrase(
            "en",
            "full_greeting",
            &[Value::from("Smith"), Value::from("Dr.")],
        )
        .unwrap();
    assert_eq!(result.to_string(), "Hello, Dr. Smith!");
}

// =============================================================================
// Phrase References Without Selectors
// =============================================================================

#[test]
fn eval_phrase_reference_without_selector() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        greeting = "Hello";
        full = "{greeting}, world!";
    "#,
        )
        .unwrap();

    let result = registry.get_phrase("en", "full").unwrap();
    assert_eq!(result.to_string(), "Hello, world!");
}

// =============================================================================
// Nested Phrase Calls
// =============================================================================

#[test]
fn eval_nested_phrase_calls() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        inner($x) = "[{$x}]";
        outer($y) = "({inner($y)})";
    "#,
        )
        .unwrap();

    let result = registry
        .call_phrase("en", "outer", &[Value::from("test")])
        .unwrap();
    assert_eq!(result.to_string(), "([test])");
}

// =============================================================================
// Phrase Parameter Count
// =============================================================================

#[test]
fn phrase_parameter_count() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        no_params = "Hello";
        one_param($x) = "{$x}";
        two_params($a, $b) = "{$a} {$b}";
    "#,
        )
        .unwrap();

    let id0 = PhraseId::from_name("no_params");
    let id1 = PhraseId::from_name("one_param");
    let id2 = PhraseId::from_name("two_params");
    let id_missing = PhraseId::from_name("missing");

    assert_eq!(registry.phrase_parameter_count(id0.as_u128()), 0);
    assert_eq!(registry.phrase_parameter_count(id1.as_u128()), 1);
    assert_eq!(registry.phrase_parameter_count(id2.as_u128()), 2);
    assert_eq!(registry.phrase_parameter_count(id_missing.as_u128()), 0);
}

// =============================================================================
// Template Caching
// =============================================================================

#[test]
fn eval_str_caches_parsed_templates() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(r#"card = { one: "card", other: "cards" };"#)
        .unwrap();

    assert_eq!(registry.template_cache_len(), 0);

    let params: HashMap<String, Value> = [("n".to_string(), Value::from(3))].into_iter().collect();
    registry
        .eval_str("Draw {$n} {card:$n}.", "en", params)
        .unwrap();
    assert_eq!(registry.template_cache_len(), 1);

    // Second call with same template should reuse cache
    let params2: HashMap<String, Value> = [("n".to_string(), Value::from(1))].into_iter().collect();
    let result = registry
        .eval_str("Draw {$n} {card:$n}.", "en", params2)
        .unwrap();
    assert_eq!(result.to_string(), "Draw 1 card.");
    assert_eq!(registry.template_cache_len(), 1);
}

#[test]
fn eval_str_caches_different_templates_separately() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(r#"card = { one: "card", other: "cards" };"#)
        .unwrap();

    let params1: HashMap<String, Value> = [("n".to_string(), Value::from(1))].into_iter().collect();
    registry.eval_str("{$n} {card:$n}", "en", params1).unwrap();
    assert_eq!(registry.template_cache_len(), 1);

    let params2: HashMap<String, Value> = [("n".to_string(), Value::from(2))].into_iter().collect();
    registry
        .eval_str("You have {$n} {card:$n}.", "en", params2)
        .unwrap();
    assert_eq!(registry.template_cache_len(), 2);
}

#[test]
fn clear_template_cache_removes_all_entries() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(r#"card = { one: "card", other: "cards" };"#)
        .unwrap();

    let params: HashMap<String, Value> = [("n".to_string(), Value::from(3))].into_iter().collect();
    registry
        .eval_str("Draw {$n} {card:$n}.", "en", params)
        .unwrap();
    assert_eq!(registry.template_cache_len(), 1);

    registry.clear_template_cache();
    assert_eq!(registry.template_cache_len(), 0);

    // Should still work after clearing cache
    let params2: HashMap<String, Value> = [("n".to_string(), Value::from(1))].into_iter().collect();
    let result = registry
        .eval_str("Draw {$n} {card:$n}.", "en", params2)
        .unwrap();
    assert_eq!(result.to_string(), "Draw 1 card.");
    assert_eq!(registry.template_cache_len(), 1);
}

#[test]
fn eval_str_cache_correct_with_varying_params() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(r#"card = { one: "card", other: "cards" };"#)
        .unwrap();

    let template = "You drew {$n} {card:$n}.";

    for n in [1, 2, 5, 1, 100, 1] {
        let params: HashMap<String, Value> =
            [("n".to_string(), Value::from(n))].into_iter().collect();
        let result = registry.eval_str(template, "en", params).unwrap();
        let expected_card = if n == 1 { "card" } else { "cards" };
        assert_eq!(result.to_string(), format!("You drew {n} {expected_card}."));
    }

    assert_eq!(registry.template_cache_len(), 1);
}

#[test]
fn eval_str_cache_parse_error_not_cached() {
    let mut registry = PhraseRegistry::new();
    registry.load_phrases(r#"hello = "Hello!";"#).unwrap();

    // An invalid template should not be cached
    let params: HashMap<String, Value> = HashMap::new();
    let result = registry.eval_str("Unclosed {brace", "en", params);
    assert!(result.is_err());
    assert_eq!(registry.template_cache_len(), 0);
}

// =============================================================================
// Polish Gender Tag Selection
// =============================================================================

#[test]
fn eval_polish_gender_tag_selection() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        card = :fem "karta";
        enemy = :masc_anim "wróg";
        sword = :masc_inan "miecz";
        kingdom = :neut "królestwo";

        destroyed = {
            masc_anim: "pokonany",
            masc_inan: "zniszczony",
            fem: "zniszczona",
            neut: "zniszczone"
        };

        destroy($thing) = "{$thing} - {destroyed:$thing}";
    "#,
        )
        .unwrap();

    let card = registry.get_phrase("pl", "card").unwrap();
    let result = registry
        .call_phrase("pl", "destroy", &[Value::Phrase(card)])
        .unwrap();
    assert_eq!(result.to_string(), "karta - zniszczona");

    let enemy = registry.get_phrase("pl", "enemy").unwrap();
    let result = registry
        .call_phrase("pl", "destroy", &[Value::Phrase(enemy)])
        .unwrap();
    assert_eq!(result.to_string(), "wróg - pokonany");

    let sword = registry.get_phrase("pl", "sword").unwrap();
    let result = registry
        .call_phrase("pl", "destroy", &[Value::Phrase(sword)])
        .unwrap();
    assert_eq!(result.to_string(), "miecz - zniszczony");

    let kingdom = registry.get_phrase("pl", "kingdom").unwrap();
    let result = registry
        .call_phrase("pl", "destroy", &[Value::Phrase(kingdom)])
        .unwrap();
    assert_eq!(result.to_string(), "królestwo - zniszczone");
}

#[test]
fn eval_polish_case_variants() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        card = :fem {
            nom: "karta",
            nom.many: "kart",
            acc: "kartę",
            acc.many: "kart",
            gen: "karty",
            gen.many: "kart",
            dat: "karcie",
            ins: "kartą",
            loc: "karcie",
            voc: "karto"
        };
        take($n) = "Weź {card:acc:$n}.";
    "#,
        )
        .unwrap();

    let one = registry
        .call_phrase("pl", "take", &[Value::from(1)])
        .unwrap();
    assert_eq!(one.to_string(), "Weź kartę.");

    let five = registry
        .call_phrase("pl", "take", &[Value::from(5)])
        .unwrap();
    assert_eq!(five.to_string(), "Weź kart.");
}

#[test]
fn eval_polish_plural_categories() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        card = :fem {
            one: "karta",
            few: "karty",
            many: "kart",
            other: "karty"
        };
        draw($n) = "Dobierz {$n} {card:$n}.";
    "#,
        )
        .unwrap();

    // Polish plural: 1=one, 2-4=few, 5-21=many (for 5-20), 22-24=few, etc.
    let one = registry
        .call_phrase("pl", "draw", &[Value::from(1)])
        .unwrap();
    assert_eq!(one.to_string(), "Dobierz 1 karta.");

    let three = registry
        .call_phrase("pl", "draw", &[Value::from(3)])
        .unwrap();
    assert_eq!(three.to_string(), "Dobierz 3 karty.");

    let five = registry
        .call_phrase("pl", "draw", &[Value::from(5)])
        .unwrap();
    assert_eq!(five.to_string(), "Dobierz 5 kart.");

    let twenty_two = registry
        .call_phrase("pl", "draw", &[Value::from(22)])
        .unwrap();
    assert_eq!(twenty_two.to_string(), "Dobierz 22 karty.");
}

#[test]
fn eval_polish_masc_anim_case_and_plural() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        enemy = :masc_anim {
            nom.one: "wróg",
            nom: "wrogowie",
            nom.many: "wrogów",
            acc: "wroga",
            acc.many: "wrogów"
        };
        defeat($n) = "Pokonaj {$n} {enemy:acc:$n}.";
    "#,
        )
        .unwrap();

    let one = registry
        .call_phrase("pl", "defeat", &[Value::from(1)])
        .unwrap();
    assert_eq!(one.to_string(), "Pokonaj 1 wroga.");

    let five = registry
        .call_phrase("pl", "defeat", &[Value::from(5)])
        .unwrap();
    assert_eq!(five.to_string(), "Pokonaj 5 wrogów.");

    let three = registry
        .call_phrase("pl", "defeat", &[Value::from(3)])
        .unwrap();
    assert_eq!(three.to_string(), "Pokonaj 3 wroga.");
}

#[test]
fn eval_polish_all_seven_cases() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        card = :fem {
            nom: "karta",
            gen: "karty",
            dat: "karcie",
            acc: "kartę",
            ins: "kartą",
            loc: "karcie",
            voc: "karto"
        };
        test_nom = "{card:nom}";
        test_gen = "{card:gen}";
        test_dat = "{card:dat}";
        test_acc = "{card:acc}";
        test_ins = "{card:ins}";
        test_loc = "{card:loc}";
        test_voc = "{card:voc}";
    "#,
        )
        .unwrap();

    assert_eq!(
        registry.get_phrase("pl", "test_nom").unwrap().to_string(),
        "karta"
    );
    assert_eq!(
        registry.get_phrase("pl", "test_gen").unwrap().to_string(),
        "karty"
    );
    assert_eq!(
        registry.get_phrase("pl", "test_dat").unwrap().to_string(),
        "karcie"
    );
    assert_eq!(
        registry.get_phrase("pl", "test_acc").unwrap().to_string(),
        "kartę"
    );
    assert_eq!(
        registry.get_phrase("pl", "test_ins").unwrap().to_string(),
        "kartą"
    );
    assert_eq!(
        registry.get_phrase("pl", "test_loc").unwrap().to_string(),
        "karcie"
    );
    assert_eq!(
        registry.get_phrase("pl", "test_voc").unwrap().to_string(),
        "karto"
    );
}

// =============================================================================
// Russian Gender Tag Selection
// =============================================================================

#[test]
fn eval_russian_gender_tag_selection() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        card = :fem :inan "карта";
        character = :masc :anim "персонаж";
        event = :neut :inan "событие";

        another_adj = {
            masc: "другой",
            fem: "другая",
            neut: "другое"
        };

        another($entity) = "{another_adj:$entity} {$entity}";
    "#,
        )
        .unwrap();

    // card is :fem -> selects "другая"
    let card = registry.get_phrase("ru", "card").unwrap();
    let result = registry
        .call_phrase("ru", "another", &[Value::Phrase(card)])
        .unwrap();
    assert_eq!(result.to_string(), "другая карта");

    // character is :masc -> selects "другой"
    let character = registry.get_phrase("ru", "character").unwrap();
    let result = registry
        .call_phrase("ru", "another", &[Value::Phrase(character)])
        .unwrap();
    assert_eq!(result.to_string(), "другой персонаж");

    // event is :neut -> selects "другое"
    let event = registry.get_phrase("ru", "event").unwrap();
    let result = registry
        .call_phrase("ru", "another", &[Value::Phrase(event)])
        .unwrap();
    assert_eq!(result.to_string(), "другое событие");
}

// =============================================================================
// Russian Animacy Tag Selection
// =============================================================================

#[test]
fn eval_russian_animacy_tag_selection() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        character = :masc :anim "персонаж";
        card = :fem :inan "карта";
        sword = :masc :inan "меч";

        target_type = {
            anim: "живая цель",
            inan: "предмет"
        };

        describe($entity) = "{$entity} - {target_type:$entity}";
    "#,
        )
        .unwrap();

    // character is :masc :anim -> tries "masc" first (no match), then "anim" -> "живая цель"
    let character = registry.get_phrase("ru", "character").unwrap();
    let result = registry
        .call_phrase("ru", "describe", &[Value::Phrase(character)])
        .unwrap();
    assert_eq!(result.to_string(), "персонаж - живая цель");

    // card is :fem :inan -> tries "fem" first (no match), then "inan" -> "предмет"
    let card = registry.get_phrase("ru", "card").unwrap();
    let result = registry
        .call_phrase("ru", "describe", &[Value::Phrase(card)])
        .unwrap();
    assert_eq!(result.to_string(), "карта - предмет");

    // sword is :masc :inan -> tries "masc" first (no match), then "inan" -> "предмет"
    let sword = registry.get_phrase("ru", "sword").unwrap();
    let result = registry
        .call_phrase("ru", "describe", &[Value::Phrase(sword)])
        .unwrap();
    assert_eq!(result.to_string(), "меч - предмет");
}

#[test]
fn eval_russian_animacy_affects_accusative() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        character = :masc :anim {
            nom.one: "персонаж",
            nom: "персонажи",
            nom.many: "персонажей",
            acc: "персонажа",
            acc.many: "персонажей",
            gen: "персонажа",
            gen.many: "персонажей"
        };

        card = :fem :inan {
            nom: "карта",
            nom.many: "карт",
            acc.one: "карту",
            acc: "карты",
            acc.many: "карт",
            gen.one: "карты",
            gen: "карт"
        };

        take($n) = "Возьмите {card:acc:$n}.";
        defeat($n) = "Победите {character:acc:$n}.";
    "#,
        )
        .unwrap();

    // Animate masculine: acc = gen (персонажа)
    let one_char = registry
        .call_phrase("ru", "defeat", &[Value::from(1)])
        .unwrap();
    assert_eq!(one_char.to_string(), "Победите персонажа.");

    let five_char = registry
        .call_phrase("ru", "defeat", &[Value::from(5)])
        .unwrap();
    assert_eq!(five_char.to_string(), "Победите персонажей.");

    // Inanimate feminine: acc has distinct forms
    let one_card = registry
        .call_phrase("ru", "take", &[Value::from(1)])
        .unwrap();
    assert_eq!(one_card.to_string(), "Возьмите карту.");

    let five_card = registry
        .call_phrase("ru", "take", &[Value::from(5)])
        .unwrap();
    assert_eq!(five_card.to_string(), "Возьмите карт.");
}

// =============================================================================
// Russian All Six Cases
// =============================================================================

#[test]
fn eval_russian_all_six_cases() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        card = :fem :inan {
            nom: "карта",
            acc: "карту",
            gen: "карты",
            dat: "карте",
            ins: "картой",
            prep: "карте"
        };
        test_nom = "{card:nom}";
        test_acc = "{card:acc}";
        test_gen = "{card:gen}";
        test_dat = "{card:dat}";
        test_ins = "{card:ins}";
        test_prep = "{card:prep}";
    "#,
        )
        .unwrap();

    assert_eq!(
        registry.get_phrase("ru", "test_nom").unwrap().to_string(),
        "карта"
    );
    assert_eq!(
        registry.get_phrase("ru", "test_acc").unwrap().to_string(),
        "карту"
    );
    assert_eq!(
        registry.get_phrase("ru", "test_gen").unwrap().to_string(),
        "карты"
    );
    assert_eq!(
        registry.get_phrase("ru", "test_dat").unwrap().to_string(),
        "карте"
    );
    assert_eq!(
        registry.get_phrase("ru", "test_ins").unwrap().to_string(),
        "картой"
    );
    assert_eq!(
        registry.get_phrase("ru", "test_prep").unwrap().to_string(),
        "карте"
    );
}

#[test]
fn eval_russian_masculine_all_six_cases() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        ally = :masc :anim {
            nom.one: "союзник",
            nom: "союзники",
            acc: "союзника",
            gen: "союзника",
            dat: "союзнику",
            ins: "союзником",
            prep: "союзнике"
        };
        test_nom = "{ally:nom}";
        test_acc = "{ally:acc}";
        test_gen = "{ally:gen}";
        test_dat = "{ally:dat}";
        test_ins = "{ally:ins}";
        test_prep = "{ally:prep}";
    "#,
        )
        .unwrap();

    assert_eq!(
        registry.get_phrase("ru", "test_nom").unwrap().to_string(),
        "союзники"
    );
    assert_eq!(
        registry.get_phrase("ru", "test_acc").unwrap().to_string(),
        "союзника"
    );
    assert_eq!(
        registry.get_phrase("ru", "test_gen").unwrap().to_string(),
        "союзника"
    );
    assert_eq!(
        registry.get_phrase("ru", "test_dat").unwrap().to_string(),
        "союзнику"
    );
    assert_eq!(
        registry.get_phrase("ru", "test_ins").unwrap().to_string(),
        "союзником"
    );
    assert_eq!(
        registry.get_phrase("ru", "test_prep").unwrap().to_string(),
        "союзнике"
    );
}

#[test]
fn eval_russian_neuter_all_six_cases() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        event = :neut :inan {
            nom: "событие",
            acc: "событие",
            gen: "события",
            dat: "событию",
            ins: "событием",
            prep: "событии"
        };
        test_nom = "{event:nom}";
        test_acc = "{event:acc}";
        test_gen = "{event:gen}";
        test_dat = "{event:dat}";
        test_ins = "{event:ins}";
        test_prep = "{event:prep}";
    "#,
        )
        .unwrap();

    assert_eq!(
        registry.get_phrase("ru", "test_nom").unwrap().to_string(),
        "событие"
    );
    assert_eq!(
        registry.get_phrase("ru", "test_acc").unwrap().to_string(),
        "событие"
    );
    assert_eq!(
        registry.get_phrase("ru", "test_gen").unwrap().to_string(),
        "события"
    );
    assert_eq!(
        registry.get_phrase("ru", "test_dat").unwrap().to_string(),
        "событию"
    );
    assert_eq!(
        registry.get_phrase("ru", "test_ins").unwrap().to_string(),
        "событием"
    );
    assert_eq!(
        registry.get_phrase("ru", "test_prep").unwrap().to_string(),
        "событии"
    );
}

// =============================================================================
// Russian Case + Plural Multi-dimensional Variants
// =============================================================================

#[test]
fn eval_russian_case_and_plural() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        card = :fem :inan {
            nom.one: "карта",
            nom: "карты",
            nom.many: "карт",
            acc.one: "карту",
            acc: "карты",
            acc.many: "карт",
            gen.one: "карты",
            gen: "карт",
            gen.many: "карт",
            ins.one: "картой",
            ins: "картами"
        };
        draw($n) = "Возьмите {card:acc:$n}.";
        count($n) = "{$n} {card:gen:$n}";
    "#,
        )
        .unwrap();

    // acc.one -> "карту"
    let one = registry
        .call_phrase("ru", "draw", &[Value::from(1)])
        .unwrap();
    assert_eq!(one.to_string(), "Возьмите карту.");

    // acc.few -> falls back to acc -> "карты"
    let two = registry
        .call_phrase("ru", "draw", &[Value::from(2)])
        .unwrap();
    assert_eq!(two.to_string(), "Возьмите карты.");

    // acc.many -> "карт"
    let five = registry
        .call_phrase("ru", "draw", &[Value::from(5)])
        .unwrap();
    assert_eq!(five.to_string(), "Возьмите карт.");

    // gen.one -> "карты"
    let count_one = registry
        .call_phrase("ru", "count", &[Value::from(1)])
        .unwrap();
    assert_eq!(count_one.to_string(), "1 карты");

    // gen.many -> "карт"
    let count_five = registry
        .call_phrase("ru", "count", &[Value::from(5)])
        .unwrap();
    assert_eq!(count_five.to_string(), "5 карт");
}

#[test]
fn eval_russian_animate_masc_case_and_plural() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        ally = :masc :anim {
            nom.one: "союзник",
            nom: "союзники",
            nom.many: "союзников",
            acc: "союзника",
            acc.many: "союзников",
            gen: "союзника",
            gen.many: "союзников",
            ins.one: "союзником",
            ins: "союзниками"
        };
        defeat($n) = "Победите {$n} {ally:acc:$n}.";
    "#,
        )
        .unwrap();

    // acc.one -> falls back to acc -> "союзника"
    let one = registry
        .call_phrase("ru", "defeat", &[Value::from(1)])
        .unwrap();
    assert_eq!(one.to_string(), "Победите 1 союзника.");

    // acc.few -> falls back to acc -> "союзника"
    let three = registry
        .call_phrase("ru", "defeat", &[Value::from(3)])
        .unwrap();
    assert_eq!(three.to_string(), "Победите 3 союзника.");

    // acc.many -> "союзников"
    let five = registry
        .call_phrase("ru", "defeat", &[Value::from(5)])
        .unwrap();
    assert_eq!(five.to_string(), "Победите 5 союзников.");
}

// =============================================================================
// Russian Plural Categories
// =============================================================================

#[test]
fn eval_russian_plural_categories() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        card = :fem :inan {
            one: "карта",
            few: "карты",
            many: "карт",
            other: "карты"
        };
        draw($n) = "Возьмите {$n} {card:$n}.";
    "#,
        )
        .unwrap();

    // Russian plural: 1=one, 2-4=few, 5-20=many, 21=one, 22-24=few, 25-30=many
    let one = registry
        .call_phrase("ru", "draw", &[Value::from(1)])
        .unwrap();
    assert_eq!(one.to_string(), "Возьмите 1 карта.");

    let two = registry
        .call_phrase("ru", "draw", &[Value::from(2)])
        .unwrap();
    assert_eq!(two.to_string(), "Возьмите 2 карты.");

    let five = registry
        .call_phrase("ru", "draw", &[Value::from(5)])
        .unwrap();
    assert_eq!(five.to_string(), "Возьмите 5 карт.");

    let twenty_one = registry
        .call_phrase("ru", "draw", &[Value::from(21)])
        .unwrap();
    assert_eq!(twenty_one.to_string(), "Возьмите 21 карта.");

    let twenty_two = registry
        .call_phrase("ru", "draw", &[Value::from(22)])
        .unwrap();
    assert_eq!(twenty_two.to_string(), "Возьмите 22 карты.");

    let twenty_five = registry
        .call_phrase("ru", "draw", &[Value::from(25)])
        .unwrap();
    assert_eq!(twenty_five.to_string(), "Возьмите 25 карт.");
}

// =============================================================================
// Russian Compositional Phrases (Phrase as Parameter)
// =============================================================================

#[test]
fn eval_russian_compositional_with_gender_agreement() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        card = :fem :inan {
            nom.one: "карта",
            nom: "карты",
            gen.many: "карт"
        };
        character = :masc :anim {
            nom.one: "персонаж",
            nom: "персонажи",
            gen.many: "персонажей"
        };
        event = :neut :inan {
            nom: "событие",
            gen.many: "событий"
        };

        allied_adj = {
            masc: "союзный",
            fem: "союзная",
            neut: "союзное"
        };
        allied($entity) = "{allied_adj:$entity} {$entity:nom:one}";
        allied_plural($entity) = "союзных {$entity:gen:many}";
    "#,
        )
        .unwrap();

    // card is :fem -> "союзная карта"
    let card = registry.get_phrase("ru", "card").unwrap();
    let result = registry
        .call_phrase("ru", "allied", &[Value::Phrase(card)])
        .unwrap();
    assert_eq!(result.to_string(), "союзная карта");

    // character is :masc -> "союзный персонаж"
    let character = registry.get_phrase("ru", "character").unwrap();
    let result = registry
        .call_phrase("ru", "allied", &[Value::Phrase(character)])
        .unwrap();
    assert_eq!(result.to_string(), "союзный персонаж");

    // event is :neut -> "союзное событие"
    let event = registry.get_phrase("ru", "event").unwrap();
    let result = registry
        .call_phrase("ru", "allied", &[Value::Phrase(event)])
        .unwrap();
    assert_eq!(result.to_string(), "союзное событие");

    // genitive plural: "союзных персонажей"
    let character2 = registry.get_phrase("ru", "character").unwrap();
    let result = registry
        .call_phrase("ru", "allied_plural", &[Value::Phrase(character2)])
        .unwrap();
    assert_eq!(result.to_string(), "союзных персонажей");
}

#[test]
fn eval_russian_cost_comparison_pattern() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        character = :masc :anim {
            nom.one: "персонаж",
            nom: "персонажи",
            gen.many: "персонажей"
        };
        card = :fem :inan {
            nom.one: "карта",
            gen.many: "карт"
        };

        with_cost_less_than_allied($base, $counting) = "{$base:nom:one} со стоимостью меньше количества союзных {$counting:gen:many}";
    "#,
        )
        .unwrap();

    // Pattern from APPENDIX_RUSSIAN_TRANSLATION.md
    let base = registry.get_phrase("ru", "character").unwrap();
    let counting = registry.get_phrase("ru", "character").unwrap();
    let result = registry
        .call_phrase(
            "ru",
            "with_cost_less_than_allied",
            &[Value::Phrase(base), Value::Phrase(counting)],
        )
        .unwrap();
    assert_eq!(
        result.to_string(),
        "персонаж со стоимостью меньше количества союзных персонажей"
    );

    // With card as counting base
    let base2 = registry.get_phrase("ru", "character").unwrap();
    let counting2 = registry.get_phrase("ru", "card").unwrap();
    let result = registry
        .call_phrase(
            "ru",
            "with_cost_less_than_allied",
            &[Value::Phrase(base2), Value::Phrase(counting2)],
        )
        .unwrap();
    assert_eq!(
        result.to_string(),
        "персонаж со стоимостью меньше количества союзных карт"
    );
}

// =============================================================================
// Russian Instrumental and Prepositional Case Usage
// =============================================================================

#[test]
fn eval_russian_instrumental_case_negation() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        character = :masc :anim {
            nom.one: "персонаж",
            ins.one: "персонажем",
            ins: "персонажами"
        };
        card = :fem :inan {
            nom.one: "карта",
            ins.one: "картой",
            ins: "картами"
        };
        event = :neut :inan {
            nom.one: "событие",
            ins.one: "событием",
            ins: "событиями"
        };

        not_a($entity) = "персонаж, который не является {$entity:ins:one}";
    "#,
        )
        .unwrap();

    // Instrumental case for negation: "не является картой"
    let card = registry.get_phrase("ru", "card").unwrap();
    let result = registry
        .call_phrase("ru", "not_a", &[Value::Phrase(card)])
        .unwrap();
    assert_eq!(result.to_string(), "персонаж, который не является картой");

    // "не является событием"
    let event = registry.get_phrase("ru", "event").unwrap();
    let result = registry
        .call_phrase("ru", "not_a", &[Value::Phrase(event)])
        .unwrap();
    assert_eq!(result.to_string(), "персонаж, который не является событием");
}

#[test]
fn eval_russian_prepositional_case_location() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        void = :fem :inan {
            nom: "бездна",
            prep: "бездне"
        };

        in_void = "в {void:prep}";
        in_your_void = "в вашей {void:prep}";
    "#,
        )
        .unwrap();

    let result = registry.get_phrase("ru", "in_void").unwrap();
    assert_eq!(result.to_string(), "в бездне");

    let result = registry.get_phrase("ru", "in_your_void").unwrap();
    assert_eq!(result.to_string(), "в вашей бездне");
}

// =============================================================================
// Russian Multi-tag Selector Fallback
// =============================================================================

#[test]
fn eval_russian_multi_tag_first_tag_matches() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        character = :masc :anim "персонаж";

        adj = {
            masc: "сильный",
            fem: "сильная",
            neut: "сильное"
        };

        describe($entity) = "{adj:$entity} {$entity}";
    "#,
        )
        .unwrap();

    // First tag (:masc) matches, so "masc" variant is selected
    let character = registry.get_phrase("ru", "character").unwrap();
    let result = registry
        .call_phrase("ru", "describe", &[Value::Phrase(character)])
        .unwrap();
    assert_eq!(result.to_string(), "сильный персонаж");
}

#[test]
fn eval_russian_multi_tag_second_tag_matches() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        character = :masc :anim "персонаж";
        sword = :masc :inan "меч";

        target = {
            anim: "одушевлённый",
            inan: "неодушевлённый"
        };

        classify($entity) = "{$entity} - {target:$entity}";
    "#,
        )
        .unwrap();

    // character: first tag "masc" doesn't match, second tag "anim" matches
    let character = registry.get_phrase("ru", "character").unwrap();
    let result = registry
        .call_phrase("ru", "classify", &[Value::Phrase(character)])
        .unwrap();
    assert_eq!(result.to_string(), "персонаж - одушевлённый");

    // sword: first tag "masc" doesn't match, second tag "inan" matches
    let sword = registry.get_phrase("ru", "sword").unwrap();
    let result = registry
        .call_phrase("ru", "classify", &[Value::Phrase(sword)])
        .unwrap();
    assert_eq!(result.to_string(), "меч - неодушевлённый");
}

// =============================================================================
// Russian Complex Scenario: Full Translation Pattern
// =============================================================================

#[test]
fn eval_russian_full_translation_pattern() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        card = :fem :inan {
            nom.one: "карта",
            nom: "карты",
            nom.many: "карт",
            acc.one: "карту",
            acc: "карты",
            acc.many: "карт",
            gen.one: "карты",
            gen: "карт",
            gen.many: "карт",
            ins.one: "картой",
            ins: "картами"
        };

        character = :masc :anim {
            nom.one: "персонаж",
            nom: "персонажи",
            nom.many: "персонажей",
            acc: "персонажа",
            acc.many: "персонажей",
            gen: "персонажа",
            gen.many: "персонажей",
            ins.one: "персонажем",
            ins: "персонажами"
        };

        enemy = :masc :anim {
            nom.one: "враг",
            nom: "враги",
            nom.many: "врагов",
            acc: "врага",
            acc.many: "врагов",
            ins.one: "врагом",
            ins: "врагами"
        };

        each_adj = {
            masc: "каждый",
            fem: "каждая",
            neut: "каждое"
        };
        for_each($entity) = "{each_adj:$entity} {$entity:nom:one}";

        with_spark($base, $spark, $op) = "{$base:nom:one} с искрой {$spark}{$op}";
        or_less = " или меньше";

        in_your_void($things) = "{$things} в вашей бездне";
    "#,
        )
        .unwrap();

    // "каждый персонаж"
    let character = registry.get_phrase("ru", "character").unwrap();
    let result = registry
        .call_phrase("ru", "for_each", &[Value::Phrase(character)])
        .unwrap();
    assert_eq!(result.to_string(), "каждый персонаж");

    // "каждая карта"
    let card = registry.get_phrase("ru", "card").unwrap();
    let result = registry
        .call_phrase("ru", "for_each", &[Value::Phrase(card)])
        .unwrap();
    assert_eq!(result.to_string(), "каждая карта");

    // "персонаж с искрой 3 или меньше"
    let base = registry.get_phrase("ru", "character").unwrap();
    let or_less = registry.get_phrase("ru", "or_less").unwrap();
    let result = registry
        .call_phrase(
            "ru",
            "with_spark",
            &[Value::Phrase(base), Value::from(3), Value::Phrase(or_less)],
        )
        .unwrap();
    assert_eq!(result.to_string(), "персонаж с искрой 3 или меньше");

    // "враг в вашей бездне" (default text from first variant nom.one)
    let enemies = registry.get_phrase("ru", "enemy").unwrap();
    let result = registry
        .call_phrase("ru", "in_your_void", &[Value::Phrase(enemies)])
        .unwrap();
    assert_eq!(result.to_string(), "враг в вашей бездне");
}

// =============================================================================
// Russian Dative Case
// =============================================================================

#[test]
fn eval_russian_dative_case() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        ally = :masc :anim {
            nom.one: "союзник",
            dat.one: "союзнику",
            dat: "союзникам"
        };
        card = :fem :inan {
            nom.one: "карта",
            dat.one: "карте",
            dat: "картам"
        };

        give_to($entity) = "дать {$entity:dat:one}";
    "#,
        )
        .unwrap();

    let ally = registry.get_phrase("ru", "ally").unwrap();
    let result = registry
        .call_phrase("ru", "give_to", &[Value::Phrase(ally)])
        .unwrap();
    assert_eq!(result.to_string(), "дать союзнику");

    let card = registry.get_phrase("ru", "card").unwrap();
    let result = registry
        .call_phrase("ru", "give_to", &[Value::Phrase(card)])
        .unwrap();
    assert_eq!(result.to_string(), "дать карте");
}

// =============================================================================
// Phrase Call in Template (v2 syntax replaces v1 auto-forwarding)
// =============================================================================

#[test]
fn eval_phrase_call_replaces_auto_forward() {
    // v2: phrases with params cannot have variant blocks. Use a term with
    // parameterized selection and a simple phrase that calls into it.
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        card = { one: "card", other: "cards" };
        draw($n) = "Draw {$n} {card:$n}.";
    "#,
        )
        .unwrap();

    let one = registry
        .call_phrase("en", "draw", &[Value::from(1)])
        .unwrap();
    assert_eq!(one.to_string(), "Draw 1 card.");

    let five = registry
        .call_phrase("en", "draw", &[Value::from(5)])
        .unwrap();
    assert_eq!(five.to_string(), "Draw 5 cards.");
}

#[test]
fn eval_bare_identifier_on_parameterized_phrase_is_error() {
    // v2: bare identifier referencing a phrase is an error — must use ()
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        cards($n) = "{$n} cards";
        bad = "{cards}";
    "#,
        )
        .unwrap();

    let err = registry.get_phrase("en", "bad").unwrap_err();
    assert!(
        matches!(err, EvalError::SelectorOnPhrase { ref name } if name == "cards"),
        "expected SelectorOnPhrase, got: {err:?}"
    );
}

#[test]
fn eval_term_with_multidimensional_variant_selection() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        noun = {
            nom.one: "card",
            nom.other: "cards",
            acc.one: "card",
            acc.other: "cards"
        };
        test($n) = "Take {noun:acc:$n}.";
    "#,
        )
        .unwrap();

    let one = registry
        .call_phrase("en", "test", &[Value::from(1)])
        .unwrap();
    assert_eq!(one.to_string(), "Take card.");

    let five = registry
        .call_phrase("en", "test", &[Value::from(5)])
        .unwrap();
    assert_eq!(five.to_string(), "Take cards.");
}

// =============================================================================
// Multi-tag Selector Candidates
// =============================================================================

#[test]
fn eval_multi_tag_selector_uses_all_tags() {
    // Verifies that selector resolution considers ALL tags on a phrase,
    // not just the first one. The "animacy" variants only match the second
    // tag (:anim or :inan), so this test would fail if only the first tag
    // (:masc) were used.
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        hero = :masc :anim "герой";
        rock = :masc :inan "камень";

        animacy = {
            anim: "живой",
            inan: "неживой"
        };

        describe($thing) = "{$thing} — {animacy:$thing}";
    "#,
        )
        .unwrap();

    let hero = registry.get_phrase("ru", "hero").unwrap();
    let result = registry
        .call_phrase("ru", "describe", &[Value::Phrase(hero)])
        .unwrap();
    assert_eq!(result.to_string(), "герой — живой");

    let rock = registry.get_phrase("ru", "rock").unwrap();
    let result = registry
        .call_phrase("ru", "describe", &[Value::Phrase(rock)])
        .unwrap();
    assert_eq!(result.to_string(), "камень — неживой");
}

// =============================================================================
// Auto-select Variants on Phrase Calls
// =============================================================================

#[test]
fn eval_variant_phrase_with_params_is_error() {
    // v2: phrases with parameters cannot have variant blocks
    let mut registry = PhraseRegistry::new();
    let result = registry.load_phrases(
        r#"
        text_number($n) = {
            one: "one",
            other: "{$n}",
        };
    "#,
    );
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("variant block"));
}

#[test]
fn eval_variant_term_selects_by_key() {
    // v2: variant blocks are for terms only
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        this_turn = {
            one: "this turn",
            other: "this turn many times",
        };
        report($n) = "{this_turn:$n}";
    "#,
        )
        .unwrap();

    let one = registry
        .call_phrase("en", "report", &[Value::from(1)])
        .unwrap();
    assert_eq!(one.to_string(), "this turn");

    let three = registry
        .call_phrase("en", "report", &[Value::from(3)])
        .unwrap();
    assert_eq!(three.to_string(), "this turn many times");
}

#[test]
fn eval_variant_term_preserves_variants() {
    // v2: variant blocks are for terms only; test variant access
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        item = {
            one: "item",
            other: "items",
        };
    "#,
        )
        .unwrap();

    let result = registry.get_phrase("en", "item").unwrap();
    assert_eq!(result.to_string(), "item");
    assert_eq!(result.variant("one"), "item");
    assert_eq!(result.variant("other"), "items");
}

#[test]
fn eval_variant_term_russian_plural_selection() {
    // v2: variant blocks are for terms only; test parameterized selection
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        card_ru = {
            one: "карта",
            few: "карты",
            many: "карт",
            other: "карты",
        };
        counted_cards($n) = "{$n} {card_ru:$n}";
    "#,
        )
        .unwrap();

    let one = registry
        .call_phrase("ru", "counted_cards", &[Value::from(1)])
        .unwrap();
    assert_eq!(one.to_string(), "1 карта");

    let three = registry
        .call_phrase("ru", "counted_cards", &[Value::from(3)])
        .unwrap();
    assert_eq!(three.to_string(), "3 карты");

    let five = registry
        .call_phrase("ru", "counted_cards", &[Value::from(5)])
        .unwrap();
    assert_eq!(five.to_string(), "5 карт");
}

#[test]
fn eval_variant_phrase_with_params_and_non_numeric_is_error() {
    // v2: phrases with parameters cannot have variant blocks
    let mut registry = PhraseRegistry::new();
    let result = registry.load_phrases(
        r#"
        greeting($name) = {
            formal: "Good day, {$name}.",
            casual: "Hey {$name}!",
        };
    "#,
    );
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("variant block"));
}

#[test]
fn eval_variant_term_selects_via_locale() {
    // v2: variant blocks are for terms only; test via Locale API
    let mut locale = Locale::builder().language("en").build();
    locale
        .load_translations_str(
            "en",
            r#"
        this_turn = {
            one: "this turn",
            other: "this turn many times",
        };
        report($n) = "{this_turn:$n}";
    "#,
        )
        .unwrap();

    let one = locale.call_phrase("report", &[Value::from(1)]).unwrap();
    assert_eq!(one.to_string(), "this turn");

    let three = locale.call_phrase("report", &[Value::from(3)]).unwrap();
    assert_eq!(three.to_string(), "this turn many times");
}

// =============================================================================
// Literal Arguments in Phrase Calls
// =============================================================================

#[test]
fn eval_phrase_call_with_number_literal() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        cards($n) = "{$n} cards";
        pair = "You have {cards(2)}.";
    "#,
        )
        .unwrap();
    let result = registry.get_phrase("en", "pair").unwrap();
    assert_eq!(result.to_string(), "You have 2 cards.");
}

#[test]
fn eval_phrase_call_with_number_literal_match() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        label($n) = "count: {$n}";
        one_label = "{label(1)}";
    "#,
        )
        .unwrap();
    let result = registry.get_phrase("en", "one_label").unwrap();
    assert_eq!(result.to_string(), "count: 1");
}

#[test]
fn eval_phrase_call_with_string_literal() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        trigger($t) = "<b>{$t}</b>";
        example = "{trigger("Attack")}";
    "#,
        )
        .unwrap();
    let result = registry.get_phrase("en", "example").unwrap();
    assert_eq!(result.to_string(), "<b>Attack</b>");
}

#[test]
fn eval_phrase_call_with_string_literal_escape() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        wrap($s) = "[{$s}]";
        example = "{wrap("He said \"hi\"")}";
    "#,
        )
        .unwrap();
    let result = registry.get_phrase("en", "example").unwrap();
    assert_eq!(result.to_string(), "[He said \"hi\"]");
}

#[test]
fn eval_phrase_call_with_number_literal_zero() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        items($n) = "{$n} items";
        none = "{items(0)}";
    "#,
        )
        .unwrap();
    let result = registry.get_phrase("en", "none").unwrap();
    assert_eq!(result.to_string(), "0 items");
}

#[test]
fn eval_phrase_call_mixed_literal_and_param() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        format($label, $n) = "{$label}: {$n}";
        example($n) = "{format("Score", $n)}";
    "#,
        )
        .unwrap();
    let result = registry
        .call_phrase("en", "example", &[Value::from(42)])
        .unwrap();
    assert_eq!(result.to_string(), "Score: 42");
}

// =============================================================================
// Term/Phrase Usage Rule Enforcement
// =============================================================================

#[test]
fn error_arguments_to_term() {
    // {card($n)} where card is a term -> ArgumentsToTerm error
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        card = { one: "card", other: "cards" };
        bad($n) = "{card($n)}";
    "#,
        )
        .unwrap();

    let err = registry
        .call_phrase("en", "bad", &[Value::from(1)])
        .unwrap_err();
    assert!(
        matches!(err, EvalError::ArgumentsToTerm { ref name } if name == "card"),
        "expected ArgumentsToTerm, got: {err:?}"
    );
    let msg = err.to_string();
    assert!(msg.contains("term"), "error should mention 'term': {msg}");
    assert!(
        msg.contains("card:variant"),
        "error should suggest :variant syntax: {msg}"
    );
}

#[test]
fn error_selector_on_phrase_with_colon() {
    // {cards:other} where cards is a phrase -> SelectorOnPhrase error
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        cards($n) = "{$n} cards";
        bad = "{cards:other}";
    "#,
        )
        .unwrap();

    let err = registry.get_phrase("en", "bad").unwrap_err();
    assert!(
        matches!(err, EvalError::SelectorOnPhrase { ref name } if name == "cards"),
        "expected SelectorOnPhrase, got: {err:?}"
    );
    let msg = err.to_string();
    assert!(
        msg.contains("phrase"),
        "error should mention 'phrase': {msg}"
    );
    assert!(
        msg.contains("cards(...)"),
        "error should suggest () syntax: {msg}"
    );
}

#[test]
fn error_bare_phrase_reference() {
    // {cards} where cards is a phrase -> SelectorOnPhrase error
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        cards($n) = "{$n} cards";
        bad = "{cards}";
    "#,
        )
        .unwrap();

    let err = registry.get_phrase("en", "bad").unwrap_err();
    assert!(
        matches!(err, EvalError::SelectorOnPhrase { ref name } if name == "cards"),
        "expected SelectorOnPhrase, got: {err:?}"
    );
}

#[test]
fn ok_term_with_static_selection() {
    // {card:other} where card is a term -> OK
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        card = { one: "card", other: "cards" };
        all_cards = "All {card:other}.";
    "#,
        )
        .unwrap();

    let result = registry.get_phrase("en", "all_cards").unwrap();
    assert_eq!(result.to_string(), "All cards.");
}

#[test]
fn ok_phrase_call_with_args() {
    // {cards($n)} where cards is a phrase -> OK
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        cards($n) = "{$n} cards";
        draw($n) = "Draw {cards($n)}.";
    "#,
        )
        .unwrap();

    let result = registry
        .call_phrase("en", "draw", &[Value::from(3)])
        .unwrap();
    assert_eq!(result.to_string(), "Draw 3 cards.");
}

#[test]
fn ok_phrase_call_then_select() {
    // {cards($n):one} -> call phrase then select variant from result -> OK
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        ancient = :an { one: "Ancient", other: "Ancients" };
        subtype($s) = :from($s) "<b>{$s}</b>";
        get_plural($s) = "{subtype($s):other}";
    "#,
        )
        .unwrap();

    let ancient = registry.get_phrase("en", "ancient").unwrap();
    let result = registry
        .call_phrase("en", "get_plural", &[Value::Phrase(ancient)])
        .unwrap();
    assert_eq!(result.to_string(), "<b>Ancients</b>");
}

#[test]
fn error_arguments_to_term_via_call_phrase_api() {
    // Calling a term via call_phrase with args -> ArgumentsToTerm error
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(r#"card = { one: "card", other: "cards" };"#)
        .unwrap();

    let err = registry
        .call_phrase("en", "card", &[Value::from(1)])
        .unwrap_err();
    assert!(
        matches!(err, EvalError::ArgumentsToTerm { ref name } if name == "card"),
        "expected ArgumentsToTerm, got: {err:?}"
    );
}

#[test]
fn error_get_phrase_on_phrase_definition() {
    // Calling get_phrase on a phrase (with params) -> SelectorOnPhrase
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(r#"greet($name) = "Hello, {$name}!";"#)
        .unwrap();

    let err = registry.get_phrase("en", "greet").unwrap_err();
    assert!(
        matches!(err, EvalError::SelectorOnPhrase { ref name } if name == "greet"),
        "expected SelectorOnPhrase, got: {err:?}"
    );
}

#[test]
fn error_arguments_to_term_message_format() {
    let err = EvalError::ArgumentsToTerm {
        name: "card".to_string(),
    };
    let msg = err.to_string();
    assert!(msg.contains("'card' is a term"));
    assert!(msg.contains("cannot use () call syntax"));
    assert!(msg.contains("{card:variant}"));
    assert!(msg.contains("{card:$param}"));
}

#[test]
fn error_selector_on_phrase_message_format() {
    let err = EvalError::SelectorOnPhrase {
        name: "cards".to_string(),
    };
    let msg = err.to_string();
    assert!(msg.contains("'cards' is a phrase"));
    assert!(msg.contains("cannot use : without ()"));
    assert!(msg.contains("{cards(...)}"));
    assert!(msg.contains("{cards(...):variant}"));
}

// =============================================================================
// Default Variant Marker (*) Tests
// =============================================================================

#[test]
fn eval_bare_ref_uses_star_default() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        card = { *one: "card", other: "cards" };
        example = "{card}";
    "#,
        )
        .unwrap();
    let result = registry.get_phrase("en", "example").unwrap();
    assert_eq!(result.to_string(), "card");
}

#[test]
fn eval_bare_ref_star_on_second_variant() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        card = { one: "card", *other: "cards" };
        example = "{card}";
    "#,
        )
        .unwrap();
    let result = registry.get_phrase("en", "example").unwrap();
    assert_eq!(result.to_string(), "cards");
}

#[test]
fn eval_bare_ref_no_star_uses_first_variant() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        card = { one: "card", other: "cards" };
        example = "{card}";
    "#,
        )
        .unwrap();
    let result = registry.get_phrase("en", "example").unwrap();
    assert_eq!(result.to_string(), "card");
}

#[test]
fn eval_star_default_with_selectors_still_works() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        card = { *one: "card", other: "cards" };
        all_cards = "All {card:other}.";
    "#,
        )
        .unwrap();
    let result = registry.get_phrase("en", "all_cards").unwrap();
    assert_eq!(result.to_string(), "All cards.");
}

#[test]
fn eval_term_default_text_reflects_star() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        go = { present: "go", *past: "went", participle: "gone" };
    "#,
        )
        .unwrap();
    let result = registry.get_phrase("en", "go").unwrap();
    assert_eq!(result.to_string(), "went");
}
