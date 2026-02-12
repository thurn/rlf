//! Static lint rules for RLF phrase definitions.
//!
//! Analyzes parsed AST to detect verbose patterns, missing `:from` annotations,
//! and other issues that may cause silent metadata loss or unnecessary verbosity.

use crate::interpreter::error::LoadWarning;
use crate::parser::ast::PhraseDefinition;

/// Runs static lint rules over parsed phrase definitions, returning warnings.
///
/// Operates purely on the parsed AST without evaluating phrases. Pass the
/// language code to include in warning messages.
pub fn lint_definitions(defs: &[PhraseDefinition], language: &str) -> Vec<LoadWarning> {
    let mut warnings = Vec::new();
    for def in defs {
        lint_redundant_passthrough_block(def, language, &mut warnings);
        lint_redundant_from_selector(def, language, &mut warnings);
        lint_likely_missing_from(def, language, &mut warnings);
        lint_verbose_transparent_wrapper(def, language, &mut warnings);
    }
    warnings
}

fn lint_redundant_passthrough_block(
    _def: &PhraseDefinition,
    _language: &str,
    _warnings: &mut Vec<LoadWarning>,
) {
    // Lint 1 implementation will be added in a follow-up task
}

fn lint_redundant_from_selector(
    _def: &PhraseDefinition,
    _language: &str,
    _warnings: &mut Vec<LoadWarning>,
) {
    // Lint 2 implementation will be added in a follow-up task
}

fn lint_likely_missing_from(
    _def: &PhraseDefinition,
    _language: &str,
    _warnings: &mut Vec<LoadWarning>,
) {
    // Lint 3 implementation will be added in a follow-up task
}

fn lint_verbose_transparent_wrapper(
    _def: &PhraseDefinition,
    _language: &str,
    _warnings: &mut Vec<LoadWarning>,
) {
    // Lint 4 implementation will be added in a follow-up task
}
