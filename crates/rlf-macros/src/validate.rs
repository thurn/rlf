//! Compile-time validation for the rlf! macro.
//!
//! Performs validation checks including:
//! 1. Undefined phrase references (MACRO-08)
//! 2. Undefined parameter references (MACRO-09)
//! 3. Invalid literal selectors (MACRO-10)
//! 4. Unknown transforms (MACRO-11)
//! 5. Transform tag requirements (MACRO-12) - infrastructure for future
//! 6. Tag-based selection compatibility (MACRO-13) - infrastructure for future
//! 7. Cyclic references (MACRO-14)
//! 8. Term/phrase usage errors: `()` on a term, `:` on a phrase
//! 9. Arity mismatches in phrase calls
//! 10. Nested phrase calls not supported as arguments
//! 11. `$name` referencing a term instead of a parameter
//! 12. Numeric keys in term variant blocks
//!
//! Also provides typo suggestions (MACRO-17) using Levenshtein distance.

use std::collections::{HashMap, HashSet};

use proc_macro2::Span;
use rlf_semantics::{accepted_transform_names, resolve_transform};
use strsim::levenshtein;

use crate::input::{
    DefinitionKind, Interpolation, MacroInput, PhraseBody, PhraseDefinition, Reference, Segment,
    Selector, Template, TransformContext, VariantEntryBody,
};

/// Validation context built from MacroInput.
pub struct ValidationContext {
    /// All defined phrase names.
    pub phrases: HashSet<String>,
    /// Phrase name -> defined variant keys (for literal selector validation).
    pub phrase_variants: HashMap<String, HashSet<String>>,
    /// Phrase name -> defined tags (infrastructure for future tag-based validation).
    #[cfg_attr(not(test), expect(dead_code))]
    pub phrase_tags: HashMap<String, HashSet<String>>,
    /// Phrase name -> DefinitionKind (Term or Phrase).
    pub phrase_kinds: HashMap<String, DefinitionKind>,
    /// Phrase name -> parameter count (0 for terms).
    pub phrase_param_counts: HashMap<String, usize>,
}

impl ValidationContext {
    /// Build validation context from macro input.
    pub fn from_input(input: &MacroInput) -> Self {
        let mut phrases = HashSet::new();
        let mut phrase_variants = HashMap::new();
        let mut phrase_tags = HashMap::new();
        let mut phrase_kinds = HashMap::new();
        let mut phrase_param_counts = HashMap::new();

        for phrase in &input.phrases {
            let name = phrase.name.name.clone();
            phrases.insert(name.clone());
            phrase_kinds.insert(name.clone(), phrase.kind);
            phrase_param_counts.insert(name.clone(), phrase.parameters.len());

            // Collect variant keys
            match &phrase.body {
                PhraseBody::Variants(variants) => {
                    let mut keys = HashSet::new();
                    for variant in variants {
                        for key in &variant.keys {
                            // Store exact variant keys; selector fallback behavior is checked
                            // later using runtime-equivalent prefix matching.
                            keys.insert(key.name.clone());
                        }
                    }
                    phrase_variants.insert(name.clone(), keys);
                }
                PhraseBody::Match(branches) => {
                    let mut keys = HashSet::new();
                    for branch in branches {
                        for key in &branch.keys {
                            keys.insert(key.value.name.clone());
                        }
                    }
                    phrase_variants.insert(name.clone(), keys);
                }
                PhraseBody::Simple(_) => {}
            }

            // Collect tags
            let tags: HashSet<String> = phrase.tags.iter().map(|t| t.name.clone()).collect();
            if !tags.is_empty() {
                phrase_tags.insert(name, tags);
            }
        }

        ValidationContext {
            phrases,
            phrase_variants,
            phrase_tags,
            phrase_kinds,
            phrase_param_counts,
        }
    }
}

/// Main validation entry point.
///
/// Performs all compile-time validation checks and returns a syn::Result.
/// On success, returns Ok(()). On failure, returns an error with span information
/// pointing to the problematic location in the source.
pub fn validate(input: &MacroInput) -> syn::Result<()> {
    let ctx = ValidationContext::from_input(input);

    // Check each phrase definition
    for phrase in &input.phrases {
        validate_phrase(phrase, &ctx)?;
    }

    // Check for cycles (separate pass after all phrases validated)
    detect_cycles(input, &ctx)?;

    Ok(())
}

/// Validate a single phrase definition.
fn validate_phrase(phrase: &PhraseDefinition, ctx: &ValidationContext) -> syn::Result<()> {
    let params: HashSet<String> = phrase.parameters.iter().map(|p| p.name.clone()).collect();

    // Check parameter shadowing (MACRO-15)
    for param in &phrase.parameters {
        if ctx.phrases.contains(&param.name) {
            return Err(syn::Error::new(
                param.span,
                format!(
                    "parameter '{}' shadows phrase '{}'\nhelp: use a different parameter name",
                    param.name, param.name
                ),
            ));
        }
    }

    // Validate: numeric keys in term variant blocks are not allowed
    if phrase.kind == DefinitionKind::Term
        && let PhraseBody::Variants(variants) = &phrase.body
    {
        for variant in variants {
            for key in &variant.keys {
                for component in key.name.split('.') {
                    if component.chars().next().is_some_and(|c| c.is_ascii_digit()) {
                        return Err(syn::Error::new(
                            key.span,
                            format!(
                                "term variant keys must be named identifiers — use ':match' for numeric branching (found '{}')",
                                key.name
                            ),
                        ));
                    }
                }
            }
        }
    }

    // Validate body references
    match &phrase.body {
        PhraseBody::Simple(template) => {
            validate_template(template, &params, ctx, &phrase.name.name)?;
        }
        PhraseBody::Variants(variants) => {
            for variant in variants {
                validate_variant_entry_body(&variant.body, &params, ctx, &phrase.name.name)?;
            }
        }
        PhraseBody::Match(branches) => {
            for branch in branches {
                validate_template(&branch.template, &params, ctx, &phrase.name.name)?;
            }
        }
    }

    Ok(())
}

/// Validate a template and all its interpolations.
fn validate_template(
    template: &Template,
    params: &HashSet<String>,
    ctx: &ValidationContext,
    _current_phrase: &str,
) -> syn::Result<()> {
    for segment in &template.segments {
        if let Segment::Interpolation(interp) = segment {
            validate_interpolation(interp, params, ctx)?;
        }
    }
    Ok(())
}

/// Validate a variant entry body (template or match block).
fn validate_variant_entry_body(
    body: &VariantEntryBody,
    params: &HashSet<String>,
    ctx: &ValidationContext,
    current_phrase: &str,
) -> syn::Result<()> {
    match body {
        VariantEntryBody::Template(template) => {
            validate_template(template, params, ctx, current_phrase)
        }
        VariantEntryBody::Match { branches, .. } => {
            for branch in branches {
                validate_template(&branch.template, params, ctx, current_phrase)?;
            }
            Ok(())
        }
    }
}

/// Validate an interpolation: reference, transforms, and selectors.
fn validate_interpolation(
    interp: &Interpolation,
    params: &HashSet<String>,
    ctx: &ValidationContext,
) -> syn::Result<()> {
    let source_transform_names = accepted_transform_names("en");

    // Validate transforms exist (MACRO-11)
    for transform in &interp.transforms {
        if resolve_transform(&transform.name.name, "en").is_none() {
            let suggestions = compute_suggestions_str(&transform.name.name, source_transform_names);
            let mut msg = format!("unknown transform '@{}'", transform.name.name);
            if !suggestions.is_empty() {
                msg.push_str(&format!("\nhelp: did you mean '@{}'?", suggestions[0]));
            } else {
                msg.push_str(&format!(
                    "\nnote: available transforms: {}",
                    source_transform_names.join(", ")
                ));
            }
            return Err(syn::Error::new(transform.name.span, msg));
        }

        // Transform tag validation (MACRO-12)
        // Note: Tag requirements are enforced at runtime by the transform itself.
        // At macro time we only validate that transform names are recognized.

        // Validate dynamic context parameter is declared
        let dynamic_param = match &transform.context {
            TransformContext::Dynamic(ident) => Some(ident),
            TransformContext::Both(_, ident) => Some(ident),
            TransformContext::None | TransformContext::Static(_) => None,
        };
        if let Some(ident) = dynamic_param
            && !params.contains(&ident.name)
        {
            return Err(syn::Error::new(
                ident.span,
                format!(
                    "undefined parameter '${0}' in transform context\nhelp: declare it as a parameter: name(${0})",
                    ident.name
                ),
            ));
        }
    }

    // Validate the reference (phrase or parameter)
    validate_reference(&interp.reference, params, ctx)?;

    // Check term/phrase usage: bare identifier with selectors on a phrase is an error
    if let Reference::Identifier(ident) = &interp.reference
        && let Some(&DefinitionKind::Phrase) = ctx.phrase_kinds.get(&ident.name)
    {
        if interp.selectors.is_empty() {
            return Err(syn::Error::new(
                ident.span,
                format!(
                    "'{}' is a phrase — cannot reference without (); use {{{}(...)}}",
                    ident.name, ident.name
                ),
            ));
        } else {
            return Err(syn::Error::new(
                ident.span,
                format!(
                    "'{}' is a phrase — use {}(...):{}",
                    ident.name,
                    ident.name,
                    interp
                        .selectors
                        .iter()
                        .map(|s| match s {
                            Selector::Literal(i) => i.name.clone(),
                            Selector::Parameter(i) => format!("${}", i.name),
                        })
                        .collect::<Vec<_>>()
                        .join(":")
                ),
            ));
        }
    }

    // Validate selectors (MACRO-10, MACRO-13)
    // Determine if reference is a literal phrase (not a parameter)
    let is_literal_phrase = match &interp.reference {
        Reference::Identifier(ident) => ctx.phrases.contains(&ident.name),
        Reference::Parameter(_) | Reference::NumberLiteral(..) | Reference::StringLiteral(..) => {
            false
        }
        Reference::Call { name, .. } => ctx.phrases.contains(&name.name),
    };

    if is_literal_phrase {
        let phrase_name = match &interp.reference {
            Reference::Identifier(ident) => &ident.name,
            Reference::Call { name, .. } => &name.name,
            Reference::Parameter(_)
            | Reference::NumberLiteral(..)
            | Reference::StringLiteral(..) => unreachable!(),
        };

        // Get phrase variants for literal selector validation
        let phrase_variants = ctx.phrase_variants.get(phrase_name);

        // Compile-time selector existence validation is only applied when all
        // selectors are literal (static). Parameterized selectors are validated
        // at runtime.
        let all_selectors_static = interp
            .selectors
            .iter()
            .all(|s| matches!(s, Selector::Literal(_)));

        // Static selector - must match a variant key under runtime-equivalent
        // fallback semantics: exact key first, then progressively shorter
        // dotted prefixes (e.g., nom.one -> nom).
        if all_selectors_static && let Some(variants) = phrase_variants {
            let selector_key = interp
                .selectors
                .iter()
                .filter_map(|selector| match selector {
                    Selector::Literal(ident) => Some(ident.name.clone()),
                    Selector::Parameter(_) => None,
                })
                .collect::<Vec<_>>()
                .join(".");

            if !selector_key.is_empty() && !variant_exists_with_fallback(variants, &selector_key) {
                let mut available: Vec<_> = variants.iter().cloned().collect();
                available.sort();
                return Err(syn::Error::new(
                    interp.span,
                    format!(
                        "phrase '{}' has no variant '{}'\nnote: available variants: {}",
                        phrase_name,
                        selector_key,
                        available.join(", ")
                    ),
                ));
            }
        }
    }

    // Validate parameter selectors reference declared parameters
    for selector in &interp.selectors {
        if let Selector::Parameter(ident) = selector
            && !params.contains(&ident.name)
        {
            return Err(syn::Error::new(
                ident.span,
                format!(
                    "undefined parameter '${0}' in selector\nhelp: declare it as a parameter: name(${0})",
                    ident.name
                ),
            ));
        }
    }

    Ok(())
}

/// Check if a static selector key can resolve against variants using runtime
/// fallback semantics (exact key, then progressively shorter dotted prefixes).
fn variant_exists_with_fallback(variants: &HashSet<String>, key: &str) -> bool {
    if variants.contains(key) {
        return true;
    }

    let mut current = key;
    while let Some(dot_pos) = current.rfind('.') {
        current = &current[..dot_pos];
        if variants.contains(current) {
            return true;
        }
    }
    false
}

/// Validate a reference (identifier, parameter, or call).
fn validate_reference(
    reference: &Reference,
    params: &HashSet<String>,
    ctx: &ValidationContext,
) -> syn::Result<()> {
    match reference {
        Reference::Identifier(ident) => {
            // Bare identifier must be a term/phrase, not a parameter.
            // If a parameter with this name exists, suggest using {$name}.
            if params.contains(&ident.name) {
                return Err(syn::Error::new(
                    ident.span,
                    format!(
                        "'{}' matches parameter '${}' — use {{${}}}",
                        ident.name, ident.name, ident.name
                    ),
                ));
            }
            // Must be a defined phrase/term
            if !ctx.phrases.contains(&ident.name) {
                let suggestions = compute_suggestions(&ident.name, ctx.phrases.iter());
                let mut msg = format!("unknown phrase '{}'", ident.name);
                if !suggestions.is_empty() {
                    msg.push_str(&format!("\nhelp: did you mean '{}'?", suggestions[0]));
                }
                return Err(syn::Error::new(ident.span, msg));
            }
        }
        Reference::Parameter(ident) => {
            // $-prefixed parameter must be declared
            if !params.contains(&ident.name) {
                // Check if the name matches a known term/phrase
                if ctx.phrases.contains(&ident.name) {
                    return Err(syn::Error::new(
                        ident.span,
                        format!(
                            "'${}' is not a declared parameter — remove '$' to reference term '{}'",
                            ident.name, ident.name
                        ),
                    ));
                }
                return Err(syn::Error::new(
                    ident.span,
                    format!(
                        "undefined parameter '${0}'\nhelp: declare it as a parameter: name(${0})",
                        ident.name
                    ),
                ));
            }
        }
        Reference::Call { name, args } => {
            // Phrase call - phrase must exist
            if !ctx.phrases.contains(&name.name) {
                let suggestions = compute_suggestions(&name.name, ctx.phrases.iter());
                let mut msg = format!("unknown phrase '{}'", name.name);
                if !suggestions.is_empty() {
                    msg.push_str(&format!("\nhelp: did you mean '{}'?", suggestions[0]));
                }
                return Err(syn::Error::new(name.span, msg));
            }

            // Check term/phrase usage: Call on a term is an error
            if let Some(&DefinitionKind::Term) = ctx.phrase_kinds.get(&name.name) {
                let suggestion = if args.len() == 1 {
                    let arg_str = match &args[0] {
                        Reference::Parameter(p) => format!("${}", p.name),
                        _ => "variant".to_string(),
                    };
                    format!(
                        "'{}' is a term — use {{{}:{}}}",
                        name.name, name.name, arg_str
                    )
                } else {
                    format!(
                        "'{}' is a term — cannot use () call syntax; use {{{}:variant}} or {{{}:$param}} to select a variant",
                        name.name, name.name, name.name
                    )
                };
                return Err(syn::Error::new(name.span, suggestion));
            }

            // Validate argument count
            if let Some(&expected) = ctx.phrase_param_counts.get(&name.name)
                && args.len() != expected
            {
                return Err(syn::Error::new(
                    name.span,
                    format!(
                        "phrase '{}' expects {} parameter{}, got {}",
                        name.name,
                        expected,
                        if expected == 1 { "" } else { "s" },
                        args.len()
                    ),
                ));
            }

            // Validate arguments recursively, checking for nested calls
            for arg in args {
                if let Reference::Call {
                    name: inner_name, ..
                } = arg
                {
                    return Err(syn::Error::new(
                        inner_name.span,
                        "nested phrase calls not supported as arguments — bind to a parameter in Rust instead",
                    ));
                }
                validate_reference(arg, params, ctx)?;
            }
        }
        Reference::NumberLiteral(..) | Reference::StringLiteral(..) => {
            // Literal values are always valid
        }
    }
    Ok(())
}

/// Compute typo suggestions using Levenshtein distance.
///
/// Match existing runtime behavior:
/// - distance <= 1 for keys <= 3 chars
/// - distance <= 2 for longer keys
/// - Limit to 3 suggestions, sorted by distance
fn compute_suggestions<'a>(name: &str, available: impl Iterator<Item = &'a String>) -> Vec<String> {
    let max_distance = if name.len() <= 3 { 1 } else { 2 };
    let mut suggestions: Vec<(usize, String)> = available
        .filter_map(|candidate| {
            let dist = levenshtein(name, candidate);
            if dist <= max_distance && dist > 0 {
                Some((dist, candidate.clone()))
            } else {
                None
            }
        })
        .collect();

    suggestions.sort_by_key(|(dist, _)| *dist);
    suggestions.into_iter().take(3).map(|(_, s)| s).collect()
}

/// Compute typo suggestions from a static slice of &str.
fn compute_suggestions_str(name: &str, available: &[&str]) -> Vec<String> {
    let max_distance = if name.len() <= 3 { 1 } else { 2 };
    let mut suggestions: Vec<(usize, String)> = available
        .iter()
        .filter_map(|candidate| {
            let dist = levenshtein(name, candidate);
            if dist <= max_distance && dist > 0 {
                Some((dist, (*candidate).to_string()))
            } else {
                None
            }
        })
        .collect();

    suggestions.sort_by_key(|(dist, _)| *dist);
    suggestions.into_iter().take(3).map(|(_, s)| s).collect()
}

// ============================================================================
// Cycle Detection (MACRO-14)
// ============================================================================

/// DFS coloring for cycle detection.
///
/// Uses a three-color algorithm:
/// - White: Node not yet visited
/// - Gray: Node is in the current DFS path (ancestor)
/// - Black: Node has been fully processed (all descendants visited)
///
/// A cycle is detected when we encounter a Gray node while traversing,
/// meaning we've found a back edge to an ancestor in the current path.
#[derive(Clone, Copy, PartialEq)]
enum Color {
    White, // Not visited
    Gray,  // In current DFS path
    Black, // Fully processed
}

/// Detect cycles in phrase references.
///
/// Uses DFS with coloring to find cycles in the phrase dependency graph.
/// Returns an error with the full cycle chain if found (e.g., "a -> b -> c -> a").
pub fn detect_cycles(input: &MacroInput, ctx: &ValidationContext) -> syn::Result<()> {
    // Build dependency graph: phrase name -> list of (phrase references, span)
    let mut deps: HashMap<String, Vec<(String, Span)>> = HashMap::new();

    for phrase in &input.phrases {
        let params: HashSet<String> = phrase.parameters.iter().map(|p| p.name.clone()).collect();
        let refs = collect_phrase_refs(&phrase.body, &params, ctx);
        deps.insert(phrase.name.name.clone(), refs);
    }

    // DFS with coloring
    let mut colors: HashMap<String, Color> =
        deps.keys().map(|k| (k.clone(), Color::White)).collect();

    // Sort keys for deterministic iteration order (stable trybuild tests)
    let mut sorted_keys: Vec<_> = deps.keys().cloned().collect();
    sorted_keys.sort();

    for name in &sorted_keys {
        if colors.get(name) == Some(&Color::White) {
            let mut path: Vec<String> = Vec::new();
            if let Some((cycle, span)) = dfs_find_cycle(name, &deps, &mut colors, &mut path) {
                // Format cycle chain: a -> b -> c -> a
                let chain = cycle.join(" -> ");
                return Err(syn::Error::new(
                    span,
                    format!("cyclic reference: {}", chain),
                ));
            }
        }
    }

    Ok(())
}

fn dfs_find_cycle(
    name: &str,
    deps: &HashMap<String, Vec<(String, Span)>>,
    colors: &mut HashMap<String, Color>,
    path: &mut Vec<String>,
) -> Option<(Vec<String>, Span)> {
    colors.insert(name.to_string(), Color::Gray);
    path.push(name.to_string());

    if let Some(refs) = deps.get(name) {
        // Sort refs for deterministic iteration order (stable trybuild tests)
        let mut sorted_refs = refs.clone();
        sorted_refs.sort_by(|(a, _), (b, _)| a.cmp(b));

        for (ref_name, span) in sorted_refs {
            match colors.get(&ref_name) {
                Some(Color::Gray) => {
                    // Found cycle - extract from path starting at the cycle point
                    let cycle_start = path.iter().position(|n| n == &ref_name).unwrap_or(0);
                    let mut cycle: Vec<String> = path[cycle_start..].to_vec();
                    cycle.push(ref_name.clone());
                    return Some((cycle, span));
                }
                Some(Color::White) | None => {
                    if let Some(result) = dfs_find_cycle(&ref_name, deps, colors, path) {
                        return Some(result);
                    }
                }
                Some(Color::Black) => {}
            }
        }
    }

    path.pop();
    colors.insert(name.to_string(), Color::Black);
    None
}

/// Collect all phrase references from a phrase body.
///
/// Filters out parameter references (only phrases can form cycles).
fn collect_phrase_refs(
    body: &PhraseBody,
    params: &HashSet<String>,
    ctx: &ValidationContext,
) -> Vec<(String, Span)> {
    let mut refs = Vec::new();
    match body {
        PhraseBody::Simple(template) => collect_template_refs(template, params, ctx, &mut refs),
        PhraseBody::Variants(variants) => {
            for variant in variants {
                collect_variant_entry_body_refs(&variant.body, params, ctx, &mut refs);
            }
        }
        PhraseBody::Match(branches) => {
            for branch in branches {
                collect_template_refs(&branch.template, params, ctx, &mut refs);
            }
        }
    }
    refs
}

fn collect_variant_entry_body_refs(
    body: &VariantEntryBody,
    params: &HashSet<String>,
    ctx: &ValidationContext,
    refs: &mut Vec<(String, Span)>,
) {
    match body {
        VariantEntryBody::Template(template) => {
            collect_template_refs(template, params, ctx, refs);
        }
        VariantEntryBody::Match { branches, .. } => {
            for branch in branches {
                collect_template_refs(&branch.template, params, ctx, refs);
            }
        }
    }
}

fn collect_template_refs(
    template: &Template,
    params: &HashSet<String>,
    ctx: &ValidationContext,
    refs: &mut Vec<(String, Span)>,
) {
    for segment in &template.segments {
        if let Segment::Interpolation(interp) = segment {
            collect_reference_refs(&interp.reference, params, ctx, refs);
        }
    }
}

fn collect_reference_refs(
    reference: &Reference,
    params: &HashSet<String>,
    ctx: &ValidationContext,
    refs: &mut Vec<(String, Span)>,
) {
    match reference {
        Reference::Identifier(ident) => {
            // Only add if it's a phrase reference (not a parameter)
            if !params.contains(&ident.name) && ctx.phrases.contains(&ident.name) {
                refs.push((ident.name.clone(), ident.span));
            }
        }
        Reference::Parameter(_) => {
            // Parameters are runtime values, not phrase references
        }
        Reference::Call { name, args } => {
            // Phrase calls are always phrase references
            if ctx.phrases.contains(&name.name) {
                refs.push((name.name.clone(), name.span));
            }
            for arg in args {
                collect_reference_refs(arg, params, ctx, refs);
            }
        }
        Reference::NumberLiteral(..) | Reference::StringLiteral(..) => {
            // Literal values are not phrase references
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    /// Helper to parse a rlf! macro input from tokens.
    fn parse_input(tokens: proc_macro2::TokenStream) -> MacroInput {
        syn::parse2(tokens).expect("should parse")
    }

    // =========================================================================
    // ValidationContext::from_input tests
    // =========================================================================

    #[test]
    fn test_context_empty_input() {
        let input = parse_input(parse_quote! {});
        let ctx = ValidationContext::from_input(&input);
        assert!(ctx.phrases.is_empty());
        assert!(ctx.phrase_variants.is_empty());
        assert!(ctx.phrase_tags.is_empty());
    }

    #[test]
    fn test_context_single_phrase() {
        let input = parse_input(parse_quote! {
            hello = "world";
        });
        let ctx = ValidationContext::from_input(&input);
        assert!(ctx.phrases.contains("hello"));
        assert_eq!(ctx.phrases.len(), 1);
    }

    #[test]
    fn test_context_phrase_with_variants() {
        let input = parse_input(parse_quote! {
            card = { one: "card", other: "cards" };
        });
        let ctx = ValidationContext::from_input(&input);
        assert!(ctx.phrases.contains("card"));

        let variants = ctx
            .phrase_variants
            .get("card")
            .expect("should have variants");
        assert!(variants.contains("one"));
        assert!(variants.contains("other"));
    }

    #[test]
    fn test_context_dotted_variant_keys() {
        let input = parse_input(parse_quote! {
            noun = {
                nom.one: "noun",
                nom.other: "nouns",
                acc.one: "noun",
                acc.other: "nouns"
            };
        });
        let ctx = ValidationContext::from_input(&input);

        let variants = ctx
            .phrase_variants
            .get("noun")
            .expect("should have variants");
        // Only exact keys are stored; fallback behavior is checked separately.
        assert!(variants.contains("nom.one"));
        assert!(variants.contains("nom.other"));
        assert!(variants.contains("acc.one"));
        assert!(variants.contains("acc.other"));
        assert!(!variants.contains("nom"));
        assert!(!variants.contains("acc"));
    }

    #[test]
    fn test_context_phrase_with_tags() {
        let input = parse_input(parse_quote! {
            item = :masc :inanimate "item";
        });
        let ctx = ValidationContext::from_input(&input);

        let tags = ctx.phrase_tags.get("item").expect("should have tags");
        assert!(tags.contains("masc"));
        assert!(tags.contains("inanimate"));
    }

    // =========================================================================
    // compute_suggestions tests
    // =========================================================================

    #[test]
    fn test_suggestions_exact_match_excluded() {
        let available = ["card".to_string()];
        let suggestions = compute_suggestions("card", available.iter());
        assert!(
            suggestions.is_empty(),
            "exact matches should not be suggested"
        );
    }

    #[test]
    fn test_suggestions_one_char_off_short() {
        let available = ["cat".to_string(), "dog".to_string()];
        let suggestions = compute_suggestions("car", available.iter());
        // "car" vs "cat" = distance 1, name len 3 <= 3, so should suggest
        assert!(suggestions.contains(&"cat".to_string()));
    }

    #[test]
    fn test_suggestions_two_chars_off_long() {
        let available = ["hello".to_string(), "world".to_string()];
        let suggestions = compute_suggestions("hallo", available.iter());
        // "hallo" vs "hello" = distance 1, should suggest
        assert!(suggestions.contains(&"hello".to_string()));
    }

    #[test]
    fn test_suggestions_three_chars_off_rejected() {
        let available = ["hello".to_string()];
        let suggestions = compute_suggestions("xxxxx", available.iter());
        // distance > 2, should not suggest
        assert!(suggestions.is_empty());
    }

    #[test]
    fn test_suggestions_sorted_by_distance() {
        let available = ["card".to_string(), "cart".to_string(), "cars".to_string()];
        let suggestions = compute_suggestions("carx", available.iter());
        // All have distance 1, should be sorted alphabetically (secondary)
        assert!(!suggestions.is_empty());
    }

    #[test]
    fn test_suggestions_limited_to_three() {
        let available = [
            "aa".to_string(),
            "ab".to_string(),
            "ac".to_string(),
            "ad".to_string(),
            "ae".to_string(),
        ];
        let suggestions = compute_suggestions("ax", available.iter());
        assert!(suggestions.len() <= 3);
    }

    // =========================================================================
    // validate() error condition tests
    // =========================================================================

    #[test]
    fn test_validate_undefined_phrase() {
        let input = parse_input(parse_quote! {
            greeting = "{unknown}";
        });
        let result = validate(&input);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("unknown phrase"));
    }

    #[test]
    fn test_validate_undefined_parameter() {
        let input = parse_input(parse_quote! {
            greet($name) = "Hello, {$unknown}";
        });
        let result = validate(&input);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("undefined parameter '$unknown'"));
    }

    #[test]
    fn test_validate_bare_identifier_matches_parameter() {
        let input = parse_input(parse_quote! {
            greet($name) = "Hello, {name}";
        });
        let result = validate(&input);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("'name' matches parameter '$name'"));
        assert!(err.contains("{$name}"));
    }

    #[test]
    fn test_validate_unknown_transform() {
        let input = parse_input(parse_quote! {
            bad = "{@nonexistent hello}";
            hello = "hello";
        });
        let result = validate(&input);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("unknown transform"));
    }

    #[test]
    fn test_validate_invalid_variant_selector() {
        let input = parse_input(parse_quote! {
            card = { one: "card", other: "cards" };
            bad = "{card:nonexistent}";
        });
        let result = validate(&input);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("has no variant"));
        assert!(err.contains("available variants"));
    }

    #[test]
    fn test_validate_static_selector_requires_resolvable_full_key() {
        let input = parse_input(parse_quote! {
            card = { nom.one: "card", nom.other: "cards" };
            bad = "{card:nom}";
        });
        let result = validate(&input);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("has no variant 'nom'"));
    }

    #[test]
    fn test_validate_mixed_selector_skips_static_variant_existence_check() {
        let input = parse_input(parse_quote! {
            card = { nom.one: "card", nom.other: "cards" };
            ok($n) = "{card:nom:$n}";
        });
        let result = validate(&input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_parameter_shadows_phrase() {
        let input = parse_input(parse_quote! {
            card = "card";
            bad($card) = "uses {$card}";
        });
        let result = validate(&input);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("shadows phrase"));
    }

    #[test]
    fn test_validate_undefined_parameter_in_selector() {
        let input = parse_input(parse_quote! {
            card = { one: "card", other: "cards" };
            draw($n) = "Draw {card:$unknown}.";
        });
        let result = validate(&input);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("undefined parameter '$unknown'"));
    }

    #[test]
    fn test_validate_valid_input() {
        let input = parse_input(parse_quote! {
            card = { one: "card", other: "cards" };
            draw($n) = "Draw {$n} {card:$n}.";
        });
        let result = validate(&input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_valid_with_transforms() {
        let input = parse_input(parse_quote! {
            hello = "hello";
            greeting = "{@cap hello} world!";
        });
        let result = validate(&input);
        assert!(result.is_ok());
    }

    // =========================================================================
    // detect_cycles tests
    // =========================================================================

    #[test]
    fn test_no_cycles() {
        let input = parse_input(parse_quote! {
            a = "see {b}";
            b = "end";
        });
        let ctx = ValidationContext::from_input(&input);
        let result = detect_cycles(&input, &ctx);
        assert!(result.is_ok());
    }

    #[test]
    fn test_direct_self_reference() {
        let input = parse_input(parse_quote! {
            a = "{a}";
        });
        let ctx = ValidationContext::from_input(&input);
        let result = detect_cycles(&input, &ctx);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("cyclic reference"));
        assert!(err.contains("a -> a"));
    }

    #[test]
    fn test_two_node_cycle() {
        let input = parse_input(parse_quote! {
            a = "{b}";
            b = "{a}";
        });
        let ctx = ValidationContext::from_input(&input);
        let result = detect_cycles(&input, &ctx);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("cyclic reference"));
        // Should contain either "a -> b -> a" or "b -> a -> b"
        assert!(err.contains("->"));
    }

    #[test]
    fn test_three_node_cycle() {
        let input = parse_input(parse_quote! {
            a = "{b}";
            b = "{c}";
            c = "{a}";
        });
        let ctx = ValidationContext::from_input(&input);
        let result = detect_cycles(&input, &ctx);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("cyclic reference"));
    }

    #[test]
    fn test_diamond_not_cycle() {
        // Diamond shape: a->b, a->c, b->d, c->d
        // This is NOT a cycle because there's no back edge
        let input = parse_input(parse_quote! {
            a = "{b} and {c}";
            b = "{d}";
            c = "{d}";
            d = "end";
        });
        let ctx = ValidationContext::from_input(&input);
        let result = detect_cycles(&input, &ctx);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parameters_dont_form_cycles() {
        // Parameters are runtime, not compile-time references
        let input = parse_input(parse_quote! {
            greet($name) = "Hello, {$name}";
        });
        let ctx = ValidationContext::from_input(&input);
        let result = detect_cycles(&input, &ctx);
        assert!(result.is_ok());
    }

    #[test]
    fn test_phrase_call_cycle() {
        let input = parse_input(parse_quote! {
            a($x) = "{b($x)}";
            b($y) = "{a($y)}";
        });
        let ctx = ValidationContext::from_input(&input);
        let result = detect_cycles(&input, &ctx);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("cyclic reference"));
    }
}
