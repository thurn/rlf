//! Tests for runtime evaluation warnings (Lints 5 and 6).

use rlf::{EvalWarning, Locale, Phrase, Value, VariantKey};
use std::collections::HashMap;

// =========================================================================
// Lint 5: Missing Selector on Multi-Dimensional Phrase
// =========================================================================

#[test]
fn lint5_warns_bare_ref_to_multi_dimensional_phrase() {
    let mut locale = Locale::new();
    locale
        .load_translations_str(
            "en",
            r#"
        wrapper($p) = :from($p) "hello {$p}";
    "#,
        )
        .unwrap();

    let noun = Phrase::builder()
        .text("enemy".to_string())
        .variants(HashMap::from([
            (VariantKey::new("nom"), "enemy".to_string()),
            (VariantKey::new("nom.one"), "enemy".to_string()),
            (VariantKey::new("acc.one"), "foe".to_string()),
        ]))
        .build();

    let (result, warnings) = locale
        .call_phrase_with_warnings("wrapper", &[Value::Phrase(noun)])
        .unwrap();
    assert_eq!(result.to_string(), "hello enemy");

    // No warning here because $p is in :from context
    assert!(
        warnings.is_empty(),
        "Expected no warnings for :from context, got: {warnings:?}"
    );
}

#[test]
fn lint5_warns_bare_ref_outside_from_context() {
    let mut locale = Locale::new();
    locale
        .load_translations_str(
            "en",
            r#"
        show($p) = "You see {$p}.";
    "#,
        )
        .unwrap();

    let noun = Phrase::builder()
        .text("enemy".to_string())
        .variants(HashMap::from([
            (VariantKey::new("nom.one"), "enemy".to_string()),
            (VariantKey::new("acc.one"), "foe".to_string()),
        ]))
        .build();

    let (result, warnings) = locale
        .call_phrase_with_warnings("show", &[Value::Phrase(noun)])
        .unwrap();
    assert_eq!(result.to_string(), "You see enemy.");

    let multi_dim_warnings: Vec<_> = warnings
        .iter()
        .filter(|w| matches!(w, EvalWarning::MissingSelectorOnMultiDimensional { .. }))
        .collect();
    assert_eq!(multi_dim_warnings.len(), 1);
    assert!(matches!(
        &multi_dim_warnings[0],
        EvalWarning::MissingSelectorOnMultiDimensional { parameter, .. }
        if parameter == "p"
    ));
}

#[test]
fn lint5_no_warn_with_explicit_selector() {
    let mut locale = Locale::new();
    locale
        .load_translations_str(
            "en",
            r#"
        show($p) = "You see {$p:acc}.";
    "#,
        )
        .unwrap();

    let noun = Phrase::builder()
        .text("enemy".to_string())
        .variants(HashMap::from([
            (VariantKey::new("nom.one"), "enemy".to_string()),
            (VariantKey::new("acc"), "foe".to_string()),
        ]))
        .build();

    let (result, warnings) = locale
        .call_phrase_with_warnings("show", &[Value::Phrase(noun)])
        .unwrap();
    assert_eq!(result.to_string(), "You see foe.");
    assert!(
        warnings.is_empty(),
        "Expected no warnings with explicit selector, got: {warnings:?}"
    );
}

#[test]
fn lint5_no_warn_with_default_selector() {
    let mut locale = Locale::new();
    locale
        .load_translations_str(
            "en",
            r#"
        show($p) = "You see {$p:*}.";
    "#,
        )
        .unwrap();

    let noun = Phrase::builder()
        .text("enemy".to_string())
        .variants(HashMap::from([
            (VariantKey::new("nom.one"), "enemy".to_string()),
            (VariantKey::new("acc.one"), "foe".to_string()),
        ]))
        .build();

    let (result, warnings) = locale
        .call_phrase_with_warnings("show", &[Value::Phrase(noun)])
        .unwrap();
    assert_eq!(result.to_string(), "You see enemy.");
    assert!(
        warnings.is_empty(),
        "Expected no warnings with :* selector, got: {warnings:?}"
    );
}

#[test]
fn lint5_no_warn_for_simple_variants() {
    let mut locale = Locale::new();
    locale
        .load_translations_str(
            "en",
            r#"
        show($p) = "You see {$p}.";
    "#,
        )
        .unwrap();

    // Simple variants (no dots) — no warning expected
    let noun = Phrase::builder()
        .text("card".to_string())
        .variants(HashMap::from([
            (VariantKey::new("one"), "card".to_string()),
            (VariantKey::new("other"), "cards".to_string()),
        ]))
        .build();

    let (_, warnings) = locale
        .call_phrase_with_warnings("show", &[Value::Phrase(noun)])
        .unwrap();
    assert!(
        warnings.is_empty(),
        "Expected no warnings for simple variants, got: {warnings:?}"
    );
}

#[test]
fn lint5_no_warn_for_number_params() {
    let mut locale = Locale::new();
    locale
        .load_translations_str(
            "en",
            r#"
        count($n) = "Count: {$n}";
    "#,
        )
        .unwrap();

    let (_, warnings) = locale
        .call_phrase_with_warnings("count", &[Value::Number(5)])
        .unwrap();
    assert!(
        warnings.is_empty(),
        "Expected no warnings for number params, got: {warnings:?}"
    );
}

#[test]
fn lint5_no_warn_for_string_params() {
    let mut locale = Locale::new();
    locale
        .load_translations_str(
            "en",
            r#"
        greet($name) = "Hello, {$name}!";
    "#,
        )
        .unwrap();

    let (_, warnings) = locale
        .call_phrase_with_warnings("greet", &[Value::String("Alice".to_string())])
        .unwrap();
    assert!(
        warnings.is_empty(),
        "Expected no warnings for string params, got: {warnings:?}"
    );
}

#[test]
fn lint5_no_warn_in_from_with_match() {
    let mut locale = Locale::new();
    locale
        .load_translations_str(
            "en",
            r#"
        count_pred($n, $base) = :from($base) :match($n) {
            1: "{$base}",
            *other: "{$n} {$base}"
        };
    "#,
        )
        .unwrap();

    let noun = Phrase::builder()
        .text("enemy".to_string())
        .variants(HashMap::from([
            (VariantKey::new("nom.one"), "enemy".to_string()),
            (VariantKey::new("acc.one"), "foe".to_string()),
        ]))
        .build();

    let (_, warnings) = locale
        .call_phrase_with_warnings("count_pred", &[Value::Number(3), Value::Phrase(noun)])
        .unwrap();

    // $base is in :from context, so bare {$base} shouldn't trigger Lint 5
    let multi_dim_warnings: Vec<_> = warnings
        .iter()
        .filter(|w| matches!(w, EvalWarning::MissingSelectorOnMultiDimensional { .. }))
        .collect();
    assert_eq!(
        multi_dim_warnings.len(),
        0,
        "Expected no Lint 5 warnings in :from context, got: {multi_dim_warnings:?}"
    );
}

// =========================================================================
// Lint 6: Phrase Argument Without :from
// =========================================================================

#[test]
fn lint6_warns_phrase_arg_without_from() {
    let mut locale = Locale::new();
    locale
        .load_translations_str(
            "en",
            r#"
        inner($p) = "inner: {$p}";
        outer($x) = "outer: {inner($x)}";
    "#,
        )
        .unwrap();

    let noun = Phrase::builder()
        .text("card".to_string())
        .tags(vec![rlf::Tag::new("a")])
        .build();

    let (result, warnings) = locale
        .call_phrase_with_warnings("outer", &[Value::Phrase(noun)])
        .unwrap();
    assert_eq!(result.to_string(), "outer: inner: card");

    // inner() has no :from and receives a Phrase argument
    let from_warnings: Vec<_> = warnings
        .iter()
        .filter(|w| matches!(w, EvalWarning::PhraseArgumentWithoutFrom { .. }))
        .collect();
    assert_eq!(from_warnings.len(), 1);
    assert!(matches!(
        &from_warnings[0],
        EvalWarning::PhraseArgumentWithoutFrom { phrase, parameter }
        if phrase == "inner" && parameter == "p"
    ));
}

#[test]
fn lint6_no_warn_with_from() {
    let mut locale = Locale::new();
    locale
        .load_translations_str(
            "en",
            r#"
        inner($p) = :from($p) "inner: {$p}";
        outer($x) = "outer: {inner($x)}";
    "#,
        )
        .unwrap();

    let noun = Phrase::builder()
        .text("card".to_string())
        .tags(vec![rlf::Tag::new("a")])
        .build();

    let (_, warnings) = locale
        .call_phrase_with_warnings("outer", &[Value::Phrase(noun)])
        .unwrap();

    let from_warnings: Vec<_> = warnings
        .iter()
        .filter(|w| matches!(w, EvalWarning::PhraseArgumentWithoutFrom { .. }))
        .collect();
    assert_eq!(
        from_warnings.len(),
        0,
        "Expected no Lint 6 warnings when :from is present, got: {from_warnings:?}"
    );
}

#[test]
fn lint6_no_warn_for_number_arg() {
    let mut locale = Locale::new();
    locale
        .load_translations_str(
            "en",
            r#"
        count($n) = "Count: {$n}";
        wrapper($x) = "-> {count($x)}";
    "#,
        )
        .unwrap();

    let (_, warnings) = locale
        .call_phrase_with_warnings("wrapper", &[Value::Number(3)])
        .unwrap();
    assert!(
        warnings.is_empty(),
        "Expected no Lint 6 warnings for number arg, got: {warnings:?}"
    );
}

#[test]
fn lint6_no_warn_for_string_arg() {
    let mut locale = Locale::new();
    locale
        .load_translations_str(
            "en",
            r#"
        greet($name) = "Hello, {$name}!";
        wrapper($x) = "{greet($x)}";
    "#,
        )
        .unwrap();

    let (_, warnings) = locale
        .call_phrase_with_warnings("wrapper", &[Value::String("Bob".to_string())])
        .unwrap();
    assert!(
        warnings.is_empty(),
        "Expected no Lint 6 warnings for string arg, got: {warnings:?}"
    );
}

#[test]
fn lint6_warns_for_multiple_phrase_args() {
    let mut locale = Locale::new();
    locale
        .load_translations_str(
            "en",
            r#"
        combine($a, $b) = "{$a} and {$b}";
        wrapper($x, $y) = "{combine($x, $y)}";
    "#,
        )
        .unwrap();

    let noun1 = Phrase::builder().text("card".to_string()).build();
    let noun2 = Phrase::builder().text("token".to_string()).build();

    let (_, warnings) = locale
        .call_phrase_with_warnings("wrapper", &[Value::Phrase(noun1), Value::Phrase(noun2)])
        .unwrap();

    let from_warnings: Vec<_> = warnings
        .iter()
        .filter(|w| matches!(w, EvalWarning::PhraseArgumentWithoutFrom { .. }))
        .collect();
    // Both $a and $b trigger warnings
    assert_eq!(from_warnings.len(), 2);
}

#[test]
fn lint6_warns_only_for_phrase_args_not_number() {
    let mut locale = Locale::new();
    locale
        .load_translations_str(
            "en",
            r#"
        display($n, $p) = "{$n} {$p}";
        wrapper($count, $thing) = "{display($count, $thing)}";
    "#,
        )
        .unwrap();

    let noun = Phrase::builder().text("card".to_string()).build();

    let (_, warnings) = locale
        .call_phrase_with_warnings("wrapper", &[Value::Number(3), Value::Phrase(noun)])
        .unwrap();

    let from_warnings: Vec<_> = warnings
        .iter()
        .filter(|w| matches!(w, EvalWarning::PhraseArgumentWithoutFrom { .. }))
        .collect();
    // Only $p triggers the warning, not $n
    assert_eq!(from_warnings.len(), 1);
    assert!(matches!(
        &from_warnings[0],
        EvalWarning::PhraseArgumentWithoutFrom { phrase, parameter }
        if phrase == "display" && parameter == "p"
    ));
}

// =========================================================================
// EvalWarning Display Tests
// =========================================================================

#[test]
fn eval_warning_phrase_argument_without_from_display() {
    let warning = EvalWarning::PhraseArgumentWithoutFrom {
        phrase: "wrapper".to_string(),
        parameter: "p".to_string(),
    };
    assert_eq!(
        warning.to_string(),
        "warning: phrase 'wrapper' receives Phrase value for parameter '$p' but has no :from; tags and variants will be lost"
    );
}

#[test]
fn eval_warning_missing_selector_display() {
    let warning = EvalWarning::MissingSelectorOnMultiDimensional {
        parameter: "target".to_string(),
        available_keys: vec!["acc.one".to_string(), "nom.one".to_string()],
    };
    assert_eq!(
        warning.to_string(),
        "warning: bare reference '${target}' to Phrase with multi-dimensional variants; use an explicit selector or ':*' for the default; available keys: acc.one, nom.one"
    );
}

// =========================================================================
// Deduplication
// =========================================================================

#[test]
fn lint_warnings_are_deduplicated() {
    let mut locale = Locale::new();
    locale
        .load_translations_str(
            "en",
            r#"
        show($p) = "A: {$p}, B: {$p}.";
    "#,
        )
        .unwrap();

    let noun = Phrase::builder()
        .text("enemy".to_string())
        .variants(HashMap::from([
            (VariantKey::new("nom.one"), "enemy".to_string()),
            (VariantKey::new("acc.one"), "foe".to_string()),
        ]))
        .build();

    let (_, warnings) = locale
        .call_phrase_with_warnings("show", &[Value::Phrase(noun)])
        .unwrap();

    // Same parameter referenced twice, but warning should be deduplicated
    let multi_dim_warnings: Vec<_> = warnings
        .iter()
        .filter(|w| matches!(w, EvalWarning::MissingSelectorOnMultiDimensional { .. }))
        .collect();
    assert_eq!(
        multi_dim_warnings.len(),
        1,
        "Expected deduplicated warnings, got: {multi_dim_warnings:?}"
    );
}

// =========================================================================
// Integration: Both Lints Together
// =========================================================================

#[test]
fn both_lints_can_fire_together() {
    let mut locale = Locale::new();
    locale
        .load_translations_str(
            "en",
            r#"
        inner($p) = "see {$p}";
        outer($x) = "{inner($x)}";
    "#,
        )
        .unwrap();

    let noun = Phrase::builder()
        .text("enemy".to_string())
        .variants(HashMap::from([
            (VariantKey::new("nom.one"), "enemy".to_string()),
            (VariantKey::new("acc.one"), "foe".to_string()),
        ]))
        .tags(vec![rlf::Tag::new("masc")])
        .build();

    let (_, warnings) = locale
        .call_phrase_with_warnings("outer", &[Value::Phrase(noun)])
        .unwrap();

    let lint5: Vec<_> = warnings
        .iter()
        .filter(|w| matches!(w, EvalWarning::MissingSelectorOnMultiDimensional { .. }))
        .collect();
    let lint6: Vec<_> = warnings
        .iter()
        .filter(|w| matches!(w, EvalWarning::PhraseArgumentWithoutFrom { .. }))
        .collect();

    // Lint 6: inner() has no :from but receives a Phrase
    assert_eq!(lint6.len(), 1);
    // Lint 5: inner's body has bare {$p} with multi-dimensional Phrase
    assert_eq!(lint5.len(), 1);
}

// =========================================================================
// Edge Cases
// =========================================================================

#[test]
fn lint5_no_warn_for_phrase_without_variants() {
    let mut locale = Locale::new();
    locale
        .load_translations_str(
            "en",
            r#"
        show($p) = "You see {$p}.";
    "#,
        )
        .unwrap();

    // Phrase with no variants at all
    let noun = Phrase::builder()
        .text("enemy".to_string())
        .tags(vec![rlf::Tag::new("masc")])
        .build();

    let (_, warnings) = locale
        .call_phrase_with_warnings("show", &[Value::Phrase(noun)])
        .unwrap();

    let multi_dim_warnings: Vec<_> = warnings
        .iter()
        .filter(|w| matches!(w, EvalWarning::MissingSelectorOnMultiDimensional { .. }))
        .collect();
    assert_eq!(
        multi_dim_warnings.len(),
        0,
        "Expected no Lint 5 warnings for phrase without variants, got: {multi_dim_warnings:?}"
    );
}

#[test]
fn lint5_no_warn_in_from_with_variant_block() {
    let mut locale = Locale::new();
    locale
        .load_translations_str(
            "en",
            r#"
        wrapper($s) = :from($s) {
            nom: "good {$s}",
            *acc: "good {$s}"
        };
    "#,
        )
        .unwrap();

    let noun = Phrase::builder()
        .text("enemy".to_string())
        .variants(HashMap::from([
            (VariantKey::new("nom.one"), "enemy".to_string()),
            (VariantKey::new("acc.one"), "foe".to_string()),
        ]))
        .build();

    let (_, warnings) = locale
        .call_phrase_with_warnings("wrapper", &[Value::Phrase(noun)])
        .unwrap();

    let multi_dim_warnings: Vec<_> = warnings
        .iter()
        .filter(|w| matches!(w, EvalWarning::MissingSelectorOnMultiDimensional { .. }))
        .collect();
    assert_eq!(
        multi_dim_warnings.len(),
        0,
        "Expected no Lint 5 warnings in :from variant block, got: {multi_dim_warnings:?}"
    );
}

#[test]
fn lint6_warns_for_term_call_with_phrase_arg() {
    let mut locale = Locale::new();
    locale
        .load_translations_str(
            "en",
            r#"
        bold($t) = "<b>{$t}</b>";
        wrapper($x) = "{bold($x)}";
    "#,
        )
        .unwrap();

    let noun = Phrase::builder()
        .text("card".to_string())
        .variants(HashMap::from([
            (VariantKey::new("one"), "card".to_string()),
            (VariantKey::new("other"), "cards".to_string()),
        ]))
        .build();

    let (_, warnings) = locale
        .call_phrase_with_warnings("wrapper", &[Value::Phrase(noun)])
        .unwrap();

    let from_warnings: Vec<_> = warnings
        .iter()
        .filter(|w| matches!(w, EvalWarning::PhraseArgumentWithoutFrom { .. }))
        .collect();
    // bold() has no :from but gets a Phrase
    assert_eq!(from_warnings.len(), 1);
    assert!(matches!(
        &from_warnings[0],
        EvalWarning::PhraseArgumentWithoutFrom { phrase, parameter }
        if phrase == "bold" && parameter == "t"
    ));
}
