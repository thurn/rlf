//! RLF file format parser.
//!
//! Parses `.rlf` files containing phrase definitions.

use std::collections::HashSet;

use super::ast::*;
use super::error::ParseError;
use crate::types::Tag;
use winnow::combinator::{alt, delimited, opt, preceded, repeat, separated, terminated};
use winnow::error::{ContextError, ErrMode};
use winnow::prelude::*;
use winnow::token::{any, none_of, take_while};

/// Parse an entire .rlf file into phrase definitions.
pub fn parse_file(input: &str) -> Result<Vec<PhraseDefinition>, ParseError> {
    let mut remaining = input;
    match file(&mut remaining) {
        Ok(phrases) => {
            // Skip any trailing whitespace/comments
            let _ = skip_ws_and_comments(&mut remaining);
            if remaining.is_empty() {
                // Validate term/phrase restrictions
                validate_definitions(&phrases)?;
                Ok(phrases)
            } else {
                let (line, column) = calculate_position(input, remaining);
                Err(ParseError::Syntax {
                    line,
                    column,
                    message: format!(
                        "unexpected character: '{}'",
                        remaining.chars().next().unwrap_or('?')
                    ),
                })
            }
        }
        Err(e) => {
            let (line, column) = calculate_position(input, remaining);
            Err(ParseError::Syntax {
                line,
                column,
                message: format!("parse error: {}", e),
            })
        }
    }
}

/// Validate term/phrase restrictions on parsed definitions.
fn validate_definitions(definitions: &[PhraseDefinition]) -> Result<(), ParseError> {
    for def in definitions {
        // Empty parameter list: name() = ... should use a term instead
        if def.has_empty_parens {
            return Err(ParseError::Syntax {
                line: 0,
                column: 0,
                message: format!(
                    "empty parameter list on '{}' — use a term instead (remove the parentheses)",
                    def.name
                ),
            });
        }

        // Phrases cannot have variant block bodies — use :match for branching
        if def.kind == DefinitionKind::Phrase && matches!(def.body, PhraseBody::Variants(_)) {
            return Err(ParseError::Syntax {
                line: 0,
                column: 0,
                message: format!(
                    "phrase '{}' cannot have a variant block — use :match($param) for branching",
                    def.name
                ),
            });
        }

        // :from requires parameters (must be a phrase)
        if def.kind == DefinitionKind::Term && def.from_param.is_some() {
            return Err(ParseError::Syntax {
                line: 0,
                column: 0,
                message: format!(":from requires parameters on definition '{}'", def.name),
            });
        }

        // :match requires parameters (must be a phrase)
        if def.kind == DefinitionKind::Term && !def.match_params.is_empty() {
            return Err(ParseError::Syntax {
                line: 0,
                column: 0,
                message: format!(":match requires parameters on definition '{}'", def.name),
            });
        }

        // :match parameters must be declared in the phrase signature
        for mp in &def.match_params {
            if !def.parameters.contains(mp) {
                return Err(ParseError::Syntax {
                    line: 0,
                    column: 0,
                    message: format!(
                        ":match parameter '{}' is not declared in phrase '{}' — add it to the parameter list",
                        mp, def.name
                    ),
                });
            }
        }

        // Validate * default markers and numeric keys in variant blocks
        if let PhraseBody::Variants(entries) = &def.body {
            let mut default_count = 0;
            for entry in entries {
                if entry.is_default {
                    default_count += 1;

                    // * cannot appear on multi-dimensional keys (keys containing a dot)
                    if entry.keys.iter().any(|k| k.contains('.')) {
                        return Err(ParseError::Syntax {
                            line: 0,
                            column: 0,
                            message: format!(
                                "'*' cannot be used on multi-dimensional key '{}' in definition '{}'",
                                entry.keys.first().unwrap_or(&String::new()),
                                def.name
                            ),
                        });
                    }
                }

                // Numeric keys in term variant blocks are not allowed
                if def.kind == DefinitionKind::Term {
                    for key in &entry.keys {
                        for component in key.split('.') {
                            if component.chars().next().is_some_and(|c| c.is_ascii_digit()) {
                                return Err(ParseError::Syntax {
                                    line: 0,
                                    column: 0,
                                    message: format!(
                                        "term variant keys must be named identifiers — use ':match' for numeric branching (found '{}' in '{}')",
                                        key, def.name
                                    ),
                                });
                            }
                        }
                    }
                }
            }

            // At most one * per variant block
            if default_count > 1 {
                return Err(ParseError::Syntax {
                    line: 0,
                    column: 0,
                    message: format!(
                        "multiple '*' default markers in variant block of '{}' — at most one is allowed",
                        def.name
                    ),
                });
            }
        }

        // Validate * default markers in match blocks
        if let PhraseBody::Match(branches) = &def.body {
            validate_match_defaults(def, branches)?;
        }
    }
    Ok(())
}

/// Validate that exactly one distinct `*` default value exists per dimension in a match block.
fn validate_match_defaults(
    def: &PhraseDefinition,
    branches: &[MatchBranch],
) -> Result<(), ParseError> {
    let num_dims = def.match_params.len();

    // Collect distinct default values per dimension
    let mut default_values_per_dim: Vec<HashSet<String>> = vec![HashSet::new(); num_dims];

    for branch in branches {
        for key in &branch.keys {
            let parts: Vec<&str> = key.value.split('.').collect();
            for (dim, &is_default) in key.default_dimensions.iter().enumerate() {
                if is_default && dim < num_dims && dim < parts.len() {
                    default_values_per_dim[dim].insert(parts[dim].to_string());
                }
            }
        }
    }

    // Validate: exactly one distinct default value per dimension
    for (dim, values) in default_values_per_dim.iter().enumerate() {
        if values.is_empty() {
            let param_name = &def.match_params[dim];
            return Err(ParseError::Syntax {
                line: 0,
                column: 0,
                message: format!(
                    "no '*' default branch for :match parameter '{}' in phrase '{}' — exactly one branch must be marked with '*'",
                    param_name, def.name
                ),
            });
        }
        if values.len() > 1 {
            let param_name = &def.match_params[dim];
            return Err(ParseError::Syntax {
                line: 0,
                column: 0,
                message: format!(
                    "multiple '*' default values for :match parameter '{}' in phrase '{}' — exactly one is allowed (found: {})",
                    param_name,
                    def.name,
                    values.iter().cloned().collect::<Vec<_>>().join(", ")
                ),
            });
        }
    }

    Ok(())
}

/// Calculate line and column from original input and remaining input.
fn calculate_position(original: &str, remaining: &str) -> (usize, usize) {
    let consumed = original.len() - remaining.len();
    let consumed_str = &original[..consumed];
    let line = consumed_str.chars().filter(|&c| c == '\n').count() + 1;
    let last_newline = consumed_str.rfind('\n');
    let column = match last_newline {
        Some(pos) => consumed - pos,
        None => consumed + 1,
    };
    (line, column)
}

/// Parse an entire file into phrase definitions.
fn file(input: &mut &str) -> ModalResult<Vec<PhraseDefinition>> {
    skip_ws_and_comments(input)?;
    let phrases: Vec<PhraseDefinition> =
        repeat(0.., terminated(phrase_definition, skip_ws_and_comments)).parse_next(input)?;
    Ok(phrases)
}

/// Skip whitespace and line comments.
fn skip_ws_and_comments(input: &mut &str) -> ModalResult<()> {
    let _: Vec<()> = repeat(0.., alt((ws_only.void(), line_comment.void()))).parse_next(input)?;
    Ok(())
}

/// Parse whitespace (no comments).
fn ws_only<'i>(input: &mut &'i str) -> ModalResult<&'i str> {
    take_while(1.., |c: char| c.is_ascii_whitespace()).parse_next(input)
}

/// Parse a line comment: // ... newline
fn line_comment<'i>(input: &mut &'i str) -> ModalResult<&'i str> {
    preceded("//", take_while(0.., |c| c != '\n')).parse_next(input)
}

/// Parse a phrase definition: name(params)? = tags? from? match? body ;
fn phrase_definition(input: &mut &str) -> ModalResult<PhraseDefinition> {
    let name = snake_case_identifier(input)?;
    skip_ws_and_comments(input)?;

    // Optional parameter list — track whether parens were present
    let parsed_params: Option<Vec<String>> = opt(parameter_list).parse_next(input)?;
    let has_empty_parens = matches!(&parsed_params, Some(v) if v.is_empty());
    let parameters: Vec<String> = parsed_params.unwrap_or_default();
    skip_ws_and_comments(input)?;

    // Equals sign
    '='.parse_next(input)?;
    skip_ws_and_comments(input)?;

    // Optional tags
    let tags: Vec<Tag> = repeat(0.., terminated(tag, skip_ws_and_comments)).parse_next(input)?;

    // Parse :from and :match in either order
    let mut from_param: Option<String> = None;
    let mut match_params: Vec<String> = Vec::new();

    // Try :from first, then :match, then :from again (covers both orders)
    if let Some(fp) = opt(terminated(from_modifier, skip_ws_and_comments)).parse_next(input)? {
        from_param = Some(fp);
    }
    if let Some(mp) = opt(terminated(match_modifier, skip_ws_and_comments)).parse_next(input)? {
        match_params = mp;
    }
    if from_param.is_none()
        && let Some(fp) = opt(terminated(from_modifier, skip_ws_and_comments)).parse_next(input)?
    {
        from_param = Some(fp);
    }

    // Body: if :match was specified, parse a match block; otherwise simple or variant block
    let body = if !match_params.is_empty() {
        match_block(match_params.len())
            .map(PhraseBody::Match)
            .parse_next(input)?
    } else {
        phrase_body(input)?
    };
    skip_ws_and_comments(input)?;

    // Semicolon
    ';'.parse_next(input)?;

    // Determine kind: parameters present -> Phrase, else -> Term
    let kind = if parameters.is_empty() {
        DefinitionKind::Term
    } else {
        DefinitionKind::Phrase
    };

    Ok(PhraseDefinition {
        kind,
        name,
        parameters,
        tags,
        from_param,
        match_params,
        body,
        has_empty_parens,
    })
}

/// Parse a snake_case identifier (lowercase start, alphanumeric + underscore).
fn snake_case_identifier(input: &mut &str) -> ModalResult<String> {
    let ident: &str =
        take_while(1.., |c: char| c.is_ascii_alphanumeric() || c == '_').parse_next(input)?;

    // Validate: must start with lowercase letter
    let first = ident.chars().next().unwrap();
    if !first.is_ascii_lowercase() {
        return Err(ErrMode::Backtrack(ContextError::new()));
    }

    Ok(ident.to_string())
}

/// Parse a parameter list: ($param1, $param2, ...)
fn parameter_list(input: &mut &str) -> ModalResult<Vec<String>> {
    delimited(
        '(',
        separated(
            0..,
            preceded(skip_ws_and_comments, parameter_name),
            (skip_ws_and_comments, ',', skip_ws_and_comments),
        ),
        preceded(skip_ws_and_comments, ')'),
    )
    .parse_next(input)
}

/// Parse a parameter name with required `$` prefix (e.g., `$name`).
/// Returns the name without the `$` prefix.
fn parameter_name(input: &mut &str) -> ModalResult<String> {
    preceded(
        '$',
        take_while(1.., |c: char| c.is_ascii_alphanumeric() || c == '_'),
    )
    .map(|s: &str| s.to_string())
    .parse_next(input)
}

/// Parse a tag: :name
fn tag(input: &mut &str) -> ModalResult<Tag> {
    preceded(
        ':',
        take_while(1.., |c: char| c.is_ascii_alphanumeric() || c == '_'),
    )
    .verify(|s: &&str| {
        // Make sure this is not a :from or :match modifier
        *s != "from" && *s != "match"
    })
    .map(|s: &str| Tag::new(s))
    .parse_next(input)
}

/// Parse a :from(param) modifier.
fn from_modifier(input: &mut &str) -> ModalResult<String> {
    preceded(":from", delimited('(', parameter_name, ')')).parse_next(input)
}

/// Parse a :match($p1, $p2, ...) modifier.
fn match_modifier(input: &mut &str) -> ModalResult<Vec<String>> {
    preceded(
        ":match",
        delimited(
            '(',
            separated(
                1..,
                preceded(skip_ws_and_comments, parameter_name),
                (skip_ws_and_comments, ',', skip_ws_and_comments),
            ),
            preceded(skip_ws_and_comments, ')'),
        ),
    )
    .parse_next(input)
}

/// Parse a match block: { key: "template", *default: "template" }
fn match_block(num_params: usize) -> impl FnMut(&mut &str) -> ModalResult<Vec<MatchBranch>> {
    move |input: &mut &str| {
        delimited(
            ('{', skip_ws_and_comments),
            |input: &mut &str| match_entries(input, num_params),
            (skip_ws_and_comments, '}'),
        )
        .parse_next(input)
    }
}

/// Parse match entries with trailing comma support.
fn match_entries(input: &mut &str, num_params: usize) -> ModalResult<Vec<MatchBranch>> {
    let entries: Vec<MatchBranch> = separated(
        0..,
        |input: &mut &str| match_entry(input, num_params),
        (skip_ws_and_comments, ',', skip_ws_and_comments),
    )
    .parse_next(input)?;

    // Allow trailing comma
    let _ = opt((skip_ws_and_comments, ',')).parse_next(input)?;

    Ok(entries)
}

/// Parse a single match entry: *?key1, key2: "template"
fn match_entry(input: &mut &str, num_params: usize) -> ModalResult<MatchBranch> {
    let keys = match_keys(input, num_params)?;
    skip_ws_and_comments(input)?;
    ':'.parse_next(input)?;
    skip_ws_and_comments(input)?;
    let template = template_string(input)?;

    Ok(MatchBranch { keys, template })
}

/// Parse match keys: key1, key2, key3 (before the colon).
fn match_keys(input: &mut &str, num_params: usize) -> ModalResult<Vec<MatchKey>> {
    separated(
        1..,
        |input: &mut &str| match_key(input, num_params),
        (skip_ws_and_comments, ',', skip_ws_and_comments),
    )
    .parse_next(input)
}

/// Parse a single match key, possibly with dot notation and `*` defaults per dimension.
///
/// Examples: `1`, `other`, `*other`, `1.masc`, `*other.*neut`
fn match_key(input: &mut &str, num_params: usize) -> ModalResult<MatchKey> {
    let mut value_parts = Vec::new();
    let mut default_dims = Vec::new();

    // Parse first dimension
    let is_default = opt('*').parse_next(input)?.is_some();
    default_dims.push(is_default);
    let part = match_key_component(input)?;
    value_parts.push(part);

    // Parse additional dimensions (dot-separated)
    while input.starts_with('.') {
        let _ = '.'.parse_next(input)?;
        let is_default = opt('*').parse_next(input)?.is_some();
        default_dims.push(is_default);
        let part = match_key_component(input)?;
        value_parts.push(part);
    }

    // Validate dimension count matches number of match parameters
    if value_parts.len() != num_params {
        return Err(ErrMode::Backtrack(ContextError::new()));
    }

    Ok(MatchKey {
        value: value_parts.join("."),
        default_dimensions: default_dims,
    })
}

/// Parse a single component of a match key (identifier or number).
fn match_key_component(input: &mut &str) -> ModalResult<String> {
    take_while(1.., |c: char| c.is_ascii_alphanumeric() || c == '_')
        .map(|s: &str| s.to_string())
        .parse_next(input)
}

/// Parse a phrase body: simple template or variant block.
fn phrase_body(input: &mut &str) -> ModalResult<PhraseBody> {
    alt((
        variant_block.map(PhraseBody::Variants),
        template_string.map(PhraseBody::Simple),
    ))
    .parse_next(input)
}

/// Parse a variant block: { key: "template", ... }
fn variant_block(input: &mut &str) -> ModalResult<Vec<VariantEntry>> {
    delimited(
        ('{', skip_ws_and_comments),
        variant_entries,
        (skip_ws_and_comments, '}'),
    )
    .parse_next(input)
}

/// Parse variant entries with trailing comma support.
fn variant_entries(input: &mut &str) -> ModalResult<Vec<VariantEntry>> {
    // Parse entries separated by commas, allow trailing comma
    let entries: Vec<VariantEntry> = separated(
        0..,
        variant_entry,
        (skip_ws_and_comments, ',', skip_ws_and_comments),
    )
    .parse_next(input)?;

    // Allow trailing comma
    let _ = opt((skip_ws_and_comments, ',')).parse_next(input)?;

    Ok(entries)
}

/// Parse a single variant entry: *? key1, key2: "template"
fn variant_entry(input: &mut &str) -> ModalResult<VariantEntry> {
    // Check for * default marker
    let is_default = opt('*').parse_next(input)?.is_some();

    let keys = variant_keys(input)?;
    skip_ws_and_comments(input)?;
    ':'.parse_next(input)?;
    skip_ws_and_comments(input)?;
    let template = template_string(input)?;

    Ok(VariantEntry {
        keys,
        template,
        is_default,
    })
}

/// Parse variant keys: key1, key2, key3 (before the colon).
fn variant_keys(input: &mut &str) -> ModalResult<Vec<String>> {
    separated(
        1..,
        variant_key,
        (skip_ws_and_comments, ',', skip_ws_and_comments),
    )
    .parse_next(input)
}

/// Parse a single variant key (may include dots for multi-dimensional: nom.one).
fn variant_key(input: &mut &str) -> ModalResult<String> {
    take_while(1.., |c: char| {
        c.is_ascii_alphanumeric() || c == '_' || c == '.'
    })
    .map(|s: &str| s.to_string())
    .parse_next(input)
}

/// Parse a template string: "content"
fn template_string(input: &mut &str) -> ModalResult<Template> {
    delimited('"', template_content, '"').parse_next(input)
}

/// Parse the content of a template string.
fn template_content(input: &mut &str) -> ModalResult<Template> {
    let segments: Vec<Segment> = repeat(0.., template_segment).parse_next(input)?;
    let merged = merge_literals(segments);
    Ok(Template { segments: merged })
}

/// Parse a single template segment.
fn template_segment(input: &mut &str) -> ModalResult<Segment> {
    alt((escape_sequence, interpolation, template_literal_char)).parse_next(input)
}

/// Parse escape sequences in templates: {{ }}
fn escape_sequence(input: &mut &str) -> ModalResult<Segment> {
    alt((
        "{{".value(Segment::Literal("{".to_string())),
        "}}".value(Segment::Literal("}".to_string())),
    ))
    .parse_next(input)
}

/// Parse a literal character in a template (not { } or ").
fn template_literal_char(input: &mut &str) -> ModalResult<Segment> {
    none_of(['{', '}', '"'])
        .map(|c: char| Segment::Literal(c.to_string()))
        .parse_next(input)
}

/// Parse an interpolation: { ... }
fn interpolation(input: &mut &str) -> ModalResult<Segment> {
    delimited('{', interpolation_content, '}').parse_next(input)
}

/// A parsed reference with an auto-capitalization flag.
struct ParsedReference {
    reference: Reference,
    auto_cap: bool,
}

/// Parse interpolation content.
fn interpolation_content(input: &mut &str) -> ModalResult<Segment> {
    ws(input)?;
    let mut transforms: Vec<Transform> =
        repeat(0.., terminated(transform, ws)).parse_next(input)?;
    let parsed_ref = reference(input)?;
    let selectors: Vec<Selector> = repeat(0.., selector).parse_next(input)?;
    ws(input)?;

    // If auto-capitalization was triggered, prepend @cap transform
    if parsed_ref.auto_cap {
        transforms.insert(
            0,
            Transform {
                name: "cap".to_string(),
                context: TransformContext::None,
            },
        );
    }

    Ok(Segment::Interpolation {
        transforms,
        reference: parsed_ref.reference,
        selectors,
    })
}

/// Parse whitespace within interpolations.
fn ws(input: &mut &str) -> ModalResult<()> {
    take_while(0.., |c: char| c.is_ascii_whitespace())
        .void()
        .parse_next(input)
}

/// Parse a transform: @name, @name:context, @name($param), or @name:context($param)
fn transform(input: &mut &str) -> ModalResult<Transform> {
    let _ = '@'.parse_next(input)?;
    let name: &str = simple_identifier(input)?;

    // Parse optional static context (:literal)
    let static_ctx =
        if input.starts_with(':') && !input[1..].starts_with(|c: char| c.is_whitespace()) {
            let _ = ':'.parse_next(input)?;
            Some(transform_context_identifier(input)?.to_string())
        } else {
            None
        };

    // Parse optional dynamic context ($param)
    let dynamic_ctx = if input.starts_with('(') {
        let _ = '('.parse_next(input)?;
        ws(input)?;
        let _ = '$'.parse_next(input)?;
        let param: &str = simple_identifier(input)?;
        ws(input)?;
        let _ = ')'.parse_next(input)?;
        Some(param.to_string())
    } else {
        None
    };

    let context = match (static_ctx, dynamic_ctx) {
        (Some(s), Some(d)) => TransformContext::Both(s, d),
        (Some(s), None) => TransformContext::Static(s),
        (None, Some(d)) => TransformContext::Dynamic(d),
        (None, None) => TransformContext::None,
    };

    Ok(Transform {
        name: name.to_string(),
        context,
    })
}

/// Parse a transform context identifier (alphanumeric, underscores, and dots).
///
/// Dots are allowed in transform contexts for compound selectors like
/// `@der:acc.other` (case + plural) or `@inflect:abl.poss1sg.pl` (suffix chains).
fn transform_context_identifier<'i>(input: &mut &'i str) -> ModalResult<&'i str> {
    take_while(1.., |c: char| {
        c.is_ascii_alphanumeric() || c == '_' || c == '.'
    })
    .parse_next(input)
}

/// Parse a reference in an interpolation.
///
/// Returns the reference and a flag indicating if auto-capitalization should
/// be applied (uppercase first letter triggers @cap, only for bare identifiers).
fn reference(input: &mut &str) -> ModalResult<ParsedReference> {
    // Check for $$ escape sequence (literal $)
    if input.starts_with("$$") {
        let _ = "$$".parse_next(input)?;
        return Ok(ParsedReference {
            reference: Reference::Identifier("$".to_string()),
            auto_cap: false,
        });
    }

    // Check for $ prefix (parameter reference)
    if input.starts_with('$') {
        let _ = '$'.parse_next(input)?;
        let name: &str = simple_identifier(input)?;
        return Ok(ParsedReference {
            reference: Reference::Parameter(name.to_string()),
            auto_cap: false,
        });
    }

    let first_char = any.parse_next(input)?;

    if !is_ident_start(first_char) {
        return Err(ErrMode::Backtrack(ContextError::new()));
    }

    let rest: &str = take_while(0.., is_ident_cont).parse_next(input)?;
    let mut name = String::with_capacity(1 + rest.len());
    name.push(first_char);
    name.push_str(rest);

    // Check for auto-capitalization: if first letter is uppercase, add @cap
    // Auto-capitalization only applies to bare identifiers, never to $-prefixed parameters
    let auto_cap = first_char.is_ascii_uppercase();
    let actual_name = if auto_cap {
        // Lowercase the first character and the first character after each underscore
        let mut result = String::with_capacity(name.len());
        let mut after_underscore = true;
        for c in name.chars() {
            if c == '_' {
                result.push(c);
                after_underscore = true;
            } else if after_underscore {
                result.push(c.to_ascii_lowercase());
                after_underscore = false;
            } else {
                result.push(c);
                after_underscore = false;
            }
        }
        result
    } else {
        name
    };

    // Check for phrase call: identifier(args)
    let args_opt: Option<Vec<Reference>> = opt(phrase_call_args).parse_next(input)?;

    let reference = match args_opt {
        Some(args) => Reference::PhraseCall {
            name: actual_name,
            args,
        },
        None => Reference::Identifier(actual_name),
    };

    Ok(ParsedReference {
        reference,
        auto_cap,
    })
}

/// Parse phrase call arguments.
fn phrase_call_args(input: &mut &str) -> ModalResult<Vec<Reference>> {
    delimited(
        ('(', ws),
        separated(0.., reference_arg, (ws, ',', ws)),
        (ws, ')'),
    )
    .parse_next(input)
}

/// Parse a reference argument: $param, bare term name, number literal, or string literal.
fn reference_arg(input: &mut &str) -> ModalResult<Reference> {
    if input.starts_with('$') {
        let _ = '$'.parse_next(input)?;
        simple_identifier
            .map(|name| Reference::Parameter(name.to_string()))
            .parse_next(input)
    } else if input.starts_with('"') {
        string_literal_arg(input)
    } else if input.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        number_literal_arg(input)
    } else {
        simple_identifier
            .map(|name| Reference::Identifier(name.to_string()))
            .parse_next(input)
    }
}

/// Parse a number literal argument: sequence of digits.
fn number_literal_arg(input: &mut &str) -> ModalResult<Reference> {
    let digits: &str = take_while(1.., |c: char| c.is_ascii_digit()).parse_next(input)?;
    let n: i64 = digits
        .parse()
        .map_err(|_| ErrMode::Backtrack(ContextError::new()))?;
    Ok(Reference::NumberLiteral(n))
}

/// Parse a string literal argument: "text" with escape support for \" and \\.
fn string_literal_arg(input: &mut &str) -> ModalResult<Reference> {
    let _ = '"'.parse_next(input)?;
    let mut result = String::new();
    loop {
        match any.parse_next(input)? {
            '"' => break,
            '\\' => {
                let escaped = any.parse_next(input)?;
                match escaped {
                    '"' => result.push('"'),
                    '\\' => result.push('\\'),
                    other => {
                        result.push('\\');
                        result.push(other);
                    }
                }
            }
            c => result.push(c),
        }
    }
    Ok(Reference::StringLiteral(result))
}

/// Parse a selector: :identifier or :$param
fn selector(input: &mut &str) -> ModalResult<Selector> {
    ':'.parse_next(input)?;
    if input.starts_with('$') {
        let _ = '$'.parse_next(input)?;
        selector_identifier
            .map(|s| Selector::Parameter(s.to_string()))
            .parse_next(input)
    } else {
        selector_identifier
            .map(|s| Selector::Identifier(s.to_string()))
            .parse_next(input)
    }
}

/// Parse a selector identifier.
fn selector_identifier<'i>(input: &mut &'i str) -> ModalResult<&'i str> {
    take_while(1.., |c: char| c.is_ascii_alphanumeric() || c == '_').parse_next(input)
}

/// Parse a simple identifier.
fn simple_identifier<'i>(input: &mut &'i str) -> ModalResult<&'i str> {
    take_while(1.., |c: char| c.is_ascii_alphanumeric() || c == '_').parse_next(input)
}

/// Check if a character can start an identifier.
fn is_ident_start(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_'
}

/// Check if a character can continue an identifier.
fn is_ident_cont(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

/// Merge adjacent literal segments.
fn merge_literals(segments: Vec<Segment>) -> Vec<Segment> {
    let mut result = Vec::with_capacity(segments.len());

    for segment in segments {
        match segment {
            Segment::Literal(text) => {
                if let Some(Segment::Literal(prev)) = result.last_mut() {
                    prev.push_str(&text);
                } else {
                    result.push(Segment::Literal(text));
                }
            }
            other @ Segment::Interpolation { .. } => result.push(other),
        }
    }

    result
}
