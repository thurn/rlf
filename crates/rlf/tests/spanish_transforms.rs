//! QA tests for Spanish @el/@un transforms with gender agreement.
//!
//! Tests the definite article (@el/@la) and indefinite article (@un/@una)
//! transforms for Spanish, covering all gender x plural combinations,
//! alias resolution, variant selection from context, error handling,
//! and integration with other transforms.

use rlf::interpreter::{EvalError, Locale, TransformKind, TransformRegistry};
use rlf::{Phrase, PhraseRegistry, Tag, Value, VariantKey};
use std::collections::HashMap;

// =============================================================================
// @el (Definite Article) - Direct TransformKind::execute Tests
// =============================================================================

#[test]
fn el_masculine_singular() {
    let phrase = Phrase::builder()
        .text("enemigo".to_string())
        .tags(vec![Tag::new("masc")])
        .build();
    let value = Value::Phrase(phrase);
    let result = TransformKind::SpanishEl
        .execute(&value, None, "es")
        .unwrap();
    assert_eq!(result, "el enemigo");
}

#[test]
fn el_feminine_singular() {
    let phrase = Phrase::builder()
        .text("carta".to_string())
        .tags(vec![Tag::new("fem")])
        .build();
    let value = Value::Phrase(phrase);
    let result = TransformKind::SpanishEl
        .execute(&value, None, "es")
        .unwrap();
    assert_eq!(result, "la carta");
}

#[test]
fn el_masculine_plural() {
    let phrase = Phrase::builder()
        .text("enemigos".to_string())
        .tags(vec![Tag::new("masc")])
        .build();
    let value = Value::Phrase(phrase);
    let context = Value::String("other".to_string());
    let result = TransformKind::SpanishEl
        .execute(&value, Some(&context), "es")
        .unwrap();
    assert_eq!(result, "los enemigos");
}

#[test]
fn el_feminine_plural() {
    let phrase = Phrase::builder()
        .text("cartas".to_string())
        .tags(vec![Tag::new("fem")])
        .build();
    let value = Value::Phrase(phrase);
    let context = Value::String("other".to_string());
    let result = TransformKind::SpanishEl
        .execute(&value, Some(&context), "es")
        .unwrap();
    assert_eq!(result, "las cartas");
}

// =============================================================================
// @un (Indefinite Article) - Direct TransformKind::execute Tests
// =============================================================================

#[test]
fn un_masculine_singular() {
    let phrase = Phrase::builder()
        .text("enemigo".to_string())
        .tags(vec![Tag::new("masc")])
        .build();
    let value = Value::Phrase(phrase);
    let result = TransformKind::SpanishUn
        .execute(&value, None, "es")
        .unwrap();
    assert_eq!(result, "un enemigo");
}

#[test]
fn un_feminine_singular() {
    let phrase = Phrase::builder()
        .text("carta".to_string())
        .tags(vec![Tag::new("fem")])
        .build();
    let value = Value::Phrase(phrase);
    let result = TransformKind::SpanishUn
        .execute(&value, None, "es")
        .unwrap();
    assert_eq!(result, "una carta");
}

#[test]
fn un_masculine_plural() {
    let phrase = Phrase::builder()
        .text("enemigos".to_string())
        .tags(vec![Tag::new("masc")])
        .build();
    let value = Value::Phrase(phrase);
    let context = Value::String("other".to_string());
    let result = TransformKind::SpanishUn
        .execute(&value, Some(&context), "es")
        .unwrap();
    assert_eq!(result, "unos enemigos");
}

#[test]
fn un_feminine_plural() {
    let phrase = Phrase::builder()
        .text("cartas".to_string())
        .tags(vec![Tag::new("fem")])
        .build();
    let value = Value::Phrase(phrase);
    let context = Value::String("other".to_string());
    let result = TransformKind::SpanishUn
        .execute(&value, Some(&context), "es")
        .unwrap();
    assert_eq!(result, "unas cartas");
}

// =============================================================================
// Missing Gender Tag Errors
// =============================================================================

#[test]
fn el_missing_gender_tag() {
    let phrase = Phrase::builder().text("cosa".to_string()).build();
    let value = Value::Phrase(phrase);
    let result = TransformKind::SpanishEl.execute(&value, None, "es");
    assert!(matches!(result, Err(EvalError::MissingTag { .. })));
    if let Err(EvalError::MissingTag {
        transform,
        expected,
        ..
    }) = result
    {
        assert_eq!(transform, "el");
        assert!(expected.contains(&"masc".to_string()));
        assert!(expected.contains(&"fem".to_string()));
    }
}

#[test]
fn un_missing_gender_tag() {
    let phrase = Phrase::builder().text("cosa".to_string()).build();
    let value = Value::Phrase(phrase);
    let result = TransformKind::SpanishUn.execute(&value, None, "es");
    assert!(matches!(result, Err(EvalError::MissingTag { .. })));
    if let Err(EvalError::MissingTag {
        transform,
        expected,
        ..
    }) = result
    {
        assert_eq!(transform, "un");
        assert!(expected.contains(&"masc".to_string()));
        assert!(expected.contains(&"fem".to_string()));
    }
}

#[test]
fn el_missing_gender_on_string_value() {
    // String values have no tags, so gender lookup should fail
    let value = Value::from("texto");
    let result = TransformKind::SpanishEl.execute(&value, None, "es");
    assert!(matches!(result, Err(EvalError::MissingTag { .. })));
}

#[test]
fn un_missing_gender_on_string_value() {
    let value = Value::from("texto");
    let result = TransformKind::SpanishUn.execute(&value, None, "es");
    assert!(matches!(result, Err(EvalError::MissingTag { .. })));
}

#[test]
fn el_missing_gender_on_number_value() {
    let value = Value::from(42);
    let result = TransformKind::SpanishEl.execute(&value, None, "es");
    assert!(matches!(result, Err(EvalError::MissingTag { .. })));
}

// =============================================================================
// Transform Alias Resolution via TransformRegistry
// =============================================================================

#[test]
fn la_alias_resolves_to_el() {
    let registry = TransformRegistry::new();
    assert_eq!(registry.get("la", "es"), Some(TransformKind::SpanishEl));
}

#[test]
fn una_alias_resolves_to_un() {
    let registry = TransformRegistry::new();
    assert_eq!(registry.get("una", "es"), Some(TransformKind::SpanishUn));
}

#[test]
fn el_direct_lookup() {
    let registry = TransformRegistry::new();
    assert_eq!(registry.get("el", "es"), Some(TransformKind::SpanishEl));
}

#[test]
fn un_direct_lookup() {
    let registry = TransformRegistry::new();
    assert_eq!(registry.get("un", "es"), Some(TransformKind::SpanishUn));
}

#[test]
fn la_alias_not_resolved_for_other_languages() {
    // @la in Italian resolves to @il (Italian), not @el (Spanish)
    let registry = TransformRegistry::new();
    assert_eq!(registry.get("la", "it"), Some(TransformKind::ItalianIl));
}

#[test]
fn una_alias_not_resolved_for_other_languages() {
    // @una in Italian resolves to @un (Italian), not @un (Spanish)
    let registry = TransformRegistry::new();
    assert_eq!(registry.get("una", "it"), Some(TransformKind::ItalianUn));
}

// =============================================================================
// Numeric Context for Plural (1 = singular, other = plural)
// =============================================================================

#[test]
fn el_numeric_context_singular() {
    let phrase = Phrase::builder()
        .text("carta".to_string())
        .tags(vec![Tag::new("fem")])
        .build();
    let value = Value::Phrase(phrase);
    let context = Value::from(1);
    let result = TransformKind::SpanishEl
        .execute(&value, Some(&context), "es")
        .unwrap();
    assert_eq!(result, "la carta");
}

#[test]
fn el_numeric_context_plural() {
    let phrase = Phrase::builder()
        .text("cartas".to_string())
        .tags(vec![Tag::new("fem")])
        .build();
    let value = Value::Phrase(phrase);
    let context = Value::from(5);
    let result = TransformKind::SpanishEl
        .execute(&value, Some(&context), "es")
        .unwrap();
    assert_eq!(result, "las cartas");
}

#[test]
fn un_numeric_context_singular() {
    let phrase = Phrase::builder()
        .text("enemigo".to_string())
        .tags(vec![Tag::new("masc")])
        .build();
    let value = Value::Phrase(phrase);
    let context = Value::from(1);
    let result = TransformKind::SpanishUn
        .execute(&value, Some(&context), "es")
        .unwrap();
    assert_eq!(result, "un enemigo");
}

#[test]
fn un_numeric_context_plural() {
    let phrase = Phrase::builder()
        .text("enemigos".to_string())
        .tags(vec![Tag::new("masc")])
        .build();
    let value = Value::Phrase(phrase);
    let context = Value::from(3);
    let result = TransformKind::SpanishUn
        .execute(&value, Some(&context), "es")
        .unwrap();
    assert_eq!(result, "unos enemigos");
}

#[test]
fn el_numeric_context_zero_is_plural() {
    let phrase = Phrase::builder()
        .text("cartas".to_string())
        .tags(vec![Tag::new("fem")])
        .build();
    let value = Value::Phrase(phrase);
    let context = Value::from(0);
    let result = TransformKind::SpanishEl
        .execute(&value, Some(&context), "es")
        .unwrap();
    assert_eq!(result, "las cartas");
}

// =============================================================================
// Context "one" Should be Singular
// =============================================================================

#[test]
fn el_context_one_is_singular() {
    let phrase = Phrase::builder()
        .text("enemigo".to_string())
        .tags(vec![Tag::new("masc")])
        .build();
    let value = Value::Phrase(phrase);
    let context = Value::String("one".to_string());
    let result = TransformKind::SpanishEl
        .execute(&value, Some(&context), "es")
        .unwrap();
    assert_eq!(result, "el enemigo");
}

#[test]
fn un_context_one_is_singular() {
    let phrase = Phrase::builder()
        .text("carta".to_string())
        .tags(vec![Tag::new("fem")])
        .build();
    let value = Value::Phrase(phrase);
    let context = Value::String("one".to_string());
    let result = TransformKind::SpanishUn
        .execute(&value, Some(&context), "es")
        .unwrap();
    assert_eq!(result, "una carta");
}

// =============================================================================
// Variant Selection with Context
// =============================================================================

#[test]
fn el_other_selects_plural_variant() {
    // When a phrase has one/other variants, @el:other should select the "other" text
    let phrase = Phrase::builder()
        .text("carta".to_string())
        .tags(vec![Tag::new("fem")])
        .variants(HashMap::from([
            (VariantKey::new("one"), "carta".to_string()),
            (VariantKey::new("other"), "cartas".to_string()),
        ]))
        .build();
    let value = Value::Phrase(phrase);
    let context = Value::String("other".to_string());
    let result = TransformKind::SpanishEl
        .execute(&value, Some(&context), "es")
        .unwrap();
    assert_eq!(result, "las cartas");
}

#[test]
fn un_other_selects_plural_variant() {
    let phrase = Phrase::builder()
        .text("enemigo".to_string())
        .tags(vec![Tag::new("masc")])
        .variants(HashMap::from([
            (VariantKey::new("one"), "enemigo".to_string()),
            (VariantKey::new("other"), "enemigos".to_string()),
        ]))
        .build();
    let value = Value::Phrase(phrase);
    let context = Value::String("other".to_string());
    let result = TransformKind::SpanishUn
        .execute(&value, Some(&context), "es")
        .unwrap();
    assert_eq!(result, "unos enemigos");
}

#[test]
fn el_one_selects_singular_variant() {
    let phrase = Phrase::builder()
        .text("carta".to_string())
        .tags(vec![Tag::new("fem")])
        .variants(HashMap::from([
            (VariantKey::new("one"), "carta".to_string()),
            (VariantKey::new("other"), "cartas".to_string()),
        ]))
        .build();
    let value = Value::Phrase(phrase);
    let context = Value::String("one".to_string());
    let result = TransformKind::SpanishEl
        .execute(&value, Some(&context), "es")
        .unwrap();
    // "one" context: singular article + singular variant text
    assert_eq!(result, "la carta");
}

// =============================================================================
// Template Integration Tests - @el
// =============================================================================

#[test]
fn template_el_masculine_term() {
    let source = r#"
        enemigo = :masc "enemigo";
        the_enemy = "{@el enemigo}";
    "#;
    let mut locale = Locale::builder().language("es").build();
    locale.load_translations_str("es", source).unwrap();
    assert_eq!(
        locale.get_phrase("the_enemy").unwrap().to_string(),
        "el enemigo"
    );
}

#[test]
fn template_el_feminine_term() {
    let source = r#"
        carta = :fem "carta";
        the_card = "{@el carta}";
    "#;
    let mut locale = Locale::builder().language("es").build();
    locale.load_translations_str("es", source).unwrap();
    assert_eq!(
        locale.get_phrase("the_card").unwrap().to_string(),
        "la carta"
    );
}

#[test]
fn template_el_plural_masculine() {
    let source = r#"
        enemigo = :masc { one: "enemigo", other: "enemigos" };
        the_enemies($t) = "{@el:other $t}";
    "#;
    let mut locale = Locale::builder().language("es").build();
    locale.load_translations_str("es", source).unwrap();

    let enemy = locale.get_phrase("enemigo").unwrap();
    let result = locale
        .call_phrase("the_enemies", &[Value::Phrase(enemy)])
        .unwrap();
    assert_eq!(result.to_string(), "los enemigos");
}

#[test]
fn template_el_plural_feminine() {
    let source = r#"
        carta = :fem { one: "carta", other: "cartas" };
        the_cards($t) = "{@el:other $t}";
    "#;
    let mut locale = Locale::builder().language("es").build();
    locale.load_translations_str("es", source).unwrap();

    let card = locale.get_phrase("carta").unwrap();
    let result = locale
        .call_phrase("the_cards", &[Value::Phrase(card)])
        .unwrap();
    assert_eq!(result.to_string(), "las cartas");
}

// =============================================================================
// Template Integration Tests - @un
// =============================================================================

#[test]
fn template_un_masculine_term() {
    let source = r#"
        enemigo = :masc "enemigo";
        an_enemy = "{@un enemigo}";
    "#;
    let mut locale = Locale::builder().language("es").build();
    locale.load_translations_str("es", source).unwrap();
    assert_eq!(
        locale.get_phrase("an_enemy").unwrap().to_string(),
        "un enemigo"
    );
}

#[test]
fn template_un_feminine_term() {
    let source = r#"
        carta = :fem "carta";
        a_card = "{@un carta}";
    "#;
    let mut locale = Locale::builder().language("es").build();
    locale.load_translations_str("es", source).unwrap();
    assert_eq!(
        locale.get_phrase("a_card").unwrap().to_string(),
        "una carta"
    );
}

#[test]
fn template_un_plural_masculine() {
    let source = r#"
        enemigo = :masc { one: "enemigo", other: "enemigos" };
        some_enemies($t) = "{@un:other $t}";
    "#;
    let mut locale = Locale::builder().language("es").build();
    locale.load_translations_str("es", source).unwrap();

    let enemy = locale.get_phrase("enemigo").unwrap();
    let result = locale
        .call_phrase("some_enemies", &[Value::Phrase(enemy)])
        .unwrap();
    assert_eq!(result.to_string(), "unos enemigos");
}

#[test]
fn template_un_plural_feminine() {
    let source = r#"
        carta = :fem { one: "carta", other: "cartas" };
        some_cards($t) = "{@un:other $t}";
    "#;
    let mut locale = Locale::builder().language("es").build();
    locale.load_translations_str("es", source).unwrap();

    let card = locale.get_phrase("carta").unwrap();
    let result = locale
        .call_phrase("some_cards", &[Value::Phrase(card)])
        .unwrap();
    assert_eq!(result.to_string(), "unas cartas");
}

// =============================================================================
// Template Integration Tests - @la and @una Aliases
// =============================================================================

#[test]
fn template_la_alias_with_feminine() {
    let source = r#"
        carta = :fem "carta";
        the_card = "{@la carta}";
    "#;
    let mut locale = Locale::builder().language("es").build();
    locale.load_translations_str("es", source).unwrap();
    // @la resolves to @el, which reads :fem tag -> "la"
    assert_eq!(
        locale.get_phrase("the_card").unwrap().to_string(),
        "la carta"
    );
}

#[test]
fn template_la_alias_with_masculine() {
    let source = r#"
        enemigo = :masc "enemigo";
        the_enemy = "{@la enemigo}";
    "#;
    let mut locale = Locale::builder().language("es").build();
    locale.load_translations_str("es", source).unwrap();
    // @la resolves to @el, which reads :masc tag -> "el"
    assert_eq!(
        locale.get_phrase("the_enemy").unwrap().to_string(),
        "el enemigo"
    );
}

#[test]
fn template_una_alias_with_feminine() {
    let source = r#"
        carta = :fem "carta";
        a_card = "{@una carta}";
    "#;
    let mut locale = Locale::builder().language("es").build();
    locale.load_translations_str("es", source).unwrap();
    // @una resolves to @un, which reads :fem tag -> "una"
    assert_eq!(
        locale.get_phrase("a_card").unwrap().to_string(),
        "una carta"
    );
}

#[test]
fn template_una_alias_with_masculine() {
    let source = r#"
        enemigo = :masc "enemigo";
        an_enemy = "{@una enemigo}";
    "#;
    let mut locale = Locale::builder().language("es").build();
    locale.load_translations_str("es", source).unwrap();
    // @una resolves to @un, which reads :masc tag -> "un"
    assert_eq!(
        locale.get_phrase("an_enemy").unwrap().to_string(),
        "un enemigo"
    );
}

// =============================================================================
// Combination with @cap Transform
// =============================================================================

#[test]
fn el_with_cap_transform() {
    let source = r#"
        carta = :fem "carta";
        sentence = "{@cap @el carta} es importante.";
    "#;
    let mut locale = Locale::builder().language("es").build();
    locale.load_translations_str("es", source).unwrap();
    assert_eq!(
        locale.get_phrase("sentence").unwrap().to_string(),
        "La carta es importante."
    );
}

#[test]
fn un_with_cap_transform() {
    let source = r#"
        enemigo = :masc "enemigo";
        sentence = "{@cap @un enemigo} aparece.";
    "#;
    let mut locale = Locale::builder().language("es").build();
    locale.load_translations_str("es", source).unwrap();
    assert_eq!(
        locale.get_phrase("sentence").unwrap().to_string(),
        "Un enemigo aparece."
    );
}

#[test]
fn el_with_upper_transform() {
    let source = r#"
        carta = :fem "carta";
        shout = "{@upper @el carta}";
    "#;
    let mut locale = Locale::builder().language("es").build();
    locale.load_translations_str("es", source).unwrap();
    assert_eq!(locale.get_phrase("shout").unwrap().to_string(), "LA CARTA");
}

// =============================================================================
// Auto-Capitalization with Spanish Transforms
// =============================================================================

#[test]
fn auto_cap_with_el() {
    // Auto-capitalization via uppercase first letter in template
    let source = r#"
        carta = :fem "carta";
        the_card = "{@el Carta}";
    "#;
    let mut locale = Locale::builder().language("es").build();
    locale.load_translations_str("es", source).unwrap();
    // Auto-cap adds @cap before @el, producing "La carta"
    assert_eq!(
        locale.get_phrase("the_card").unwrap().to_string(),
        "La carta"
    );
}

// =============================================================================
// Multiple Transforms in One Template
// =============================================================================

#[test]
fn el_and_un_in_same_template() {
    let source = r#"
        carta = :fem "carta";
        enemigo = :masc "enemigo";
        test = "Roba {@el carta}, mata {@un enemigo}.";
    "#;
    let mut locale = Locale::builder().language("es").build();
    locale.load_translations_str("es", source).unwrap();
    assert_eq!(
        locale.get_phrase("test").unwrap().to_string(),
        "Roba la carta, mata un enemigo."
    );
}

#[test]
fn mixed_genders_in_one_template() {
    let source = r#"
        carta = :fem "carta";
        enemigo = :masc "enemigo";
        test = "{@el carta} y {@el enemigo}";
    "#;
    let mut locale = Locale::builder().language("es").build();
    locale.load_translations_str("es", source).unwrap();
    assert_eq!(
        locale.get_phrase("test").unwrap().to_string(),
        "la carta y el enemigo"
    );
}

#[test]
fn singular_and_plural_in_one_template() {
    let source = r#"
        carta = :fem { one: "carta", other: "cartas" };
        result_singular = "{@el carta}";
        result_plural($t) = "{@el:other $t}";
    "#;
    let mut locale = Locale::builder().language("es").build();
    locale.load_translations_str("es", source).unwrap();

    assert_eq!(
        locale.get_phrase("result_singular").unwrap().to_string(),
        "la carta"
    );

    let card = locale.get_phrase("carta").unwrap();
    let result = locale
        .call_phrase("result_plural", &[Value::Phrase(card)])
        .unwrap();
    assert_eq!(result.to_string(), "las cartas");
}

// =============================================================================
// PhraseRegistry API Tests (call_phrase)
// =============================================================================

#[test]
fn registry_call_phrase_el() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        carta = :fem "carta";
        show_card($t) = "Muestra {@el $t}.";
    "#,
        )
        .unwrap();

    let card = registry.get_phrase("es", "carta").unwrap();
    let result = registry
        .call_phrase("es", "show_card", &[Value::Phrase(card)])
        .unwrap();
    assert_eq!(result.to_string(), "Muestra la carta.");
}

#[test]
fn registry_call_phrase_un() {
    let mut registry = PhraseRegistry::new();
    registry
        .load_phrases(
            r#"
        enemigo = :masc "enemigo";
        find_enemy($t) = "Encuentra {@un $t}.";
    "#,
        )
        .unwrap();

    let enemy = registry.get_phrase("es", "enemigo").unwrap();
    let result = registry
        .call_phrase("es", "find_enemy", &[Value::Phrase(enemy)])
        .unwrap();
    assert_eq!(result.to_string(), "Encuentra un enemigo.");
}

// =============================================================================
// Template Integration - Error in Template
// =============================================================================

#[test]
fn template_el_missing_gender_error() {
    let source = r#"
        cosa = "cosa";
        the_thing = "{@el cosa}";
    "#;
    let mut locale = Locale::builder().language("es").build();
    locale.load_translations_str("es", source).unwrap();
    let result = locale.get_phrase("the_thing");
    assert!(result.is_err());
}

#[test]
fn template_un_missing_gender_error() {
    let source = r#"
        cosa = "cosa";
        a_thing = "{@un cosa}";
    "#;
    let mut locale = Locale::builder().language("es").build();
    locale.load_translations_str("es", source).unwrap();
    let result = locale.get_phrase("a_thing");
    assert!(result.is_err());
}

// =============================================================================
// Phrase with Multiple Tags (masc + other tags)
// =============================================================================

#[test]
fn el_with_additional_tags() {
    // Phrase has :masc plus unrelated tags; transform should still work
    let phrase = Phrase::builder()
        .text("guerrero".to_string())
        .tags(vec![Tag::new("masc"), Tag::new("animate")])
        .build();
    let value = Value::Phrase(phrase);
    let result = TransformKind::SpanishEl
        .execute(&value, None, "es")
        .unwrap();
    assert_eq!(result, "el guerrero");
}

#[test]
fn un_with_additional_tags() {
    let phrase = Phrase::builder()
        .text("guerrera".to_string())
        .tags(vec![Tag::new("fem"), Tag::new("animate")])
        .build();
    let value = Value::Phrase(phrase);
    let result = TransformKind::SpanishUn
        .execute(&value, None, "es")
        .unwrap();
    assert_eq!(result, "una guerrera");
}

// =============================================================================
// Realistic Game Localization Scenarios
// =============================================================================

#[test]
fn draw_card_scenario() {
    let source = r#"
        carta = :fem { one: "carta", other: "cartas" };
        draw_one = "Roba {@un carta}.";
        draw_many($t) = "Roba {@el:other $t}.";
    "#;
    let mut locale = Locale::builder().language("es").build();
    locale.load_translations_str("es", source).unwrap();

    assert_eq!(
        locale.get_phrase("draw_one").unwrap().to_string(),
        "Roba una carta."
    );

    let card = locale.get_phrase("carta").unwrap();
    let result = locale
        .call_phrase("draw_many", &[Value::Phrase(card)])
        .unwrap();
    assert_eq!(result.to_string(), "Roba las cartas.");
}

#[test]
fn destroy_scenario() {
    let source = r#"
        enemigo = :masc { one: "enemigo", other: "enemigos" };
        destruye_uno = "Destruye {@el enemigo}.";
        destruye_todos($t) = "Destruye {@el:other $t}.";
    "#;
    let mut locale = Locale::builder().language("es").build();
    locale.load_translations_str("es", source).unwrap();

    assert_eq!(
        locale.get_phrase("destruye_uno").unwrap().to_string(),
        "Destruye el enemigo."
    );

    let enemy = locale.get_phrase("enemigo").unwrap();
    let result = locale
        .call_phrase("destruye_todos", &[Value::Phrase(enemy)])
        .unwrap();
    assert_eq!(result.to_string(), "Destruye los enemigos.");
}

#[test]
fn complex_sentence_with_both_articles() {
    let source = r#"
        espada = :fem "espada";
        dragon = :masc "dragón";
        sentence = "Usa {@la espada} contra {@un dragon}.";
    "#;
    let mut locale = Locale::builder().language("es").build();
    locale.load_translations_str("es", source).unwrap();
    assert_eq!(
        locale.get_phrase("sentence").unwrap().to_string(),
        "Usa la espada contra un dragón."
    );
}

#[test]
fn sentence_start_capitalization() {
    let source = r#"
        tesoro = :masc "tesoro";
        sentence = "{@cap @el tesoro} brilla.";
    "#;
    let mut locale = Locale::builder().language("es").build();
    locale.load_translations_str("es", source).unwrap();
    assert_eq!(
        locale.get_phrase("sentence").unwrap().to_string(),
        "El tesoro brilla."
    );
}

// =============================================================================
// All 8 Article Forms - Exhaustive Coverage
// =============================================================================

#[test]
fn all_definite_forms() {
    // Test all 4 definite article forms: el, la, los, las
    let source = r#"
        masc_term = :masc { one: "libro", other: "libros" };
        fem_term = :fem { one: "mesa", other: "mesas" };
        el($t) = "{@el $t}";
        los($t) = "{@el:other $t}";
    "#;
    let mut locale = Locale::builder().language("es").build();
    locale.load_translations_str("es", source).unwrap();

    let masc = locale.get_phrase("masc_term").unwrap();
    let fem = locale.get_phrase("fem_term").unwrap();

    // el (masc singular)
    let result = locale
        .call_phrase("el", &[Value::Phrase(masc.clone())])
        .unwrap();
    assert_eq!(result.to_string(), "el libro");

    // la (fem singular)
    let result = locale
        .call_phrase("el", &[Value::Phrase(fem.clone())])
        .unwrap();
    assert_eq!(result.to_string(), "la mesa");

    // los (masc plural)
    let result = locale.call_phrase("los", &[Value::Phrase(masc)]).unwrap();
    assert_eq!(result.to_string(), "los libros");

    // las (fem plural)
    let result = locale.call_phrase("los", &[Value::Phrase(fem)]).unwrap();
    assert_eq!(result.to_string(), "las mesas");
}

#[test]
fn all_indefinite_forms() {
    // Test all 4 indefinite article forms: un, una, unos, unas
    let source = r#"
        masc_term = :masc { one: "gato", other: "gatos" };
        fem_term = :fem { one: "casa", other: "casas" };
        un_phrase($t) = "{@un $t}";
        unos_phrase($t) = "{@un:other $t}";
    "#;
    let mut locale = Locale::builder().language("es").build();
    locale.load_translations_str("es", source).unwrap();

    let masc = locale.get_phrase("masc_term").unwrap();
    let fem = locale.get_phrase("fem_term").unwrap();

    // un (masc singular)
    let result = locale
        .call_phrase("un_phrase", &[Value::Phrase(masc.clone())])
        .unwrap();
    assert_eq!(result.to_string(), "un gato");

    // una (fem singular)
    let result = locale
        .call_phrase("un_phrase", &[Value::Phrase(fem.clone())])
        .unwrap();
    assert_eq!(result.to_string(), "una casa");

    // unos (masc plural)
    let result = locale
        .call_phrase("unos_phrase", &[Value::Phrase(masc)])
        .unwrap();
    assert_eq!(result.to_string(), "unos gatos");

    // unas (fem plural)
    let result = locale
        .call_phrase("unos_phrase", &[Value::Phrase(fem)])
        .unwrap();
    assert_eq!(result.to_string(), "unas casas");
}

// =============================================================================
// eval_str Integration
// =============================================================================

#[test]
fn eval_str_with_el() {
    let mut locale = Locale::builder().language("es").build();
    locale
        .load_translations_str(
            "es",
            r#"
        carta = :fem "carta";
    "#,
        )
        .unwrap();
    let result = locale.eval_str("{@el carta}", HashMap::new()).unwrap();
    assert_eq!(result.to_string(), "la carta");
}

#[test]
fn eval_str_with_un() {
    let mut locale = Locale::builder().language("es").build();
    locale
        .load_translations_str(
            "es",
            r#"
        enemigo = :masc "enemigo";
    "#,
        )
        .unwrap();
    let result = locale.eval_str("{@un enemigo}", HashMap::new()).unwrap();
    assert_eq!(result.to_string(), "un enemigo");
}
