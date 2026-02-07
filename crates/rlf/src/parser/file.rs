//! RLF file format parser.
//!
//! Parses `.rlf` files containing phrase definitions.

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

/// Parse a phrase definition: name(params)? = tags? from? body ;
fn phrase_definition(input: &mut &str) -> ModalResult<PhraseDefinition> {
    let name = snake_case_identifier(input)?;
    skip_ws_and_comments(input)?;

    // Optional parameter list
    let parameters: Vec<String> = opt(parameter_list).parse_next(input)?.unwrap_or_default();
    skip_ws_and_comments(input)?;

    // Equals sign
    '='.parse_next(input)?;
    skip_ws_and_comments(input)?;

    // Optional tags
    let tags: Vec<Tag> = repeat(0.., terminated(tag, skip_ws_and_comments)).parse_next(input)?;

    // Optional :from(param)
    let from_param: Option<String> =
        opt(terminated(from_modifier, skip_ws_and_comments)).parse_next(input)?;

    // Body (simple template or variant block)
    let body = phrase_body(input)?;
    skip_ws_and_comments(input)?;

    // Semicolon
    ';'.parse_next(input)?;

    Ok(PhraseDefinition {
        name,
        parameters,
        tags,
        from_param,
        body,
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

/// Parse a parameter list: (param1, param2, ...)
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
        // Make sure this is not a :from modifier
        *s != "from"
    })
    .map(|s: &str| Tag::new(s))
    .parse_next(input)
}

/// Parse a :from(param) modifier.
fn from_modifier(input: &mut &str) -> ModalResult<String> {
    preceded(":from", delimited('(', parameter_name, ')')).parse_next(input)
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

/// Parse a single variant entry: key1, key2: "template"
fn variant_entry(input: &mut &str) -> ModalResult<VariantEntry> {
    let keys = variant_keys(input)?;
    skip_ws_and_comments(input)?;
    ':'.parse_next(input)?;
    skip_ws_and_comments(input)?;
    let template = template_string(input)?;

    Ok(VariantEntry { keys, template })
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
                context: None,
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

/// Parse a transform: @name or @name:context
fn transform(input: &mut &str) -> ModalResult<Transform> {
    preceded(
        '@',
        (
            simple_identifier,
            opt(preceded(':', transform_context_identifier)),
        ),
    )
    .map(|(name, context)| Transform {
        name: name.to_string(),
        context: context.map(|s| Selector::Identifier(s.to_string())),
    })
    .parse_next(input)
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

/// Parse a reference argument: $param or bare term name.
fn reference_arg(input: &mut &str) -> ModalResult<Reference> {
    if input.starts_with('$') {
        let _ = '$'.parse_next(input)?;
        simple_identifier
            .map(|name| Reference::Parameter(name.to_string()))
            .parse_next(input)
    } else {
        simple_identifier
            .map(|name| Reference::Identifier(name.to_string()))
            .parse_next(input)
    }
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
