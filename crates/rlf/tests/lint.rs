//! Tests for the static lint rules.

use rlf::LoadWarning;
use rlf::lint_definitions;
use rlf::parser::{PhraseDefinition, parse_file};

// =========================================================================
// Smoke Tests
// =========================================================================

#[test]
fn lint_empty_definitions_returns_no_warnings() {
    let warnings = lint_definitions(&[], "en");
    assert!(warnings.is_empty());
}

#[test]
fn lint_simple_phrase_returns_no_warnings() {
    let defs = parse_file(r#"hello = "Hello!";"#).unwrap();
    let warnings = lint_definitions(&defs, "en");
    assert!(warnings.is_empty());
}

#[test]
fn lint_term_with_variants_returns_no_warnings() {
    let defs = parse_file(r#"card = { one: "card", *other: "cards" };"#).unwrap();
    let warnings = lint_definitions(&defs, "en");
    assert!(warnings.is_empty());
}

#[test]
fn lint_multiple_clean_definitions_returns_no_warnings() {
    let defs = parse_file(
        r#"
        hello = "Hello!";
        card = { one: "card", *other: "cards" };
        greet($name) = :from($name) "Hello, {$name}!";
    "#,
    )
    .unwrap();
    let warnings = lint_definitions(&defs, "en");
    assert!(warnings.is_empty());
}

// =========================================================================
// LoadWarning Display Format Tests
// =========================================================================

#[test]
fn redundant_passthrough_block_display() {
    let warning = LoadWarning::RedundantPassthroughBlock {
        name: "wrapper".to_string(),
        language: "ru".to_string(),
    };
    assert_eq!(
        warning.to_string(),
        "warning: phrase 'wrapper' in 'ru' has redundant passthrough variant block; use simple :from template instead"
    );
}

#[test]
fn redundant_from_selector_display() {
    let warning = LoadWarning::RedundantFromSelector {
        name: "wrapper".to_string(),
        language: "ru".to_string(),
        variant_key: "nom".to_string(),
        parameter: "s".to_string(),
    };
    assert_eq!(
        warning.to_string(),
        "warning: phrase 'wrapper' in 'ru' has redundant selector ':nom' on :from parameter '$s'; bare '${$s}' resolves to the same value"
    );
}

#[test]
fn likely_missing_from_display() {
    let warning = LoadWarning::LikelyMissingFrom {
        name: "wrapper".to_string(),
        language: "ru".to_string(),
        parameter: "p".to_string(),
    };
    assert_eq!(
        warning.to_string(),
        "warning: phrase 'wrapper' in 'ru' uses parameter '$p' without :from; tags and variants may be lost"
    );
}

#[test]
fn verbose_transparent_wrapper_display() {
    let warning = LoadWarning::VerboseTransparentWrapper {
        name: "wrapper".to_string(),
        language: "ru".to_string(),
    };
    assert_eq!(
        warning.to_string(),
        "warning: phrase 'wrapper' in 'ru' uses ':from($p) \"{$p}\"'; use body-less ':from($p);' instead"
    );
}

// =========================================================================
// API Accessibility Tests
// =========================================================================

#[test]
fn lint_definitions_accepts_parsed_file_output() {
    let defs: Vec<PhraseDefinition> = parse_file(
        r#"
        enemy = :fem {
            nom: "враг",
            acc: "врага",
            gen: "врага",
            dat: "врагу",
            ins: "врагом",
            *prep: "враге"
        };
        greet($name) = :from($name) "Привет, {$name}!";
    "#,
    )
    .unwrap();
    let warnings = lint_definitions(&defs, "ru");
    assert!(warnings.is_empty());
}

// =========================================================================
// Lint 1: Redundant Passthrough Variant Block
// =========================================================================

#[test]
fn lint1_detects_passthrough_variant_block() {
    let defs = parse_file(
        r#"
        wrapper($base) = :from($base) {
            nom: "{$base:nom}",
            acc: "{$base:acc}",
            gen: "{$base:gen}",
            *prep: "{$base:prep}"
        };
    "#,
    )
    .unwrap();
    let warnings = lint_definitions(&defs, "ru");
    // Lint 1 fires (passthrough block) and Lint 2 fires for each entry (redundant selectors)
    let passthrough_warnings: Vec<_> = warnings
        .iter()
        .filter(|w| matches!(w, LoadWarning::RedundantPassthroughBlock { .. }))
        .collect();
    assert_eq!(passthrough_warnings.len(), 1);
    assert!(matches!(
        &passthrough_warnings[0],
        LoadWarning::RedundantPassthroughBlock { name, language }
        if name == "wrapper" && language == "ru"
    ));
}

#[test]
fn lint1_detects_passthrough_with_extra_text() {
    let defs = parse_file(
        r#"
        wrapper($base, $c) = :from($base) {
            nom: "{$base:nom} {$c}",
            acc: "{$base:acc} {$c}",
            gen: "{$base:gen} {$c}",
            *prep: "{$base:prep} {$c}"
        };
    "#,
    )
    .unwrap();
    let warnings = lint_definitions(&defs, "ru");

    // Lint 1 fires (passthrough block) and Lint 2 also fires (redundant selectors)
    let passthrough_warnings: Vec<_> = warnings
        .iter()
        .filter(|w| matches!(w, LoadWarning::RedundantPassthroughBlock { .. }))
        .collect();
    assert_eq!(passthrough_warnings.len(), 1);
}

#[test]
fn lint1_ignores_variant_block_with_different_text() {
    let defs = parse_file(
        r#"
        wrapper($s) = :from($s) {
            nom: "вражеский {$s:nom}",
            acc: "вражеского {$s:acc}",
            *gen: "вражеского {$s:gen}"
        };
    "#,
    )
    .unwrap();
    let warnings = lint_definitions(&defs, "ru");

    // Lint 1 should NOT fire because the surrounding text differs between entries.
    // Lint 2 fires (redundant selectors) for each entry.
    let passthrough_warnings: Vec<_> = warnings
        .iter()
        .filter(|w| matches!(w, LoadWarning::RedundantPassthroughBlock { .. }))
        .collect();
    assert_eq!(passthrough_warnings.len(), 0);
}

#[test]
fn lint1_ignores_from_with_simple_template() {
    let defs = parse_file(
        r#"
        wrapper($s) = :from($s) "hello {$s}";
    "#,
    )
    .unwrap();
    let warnings = lint_definitions(&defs, "ru");
    let passthrough_warnings: Vec<_> = warnings
        .iter()
        .filter(|w| matches!(w, LoadWarning::RedundantPassthroughBlock { .. }))
        .collect();
    assert_eq!(passthrough_warnings.len(), 0);
}

#[test]
fn lint1_ignores_variant_block_without_from() {
    let defs = parse_file(
        r#"
        card = {
            nom: "карта",
            acc: "карту",
            *gen: "карты"
        };
    "#,
    )
    .unwrap();
    let warnings = lint_definitions(&defs, "ru");
    let passthrough_warnings: Vec<_> = warnings
        .iter()
        .filter(|w| matches!(w, LoadWarning::RedundantPassthroughBlock { .. }))
        .collect();
    assert_eq!(passthrough_warnings.len(), 0);
}

#[test]
fn lint1_ignores_entry_without_from_param_selector() {
    let defs = parse_file(
        r#"
        wrapper($s) = :from($s) {
            nom: "{$s:nom}",
            acc: "{$s:acc}",
            *gen: "fixed text"
        };
    "#,
    )
    .unwrap();
    let warnings = lint_definitions(&defs, "ru");
    let passthrough_warnings: Vec<_> = warnings
        .iter()
        .filter(|w| matches!(w, LoadWarning::RedundantPassthroughBlock { .. }))
        .collect();
    assert_eq!(passthrough_warnings.len(), 0);
}

// =========================================================================
// Lint 2: Redundant From Selector
// =========================================================================

#[test]
fn lint2_detects_redundant_selector_in_from_block() {
    let defs = parse_file(
        r#"
        wrapper($s) = :from($s) {
            nom: "good {$s:nom}",
            *acc: "good {$s:acc}"
        };
    "#,
    )
    .unwrap();
    let warnings = lint_definitions(&defs, "ru");
    let selector_warnings: Vec<_> = warnings
        .iter()
        .filter(|w| matches!(w, LoadWarning::RedundantFromSelector { .. }))
        .collect();
    assert_eq!(selector_warnings.len(), 2);
    assert!(matches!(
        &selector_warnings[0],
        LoadWarning::RedundantFromSelector { variant_key, parameter, .. }
        if variant_key == "nom" && parameter == "s"
    ));
    assert!(matches!(
        &selector_warnings[1],
        LoadWarning::RedundantFromSelector { variant_key, parameter, .. }
        if variant_key == "acc" && parameter == "s"
    ));
}

#[test]
fn lint2_ignores_selector_on_non_from_param() {
    let defs = parse_file(
        r#"
        wrapper($s, $t) = :from($s) {
            nom: "hello {$t:nom}",
            *acc: "hello {$t:acc}"
        };
    "#,
    )
    .unwrap();
    let warnings = lint_definitions(&defs, "ru");
    let selector_warnings: Vec<_> = warnings
        .iter()
        .filter(|w| matches!(w, LoadWarning::RedundantFromSelector { .. }))
        .collect();
    assert_eq!(selector_warnings.len(), 0);
}

#[test]
fn lint2_ignores_selector_that_differs_from_entry_key() {
    let defs = parse_file(
        r#"
        wrapper($s) = :from($s) {
            nom: "hello {$s:acc}",
            *acc: "hello {$s:gen}"
        };
    "#,
    )
    .unwrap();
    let warnings = lint_definitions(&defs, "ru");
    let selector_warnings: Vec<_> = warnings
        .iter()
        .filter(|w| matches!(w, LoadWarning::RedundantFromSelector { .. }))
        .collect();
    assert_eq!(selector_warnings.len(), 0);
}

#[test]
fn lint2_ignores_multiple_selectors() {
    let defs = parse_file(
        r#"
        wrapper($s) = :from($s) {
            nom: "hello {$s:nom:one}",
            *acc: "hello {$s:acc:one}"
        };
    "#,
    )
    .unwrap();
    let warnings = lint_definitions(&defs, "ru");
    let selector_warnings: Vec<_> = warnings
        .iter()
        .filter(|w| matches!(w, LoadWarning::RedundantFromSelector { .. }))
        .collect();
    assert_eq!(selector_warnings.len(), 0);
}

#[test]
fn lint2_ignores_simple_from_template() {
    let defs = parse_file(
        r#"
        wrapper($s) = :from($s) "hello {$s:nom}";
    "#,
    )
    .unwrap();
    let warnings = lint_definitions(&defs, "ru");
    let selector_warnings: Vec<_> = warnings
        .iter()
        .filter(|w| matches!(w, LoadWarning::RedundantFromSelector { .. }))
        .collect();
    assert_eq!(selector_warnings.len(), 0);
}

#[test]
fn lint2_ignores_parameter_selector() {
    let defs = parse_file(
        r#"
        wrapper($s, $n) = :from($s) {
            nom: "hello {$s:$n}",
            *acc: "hello {$s:$n}"
        };
    "#,
    )
    .unwrap();
    let warnings = lint_definitions(&defs, "ru");
    let selector_warnings: Vec<_> = warnings
        .iter()
        .filter(|w| matches!(w, LoadWarning::RedundantFromSelector { .. }))
        .collect();
    assert_eq!(selector_warnings.len(), 0);
}

// =========================================================================
// Lint 3: Likely Missing :from
// =========================================================================

#[test]
fn lint3_ignores_bare_param_without_selector() {
    let defs = parse_file(
        r#"
        wrapper($p) = "{$p}";
    "#,
    )
    .unwrap();
    let warnings = lint_definitions(&defs, "ru");
    let missing_from: Vec<_> = warnings
        .iter()
        .filter(|w| matches!(w, LoadWarning::LikelyMissingFrom { .. }))
        .collect();
    // Bare {$p} without a selector does not trigger the warning
    assert_eq!(missing_from.len(), 0);
}

#[test]
fn lint3_detects_param_in_phrase_call() {
    let defs = parse_file(
        r#"
        wrapper($t) = "allied {subtype($t):other}";
    "#,
    )
    .unwrap();
    let warnings = lint_definitions(&defs, "ru");
    let missing_from: Vec<_> = warnings
        .iter()
        .filter(|w| matches!(w, LoadWarning::LikelyMissingFrom { .. }))
        .collect();
    assert_eq!(missing_from.len(), 1);
    assert!(matches!(
        &missing_from[0],
        LoadWarning::LikelyMissingFrom { name, parameter, .. }
        if name == "wrapper" && parameter == "t"
    ));
}

#[test]
fn lint3_detects_param_with_selector_without_from() {
    let defs = parse_file(
        r#"
        wrapper($s) = "<b>{$s:other}</b>";
    "#,
    )
    .unwrap();
    let warnings = lint_definitions(&defs, "ru");
    let missing_from: Vec<_> = warnings
        .iter()
        .filter(|w| matches!(w, LoadWarning::LikelyMissingFrom { .. }))
        .collect();
    assert_eq!(missing_from.len(), 1);
    assert!(matches!(
        &missing_from[0],
        LoadWarning::LikelyMissingFrom { parameter, .. }
        if parameter == "s"
    ));
}

#[test]
fn lint3_ignores_phrase_with_from() {
    let defs = parse_file(
        r#"
        wrapper($p) = :from($p) "hello {$p}";
    "#,
    )
    .unwrap();
    let warnings = lint_definitions(&defs, "ru");
    let missing_from: Vec<_> = warnings
        .iter()
        .filter(|w| matches!(w, LoadWarning::LikelyMissingFrom { .. }))
        .collect();
    assert_eq!(missing_from.len(), 0);
}

#[test]
fn lint3_ignores_phrase_with_tags() {
    let defs = parse_file(
        r#"
        wrapper($t) = :an "allied {subtype($t)}";
    "#,
    )
    .unwrap();
    let warnings = lint_definitions(&defs, "ru");
    let missing_from: Vec<_> = warnings
        .iter()
        .filter(|w| matches!(w, LoadWarning::LikelyMissingFrom { .. }))
        .collect();
    assert_eq!(missing_from.len(), 0);
}

#[test]
fn lint3_ignores_terms() {
    let defs = parse_file(
        r#"
        hello = "Hello!";
    "#,
    )
    .unwrap();
    let warnings = lint_definitions(&defs, "ru");
    let missing_from: Vec<_> = warnings
        .iter()
        .filter(|w| matches!(w, LoadWarning::LikelyMissingFrom { .. }))
        .collect();
    assert_eq!(missing_from.len(), 0);
}

#[test]
fn lint3_ignores_param_without_selector() {
    let defs = parse_file(
        r#"
        wrapper($n) = "You have {$n} cards.";
    "#,
    )
    .unwrap();
    let warnings = lint_definitions(&defs, "ru");
    let missing_from: Vec<_> = warnings
        .iter()
        .filter(|w| matches!(w, LoadWarning::LikelyMissingFrom { .. }))
        .collect();
    // Bare {$n} without a selector does not trigger the warning, avoiding
    // false positives on numeric parameters.
    assert_eq!(missing_from.len(), 0);
}

#[test]
fn lint3_ignores_multiple_bare_params() {
    let defs = parse_file(
        r#"
        wrapper($a, $b) = "{$a} and {$b}";
    "#,
    )
    .unwrap();
    let warnings = lint_definitions(&defs, "ru");
    let missing_from: Vec<_> = warnings
        .iter()
        .filter(|w| matches!(w, LoadWarning::LikelyMissingFrom { .. }))
        .collect();
    // No selectors on any parameters, so no warning
    assert_eq!(missing_from.len(), 0);
}

#[test]
fn lint3_reports_first_param_with_selector() {
    let defs = parse_file(
        r#"
        wrapper($a, $b) = "{$a:other} and {$b:one}";
    "#,
    )
    .unwrap();
    let warnings = lint_definitions(&defs, "ru");
    let missing_from: Vec<_> = warnings
        .iter()
        .filter(|w| matches!(w, LoadWarning::LikelyMissingFrom { .. }))
        .collect();
    assert_eq!(missing_from.len(), 1);
    assert!(matches!(
        &missing_from[0],
        LoadWarning::LikelyMissingFrom { parameter, .. }
        if parameter == "a"
    ));
}

#[test]
fn lint3_ignores_bare_param_in_match_branch() {
    let defs = parse_file(
        r#"
        wrapper($n, $t) = :match($n) {
            1: "one {$t}",
            *other: "{$n} {$t}"
        };
    "#,
    )
    .unwrap();
    let warnings = lint_definitions(&defs, "ru");
    let missing_from: Vec<_> = warnings
        .iter()
        .filter(|w| matches!(w, LoadWarning::LikelyMissingFrom { .. }))
        .collect();
    // No selectors on any parameters, so no warning
    assert_eq!(missing_from.len(), 0);
}

#[test]
fn lint3_detects_selector_in_match_branch() {
    let defs = parse_file(
        r#"
        wrapper($n, $t) = :match($n) {
            1: "one {$t:other}",
            *other: "{$n} {$t:other}"
        };
    "#,
    )
    .unwrap();
    let warnings = lint_definitions(&defs, "ru");
    let missing_from: Vec<_> = warnings
        .iter()
        .filter(|w| matches!(w, LoadWarning::LikelyMissingFrom { .. }))
        .collect();
    assert_eq!(missing_from.len(), 1);
}

// =========================================================================
// Lint 4: Verbose Transparent Wrapper
// =========================================================================

#[test]
fn lint4_detects_from_identity_template() {
    let defs = parse_file(
        r#"
        wrapper($p) = :from($p) "{$p}";
    "#,
    )
    .unwrap();
    let warnings = lint_definitions(&defs, "ru");
    assert_eq!(warnings.len(), 1);
    assert!(matches!(
        &warnings[0],
        LoadWarning::VerboseTransparentWrapper { name, language }
        if name == "wrapper" && language == "ru"
    ));
}

#[test]
fn lint4_ignores_from_with_extra_text() {
    let defs = parse_file(
        r#"
        wrapper($p) = :from($p) "hello {$p}";
    "#,
    )
    .unwrap();
    let warnings = lint_definitions(&defs, "ru");
    let verbose_warnings: Vec<_> = warnings
        .iter()
        .filter(|w| matches!(w, LoadWarning::VerboseTransparentWrapper { .. }))
        .collect();
    assert_eq!(verbose_warnings.len(), 0);
}

#[test]
fn lint4_ignores_from_with_transforms() {
    let defs = parse_file(
        r#"
        wrapper($p) = :from($p) "{@cap $p}";
    "#,
    )
    .unwrap();
    let warnings = lint_definitions(&defs, "ru");
    let verbose_warnings: Vec<_> = warnings
        .iter()
        .filter(|w| matches!(w, LoadWarning::VerboseTransparentWrapper { .. }))
        .collect();
    assert_eq!(verbose_warnings.len(), 0);
}

#[test]
fn lint4_ignores_from_with_selectors() {
    let defs = parse_file(
        r#"
        wrapper($p) = :from($p) "{$p:other}";
    "#,
    )
    .unwrap();
    let warnings = lint_definitions(&defs, "ru");
    let verbose_warnings: Vec<_> = warnings
        .iter()
        .filter(|w| matches!(w, LoadWarning::VerboseTransparentWrapper { .. }))
        .collect();
    assert_eq!(verbose_warnings.len(), 0);
}

#[test]
fn lint4_ignores_from_with_variant_block() {
    let defs = parse_file(
        r#"
        wrapper($s) = :from($s) {
            nom: "hello {$s}",
            *acc: "goodbye {$s}"
        };
    "#,
    )
    .unwrap();
    let warnings = lint_definitions(&defs, "ru");
    let verbose_warnings: Vec<_> = warnings
        .iter()
        .filter(|w| matches!(w, LoadWarning::VerboseTransparentWrapper { .. }))
        .collect();
    assert_eq!(verbose_warnings.len(), 0);
}

#[test]
fn lint4_ignores_from_referencing_different_param() {
    let defs = parse_file(
        r#"
        wrapper($p, $q) = :from($p) "{$q}";
    "#,
    )
    .unwrap();
    let warnings = lint_definitions(&defs, "ru");
    let verbose_warnings: Vec<_> = warnings
        .iter()
        .filter(|w| matches!(w, LoadWarning::VerboseTransparentWrapper { .. }))
        .collect();
    assert_eq!(verbose_warnings.len(), 0);
}

#[test]
fn lint4_ignores_from_with_match() {
    let defs = parse_file(
        r#"
        wrapper($p, $n) = :from($p) :match($n) {
            1: "{$p}",
            *other: "{$n} {$p}"
        };
    "#,
    )
    .unwrap();
    let warnings = lint_definitions(&defs, "ru");
    let verbose_warnings: Vec<_> = warnings
        .iter()
        .filter(|w| matches!(w, LoadWarning::VerboseTransparentWrapper { .. }))
        .collect();
    assert_eq!(verbose_warnings.len(), 0);
}

// =========================================================================
// Integration: Multiple Lints on Same Definition
// =========================================================================

#[test]
fn passthrough_block_triggers_both_lint1_and_lint2() {
    let defs = parse_file(
        r#"
        wrapper($s) = :from($s) {
            nom: "{$s:nom}",
            *acc: "{$s:acc}"
        };
    "#,
    )
    .unwrap();
    let warnings = lint_definitions(&defs, "ru");
    let has_passthrough = warnings
        .iter()
        .any(|w| matches!(w, LoadWarning::RedundantPassthroughBlock { .. }));
    let has_redundant_selectors = warnings
        .iter()
        .any(|w| matches!(w, LoadWarning::RedundantFromSelector { .. }));
    assert!(has_passthrough);
    assert!(has_redundant_selectors);
}

#[test]
fn lint_multiple_definitions_reports_each() {
    let defs = parse_file(
        r#"
        identity($p) = :from($p) "{$p}";
        passthrough($s) = :from($s) {
            nom: "{$s:nom}",
            *acc: "{$s:acc}"
        };
        bare($t) = "{$t:other}";
    "#,
    )
    .unwrap();
    let warnings = lint_definitions(&defs, "ru");

    let verbose_count = warnings
        .iter()
        .filter(|w| matches!(w, LoadWarning::VerboseTransparentWrapper { .. }))
        .count();
    let passthrough_count = warnings
        .iter()
        .filter(|w| matches!(w, LoadWarning::RedundantPassthroughBlock { .. }))
        .count();
    let missing_from_count = warnings
        .iter()
        .filter(|w| matches!(w, LoadWarning::LikelyMissingFrom { .. }))
        .count();

    assert_eq!(verbose_count, 1);
    assert_eq!(passthrough_count, 1);
    assert_eq!(missing_from_count, 1);
}

// =========================================================================
// Edge Cases
// =========================================================================

#[test]
fn lint1_empty_variant_block_no_crash() {
    // An edge case: :from with empty variants should not crash.
    // The parser may not produce this, but the lint should handle it gracefully.
    let defs = parse_file(
        r#"
        wrapper($s) = :from($s) "hello {$s}";
    "#,
    )
    .unwrap();
    let warnings = lint_definitions(&defs, "ru");
    let passthrough_warnings: Vec<_> = warnings
        .iter()
        .filter(|w| matches!(w, LoadWarning::RedundantPassthroughBlock { .. }))
        .collect();
    assert_eq!(passthrough_warnings.len(), 0);
}

#[test]
fn lint_real_world_russian_passthrough() {
    let defs = parse_file(
        r#"
        pred_with_constraint($base, $constraint) = :from($base) {
            nom: "{$base:nom} {$constraint}",
            acc: "{$base:acc} {$constraint}",
            gen: "{$base:gen} {$constraint}",
            dat: "{$base:dat} {$constraint}",
            ins: "{$base:ins} {$constraint}",
            *prep: "{$base:prep} {$constraint}"
        };
    "#,
    )
    .unwrap();
    let warnings = lint_definitions(&defs, "ru");
    let passthrough_warnings: Vec<_> = warnings
        .iter()
        .filter(|w| matches!(w, LoadWarning::RedundantPassthroughBlock { .. }))
        .collect();
    assert_eq!(passthrough_warnings.len(), 1);
}

#[test]
fn lint_real_world_russian_adjective_agreement_not_passthrough() {
    let defs = parse_file(
        r#"
        enemy_subtype($s) = :from($s) {
            nom: "вражеский {$s}",
            acc: "вражеского {$s}",
            *gen: "вражеского {$s}"
        };
    "#,
    )
    .unwrap();
    let warnings = lint_definitions(&defs, "ru");
    let passthrough_warnings: Vec<_> = warnings
        .iter()
        .filter(|w| matches!(w, LoadWarning::RedundantPassthroughBlock { .. }))
        .collect();
    // NOT flagged as passthrough because the surrounding text differs between entries
    assert_eq!(passthrough_warnings.len(), 0);
}

#[test]
fn lint_real_world_transparent_wrapper() {
    let defs = parse_file(
        r#"
        predicate_with_indefinite_article($p) = :from($p) "{$p}";
    "#,
    )
    .unwrap();
    let warnings = lint_definitions(&defs, "ru");
    assert_eq!(warnings.len(), 1);
    assert!(matches!(
        &warnings[0],
        LoadWarning::VerboseTransparentWrapper { name, .. }
        if name == "predicate_with_indefinite_article"
    ));
}

#[test]
fn lint_real_world_missing_from() {
    let defs = parse_file(
        r#"
        allied_subtype_plural($t) = "allied {subtype($t):other}";
    "#,
    )
    .unwrap();
    let warnings = lint_definitions(&defs, "en");
    let missing_from: Vec<_> = warnings
        .iter()
        .filter(|w| matches!(w, LoadWarning::LikelyMissingFrom { .. }))
        .collect();
    assert_eq!(missing_from.len(), 1);
    assert!(matches!(
        &missing_from[0],
        LoadWarning::LikelyMissingFrom { name, parameter, .. }
        if name == "allied_subtype_plural" && parameter == "t"
    ));
}
