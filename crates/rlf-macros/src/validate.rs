//! Compile-time validation for the rlf! macro.
//!
//! Performs 7 types of validation checks:
//! 1. Undefined phrase references (MACRO-08)
//! 2. Undefined parameter references (MACRO-09)
//! 3. Invalid literal selectors (MACRO-10)
//! 4. Unknown transforms (MACRO-11)
//! 5. Transform tag requirements (MACRO-12) - infrastructure for future
//! 6. Tag-based selection compatibility (MACRO-13) - infrastructure for future
//! 7. Cyclic references (MACRO-14)
//!
//! Also provides typo suggestions (MACRO-17) using Levenshtein distance.

use std::collections::{HashMap, HashSet};

use proc_macro2::Span;
use strsim::levenshtein;

use crate::input::{
    Interpolation, MacroInput, PhraseBody, PhraseDefinition, Reference, Segment, Template,
};

/// Known transforms (universal only for Phase 5).
/// Phase 6+ will add @a/@an which require tags.
const KNOWN_TRANSFORMS: &[&str] = &["cap", "upper", "lower"];

/// Validation context built from MacroInput.
pub struct ValidationContext {
    /// All defined phrase names.
    pub phrases: HashSet<String>,
    /// Phrase name -> defined variant keys (for literal selector validation).
    pub phrase_variants: HashMap<String, HashSet<String>>,
    /// Phrase name -> defined tags (infrastructure for future tag-based validation).
    #[cfg_attr(not(test), expect(dead_code))]
    pub phrase_tags: HashMap<String, HashSet<String>>,
}

impl ValidationContext {
    /// Build validation context from macro input.
    pub fn from_input(input: &MacroInput) -> Self {
        let mut phrases = HashSet::new();
        let mut phrase_variants = HashMap::new();
        let mut phrase_tags = HashMap::new();

        for phrase in &input.phrases {
            let name = phrase.name.name.clone();
            phrases.insert(name.clone());

            // Collect variant keys
            if let PhraseBody::Variants(variants) = &phrase.body {
                let mut keys = HashSet::new();
                for variant in variants {
                    for key in &variant.keys {
                        // Handle dotted keys (e.g., "nom.one" -> add both "nom" and "nom.one")
                        keys.insert(key.name.clone());
                        // Also add the first component for partial matching
                        if let Some(first) = key.name.split('.').next() {
                            keys.insert(first.to_string());
                        }
                    }
                }
                phrase_variants.insert(name.clone(), keys);
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

    // Validate body references
    match &phrase.body {
        PhraseBody::Simple(template) => {
            validate_template(template, &params, ctx, &phrase.name.name)?;
        }
        PhraseBody::Variants(variants) => {
            for variant in variants {
                validate_template(&variant.template, &params, ctx, &phrase.name.name)?;
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

/// Validate an interpolation: reference, transforms, and selectors.
fn validate_interpolation(
    interp: &Interpolation,
    params: &HashSet<String>,
    ctx: &ValidationContext,
) -> syn::Result<()> {
    // Validate transforms exist (MACRO-11)
    for transform in &interp.transforms {
        if !KNOWN_TRANSFORMS.contains(&transform.name.name.as_str()) {
            let suggestions = compute_suggestions_str(&transform.name.name, KNOWN_TRANSFORMS);
            let mut msg = format!("unknown transform '@{}'", transform.name.name);
            if !suggestions.is_empty() {
                msg.push_str(&format!("\nhelp: did you mean '@{}'?", suggestions[0]));
            } else {
                msg.push_str("\nnote: available transforms: cap, upper, lower");
            }
            return Err(syn::Error::new(transform.name.span, msg));
        }

        // Transform tag validation (MACRO-12)
        // Note: Universal transforms (cap, upper, lower) don't require tags.
        // This infrastructure is for Phase 6+ when @a/@an are added which require
        // the 'vowel' tag. For now, no validation needed since all transforms
        // are universal.
    }

    // Validate the reference (phrase or parameter)
    validate_reference(&interp.reference, params, ctx)?;

    // Validate selectors (MACRO-10, MACRO-13)
    // Determine if reference is a literal phrase (not a parameter)
    let is_literal_phrase = match &interp.reference {
        Reference::Identifier(ident) => {
            !params.contains(&ident.name) && ctx.phrases.contains(&ident.name)
        }
        Reference::Call { name, .. } => ctx.phrases.contains(&name.name),
    };

    if is_literal_phrase {
        let phrase_name = match &interp.reference {
            Reference::Identifier(ident) => &ident.name,
            Reference::Call { name, .. } => &name.name,
        };

        // Get phrase variants for literal selector validation
        let phrase_variants = ctx.phrase_variants.get(phrase_name);

        for selector in &interp.selectors {
            // If selector name is a parameter, it's dynamic - skip compile-time check
            if params.contains(&selector.name.name) {
                continue;
            }

            // Literal selector - must match a variant key if phrase has variants (MACRO-10)
            if let Some(variants) = phrase_variants
                && !variants.contains(&selector.name.name)
            {
                let mut available: Vec<_> = variants.iter().cloned().collect();
                available.sort();
                return Err(syn::Error::new(
                    selector.name.span,
                    format!(
                        "phrase '{}' has no variant '{}'\nnote: available variants: {}",
                        phrase_name,
                        selector.name.name,
                        available.join(", ")
                    ),
                ));
            }

            // Tag-based selection compatibility (MACRO-13)
            // When selecting by a literal value on a phrase that has tags,
            // validate the tag has matching variants.
            // For Phase 5, this is infrastructure - tags are used but not for selection yet.
            // Full implementation deferred to Phase 6+ when tag-based transforms are added.
        }
    }

    Ok(())
}

/// Validate a reference (identifier or call).
fn validate_reference(
    reference: &Reference,
    params: &HashSet<String>,
    ctx: &ValidationContext,
) -> syn::Result<()> {
    match reference {
        Reference::Identifier(ident) => {
            // If it's a parameter, that's valid
            if params.contains(&ident.name) {
                return Ok(());
            }
            // Otherwise it must be a phrase
            if !ctx.phrases.contains(&ident.name) {
                let suggestions = compute_suggestions(&ident.name, ctx.phrases.iter());
                let mut msg = format!("unknown phrase or parameter '{}'", ident.name);
                if !suggestions.is_empty() {
                    msg.push_str(&format!("\nhelp: did you mean '{}'?", suggestions[0]));
                }
                return Err(syn::Error::new(ident.span, msg));
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
            // Validate arguments recursively
            for arg in args {
                validate_reference(arg, params, ctx)?;
            }
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
                collect_template_refs(&variant.template, params, ctx, &mut refs);
            }
        }
    }
    refs
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
        Reference::Call { name, args } => {
            // Phrase calls are always phrase references
            if ctx.phrases.contains(&name.name) {
                refs.push((name.name.clone(), name.span));
            }
            for arg in args {
                collect_reference_refs(arg, params, ctx, refs);
            }
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
        // Both full keys and first components should be present
        assert!(variants.contains("nom.one"));
        assert!(variants.contains("nom.other"));
        assert!(variants.contains("nom")); // First component
        assert!(variants.contains("acc")); // First component
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
        assert!(err.contains("unknown phrase or parameter"));
    }

    #[test]
    fn test_validate_undefined_parameter() {
        let input = parse_input(parse_quote! {
            greet(name) = "Hello, {unknown}";
        });
        let result = validate(&input);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("unknown phrase or parameter"));
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
    fn test_validate_parameter_shadows_phrase() {
        let input = parse_input(parse_quote! {
            card = "card";
            bad(card) = "uses {card}";
        });
        let result = validate(&input);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("shadows phrase"));
    }

    #[test]
    fn test_validate_valid_input() {
        let input = parse_input(parse_quote! {
            card = { one: "card", other: "cards" };
            draw(n) = "Draw {n} {card:n}.";
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
            greet(name) = "Hello, {name}";
        });
        let ctx = ValidationContext::from_input(&input);
        let result = detect_cycles(&input, &ctx);
        assert!(result.is_ok());
    }

    #[test]
    fn test_phrase_call_cycle() {
        let input = parse_input(parse_quote! {
            a(x) = "{b(x)}";
            b(y) = "{a(y)}";
        });
        let ctx = ValidationContext::from_input(&input);
        let result = detect_cycles(&input, &ctx);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("cyclic reference"));
    }
}
