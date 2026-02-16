//! Static lint rules for RLF phrase definitions.
//!
//! Analyzes parsed AST to detect verbose patterns, missing `:from` annotations,
//! and other issues that may cause silent metadata loss or unnecessary verbosity.

use crate::interpreter::error::LoadWarning;
use crate::interpreter::locale::Locale;
use crate::parser::ast::{
    DefinitionKind, PhraseBody, PhraseDefinition, Reference, Segment, Selector, Template,
    VariantEntryBody,
};
use crate::types::Value;

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

/// Runs all static and runtime lint checks over registered phrases, prints any
/// warnings to stdout, and exits with status 0 (no warnings) or 1 (warnings
/// found).
///
/// This is a convenience entry point for CLI lint tools. After registering
/// your phrases, call this function to perform a complete lint pass:
///
/// ```ignore
/// fn main() {
///     my_strings::register_source_phrases();
///     rlf::with_locale(rlf::run_lints);
/// }
/// ```
pub fn run_lints(locale: &Locale) -> ! {
    let Some(registry) = locale.registry() else {
        eprintln!("No phrase registry found for current language");
        std::process::exit(1);
    };

    let mut all_warnings: Vec<String> = Vec::new();

    // Static lints: analyze phrase definition ASTs without evaluation.
    let definitions: Vec<_> = registry
        .phrase_names()
        .filter_map(|name| registry.get(name).cloned())
        .collect();
    for warning in &lint_definitions(&definitions, locale.language()) {
        all_warnings.push(format!("[static] {warning}"));
    }

    // Runtime lints: evaluate each phrase with representative arguments and
    // collect any EvalWarning values produced.
    for name in registry.phrase_names() {
        let Some(def) = registry.get(name) else {
            continue;
        };
        let args: Vec<Value> = def.parameters.iter().map(|_| Value::Number(1)).collect();
        if let Ok((_phrase, warnings)) = locale.call_phrase_with_warnings(name, &args) {
            for warning in &warnings {
                all_warnings.push(format!("[runtime] phrase '{name}': {warning}"));
            }
        }
    }

    if all_warnings.is_empty() {
        println!("RLF lint passed: no warnings found");
        std::process::exit(0);
    } else {
        println!("RLF lint found {} warning(s):\n", all_warnings.len());
        for warning in &all_warnings {
            println!("  {warning}");
        }
        std::process::exit(1);
    }
}

/// Detects `:from($p)` phrases with variant blocks where every entry just passes
/// its own key through to `$p` (e.g., `nom: "{$p:nom} extra"` for all entries).
///
/// These can be simplified to a single `:from` template.
fn lint_redundant_passthrough_block(
    def: &PhraseDefinition,
    language: &str,
    warnings: &mut Vec<LoadWarning>,
) {
    let Some(from_param) = &def.from_param else {
        return;
    };
    let PhraseBody::Variants(entries) = &def.body else {
        return;
    };
    if entries.is_empty() {
        return;
    }

    // Check that every entry's template has a reference to the :from parameter
    // with a single static selector matching the entry's key AND that the
    // surrounding text is identical across all entries.
    let mut reference_template: Option<Vec<NormalizedSegment>> = None;
    let mut all_passthrough = true;

    for entry in entries {
        let VariantEntryBody::Template(template) = &entry.body else {
            // Entry has a :match block, not a simple template -- not a passthrough
            all_passthrough = false;
            break;
        };

        // Each entry may have multiple keys (multi-key shorthand).
        // For this lint, check if the template passes through the :from param
        // with a selector matching ANY of the entry's keys.
        let normalized = normalize_template_for_passthrough(template, from_param, &entry.keys);
        let Some(normalized) = normalized else {
            all_passthrough = false;
            break;
        };

        // Check that the surrounding text pattern is the same across all entries
        match &reference_template {
            None => reference_template = Some(normalized),
            Some(reference) => {
                if *reference != normalized {
                    all_passthrough = false;
                    break;
                }
            }
        }
    }

    if all_passthrough {
        warnings.push(LoadWarning::RedundantPassthroughBlock {
            name: def.name.clone(),
            language: language.to_string(),
        });
    }
}

/// Detects redundant explicit selectors on `:from` parameters inside variant
/// blocks (e.g., `{$s:nom}` inside the `nom:` entry of a `:from($s)` block).
fn lint_redundant_from_selector(
    def: &PhraseDefinition,
    language: &str,
    warnings: &mut Vec<LoadWarning>,
) {
    let Some(from_param) = &def.from_param else {
        return;
    };
    let PhraseBody::Variants(entries) = &def.body else {
        return;
    };

    for entry in entries {
        let VariantEntryBody::Template(template) = &entry.body else {
            continue;
        };

        for segment in &template.segments {
            let Segment::Interpolation {
                reference,
                selectors,
                ..
            } = segment
            else {
                continue;
            };

            // Check if this interpolation references the :from parameter
            let Reference::Parameter(param_name) = reference else {
                continue;
            };
            if param_name != from_param {
                continue;
            }

            // Check if there's exactly one static selector that matches
            // one of the entry's keys
            if selectors.len() != 1 {
                continue;
            }
            let Selector::Identifier(selector_key) = &selectors[0] else {
                continue;
            };

            if entry.keys.contains(selector_key) {
                warnings.push(LoadWarning::RedundantFromSelector {
                    name: def.name.clone(),
                    language: language.to_string(),
                    variant_key: selector_key.clone(),
                    parameter: from_param.clone(),
                });
            }
        }
    }
}

/// Detects phrases without `:from` or tags where a parameter is used with an
/// explicit variant selector, which likely causes silent metadata loss.
///
/// Only fires when a parameter is used with a selector (e.g. `{$p:other}` or
/// `{subtype($t):other}`), indicating the caller expects variant metadata
/// that won't be propagated without `:from`. Bare parameter references like
/// `{$n}` or `{phrase($p)}` without selectors do not trigger this warning.
fn lint_likely_missing_from(
    def: &PhraseDefinition,
    language: &str,
    warnings: &mut Vec<LoadWarning>,
) {
    // Only check phrases (not terms)
    if def.kind != DefinitionKind::Phrase {
        return;
    }
    // Skip if :from is already present
    if def.from_param.is_some() {
        return;
    }
    // Skip if the phrase has its own explicit tags
    if !def.tags.is_empty() {
        return;
    }

    // Find the first parameter used with an explicit variant selector
    let param_with_selector = match &def.body {
        PhraseBody::Simple(template) => find_parameter_with_selector(template),
        PhraseBody::Match(branches) => branches
            .iter()
            .find_map(|b| find_parameter_with_selector(&b.template)),
        PhraseBody::Variants(entries) => entries.iter().find_map(|entry| match &entry.body {
            VariantEntryBody::Template(template) => find_parameter_with_selector(template),
            VariantEntryBody::Match { branches, .. } => branches
                .iter()
                .find_map(|b| find_parameter_with_selector(&b.template)),
        }),
    };

    if let Some(param_name) = param_with_selector {
        warnings.push(LoadWarning::LikelyMissingFrom {
            name: def.name.clone(),
            language: language.to_string(),
            parameter: param_name,
        });
    }
}

/// Detects `:from($p) "{$p}"` patterns that can be simplified to `:from($p);`.
fn lint_verbose_transparent_wrapper(
    def: &PhraseDefinition,
    language: &str,
    warnings: &mut Vec<LoadWarning>,
) {
    let Some(from_param) = &def.from_param else {
        return;
    };
    let PhraseBody::Simple(template) = &def.body else {
        return;
    };

    // Check if template has exactly one segment: an interpolation of the
    // :from parameter with no selectors and no transforms
    if template.segments.len() != 1 {
        return;
    }
    let Segment::Interpolation {
        transforms,
        reference,
        selectors,
    } = &template.segments[0]
    else {
        return;
    };
    if !transforms.is_empty() || !selectors.is_empty() {
        return;
    }
    let Reference::Parameter(param_name) = reference else {
        return;
    };
    if param_name == from_param {
        warnings.push(LoadWarning::VerboseTransparentWrapper {
            name: def.name.clone(),
            language: language.to_string(),
        });
    }
}

/// A normalized template segment for passthrough comparison.
///
/// Replaces the `:from` parameter interpolation with a placeholder so that
/// templates like `"{$p:nom} extra"` and `"{$p:acc} extra"` normalize to
/// the same pattern.
#[derive(Debug, PartialEq)]
enum NormalizedSegment {
    Literal(String),
    FromParamPlaceholder,
    OtherInterpolation,
}

/// Normalizes a template for passthrough detection.
///
/// Returns `Some(segments)` if the template contains exactly one interpolation
/// of the `:from` parameter with a single static selector matching one of the
/// entry's keys. Returns `None` otherwise.
fn normalize_template_for_passthrough(
    template: &Template,
    from_param: &str,
    entry_keys: &[String],
) -> Option<Vec<NormalizedSegment>> {
    let mut found_from_ref = false;
    let mut normalized = Vec::new();

    for segment in &template.segments {
        match segment {
            Segment::Literal(s) => {
                normalized.push(NormalizedSegment::Literal(s.clone()));
            }
            Segment::Interpolation {
                transforms,
                reference,
                selectors,
            } => {
                if let Reference::Parameter(param_name) = reference
                    && param_name == from_param
                    && transforms.is_empty()
                    && selectors.len() == 1
                    && matches!(&selectors[0], Selector::Identifier(k) if entry_keys.contains(k))
                {
                    if found_from_ref {
                        // Multiple references to :from param -- not a simple passthrough
                        return None;
                    }
                    found_from_ref = true;
                    normalized.push(NormalizedSegment::FromParamPlaceholder);
                    continue;
                }
                // Some other interpolation (other params, transforms, etc.)
                normalized.push(NormalizedSegment::OtherInterpolation);
            }
        }
    }

    if found_from_ref {
        Some(normalized)
    } else {
        None
    }
}

/// Finds the first parameter used with an explicit variant selector in a
/// template.
///
/// Returns the parameter name if an interpolation has non-empty selectors and
/// contains a parameter reference (directly or as a phrase call argument).
fn find_parameter_with_selector(template: &Template) -> Option<String> {
    for segment in &template.segments {
        let Segment::Interpolation {
            reference,
            selectors,
            ..
        } = segment
        else {
            continue;
        };
        if selectors.is_empty() {
            continue;
        }
        if let Some(name) = extract_first_parameter(reference) {
            return Some(name);
        }
    }
    None
}

/// Recursively extracts the first parameter name from a reference.
fn extract_first_parameter(reference: &Reference) -> Option<String> {
    match reference {
        Reference::Parameter(name) => Some(name.clone()),
        Reference::PhraseCall { args, .. } => args.iter().find_map(extract_first_parameter),
        Reference::Identifier(_) | Reference::NumberLiteral(_) | Reference::StringLiteral(_) => {
            None
        }
    }
}
