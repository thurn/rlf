//! Template evaluation engine for RLF phrases.
//!
//! This module provides the core evaluation logic that transforms parsed templates
//! into formatted strings. It resolves references, applies selectors, builds variant
//! keys, and handles phrase calls with cycle detection.

use std::collections::HashMap;

use crate::interpreter::error::compute_suggestions;
use crate::interpreter::plural::plural_category;
use crate::interpreter::transforms::TransformRegistry;
use crate::interpreter::{EvalContext, EvalError, PhraseRegistry};
use crate::parser::ast::{
    PhraseBody, PhraseDefinition, Reference, Segment, Selector, Template, Transform, VariantEntry,
};
use crate::types::{Phrase, Tag, Value, VariantKey};

/// Evaluate a template AST, producing a formatted string.
///
/// This is the core evaluation function that processes a parsed template:
/// - Literal segments are copied directly to output
/// - Interpolations resolve references, apply selectors, and produce strings
///
/// # Arguments
///
/// * `template` - The parsed template AST
/// * `ctx` - Evaluation context with parameters and call stack
/// * `registry` - Registry for looking up phrase definitions
/// * `transform_registry` - Registry for looking up transforms
/// * `lang` - Language code for plural rules
///
/// # Errors
///
/// Returns an error if:
/// - A reference cannot be resolved (missing parameter or phrase)
/// - A variant is missing after selector resolution
/// - Maximum recursion depth is exceeded
/// - A cyclic reference is detected
/// - An unknown transform is used
pub fn eval_template(
    template: &Template,
    ctx: &mut EvalContext<'_>,
    registry: &PhraseRegistry,
    transform_registry: &TransformRegistry,
    lang: &str,
) -> Result<String, EvalError> {
    let mut output = String::new();
    for segment in &template.segments {
        match segment {
            Segment::Literal(s) => output.push_str(s),
            Segment::Interpolation {
                transforms,
                reference,
                selectors,
            } => {
                // 1. Resolve reference to Value
                let value = resolve_reference(
                    reference,
                    selectors,
                    ctx,
                    registry,
                    transform_registry,
                    lang,
                )?;
                // 2. Apply selectors to get variant/final value (returns Value to preserve tags)
                let selected = apply_selectors(&value, selectors, ctx, lang)?;
                // 3. Apply transforms (right-to-left per DESIGN.md)
                // Pass Value directly so transforms can access tags on first call
                let transformed =
                    apply_transforms(&selected, transforms, transform_registry, ctx, lang)?;
                output.push_str(&transformed);
            }
        }
    }
    Ok(output)
}

/// Resolve a reference to a Value.
///
/// Resolution order:
/// 1. Parameters (from the evaluation context)
/// 2. Phrases (from the registry)
///
/// When a `Reference::Identifier` refers to a parameterized phrase and the
/// selector count matches the parameter count, selectors are automatically
/// forwarded as arguments. This allows `{cards:n}` where `cards(n)` is
/// defined with variants.
fn resolve_reference(
    reference: &Reference,
    selectors: &[Selector],
    ctx: &mut EvalContext<'_>,
    registry: &PhraseRegistry,
    transform_registry: &TransformRegistry,
    lang: &str,
) -> Result<Value, EvalError> {
    match reference {
        Reference::Identifier(name) => {
            // First try parameters
            if let Some(value) = ctx.get_param(name) {
                return Ok(value.clone());
            }

            // Then try phrase lookup
            if let Some(def) = registry.get(name) {
                if !def.parameters.is_empty() {
                    // Auto-forward selectors as arguments when counts match
                    if selectors.len() == def.parameters.len() {
                        let resolved_args: Vec<Value> = selectors
                            .iter()
                            .map(|sel| resolve_selector_to_value(sel, ctx))
                            .collect::<Result<Vec<_>, _>>()?;

                        let params: HashMap<String, Value> = def
                            .parameters
                            .iter()
                            .zip(resolved_args)
                            .map(|(name, value)| (name.clone(), value))
                            .collect();

                        let mut child_ctx = EvalContext::with_string_context(
                            &params,
                            ctx.string_context().map(ToString::to_string),
                        );
                        child_ctx.push_call(name)?;
                        let result = eval_phrase_def(
                            def,
                            &mut child_ctx,
                            registry,
                            transform_registry,
                            lang,
                        )?;
                        child_ctx.pop_call();
                        return Ok(Value::Phrase(result));
                    }

                    return Err(EvalError::ArgumentCount {
                        phrase: name.clone(),
                        expected: def.parameters.len(),
                        got: 0,
                    });
                }

                // Evaluate the phrase
                ctx.push_call(name)?;
                let result = eval_phrase_def(def, ctx, registry, transform_registry, lang)?;
                ctx.pop_call();
                return Ok(Value::Phrase(result));
            }

            // Not found
            Err(EvalError::PhraseNotFound { name: name.clone() })
        }
        Reference::PhraseCall { name, args } => {
            let def = registry
                .get(name)
                .ok_or_else(|| EvalError::PhraseNotFound { name: name.clone() })?;

            // Validate argument count
            if def.parameters.len() != args.len() {
                return Err(EvalError::ArgumentCount {
                    phrase: name.clone(),
                    expected: def.parameters.len(),
                    got: args.len(),
                });
            }

            // Resolve arguments to values
            let resolved_args: Vec<Value> = args
                .iter()
                .map(|arg| resolve_reference(arg, &[], ctx, registry, transform_registry, lang))
                .collect::<Result<Vec<_>, _>>()?;

            // Build param map for child context
            let params: HashMap<String, Value> = def
                .parameters
                .iter()
                .zip(resolved_args)
                .map(|(name, value)| (name.clone(), value))
                .collect();

            // Create child context (no scope inheritance per RESEARCH.md,
            // but string_context propagates for consistent format selection)
            let mut child_ctx = EvalContext::with_string_context(
                &params,
                ctx.string_context().map(ToString::to_string),
            );
            child_ctx.push_call(name)?;
            let result = eval_phrase_def(def, &mut child_ctx, registry, transform_registry, lang)?;
            child_ctx.pop_call();

            Ok(Value::Phrase(result))
        }
    }
}

/// Evaluate a phrase definition to produce a Phrase.
///
/// Handles:
/// - Simple phrases (single template)
/// - Variant phrases (multiple templates with keys)
/// - :from modifier for metadata inheritance
pub fn eval_phrase_def(
    def: &PhraseDefinition,
    ctx: &mut EvalContext<'_>,
    registry: &PhraseRegistry,
    transform_registry: &TransformRegistry,
    lang: &str,
) -> Result<Phrase, EvalError> {
    // Convert definition tags to Phrase tags
    let tags: Vec<Tag> = def.tags.clone();

    // Handle :from modifier for metadata inheritance
    if let Some(from_param) = &def.from_param {
        return eval_with_from_modifier(def, from_param, ctx, registry, transform_registry, lang);
    }

    match &def.body {
        PhraseBody::Simple(template) => {
            let text = eval_template(template, ctx, registry, transform_registry, lang)?;
            Ok(Phrase::builder().text(text).tags(tags).build())
        }
        PhraseBody::Variants(entries) => {
            let (text, variants) =
                build_phrase_from_variants(entries, ctx, registry, transform_registry, lang)?;
            let text = auto_select_variant(&def.parameters, ctx, &variants, lang).unwrap_or(text);
            Ok(Phrase::builder()
                .text(text)
                .variants(variants)
                .tags(tags)
                .build())
        }
    }
}

/// Evaluate a phrase with :from modifier for metadata inheritance.
///
/// The :from modifier copies tags from a source phrase and evaluates
/// the template once per variant if the source has variants.
fn eval_with_from_modifier(
    def: &PhraseDefinition,
    from_param: &str,
    ctx: &mut EvalContext<'_>,
    registry: &PhraseRegistry,
    transform_registry: &TransformRegistry,
    lang: &str,
) -> Result<Phrase, EvalError> {
    // Get the source phrase from the parameter and clone its data upfront
    // to avoid borrow conflicts when we later use ctx mutably
    let (inherited_tags, source_variants) = {
        let source = ctx
            .get_param(from_param)
            .ok_or_else(|| EvalError::PhraseNotFound {
                name: format!("parameter '{}' not found for :from modifier", from_param),
            })?;

        let source_phrase = source
            .as_phrase()
            .ok_or_else(|| EvalError::PhraseNotFound {
                name: format!(
                    "parameter '{}' must be a Phrase for :from modifier",
                    from_param
                ),
            })?;

        (source_phrase.tags.clone(), source_phrase.variants.clone())
    };

    // Get the template from the definition body
    let template = match &def.body {
        PhraseBody::Simple(t) => t,
        PhraseBody::Variants(_) => {
            // :from with variants is not supported - use simple template
            return Err(EvalError::PhraseNotFound {
                name: ":from modifier cannot be used with variant definitions".to_string(),
            });
        }
    };

    // If source has variants, evaluate template once per variant
    if !source_variants.is_empty() {
        let mut variants = HashMap::new();

        // Evaluate for default text first
        let default_text = eval_template(template, ctx, registry, transform_registry, lang)?;

        // Evaluate for each variant (sorted keys for deterministic order)
        let mut sorted_keys: Vec<_> = source_variants.keys().collect();
        sorted_keys.sort();
        for key in sorted_keys {
            let variant_text = &source_variants[key];
            // Create a context with the variant text substituted for the from_param
            let simple_params: HashMap<String, Value> =
                [(from_param.to_string(), Value::String(variant_text.clone()))]
                    .into_iter()
                    .collect();
            let mut variant_ctx = EvalContext::with_string_context(
                &simple_params,
                ctx.string_context().map(ToString::to_string),
            );

            let variant_result = eval_template(
                template,
                &mut variant_ctx,
                registry,
                transform_registry,
                lang,
            )?;
            variants.insert(key.clone(), variant_result);
        }

        Ok(Phrase::builder()
            .text(default_text)
            .variants(variants)
            .tags(inherited_tags)
            .build())
    } else {
        // No variants - just evaluate the template
        let text = eval_template(template, ctx, registry, transform_registry, lang)?;
        Ok(Phrase::builder().text(text).tags(inherited_tags).build())
    }
}

/// Apply selectors to a value, producing a Value.
///
/// Selectors are resolved and combined with "." to form a compound key.
/// The key is then used to look up a variant in the phrase.
///
/// When a selector resolves from a Phrase parameter with multiple tags
/// (e.g., `:masc :anim`), all tags are tried as candidates. This enables
/// languages like Russian where gender and animacy are separate tags.
///
/// If no selectors are present, returns the original Value unchanged,
/// preserving Phrase type with its tags for transform access.
fn apply_selectors(
    value: &Value,
    selectors: &[Selector],
    ctx: &EvalContext<'_>,
    lang: &str,
) -> Result<Value, EvalError> {
    if selectors.is_empty() {
        // No selectors - return the original Value (preserves Phrase type with tags)
        return Ok(value.clone());
    }

    // Build candidate key parts from selectors. Each selector position may
    // have multiple candidates (e.g., a Phrase with tags [:masc, :anim]).
    let mut candidate_parts: Vec<Vec<String>> = Vec::new();
    for selector in selectors {
        let candidates = resolve_selector_candidates(selector, ctx, lang)?;
        candidate_parts.push(candidates);
    }

    // Generate compound keys by taking the cartesian product of candidates,
    // trying each combination until one matches. For typical usage (most
    // selectors have one candidate), this is a single lookup.
    let compound_keys = build_compound_keys(&candidate_parts);

    match value {
        Value::Phrase(phrase) => {
            for key in &compound_keys {
                if let Ok(variant_text) = variant_lookup(phrase, key) {
                    // Preserve tags through variant selection so transforms can
                    // still access metadata (e.g., @a needs :a tag after :n selector)
                    return Ok(Value::Phrase(
                        Phrase::builder()
                            .text(variant_text)
                            .tags(phrase.tags.clone())
                            .build(),
                    ));
                }
            }
            // None matched - report error using the first (most specific) key
            let primary_key = compound_keys
                .first()
                .cloned()
                .unwrap_or_else(|| "?".to_string());
            variant_lookup(phrase, &primary_key).map(|text| {
                Value::Phrase(
                    Phrase::builder()
                        .text(text)
                        .tags(phrase.tags.clone())
                        .build(),
                )
            })
        }
        _ => {
            // Non-phrase values don't have variants
            let primary_key = compound_keys
                .first()
                .cloned()
                .unwrap_or_else(|| "?".to_string());
            let available: Vec<String> = vec![];
            Err(EvalError::MissingVariant {
                phrase: value.to_string(),
                key: primary_key.clone(),
                suggestions: compute_suggestions(&primary_key, &available),
                available,
            })
        }
    }
}

/// Build compound keys from candidate parts via cartesian product.
///
/// Each position in `parts` may have multiple candidates. This generates
/// all combinations joined with ".".
fn build_compound_keys(parts: &[Vec<String>]) -> Vec<String> {
    if parts.is_empty() {
        return vec![String::new()];
    }

    let mut result = vec![String::new()];
    for candidates in parts {
        let mut next = Vec::new();
        for prefix in &result {
            for candidate in candidates {
                let key = if prefix.is_empty() {
                    candidate.clone()
                } else {
                    format!("{prefix}.{candidate}")
                };
                next.push(key);
            }
        }
        result = next;
    }
    result
}

/// Resolve a selector to a Value for use as a forwarded argument.
///
/// Used when auto-forwarding selectors as arguments to parameterized phrases.
/// Returns the parameter's value if it references one, or a string literal otherwise.
fn resolve_selector_to_value(
    selector: &Selector,
    ctx: &EvalContext<'_>,
) -> Result<Value, EvalError> {
    match selector {
        Selector::Identifier(name) => {
            if let Some(value) = ctx.get_param(name) {
                Ok(value.clone())
            } else {
                Ok(Value::String(name.clone()))
            }
        }
    }
}

/// Resolve a selector to candidate key strings.
///
/// Returns one or more candidate keys for a selector position:
/// - Number -> single candidate: plural_category(lang, n)
/// - Phrase -> all tags as candidates (enables multi-tag languages like Russian)
/// - String -> single candidate: parsed number or literal
/// - Literal -> single candidate: the identifier itself
fn resolve_selector_candidates(
    selector: &Selector,
    ctx: &EvalContext<'_>,
    lang: &str,
) -> Result<Vec<String>, EvalError> {
    match selector {
        Selector::Identifier(name) => {
            // Check if this is a parameter reference
            if let Some(value) = ctx.get_param(name) {
                match value {
                    Value::Number(n) => Ok(vec![plural_category(lang, *n).to_string()]),
                    Value::Float(f) => Ok(vec![plural_category(lang, *f as i64).to_string()]),
                    Value::Phrase(phrase) => {
                        // Use all tags as candidates, preserving order
                        let tags: Vec<String> =
                            phrase.tags.iter().map(ToString::to_string).collect();
                        if tags.is_empty() {
                            return Err(EvalError::MissingTag {
                                transform: "selector".to_string(),
                                expected: vec!["any".to_string()],
                                phrase: phrase.text.clone(),
                            });
                        }
                        Ok(tags)
                    }
                    Value::String(s) => {
                        if let Ok(n) = s.parse::<i64>() {
                            Ok(vec![plural_category(lang, n).to_string()])
                        } else {
                            Ok(vec![s.clone()])
                        }
                    }
                }
            } else {
                // Not a parameter - use as literal key
                Ok(vec![name.clone()])
            }
        }
    }
}

/// Look up a variant with fallback resolution.
///
/// Resolution order:
/// 1. Try exact key
/// 2. Progressively strip trailing ".segment"
///
/// Returns MissingVariant error if no match found.
fn variant_lookup(phrase: &Phrase, key: &str) -> Result<String, EvalError> {
    // Try exact match
    if let Some(v) = phrase.variants.get(&VariantKey::new(key)) {
        return Ok(v.clone());
    }

    // Try progressively shorter keys (fallback resolution)
    let mut current = key;
    while let Some(dot_pos) = current.rfind('.') {
        current = &current[..dot_pos];
        if let Some(v) = phrase.variants.get(&VariantKey::new(current)) {
            return Ok(v.clone());
        }
    }

    // If no variants exist but we have a key, this might be a simple phrase
    // being used with a selector - return the default text
    if phrase.variants.is_empty() {
        return Ok(phrase.text.clone());
    }

    // No match found - return error with available variants
    let mut available: Vec<String> = phrase.variants.keys().map(ToString::to_string).collect();
    available.sort();
    let suggestions = compute_suggestions(key, &available);
    Err(EvalError::MissingVariant {
        phrase: phrase.text.clone(),
        key: key.to_string(),
        suggestions,
        available,
    })
}

/// Apply transforms to a Value, executing right-to-left.
///
/// Per DESIGN.md: `{@cap @a card}` executes @a first, then @cap.
/// The first transform receives the original Value (possibly a Phrase with tags).
/// After each transform executes, the result is wrapped as Value::String for subsequent transforms.
fn apply_transforms(
    initial_value: &Value,
    transforms: &[Transform],
    transform_registry: &TransformRegistry,
    ctx: &EvalContext<'_>,
    lang: &str,
) -> Result<String, EvalError> {
    if transforms.is_empty() {
        return Ok(initial_value.to_string());
    }

    // Start with the initial Value (preserves Phrase type with tags for first transform)
    let mut current = initial_value.clone();

    // Process right-to-left (reverse iteration)
    for transform in transforms.iter().rev() {
        let transform_kind = transform_registry
            .get(&transform.name, lang)
            .ok_or_else(|| EvalError::UnknownTransform {
                name: transform.name.clone(),
            })?;

        // Resolve context selector if present
        let context_value = if let Some(ctx_selector) = &transform.context {
            match ctx_selector {
                Selector::Identifier(name) => {
                    // Try parameter lookup first
                    if let Some(param) = ctx.get_param(name) {
                        Some(param.clone())
                    } else {
                        // Use as literal string (e.g., "acc", "nom", "dat", "gen")
                        Some(Value::String(name.clone()))
                    }
                }
            }
        } else {
            None
        };

        // Pass full Value to transform so it can read tags (on first iteration)
        let result = transform_kind.execute(&current, context_value.as_ref(), lang)?;
        // After transform, result is String - wrap for next iteration
        current = Value::String(result);
    }

    Ok(current.to_string())
}

/// Build a Phrase from variant entries.
///
/// Evaluates each variant template and populates the variants HashMap.
/// When a string context is set, uses the context-matching variant as the
/// default text. Otherwise, uses the first entry's text.
fn build_phrase_from_variants(
    entries: &[VariantEntry],
    ctx: &mut EvalContext<'_>,
    registry: &PhraseRegistry,
    transform_registry: &TransformRegistry,
    lang: &str,
) -> Result<(String, HashMap<VariantKey, String>), EvalError> {
    let mut variants = HashMap::new();
    let mut default_text = String::new();
    let mut context_text: Option<String> = None;

    for (i, entry) in entries.iter().enumerate() {
        let text = eval_template(&entry.template, ctx, registry, transform_registry, lang)?;

        // First variant's text becomes the default
        if i == 0 {
            default_text = text.clone();
        }

        // Check if any key matches the string context
        if let Some(string_ctx) = ctx.string_context()
            && context_text.is_none()
        {
            for key in &entry.keys {
                if key == string_ctx {
                    context_text = Some(text.clone());
                    break;
                }
            }
        }

        // Add to variants map for each key
        for key in &entry.keys {
            variants.insert(VariantKey::new(key.clone()), text.clone());
        }
    }

    // Prefer context-matching variant as default text
    if let Some(ctx_text) = context_text {
        default_text = ctx_text;
    }

    Ok((default_text, variants))
}

/// Auto-select a variant based on numeric parameters.
///
/// When a variant phrase is called with a numeric parameter, the CLDR plural
/// category of that number determines the default text. This allows patterns
/// like `this_turn_times(n) = { one: "this turn", other: "this turn {n} times" }`
/// to automatically produce the correct variant when called directly.
///
/// Returns `None` if no numeric parameter is found or no variant matches.
fn auto_select_variant(
    param_names: &[String],
    ctx: &EvalContext<'_>,
    variants: &HashMap<VariantKey, String>,
    lang: &str,
) -> Option<String> {
    if variants.is_empty() {
        return None;
    }

    for name in param_names {
        let Some(value) = ctx.get_param(name) else {
            continue;
        };
        let category = match value {
            Value::Number(n) => plural_category(lang, *n),
            Value::Float(f) => plural_category(lang, *f as i64),
            _ => continue,
        };

        // Try exact match, then progressively shorter fallback keys
        let key = category.to_string();
        if let Some(text) = variants.get(&VariantKey::new(&key)) {
            return Some(text.clone());
        }
        // Try fallback (strip trailing .segment)
        let mut current = key.as_str();
        while let Some(dot_pos) = current.rfind('.') {
            current = &current[..dot_pos];
            if let Some(text) = variants.get(&VariantKey::new(current)) {
                return Some(text.clone());
            }
        }
    }

    None
}
