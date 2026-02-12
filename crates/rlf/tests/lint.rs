//! Tests for the static lint infrastructure.

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
fn lint_phrase_with_params_returns_no_warnings() {
    let defs = parse_file(r#"greet($name) = "Hello, {$name}!";"#).unwrap();
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
fn lint_multiple_definitions_returns_no_warnings() {
    let defs = parse_file(
        r#"
        hello = "Hello!";
        card = { one: "card", *other: "cards" };
        greet($name) = "Hello, {$name}!";
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
    let warning = rlf::LoadWarning::RedundantPassthroughBlock {
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
    let warning = rlf::LoadWarning::RedundantFromSelector {
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
    let warning = rlf::LoadWarning::LikelyMissingFrom {
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
    let warning = rlf::LoadWarning::VerboseTransparentWrapper {
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
        greet($name) = "Привет, {$name}!";
    "#,
    )
    .unwrap();
    let warnings = lint_definitions(&defs, "ru");
    assert!(warnings.is_empty());
}
