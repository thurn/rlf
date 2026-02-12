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
    DefinitionKind, MatchBranch, PhraseBody, PhraseDefinition, Reference, Segment, Selector,
    Template, Transform, TransformContext, VariantEntry, VariantEntryBody,
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
                let value = resolve_reference(reference, ctx, registry, transform_registry, lang)?;
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
/// Uses the AST distinction between parameters and identifiers:
/// - `Reference::Parameter(name)` → look up in current parameter bindings
/// - `Reference::Identifier(name)` → look up as a term/phrase in the registry
/// - `Reference::PhraseCall { name, args }` → evaluate phrase call
///
/// No implicit fallback: parameters never check the registry, and identifiers
/// never check parameter bindings.
fn resolve_reference(
    reference: &Reference,
    ctx: &mut EvalContext<'_>,
    registry: &PhraseRegistry,
    transform_registry: &TransformRegistry,
    lang: &str,
) -> Result<Value, EvalError> {
    match reference {
        Reference::Parameter(name) => {
            // Parameter reference ($name): look up in context only
            ctx.get_param(name)
                .cloned()
                .ok_or_else(|| EvalError::UnknownParameter { name: name.clone() })
        }
        Reference::Identifier(name) => {
            // Bare identifier: look up as term/phrase in registry only
            let def = registry
                .get(name)
                .ok_or_else(|| EvalError::PhraseNotFound { name: name.clone() })?;

            if def.kind == DefinitionKind::Phrase {
                // A bare identifier referencing a phrase (with parameters)
                // is always an error — phrases must be called with ()
                return Err(EvalError::SelectorOnPhrase { name: name.clone() });
            }

            // Evaluate the term
            ctx.push_call(name)?;
            let result = eval_phrase_def(def, ctx, registry, transform_registry, lang)?;
            ctx.pop_call();
            Ok(Value::Phrase(result))
        }
        Reference::NumberLiteral(n) => Ok(Value::Number(*n)),
        Reference::StringLiteral(s) => Ok(Value::String(s.clone())),
        Reference::PhraseCall { name, args } => {
            let def = registry
                .get(name)
                .ok_or_else(|| EvalError::PhraseNotFound { name: name.clone() })?;

            if def.kind == DefinitionKind::Term {
                // Terms cannot be called with () — use : for variant selection
                return Err(EvalError::ArgumentsToTerm { name: name.clone() });
            }

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
                .map(|arg| resolve_reference(arg, ctx, registry, transform_registry, lang))
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
            Ok(Phrase::builder()
                .text(text)
                .variants(variants)
                .tags(tags)
                .build())
        }
        PhraseBody::Match(branches) => {
            let text = eval_match_branches(
                branches,
                &def.match_params,
                ctx,
                registry,
                transform_registry,
                lang,
            )?;
            Ok(Phrase::builder().text(text).tags(tags).build())
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

    // Get the template from the definition body (or evaluate match/variants)
    let template = match &def.body {
        PhraseBody::Simple(t) => t,
        PhraseBody::Variants(entries) => {
            return eval_from_with_variants(
                def,
                from_param,
                entries,
                ctx,
                registry,
                transform_registry,
                lang,
            );
        }
        PhraseBody::Match(_) => {
            // :from + :match: evaluate the match for each inherited variant
            // The match selection runs within each variant evaluation
            return eval_from_with_match(def, from_param, ctx, registry, transform_registry, lang);
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
            // Create a Phrase with variant text as display but preserving
            // original tags and variants for phrase call arguments.
            let mut variant_params: HashMap<String, Value> = HashMap::new();
            variant_params.insert(
                from_param.to_string(),
                Value::Phrase(
                    Phrase::builder()
                        .text(variant_text.clone())
                        .tags(inherited_tags.clone())
                        .variants(source_variants.clone())
                        .build(),
                ),
            );
            // Copy all other parameters from the parent context
            for param_name in &def.parameters {
                if param_name != from_param
                    && let Some(v) = ctx.get_param(param_name)
                {
                    variant_params.insert(param_name.clone(), v.clone());
                }
            }
            let mut variant_ctx = EvalContext::with_string_context(
                &variant_params,
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

/// Evaluate a phrase with :from modifier and variant blocks.
///
/// Each variant key in the definition provides a per-variant body (either a
/// simple template or a `:match` block). When a variant key is requested on
/// the result, the corresponding body is evaluated with the :from parameter
/// bound to that variant of the source phrase. If the definition lacks the
/// requested key, it falls back to the `*`-marked default entry.
fn eval_from_with_variants(
    def: &PhraseDefinition,
    from_param: &str,
    entries: &[VariantEntry],
    ctx: &mut EvalContext<'_>,
    registry: &PhraseRegistry,
    transform_registry: &TransformRegistry,
    lang: &str,
) -> Result<Phrase, EvalError> {
    // Get the source phrase from the parameter and clone its data upfront
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

    // Build lookup from variant key -> entry body for the definition's entries
    let mut def_variants: HashMap<String, &VariantEntryBody> = HashMap::new();
    let mut default_body: Option<&VariantEntryBody> = None;

    for entry in entries {
        if entry.is_default {
            default_body = Some(&entry.body);
        }
        for key in &entry.keys {
            def_variants.insert(key.clone(), &entry.body);
        }
    }

    // Evaluate the default text using the first available default body
    // or the source's default text
    let default_text = if let Some(body) = default_body {
        eval_variant_entry_body(body, ctx, registry, transform_registry, lang)?
    } else if let Some(first_entry) = entries.first() {
        eval_variant_entry_body(&first_entry.body, ctx, registry, transform_registry, lang)?
    } else {
        String::new()
    };

    if source_variants.is_empty() {
        // No source variants: evaluate the default body with the source as-is
        return Ok(Phrase::builder()
            .text(default_text)
            .tags(inherited_tags)
            .build());
    }

    // For each source variant key, find the matching definition entry and evaluate
    let mut result_variants = HashMap::new();
    let mut sorted_keys: Vec<_> = source_variants.keys().collect();
    sorted_keys.sort();

    for key in sorted_keys {
        let variant_text = &source_variants[key];

        // Find the matching entry body: try exact key match, then progressive
        // fallback by stripping trailing ".segment", then default
        let body = find_variant_entry_body(key.as_ref(), &def_variants)
            .or(default_body)
            .ok_or_else(|| EvalError::MissingVariant {
                phrase: format!(":from variant block in '{}'", def.name),
                key: key.to_string(),
                suggestions: vec![],
                available: def_variants.keys().cloned().collect(),
            })?;

        // Build params map: substitute from_param with the variant-specific Phrase
        let mut variant_params: HashMap<String, Value> = HashMap::new();
        variant_params.insert(
            from_param.to_string(),
            Value::Phrase(
                Phrase::builder()
                    .text(variant_text.clone())
                    .tags(inherited_tags.clone())
                    .variants(source_variants.clone())
                    .build(),
            ),
        );
        // Copy all other parameters from the parent context
        for param_name in &def.parameters {
            if param_name != from_param
                && let Some(v) = ctx.get_param(param_name)
            {
                variant_params.insert(param_name.clone(), v.clone());
            }
        }
        let mut variant_ctx = EvalContext::with_string_context(
            &variant_params,
            ctx.string_context().map(ToString::to_string),
        );

        let variant_result =
            eval_variant_entry_body(body, &mut variant_ctx, registry, transform_registry, lang)?;
        result_variants.insert(key.clone(), variant_result);
    }

    Ok(Phrase::builder()
        .text(default_text)
        .variants(result_variants)
        .tags(inherited_tags)
        .build())
}

/// Evaluate a variant entry body (template or match block).
fn eval_variant_entry_body(
    body: &VariantEntryBody,
    ctx: &mut EvalContext<'_>,
    registry: &PhraseRegistry,
    transform_registry: &TransformRegistry,
    lang: &str,
) -> Result<String, EvalError> {
    match body {
        VariantEntryBody::Template(template) => {
            eval_template(template, ctx, registry, transform_registry, lang)
        }
        VariantEntryBody::Match {
            match_params,
            branches,
        } => eval_match_branches(
            branches,
            match_params,
            ctx,
            registry,
            transform_registry,
            lang,
        ),
    }
}

/// Find a variant entry body matching a variant key with progressive fallback.
///
/// Tries exact key match first, then progressively strips trailing ".segment"
/// to find a broader match.
fn find_variant_entry_body<'a>(
    key: &str,
    def_variants: &HashMap<String, &'a VariantEntryBody>,
) -> Option<&'a VariantEntryBody> {
    // Try exact match
    if let Some(body) = def_variants.get(key) {
        return Some(body);
    }

    // Try progressively shorter keys (fallback resolution)
    let mut current = key;
    while let Some(dot_pos) = current.rfind('.') {
        current = &current[..dot_pos];
        if let Some(body) = def_variants.get(current) {
            return Some(body);
        }
    }

    None
}

/// Evaluate a phrase with both :from and :match modifiers.
///
/// Inherits tags/variants from :from parameter, evaluates :match branches
/// within each inherited variant's context.
fn eval_from_with_match(
    def: &PhraseDefinition,
    from_param: &str,
    ctx: &mut EvalContext<'_>,
    registry: &PhraseRegistry,
    transform_registry: &TransformRegistry,
    lang: &str,
) -> Result<Phrase, EvalError> {
    let PhraseBody::Match(branches) = &def.body else {
        return Err(EvalError::PhraseNotFound {
            name: "expected Match body in eval_from_with_match".to_string(),
        });
    };

    // Get source phrase from the parameter
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

    // Evaluate match for default text
    let default_text = eval_match_branches(
        branches,
        &def.match_params,
        ctx,
        registry,
        transform_registry,
        lang,
    )?;

    if !source_variants.is_empty() {
        let mut variants = HashMap::new();
        let mut sorted_keys: Vec<_> = source_variants.keys().collect();
        sorted_keys.sort();
        for key in sorted_keys {
            let variant_text = &source_variants[key];
            // Build params map: substitute from_param with a Phrase that has the
            // variant text as its display text but retains the original tags and
            // variants. This ensures bare {$s} interpolation shows the variant
            // text while phrase calls like {subtype($s)} still receive a Phrase
            // value with full metadata for :from inheritance.
            let mut variant_params: HashMap<String, Value> = HashMap::new();
            variant_params.insert(
                from_param.to_string(),
                Value::Phrase(
                    Phrase::builder()
                        .text(variant_text.clone())
                        .tags(inherited_tags.clone())
                        .variants(source_variants.clone())
                        .build(),
                ),
            );
            for param_name in &def.parameters {
                if param_name != from_param
                    && let Some(v) = ctx.get_param(param_name)
                {
                    variant_params.insert(param_name.clone(), v.clone());
                }
            }
            let mut variant_ctx = EvalContext::with_string_context(
                &variant_params,
                ctx.string_context().map(ToString::to_string),
            );
            let variant_result = eval_match_branches(
                branches,
                &def.match_params,
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
        Ok(Phrase::builder()
            .text(default_text)
            .tags(inherited_tags)
            .build())
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

    // Handle :* (explicit default selector) — return the phrase's default text
    if selectors.iter().any(|s| matches!(s, Selector::Default)) {
        return match value {
            Value::Phrase(phrase) => Ok(Value::Phrase(
                Phrase::builder()
                    .text(phrase.text.clone())
                    .tags(phrase.tags.clone())
                    .build(),
            )),
            _ => Ok(value.clone()),
        };
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

/// Resolve a selector to candidate key strings.
///
/// Uses the AST distinction directly:
/// - `Selector::Identifier(name)` → use as a literal variant key
/// - `Selector::Parameter(name)` → look up parameter value, then resolve:
///   Number → CLDR plural category, Phrase → all tags, String → literal or parsed number
/// - `Selector::Default` → handled before this function is called (short-circuit in apply_selectors)
fn resolve_selector_candidates(
    selector: &Selector,
    ctx: &EvalContext<'_>,
    lang: &str,
) -> Result<Vec<String>, EvalError> {
    match selector {
        Selector::Identifier(name) => {
            // Static selector: use as literal key
            Ok(vec![name.clone()])
        }
        Selector::Default => {
            // Handled by apply_selectors before reaching here
            unreachable!("Selector::Default should be handled in apply_selectors")
        }
        Selector::Parameter(name) => {
            // Parameterized selector: look up parameter value
            let value = ctx
                .get_param(name)
                .ok_or_else(|| EvalError::UnknownParameter { name: name.clone() })?;
            match value {
                Value::Number(n) => Ok(vec![plural_category(lang, *n).to_string()]),
                Value::Float(f) => Ok(vec![plural_category(lang, *f as i64).to_string()]),
                Value::Phrase(phrase) => {
                    // Use all tags as candidates, preserving order
                    let tags: Vec<String> = phrase.tags.iter().map(ToString::to_string).collect();
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

        // Resolve transform context
        let context_value = resolve_transform_context(&transform.context, ctx)?;

        // Pass full Value to transform so it can read tags (on first iteration)
        let result = transform_kind.execute(&current, context_value.as_ref(), lang)?;
        // After transform, result is String - wrap for next iteration
        current = Value::String(result);
    }

    Ok(current.to_string())
}

/// Resolve a transform context to an optional Value.
///
/// Static context becomes a literal string value. Dynamic context looks up
/// the named parameter. Both produces a compound "static.dynamic" string
/// when both resolve to strings, or just the dynamic value if the static
/// part should be preserved as a prefix.
fn resolve_transform_context(
    context: &TransformContext,
    ctx: &EvalContext<'_>,
) -> Result<Option<Value>, EvalError> {
    match context {
        TransformContext::None => Ok(None),
        TransformContext::Static(name) => Ok(Some(Value::String(name.clone()))),
        TransformContext::Dynamic(name) => {
            if let Some(param) = ctx.get_param(name) {
                Ok(Some(param.clone()))
            } else {
                Err(EvalError::UnknownParameter { name: name.clone() })
            }
        }
        TransformContext::Both(static_name, dynamic_name) => {
            let dynamic_value = ctx.get_param(dynamic_name).cloned().ok_or_else(|| {
                EvalError::UnknownParameter {
                    name: dynamic_name.clone(),
                }
            })?;
            // Combine static context with dynamic value as "static.dynamic"
            // This supports patterns like @transform:lit($param) where the
            // transform needs both pieces of information
            let combined = format!("{static_name}.{}", dynamic_value);
            Ok(Some(Value::String(combined)))
        }
    }
}

/// Evaluate match branches, selecting the best-matching branch template.
///
/// Resolution per dimension:
/// 1. Exact numeric key (for Number values)
/// 2. CLDR plural category (for Number values)
/// 3. Tag-based matching (for Phrase values, first matching tag)
/// 4. Default (`*`-marked) branch
fn eval_match_branches(
    branches: &[MatchBranch],
    match_params: &[String],
    ctx: &mut EvalContext<'_>,
    registry: &PhraseRegistry,
    transform_registry: &TransformRegistry,
    lang: &str,
) -> Result<String, EvalError> {
    // Resolve each match parameter to its current value
    let mut resolved_keys: Vec<Vec<String>> = Vec::new();
    for param_name in match_params {
        let value = ctx
            .get_param(param_name)
            .ok_or_else(|| EvalError::UnknownParameter {
                name: param_name.clone(),
            })?;
        match value {
            Value::Number(n) => {
                // For numbers, try exact match first, then CLDR
                let exact = n.to_string();
                let cldr = plural_category(lang, *n).to_string();
                if exact == cldr {
                    resolved_keys.push(vec![exact]);
                } else {
                    resolved_keys.push(vec![exact, cldr]);
                }
            }
            Value::Phrase(phrase) => {
                let tags: Vec<String> = phrase.tags.iter().map(ToString::to_string).collect();
                resolved_keys.push(tags);
            }
            Value::String(s) => {
                if let Ok(n) = s.parse::<i64>() {
                    let exact = n.to_string();
                    let cldr = plural_category(lang, n).to_string();
                    if exact == cldr {
                        resolved_keys.push(vec![exact]);
                    } else {
                        resolved_keys.push(vec![exact, cldr]);
                    }
                } else {
                    resolved_keys.push(vec![s.clone()]);
                }
            }
            Value::Float(f) => {
                let cldr = plural_category(lang, *f as i64).to_string();
                resolved_keys.push(vec![cldr]);
            }
        }
    }

    // Try to find a matching branch, considering all candidate key combinations
    // and falling back to * default branches.
    let selected_template = select_match_branch(branches, &resolved_keys, match_params.len())?;
    eval_template(selected_template, ctx, registry, transform_registry, lang)
}

/// Select the best matching branch template from match branches.
///
/// Tries all combinations of resolved keys, then falls back to default branches.
fn select_match_branch<'a>(
    branches: &'a [MatchBranch],
    resolved_keys: &[Vec<String>],
    num_dims: usize,
) -> Result<&'a Template, EvalError> {
    // Build candidate compound keys from resolved keys via cartesian product
    let compound_keys = build_compound_keys(resolved_keys);

    // Try each candidate key against each branch (exact match)
    for candidate in &compound_keys {
        for branch in branches {
            for key in &branch.keys {
                if key.value == *candidate {
                    return Ok(&branch.template);
                }
            }
        }
    }

    // Try compound tag matching: a key like "masc.anim" with a single-param
    // :match means "match if the parameter has ALL of these tags". Check if all
    // dot-separated parts of a branch key are present in the resolved tags.
    for branch in branches {
        for key in &branch.keys {
            let key_parts: Vec<&str> = key.value.split('.').collect();
            if key_parts.len() <= num_dims {
                continue;
            }
            // Compound tag key: more parts than dimensions. Check that each
            // part exists in the resolved tags for this compound's dimension.
            if compound_tag_matches(&key_parts, resolved_keys, num_dims) {
                return Ok(&branch.template);
            }
        }
    }

    // Try partial matching: for each candidate, check if it matches a branch
    // when accounting for * defaults in unmatched dimensions.
    for candidate in &compound_keys {
        let candidate_parts: Vec<&str> = candidate.split('.').collect();
        for branch in branches {
            for key in &branch.keys {
                let key_parts: Vec<&str> = key.value.split('.').collect();
                if key_parts.len() != candidate_parts.len() {
                    continue;
                }
                // Check if this key matches (exact match or default dimension)
                let matches = key_parts
                    .iter()
                    .zip(candidate_parts.iter())
                    .enumerate()
                    .all(|(dim, (key_part, cand_part))| {
                        key_part == cand_part
                            || key.default_dimensions.get(dim).copied().unwrap_or(false)
                    });
                if matches {
                    return Ok(&branch.template);
                }
            }
        }
    }

    // Last resort: find the branch where all dimensions are defaults
    for branch in branches {
        for key in &branch.keys {
            if key.default_dimensions.len() == num_dims && key.default_dimensions.iter().all(|d| *d)
            {
                return Ok(&branch.template);
            }
        }
    }

    Err(EvalError::MissingMatchDefault {
        keys: resolved_keys.to_vec(),
    })
}

/// Check if a compound tag key matches resolved keys.
///
/// A compound key like `"masc.anim"` in a single-param `:match` means "match
/// if the parameter has ALL of these tags". For multi-param matches, excess
/// parts beyond `num_dims` form compound constraints on the last dimension.
fn compound_tag_matches(
    key_parts: &[&str],
    resolved_keys: &[Vec<String>],
    num_dims: usize,
) -> bool {
    // Split key_parts into per-dimension groups. The first (num_dims - 1)
    // dimensions each get one part; the last dimension gets all remaining parts.
    if key_parts.len() <= num_dims || resolved_keys.len() < num_dims {
        return false;
    }

    // Check dimensions before the last one (exact match required)
    for dim in 0..num_dims.saturating_sub(1) {
        if !resolved_keys[dim].contains(&key_parts[dim].to_string()) {
            return false;
        }
    }

    // For the last dimension, ALL remaining key parts must be present in resolved tags
    let last_dim = num_dims - 1;
    let compound_parts = &key_parts[last_dim..];
    let available_tags = &resolved_keys[last_dim];
    compound_parts
        .iter()
        .all(|part| available_tags.contains(&part.to_string()))
}

/// Build a Phrase from variant entries.
///
/// Evaluates each variant template and populates the variants HashMap.
/// Default text priority:
/// 1. String context match (if set)
/// 2. `*`-marked default variant
/// 3. First entry's text (backward compatibility)
fn build_phrase_from_variants(
    entries: &[VariantEntry],
    ctx: &mut EvalContext<'_>,
    registry: &PhraseRegistry,
    transform_registry: &TransformRegistry,
    lang: &str,
) -> Result<(String, HashMap<VariantKey, String>), EvalError> {
    let mut variants = HashMap::new();
    let mut first_text = String::new();
    let mut default_text: Option<String> = None;
    let mut context_text: Option<String> = None;

    for (i, entry) in entries.iter().enumerate() {
        let text = eval_variant_entry_body(&entry.body, ctx, registry, transform_registry, lang)?;

        // First variant's text becomes the fallback default
        if i == 0 {
            first_text = text.clone();
        }

        // *-marked variant becomes the default
        if entry.is_default && default_text.is_none() {
            default_text = Some(text.clone());
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

    // Priority: context match > *-marked default > first entry
    let result_text = context_text.or(default_text).unwrap_or(first_text);

    Ok((result_text, variants))
}
