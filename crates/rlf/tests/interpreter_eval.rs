//! Integration tests for interpreter evaluation.

use rlf::interpreter::EvalError;
use rlf::{PhraseId, PhraseRegistry, Value};
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
        .load_phrases(r#"greet(name) = "Hello, {name}!";"#)
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
        .load_phrases(r#"count(n) = "Count: {n}";"#)
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
        draw(n) = "Draw {n} {card:n}.";
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
        draw(n) = "Возьмите {n} {card:n}.";
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
        draw(n) = "Возьмите {card:acc:n}.";
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
        destroy(thing) = "{thing} fue {destroyed:thing}.";
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
        draw(n) = "Draw {n} {card:n}.";
        draw_and_play(n) = "{draw(n)} Then play one.";
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
        .eval_str("Draw {n} {card:n}.", "en", params)
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
        .load_phrases(r#"greet(name) = "Hello, {name}!";"#)
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
        .load_phrases(r#"greet(name) = "Hello, {name}!";"#)
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
        .load_phrases(r#"greet(name) = "Hello, {name}!";"#)
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
        subtype(s) = :from(s) "<b>{s}</b>";
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
        braces = "Use {{name}} for interpolation.";
        at_sign = "Use @@ for transforms.";
        colon = "Ratio 1::2.";
    "#,
        )
        .unwrap();

    assert_eq!(
        registry.get_phrase("en", "braces").unwrap().to_string(),
        "Use {name} for interpolation."
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
        .load_phrases(r#"full_greeting(name, title) = "Hello, {title} {name}!";"#)
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
        inner(x) = "[{x}]";
        outer(y) = "({inner(y)})";
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
        one_param(x) = "{x}";
        two_params(a, b) = "{a} {b}";
    "#,
        )
        .unwrap();

    let id0 = PhraseId::from_name("no_params");
    let id1 = PhraseId::from_name("one_param");
    let id2 = PhraseId::from_name("two_params");
    let id_missing = PhraseId::from_name("missing");

    assert_eq!(registry.phrase_parameter_count(id0.as_u64()), 0);
    assert_eq!(registry.phrase_parameter_count(id1.as_u64()), 1);
    assert_eq!(registry.phrase_parameter_count(id2.as_u64()), 2);
    assert_eq!(registry.phrase_parameter_count(id_missing.as_u64()), 0);
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
        .eval_str("Draw {n} {card:n}.", "en", params)
        .unwrap();
    assert_eq!(registry.template_cache_len(), 1);

    // Second call with same template should reuse cache
    let params2: HashMap<String, Value> = [("n".to_string(), Value::from(1))].into_iter().collect();
    let result = registry
        .eval_str("Draw {n} {card:n}.", "en", params2)
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
    registry.eval_str("{n} {card:n}", "en", params1).unwrap();
    assert_eq!(registry.template_cache_len(), 1);

    let params2: HashMap<String, Value> = [("n".to_string(), Value::from(2))].into_iter().collect();
    registry
        .eval_str("You have {n} {card:n}.", "en", params2)
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
        .eval_str("Draw {n} {card:n}.", "en", params)
        .unwrap();
    assert_eq!(registry.template_cache_len(), 1);

    registry.clear_template_cache();
    assert_eq!(registry.template_cache_len(), 0);

    // Should still work after clearing cache
    let params2: HashMap<String, Value> = [("n".to_string(), Value::from(1))].into_iter().collect();
    let result = registry
        .eval_str("Draw {n} {card:n}.", "en", params2)
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

    let template = "You drew {n} {card:n}.";

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

        destroy(thing) = "{thing} - {destroyed:thing}";
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
        take(n) = "Weź {card:acc:n}.";
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
        draw(n) = "Dobierz {n} {card:n}.";
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
        defeat(n) = "Pokonaj {n} {enemy:acc:n}.";
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
