//! Parse implementations for converting TokenStream to macro AST.
//!
//! Implements syn::parse::Parse for all AST types defined in input.rs.

use std::collections::HashSet;
use std::mem;

use crate::input::{
    DefinitionKind, Interpolation, MacroInput, MatchBranch, MatchKey, PhraseBody, PhraseDefinition,
    Reference, Segment, Selector, SpannedIdent, Template, TransformContext, TransformRef,
    VariantEntry,
};
use proc_macro2::Span;
use syn::parse::{Parse, ParseStream};
use syn::token::{Brace, Paren};
use syn::{Ident, LitStr, Token};

impl Parse for MacroInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut phrases = Vec::new();
        while !input.is_empty() {
            phrases.push(input.parse()?);
        }
        Ok(MacroInput { phrases })
    }
}

/// Parsed tags, optional :from modifier, and optional :match modifier.
struct TagsFromMatch {
    tags: Vec<SpannedIdent>,
    from_param: Option<SpannedIdent>,
    match_params: Vec<SpannedIdent>,
}

/// Parse optional tags (:tag), :from(param), and :match($p1, $p2) modifiers.
///
/// :from and :match can appear in either order. Tags must come before both.
/// Stops when it encounters something that isn't `:ident` (e.g., a string
/// literal, brace block, or identifier without colon prefix).
fn parse_tags_from_match(input: ParseStream) -> syn::Result<TagsFromMatch> {
    let mut tags = Vec::new();
    let mut from_param = None;
    let mut match_params = Vec::new();

    while input.peek(Token![:]) && !input.peek2(Brace) {
        let colon_span = input.parse::<Token![:]>()?.span;

        // `match` is a Rust keyword, so syn::Ident won't parse it.
        // We check for it explicitly before trying to parse a regular Ident.
        if input.peek(Token![match]) {
            input.parse::<Token![match]>()?;
            let content;
            syn::parenthesized!(content in input);
            while !content.is_empty() {
                if content.peek(Token![$]) {
                    content.parse::<Token![$]>()?;
                } else {
                    return Err(syn::Error::new(
                        content.span(),
                        "expected '$' before parameter name in :match",
                    ));
                }
                let param: Ident = content.parse()?;
                match_params.push(SpannedIdent::new(&param));
                if content.peek(Token![,]) {
                    content.parse::<Token![,]>()?;
                }
            }
            if match_params.is_empty() {
                return Err(syn::Error::new(
                    colon_span,
                    ":match requires at least one parameter",
                ));
            }
        } else {
            let ident: Ident = input.parse()?;

            if ident == "from" {
                let content;
                syn::parenthesized!(content in input);
                // v2: require $ prefix on :from parameter
                if content.peek(Token![$]) {
                    content.parse::<Token![$]>()?;
                } else {
                    return Err(syn::Error::new(
                        content.span(),
                        "expected '$' before parameter name in :from",
                    ));
                }
                let param: Ident = content.parse()?;
                from_param = Some(SpannedIdent::new(&param));
            } else {
                tags.push(SpannedIdent::from_str(ident.to_string(), colon_span));
            }
        }
    }

    Ok(TagsFromMatch {
        tags,
        from_param,
        match_params,
    })
}

impl Parse for PhraseDefinition {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Parse phrase name
        let name_ident: Ident = input.parse()?;
        let name = SpannedIdent::new(&name_ident);

        // Parse optional parameters (v2: $-prefixed)
        let parameters = if input.peek(Paren) {
            let content;
            let paren = syn::parenthesized!(content in input);
            let mut params = Vec::new();
            while !content.is_empty() {
                // Require $ prefix
                if content.peek(Token![$]) {
                    content.parse::<Token![$]>()?;
                } else {
                    return Err(syn::Error::new(
                        content.span(),
                        "expected '$' before parameter name",
                    ));
                }
                let ident: Ident = content.parse()?;
                params.push(SpannedIdent::new(&ident));
                if content.peek(Token![,]) {
                    content.parse::<Token![,]>()?;
                }
            }

            // Empty parameter list is an error
            if params.is_empty() {
                return Err(syn::Error::new(
                    paren.span.join(),
                    "empty parameter list — use a term instead (remove the parentheses)",
                ));
            }

            params
        } else {
            Vec::new()
        };

        // Parse =
        input.parse::<Token![=]>()?;

        // Parse optional tags/from/match AFTER = sign
        let TagsFromMatch {
            tags,
            from_param,
            match_params,
        } = parse_tags_from_match(input)?;

        // Parse body: if :match was specified, parse a match block
        let body = if !match_params.is_empty() {
            parse_match_block(input, match_params.len())?
        } else {
            input.parse()?
        };

        // Parse ;
        input.parse::<Token![;]>()?;

        // Determine kind: parameters present -> Phrase, else -> Term
        let kind = if parameters.is_empty() {
            DefinitionKind::Term
        } else {
            DefinitionKind::Phrase
        };

        // Validate: phrases cannot have variant block bodies without :match
        if kind == DefinitionKind::Phrase && matches!(body, PhraseBody::Variants(_)) {
            return Err(syn::Error::new(
                name.span,
                format!(
                    "phrase '{}' cannot have a variant block — use :match($param) for branching",
                    name.name
                ),
            ));
        }

        // Validate: :from requires parameters (must be a phrase)
        if kind == DefinitionKind::Term
            && let Some(ref from) = from_param
        {
            return Err(syn::Error::new(from.span, ":from requires parameters"));
        }

        // Validate: :match requires parameters (must be a phrase)
        if kind == DefinitionKind::Term && !match_params.is_empty() {
            return Err(syn::Error::new(
                name.span,
                ":match requires parameters on definition",
            ));
        }

        // Validate: :match parameters must be declared in the phrase signature
        let param_names: HashSet<String> = parameters.iter().map(|p| p.name.clone()).collect();
        for mp in &match_params {
            if !param_names.contains(&mp.name) {
                return Err(syn::Error::new(
                    mp.span,
                    format!(
                        ":match parameter '{}' is not declared in phrase '{}' — add it to the parameter list",
                        mp.name, name.name
                    ),
                ));
            }
        }

        // Validate match defaults
        if let PhraseBody::Match(ref branches) = body {
            validate_match_defaults(&name, &match_params, branches)?;
        }

        Ok(PhraseDefinition {
            kind,
            name,
            parameters,
            tags,
            from_param,
            match_params,
            body,
        })
    }
}

impl Parse for PhraseBody {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // If starts with { it's a variant block, otherwise simple template
        if input.peek(Brace) {
            let content;
            syn::braced!(content in input);

            let mut entries = Vec::new();
            while !content.is_empty() {
                entries.push(content.parse::<VariantEntry>()?);
            }

            // Validate: at most one * per variant block
            let default_count = entries.iter().filter(|e| e.is_default).count();
            if default_count > 1 {
                let second_default = entries.iter().filter(|e| e.is_default).nth(1).unwrap();
                return Err(syn::Error::new(
                    second_default.keys[0].span,
                    "multiple '*' default markers in variant block — at most one is allowed",
                ));
            }

            Ok(PhraseBody::Variants(entries))
        } else {
            let template: Template = input.parse()?;
            Ok(PhraseBody::Simple(template))
        }
    }
}

impl Parse for VariantEntry {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Check for * default marker
        let is_default = input.peek(Token![*]);
        if is_default {
            input.parse::<Token![*]>()?;
        }

        // Parse keys (comma-separated identifiers, possibly with dots)
        // Format: key1, key2: "template"
        // Or: nom.one, nom.few: "template"
        let mut keys = Vec::new();

        loop {
            // Parse a key which may be dotted (e.g., nom.one)
            let first: Ident = input.parse()?;
            let mut key_str = first.to_string();
            let key_span = first.span();

            // Check for dot-separated parts
            while input.peek(Token![.]) {
                input.parse::<Token![.]>()?;
                let part: Ident = input.parse()?;
                key_str.push('.');
                key_str.push_str(&part.to_string());
            }

            keys.push(SpannedIdent::from_str(key_str, key_span));

            // Check for comma (more keys) or colon (end of keys)
            if input.peek(Token![,]) && !input.peek2(Token![:]) && !input.peek2(LitStr) {
                // More keys coming, but need to be careful about trailing comma before string
                let fork = input.fork();
                fork.parse::<Token![,]>().ok();
                if fork.peek(Ident) {
                    input.parse::<Token![,]>()?;
                    continue;
                }
            }
            break;
        }

        // Validate: * cannot appear on multi-dimensional keys
        if is_default && keys.iter().any(|k| k.name.contains('.')) {
            return Err(syn::Error::new(
                keys[0].span,
                format!(
                    "'*' cannot be used on multi-dimensional key '{}'",
                    keys[0].name
                ),
            ));
        }

        // Parse colon
        input.parse::<Token![:]>()?;

        // Parse template string
        let template: Template = input.parse()?;

        // Parse optional trailing comma
        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
        }

        Ok(VariantEntry {
            keys,
            template,
            is_default,
        })
    }
}

/// Parse a match block: `{ key: "template", *default: "template" }`.
fn parse_match_block(input: ParseStream, num_params: usize) -> syn::Result<PhraseBody> {
    let content;
    syn::braced!(content in input);

    let mut branches = Vec::new();
    while !content.is_empty() {
        branches.push(parse_match_entry(&content, num_params)?);
    }

    Ok(PhraseBody::Match(branches))
}

/// Parse a single match entry: `*?key1, key2: "template",?`
fn parse_match_entry(input: ParseStream, num_params: usize) -> syn::Result<MatchBranch> {
    let mut keys = Vec::new();

    loop {
        let key = parse_match_key(input, num_params)?;
        keys.push(key);

        // Check for comma (more keys) or colon (end of keys)
        if input.peek(Token![,]) && !input.peek2(Token![:]) && !input.peek2(LitStr) {
            let fork = input.fork();
            fork.parse::<Token![,]>().ok();
            // Look ahead: if the next token after comma could start a key (ident,
            // number, or *), consume the comma and continue.
            if fork.peek(Ident) || fork.peek(syn::LitInt) || fork.peek(Token![*]) {
                input.parse::<Token![,]>()?;
                continue;
            }
        }
        break;
    }

    // Parse colon
    input.parse::<Token![:]>()?;

    // Parse template string
    let template: Template = input.parse()?;

    // Parse optional trailing comma
    if input.peek(Token![,]) {
        input.parse::<Token![,]>()?;
    }

    Ok(MatchBranch { keys, template })
}

/// Parse a single match key, possibly with dot notation and `*` defaults per dimension.
///
/// Examples: `1`, `other`, `*other`, `1.masc`, `*other.*neut`
fn parse_match_key(input: ParseStream, num_params: usize) -> syn::Result<MatchKey> {
    let mut value_parts = Vec::new();
    let mut default_dims = Vec::new();

    // Parse first dimension
    let is_default = if input.peek(Token![*]) {
        input.parse::<Token![*]>()?;
        true
    } else {
        false
    };
    default_dims.push(is_default);

    let (part, span) = parse_match_key_component(input)?;
    value_parts.push(part);

    // Parse additional dimensions (dot-separated)
    while input.peek(Token![.]) {
        input.parse::<Token![.]>()?;
        let is_default = if input.peek(Token![*]) {
            input.parse::<Token![*]>()?;
            true
        } else {
            false
        };
        default_dims.push(is_default);
        let (part, _) = parse_match_key_component(input)?;
        value_parts.push(part);
    }

    // Validate dimension count matches number of match parameters
    if value_parts.len() != num_params {
        return Err(syn::Error::new(
            span,
            format!(
                "match key has {} dimension(s) but :match has {} parameter(s)",
                value_parts.len(),
                num_params
            ),
        ));
    }

    Ok(MatchKey {
        value: SpannedIdent::from_str(value_parts.join("."), span),
        default_dimensions: default_dims,
    })
}

/// Parse a single component of a match key: an identifier or integer literal.
fn parse_match_key_component(input: ParseStream) -> syn::Result<(String, Span)> {
    if input.peek(syn::LitInt) {
        let lit: syn::LitInt = input.parse()?;
        Ok((lit.to_string(), lit.span()))
    } else {
        let ident: Ident = input.parse()?;
        Ok((ident.to_string(), ident.span()))
    }
}

/// Validate that exactly one distinct `*` default value exists per dimension in match branches.
fn validate_match_defaults(
    name: &SpannedIdent,
    match_params: &[SpannedIdent],
    branches: &[MatchBranch],
) -> syn::Result<()> {
    let num_dims = match_params.len();

    // Collect distinct default values per dimension
    let mut default_values_per_dim: Vec<HashSet<String>> = vec![HashSet::new(); num_dims];

    for branch in branches {
        for key in &branch.keys {
            let parts: Vec<&str> = key.value.name.split('.').collect();
            for (dim, &is_default) in key.default_dimensions.iter().enumerate() {
                if is_default && dim < num_dims && dim < parts.len() {
                    default_values_per_dim[dim].insert(parts[dim].to_string());
                }
            }
        }
    }

    for (dim, values) in default_values_per_dim.iter().enumerate() {
        if values.is_empty() {
            return Err(syn::Error::new(
                match_params[dim].span,
                format!(
                    "no '*' default branch for :match parameter '{}' — exactly one branch must be marked with '*'",
                    match_params[dim].name
                ),
            ));
        }
        if values.len() > 1 {
            return Err(syn::Error::new(
                name.span,
                format!(
                    "multiple '*' default markers for :match parameter '{}' — exactly one is allowed",
                    match_params[dim].name
                ),
            ));
        }
    }

    Ok(())
}

impl Parse for Template {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lit: LitStr = input.parse()?;
        let span = lit.span();
        let value = lit.value();

        let segments = parse_template_string(&value, span)?;

        Ok(Template { segments, span })
    }
}

/// Parse a template string into segments.
///
/// Handles:
/// - Literal text
/// - Escaped braces: {{ and }}
/// - Interpolations: {reference}, {@transform reference}, {reference:selector}
pub(crate) fn parse_template_string(s: &str, span: Span) -> syn::Result<Vec<Segment>> {
    let mut segments = Vec::new();
    let mut current_literal = String::new();
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '{' => {
                if chars.peek() == Some(&'{') {
                    // Escaped brace
                    chars.next();
                    current_literal.push('{');
                } else {
                    // Start of interpolation
                    if !current_literal.is_empty() {
                        segments.push(Segment::Literal(mem::take(&mut current_literal)));
                    }

                    // Collect until closing brace
                    let mut interp_content = String::new();
                    let mut depth = 1;
                    // Using while-let instead of for because we break out early
                    #[expect(clippy::while_let_on_iterator)]
                    while let Some(c) = chars.next() {
                        if c == '{' {
                            depth += 1;
                            interp_content.push(c);
                        } else if c == '}' {
                            depth -= 1;
                            if depth == 0 {
                                break;
                            }
                            interp_content.push(c);
                        } else {
                            interp_content.push(c);
                        }
                    }

                    if depth != 0 {
                        return Err(syn::Error::new(span, "unclosed interpolation brace"));
                    }

                    if interp_content.is_empty() {
                        return Err(syn::Error::new(span, "empty interpolation"));
                    }

                    let interpolation = parse_interpolation(&interp_content, span)?;
                    segments.push(Segment::Interpolation(interpolation));
                }
            }
            '}' => {
                if chars.peek() == Some(&'}') {
                    // Escaped brace
                    chars.next();
                    current_literal.push('}');
                } else {
                    return Err(syn::Error::new(span, "unexpected closing brace"));
                }
            }
            // Per v2 spec: $, @, : are literal in regular text outside {}.
            // No escaping needed — they are only special inside {} expressions.
            _ => {
                current_literal.push(c);
            }
        }
    }

    if !current_literal.is_empty() {
        segments.push(Segment::Literal(current_literal));
    }

    Ok(segments)
}

/// Parse the content of an interpolation: @transforms reference :selectors
fn parse_interpolation(content: &str, span: Span) -> syn::Result<Interpolation> {
    let content = content.trim();
    let mut transforms = Vec::new();
    let mut rest = content;

    // Parse transforms (start with @, but @@ is an escape for literal @)
    while rest.starts_with('@') && !rest.starts_with("@@") {
        rest = &rest[1..]; // Skip @

        // Find end of transform name (space, colon, open paren, or end of string)
        let end = rest
            .find(|c: char| c.is_whitespace() || c == ':' || c == '(')
            .unwrap_or(rest.len());

        if end == 0 {
            return Err(syn::Error::new(span, "empty transform name after @"));
        }

        let transform_name = &rest[..end];
        rest = &rest[end..];

        // Parse optional static context (:literal)
        let static_ctx = if rest.starts_with(':') && !rest[1..].starts_with(char::is_whitespace) {
            let after_colon = &rest[1..];
            let ctx_end = after_colon
                .find(|c: char| c.is_whitespace() || c == ':' || c == '(')
                .unwrap_or(after_colon.len());

            if ctx_end > 0 {
                let ctx_name = &after_colon[..ctx_end];
                rest = &after_colon[ctx_end..];
                Some(SpannedIdent::from_str(ctx_name, span))
            } else {
                None
            }
        } else {
            None
        };

        // Parse optional dynamic context ($param)
        let dynamic_ctx = if rest.starts_with('(') {
            let after_paren = &rest[1..];
            let trimmed = after_paren.trim_start();
            if !trimmed.starts_with('$') {
                return Err(syn::Error::new(
                    span,
                    "expected '$' before parameter name in transform dynamic context",
                ));
            }
            let after_dollar = &trimmed[1..];
            let param_end = after_dollar
                .find(|c: char| !c.is_alphanumeric() && c != '_')
                .unwrap_or(after_dollar.len());
            if param_end == 0 {
                return Err(syn::Error::new(
                    span,
                    "expected parameter name after '$' in transform context",
                ));
            }
            let param_name = &after_dollar[..param_end];
            let after_param = after_dollar[param_end..].trim_start();
            if !after_param.starts_with(')') {
                return Err(syn::Error::new(
                    span,
                    "expected ')' after parameter name in transform context",
                ));
            }
            rest = &after_param[1..];
            Some(SpannedIdent::from_str(param_name, span))
        } else {
            None
        };

        rest = rest.trim_start();

        let context = match (static_ctx, dynamic_ctx) {
            (Some(s), Some(d)) => TransformContext::Both(s, d),
            (Some(s), None) => TransformContext::Static(s),
            (None, Some(d)) => TransformContext::Dynamic(d),
            (None, None) => TransformContext::None,
        };

        transforms.push(TransformRef {
            name: SpannedIdent::from_str(transform_name, span),
            context,
        });
    }

    // Parse reference (identifier or call)
    let (reference, remaining, auto_cap) = parse_reference(rest, span)?;

    // If auto-capitalization was triggered, prepend @cap transform
    if auto_cap {
        transforms.insert(
            0,
            TransformRef {
                name: SpannedIdent::from_str("cap", span),
                context: TransformContext::None,
            },
        );
    }

    // Extract selectors from the rest (they come after the reference)
    let selectors = extract_selectors(&remaining, span)?;

    Ok(Interpolation {
        transforms,
        reference,
        selectors,
        span,
    })
}

/// Parse a reference from a string, returning the Reference, auto-cap flag, and remaining content.
///
/// v2: `$name` -> Parameter reference. Bare `name` -> Identifier (term) reference.
/// Auto-capitalization: if the identifier starts with an uppercase ASCII letter,
/// the first character is lowercased and `auto_cap` is set to true. Only applies
/// to bare identifiers, not `$` parameters. The caller should prepend `@cap` to
/// the transform list.
fn parse_reference(content: &str, span: Span) -> syn::Result<(Reference, String, bool)> {
    let content = content.trim();

    if content.is_empty() {
        return Err(syn::Error::new(span, "empty reference in interpolation"));
    }

    // Check for number literal (digit start)
    if content.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        let digit_end = content
            .find(|c: char| !c.is_ascii_digit())
            .unwrap_or(content.len());
        let digits = &content[..digit_end];
        let rest = &content[digit_end..];
        let n: i64 = digits
            .parse()
            .map_err(|e| syn::Error::new(span, format!("invalid number literal: {e}")))?;
        return Ok((Reference::NumberLiteral(n, span), rest.to_string(), false));
    }

    // Check for string literal ("..." start)
    if let Some(after_quote) = content.strip_prefix('"') {
        let (s, rest) = parse_string_literal_arg(after_quote, span)?;
        return Ok((Reference::StringLiteral(s, span), rest, false));
    }

    // Check for $$ escape sequence (literal $)
    if let Some(rest) = content.strip_prefix("$$") {
        return Ok((
            Reference::Identifier(SpannedIdent::from_str("$", span)),
            rest.to_string(),
            false,
        ));
    }

    // Check for @@ escape sequence (literal @)
    if let Some(rest) = content.strip_prefix("@@") {
        return Ok((
            Reference::Identifier(SpannedIdent::from_str("@", span)),
            rest.to_string(),
            false,
        ));
    }

    // Check for :: escape sequence (literal :)
    if let Some(rest) = content.strip_prefix("::") {
        return Ok((
            Reference::Identifier(SpannedIdent::from_str(":", span)),
            rest.to_string(),
            false,
        ));
    }

    // v2: Check for $ prefix (parameter reference)
    if let Some(after_dollar) = content.strip_prefix('$') {
        let ident_end = after_dollar
            .find(|c: char| !c.is_alphanumeric() && c != '_')
            .unwrap_or(after_dollar.len());

        if ident_end == 0 {
            return Err(syn::Error::new(span, "expected parameter name after '$'"));
        }

        let ident = &after_dollar[..ident_end];
        let rest = &after_dollar[ident_end..];

        // No auto-capitalization for parameter references
        return Ok((
            Reference::Parameter(SpannedIdent::from_str(ident, span)),
            rest.to_string(),
            false,
        ));
    }

    // Find the identifier
    let ident_end = content
        .find(|c: char| !c.is_alphanumeric() && c != '_')
        .unwrap_or(content.len());

    if ident_end == 0 {
        return Err(syn::Error::new(
            span,
            format!("invalid reference: {content}"),
        ));
    }

    let ident = &content[..ident_end];
    let rest = &content[ident_end..];

    // Detect auto-capitalization: uppercase first ASCII letter (bare identifiers only)
    let first_char = ident.chars().next().unwrap();
    let auto_cap = first_char.is_ascii_uppercase();
    let actual_ident = if auto_cap {
        // Lowercase the first character and the first character after each underscore
        let mut result = String::with_capacity(ident.len());
        let mut after_underscore = true;
        for c in ident.chars() {
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
        ident.to_string()
    };

    // Check for call syntax: name(args)
    if rest.starts_with('(') {
        // Find matching closing paren
        let mut depth = 0;
        let mut end = 0;
        for (i, c) in rest.chars().enumerate() {
            match c {
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth == 0 {
                        end = i;
                        break;
                    }
                }
                _ => {}
            }
        }

        if depth != 0 {
            return Err(syn::Error::new(span, "unclosed parenthesis in phrase call"));
        }

        let args_str = &rest[1..end]; // Content between parens
        let after_call = &rest[end + 1..];

        // Parse arguments (comma-separated: $param, term name, number, or string literal)
        let mut args = Vec::new();
        if !args_str.is_empty() {
            for arg in split_args(args_str) {
                let (arg_ref, remaining, _) = parse_reference(arg.trim(), span)?;
                if !remaining.trim().is_empty() {
                    return Err(syn::Error::new(
                        span,
                        format!(
                            "expressions not supported as phrase call arguments — use a simple $param, term name, number, or string (unexpected trailing '{remaining}')",
                        ),
                    ));
                }
                args.push(arg_ref);
            }
        }

        Ok((
            Reference::Call {
                name: SpannedIdent::from_str(actual_ident, span),
                args,
            },
            after_call.to_string(),
            auto_cap,
        ))
    } else {
        Ok((
            Reference::Identifier(SpannedIdent::from_str(actual_ident, span)),
            rest.to_string(),
            auto_cap,
        ))
    }
}

/// Split argument string by commas, respecting string literals.
fn split_args(s: &str) -> Vec<&str> {
    let mut args = Vec::new();
    let mut start = 0;
    let mut in_string = false;
    let mut prev_backslash = false;

    for (i, c) in s.char_indices() {
        if in_string {
            if c == '\\' && !prev_backslash {
                prev_backslash = true;
                continue;
            }
            if c == '"' && !prev_backslash {
                in_string = false;
            }
            prev_backslash = false;
        } else if c == '"' {
            in_string = true;
        } else if c == ',' {
            args.push(&s[start..i]);
            start = i + 1;
        }
    }

    args.push(&s[start..]);
    args
}

/// Parse a string literal argument from content after the opening `"`.
///
/// Returns the parsed string and the remaining content after the closing `"`.
fn parse_string_literal_arg(content: &str, span: Span) -> syn::Result<(String, String)> {
    let mut result = String::new();
    let mut chars = content.chars();
    let mut byte_offset = 0;

    loop {
        let Some(c) = chars.next() else {
            return Err(syn::Error::new(span, "unclosed string literal in argument"));
        };
        byte_offset += c.len_utf8();
        match c {
            '"' => break,
            '\\' => {
                let Some(escaped) = chars.next() else {
                    return Err(syn::Error::new(
                        span,
                        "unexpected end of string after backslash",
                    ));
                };
                byte_offset += escaped.len_utf8();
                match escaped {
                    '"' => result.push('"'),
                    '\\' => result.push('\\'),
                    other => {
                        result.push('\\');
                        result.push(other);
                    }
                }
            }
            other => result.push(other),
        }
    }

    let rest = &content[byte_offset..];
    Ok((result, rest.to_string()))
}

/// Extract selectors from the remaining content after a reference.
///
/// v2: After `:`, if `$` -> `Selector::Parameter`, otherwise -> `Selector::Literal`.
fn extract_selectors(content: &str, span: Span) -> syn::Result<Vec<Selector>> {
    let mut selectors = Vec::new();
    let mut rest = content.trim();

    while rest.starts_with(':') && !rest.starts_with("::") {
        rest = &rest[1..]; // Skip :

        // v2: Check for $ prefix (parameterized selector)
        if rest.starts_with('$') {
            rest = &rest[1..]; // Skip $
            let end = rest
                .find(|c: char| c.is_whitespace() || c == ':')
                .unwrap_or(rest.len());

            if end == 0 {
                return Err(syn::Error::new(span, "expected parameter name after ':$'"));
            }

            let selector_name = &rest[..end];
            selectors.push(Selector::Parameter(SpannedIdent::from_str(
                selector_name,
                span,
            )));
            rest = rest[end..].trim_start();
        } else {
            // Literal selector
            let end = rest
                .find(|c: char| c.is_whitespace() || c == ':')
                .unwrap_or(rest.len());

            if end == 0 {
                return Err(syn::Error::new(span, "empty selector after :"));
            }

            let selector_name = &rest[..end];
            selectors.push(Selector::Literal(SpannedIdent::from_str(
                selector_name,
                span,
            )));
            rest = rest[end..].trim_start();
        }
    }

    if !rest.is_empty() {
        return Err(syn::Error::new(
            span,
            format!("unexpected content after selectors: {rest}"),
        ));
    }

    Ok(selectors)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper to assert segment count and get segments.
    fn parse_ok(s: &str) -> Vec<Segment> {
        parse_template_string(s, Span::call_site()).expect("should parse successfully")
    }

    /// Helper to assert parse failure.
    fn parse_err(s: &str) -> syn::Error {
        parse_template_string(s, Span::call_site()).expect_err("should fail to parse")
    }

    fn count_literals(segments: &[Segment]) -> usize {
        segments
            .iter()
            .filter(|s| matches!(s, Segment::Literal(_)))
            .count()
    }

    fn count_interpolations(segments: &[Segment]) -> usize {
        segments
            .iter()
            .filter(|s| matches!(s, Segment::Interpolation(_)))
            .count()
    }

    fn get_literal(segment: &Segment) -> &str {
        match segment {
            Segment::Literal(s) => s,
            Segment::Interpolation(_) => panic!("expected Literal segment"),
        }
    }

    fn get_interpolation(segment: &Segment) -> &Interpolation {
        match segment {
            Segment::Interpolation(i) => i,
            Segment::Literal(_) => panic!("expected Interpolation segment"),
        }
    }

    /// Helper to extract the name from a Selector.
    fn selector_name(sel: &Selector) -> &str {
        match sel {
            Selector::Literal(ident) => &ident.name,
            Selector::Parameter(ident) => &ident.name,
        }
    }

    // =========================================================================
    // Literal-only templates
    // =========================================================================

    #[test]
    fn test_literal_only_simple() {
        let segments = parse_ok("hello");
        assert_eq!(segments.len(), 1);
        assert_eq!(count_literals(&segments), 1);
        assert_eq!(get_literal(&segments[0]), "hello");
    }

    #[test]
    fn test_literal_with_spaces() {
        let segments = parse_ok("hello world");
        assert_eq!(segments.len(), 1);
        assert_eq!(get_literal(&segments[0]), "hello world");
    }

    #[test]
    fn test_empty_string() {
        let segments = parse_ok("");
        assert!(segments.is_empty());
    }

    // =========================================================================
    // Escape sequences
    // =========================================================================

    #[test]
    fn test_escaped_open_brace() {
        let segments = parse_ok("{{escaped}}");
        assert_eq!(segments.len(), 1);
        assert_eq!(get_literal(&segments[0]), "{escaped}");
    }

    #[test]
    fn test_escaped_braces_mixed() {
        let segments = parse_ok("a {{b}} c");
        assert_eq!(segments.len(), 1);
        assert_eq!(get_literal(&segments[0]), "a {b} c");
    }

    #[test]
    fn test_at_literal_in_text() {
        // v2: @ is literal in regular text, no escaping needed
        let segments = parse_ok("user@example.com");
        assert_eq!(segments.len(), 1);
        assert_eq!(get_literal(&segments[0]), "user@example.com");
    }

    #[test]
    fn test_colon_literal_in_text() {
        // v2: : is literal in regular text, no escaping needed
        let segments = parse_ok("ratio 1:2");
        assert_eq!(segments.len(), 1);
        assert_eq!(get_literal(&segments[0]), "ratio 1:2");
    }

    #[test]
    fn test_dollar_literal_in_text() {
        // v2: $ is literal in regular text, no escaping needed
        let segments = parse_ok("The cost is $5.");
        assert_eq!(segments.len(), 1);
        assert_eq!(get_literal(&segments[0]), "The cost is $5.");
    }

    #[test]
    fn test_double_at_literal_in_text() {
        // v2: @@ in text produces two @ characters (both are literal)
        let segments = parse_ok("Use @@ for transforms.");
        assert_eq!(segments.len(), 1);
        assert_eq!(get_literal(&segments[0]), "Use @@ for transforms.");
    }

    #[test]
    fn test_double_colon_literal_in_text() {
        // v2: :: in text produces two : characters (both are literal)
        let segments = parse_ok("Ratio 1::2.");
        assert_eq!(segments.len(), 1);
        assert_eq!(get_literal(&segments[0]), "Ratio 1::2.");
    }

    #[test]
    fn test_double_dollar_literal_in_text() {
        // v2: $$ in text produces two $ characters (both are literal)
        let segments = parse_ok("Use $$ for literal dollar.");
        assert_eq!(segments.len(), 1);
        assert_eq!(get_literal(&segments[0]), "Use $$ for literal dollar.");
    }

    #[test]
    fn test_dollar_escape_in_interpolation() {
        // v2: $$ inside {} produces literal $
        let segments = parse_ok("{$$}");
        assert_eq!(segments.len(), 1);
        let interp = get_interpolation(&segments[0]);
        assert!(matches!(&interp.reference, Reference::Identifier(ident) if ident.name == "$"));
    }

    #[test]
    fn test_at_escape_in_interpolation() {
        // v2: @@ inside {} produces literal @
        let segments = parse_ok("{@@}");
        assert_eq!(segments.len(), 1);
        let interp = get_interpolation(&segments[0]);
        assert!(matches!(&interp.reference, Reference::Identifier(ident) if ident.name == "@"));
    }

    #[test]
    fn test_colon_escape_in_interpolation() {
        // v2: :: inside {} produces literal :
        let segments = parse_ok("{::}");
        assert_eq!(segments.len(), 1);
        let interp = get_interpolation(&segments[0]);
        assert!(matches!(&interp.reference, Reference::Identifier(ident) if ident.name == ":"));
    }

    #[test]
    fn test_escaped_braces_with_dollar_param() {
        // v2: "Use {{$name}} for params" -> "Use {$name} for params"
        let segments = parse_ok("Use {{$name}} for params");
        assert_eq!(segments.len(), 1);
        assert_eq!(get_literal(&segments[0]), "Use {$name} for params");
    }

    // =========================================================================
    // Single interpolation
    // =========================================================================

    #[test]
    fn test_single_identifier_interpolation() {
        let segments = parse_ok("{name}");
        assert_eq!(segments.len(), 1);
        assert_eq!(count_interpolations(&segments), 1);

        let interp = get_interpolation(&segments[0]);
        assert!(interp.transforms.is_empty());
        assert!(interp.selectors.is_empty());
        assert!(matches!(&interp.reference, Reference::Identifier(ident) if ident.name == "name"));
    }

    #[test]
    fn test_single_parameter_interpolation() {
        let segments = parse_ok("{$name}");
        assert_eq!(segments.len(), 1);
        assert_eq!(count_interpolations(&segments), 1);

        let interp = get_interpolation(&segments[0]);
        assert!(interp.transforms.is_empty());
        assert!(interp.selectors.is_empty());
        assert!(matches!(&interp.reference, Reference::Parameter(ident) if ident.name == "name"));
    }

    // =========================================================================
    // Mixed literal and interpolation
    // =========================================================================

    #[test]
    fn test_mixed_literal_interpolation() {
        let segments = parse_ok("Hello, {$name}!");
        assert_eq!(segments.len(), 3);
        assert_eq!(count_literals(&segments), 2);
        assert_eq!(count_interpolations(&segments), 1);

        assert_eq!(get_literal(&segments[0]), "Hello, ");
        let interp = get_interpolation(&segments[1]);
        assert!(matches!(&interp.reference, Reference::Parameter(ident) if ident.name == "name"));
        assert_eq!(get_literal(&segments[2]), "!");
    }

    // =========================================================================
    // Multiple interpolations
    // =========================================================================

    #[test]
    fn test_multiple_interpolations() {
        let segments = parse_ok("{a} and {b}");
        assert_eq!(segments.len(), 3);
        assert_eq!(count_interpolations(&segments), 2);

        let interp1 = get_interpolation(&segments[0]);
        assert!(matches!(&interp1.reference, Reference::Identifier(ident) if ident.name == "a"));

        assert_eq!(get_literal(&segments[1]), " and ");

        let interp2 = get_interpolation(&segments[2]);
        assert!(matches!(&interp2.reference, Reference::Identifier(ident) if ident.name == "b"));
    }

    #[test]
    fn test_adjacent_interpolations() {
        let segments = parse_ok("{a}{b}");
        assert_eq!(segments.len(), 2);
        assert_eq!(count_interpolations(&segments), 2);
    }

    // =========================================================================
    // Transforms
    // =========================================================================

    #[test]
    fn test_transform_cap() {
        let segments = parse_ok("{@cap name}");
        assert_eq!(segments.len(), 1);

        let interp = get_interpolation(&segments[0]);
        assert_eq!(interp.transforms.len(), 1);
        assert_eq!(interp.transforms[0].name.name, "cap");
        assert!(matches!(
            interp.transforms[0].context,
            TransformContext::None
        ));
    }

    #[test]
    fn test_transform_on_parameter() {
        let segments = parse_ok("{@cap $name}");
        assert_eq!(segments.len(), 1);

        let interp = get_interpolation(&segments[0]);
        assert_eq!(interp.transforms.len(), 1);
        assert_eq!(interp.transforms[0].name.name, "cap");
        assert!(matches!(&interp.reference, Reference::Parameter(ident) if ident.name == "name"));
    }

    #[test]
    fn test_multiple_transforms() {
        let segments = parse_ok("{@cap @upper name}");
        assert_eq!(segments.len(), 1);

        let interp = get_interpolation(&segments[0]);
        assert_eq!(interp.transforms.len(), 2);
        assert_eq!(interp.transforms[0].name.name, "cap");
        assert_eq!(interp.transforms[1].name.name, "upper");
    }

    #[test]
    fn test_transform_with_context() {
        let segments = parse_ok("{@der:acc item}");
        assert_eq!(segments.len(), 1);

        let interp = get_interpolation(&segments[0]);
        assert_eq!(interp.transforms.len(), 1);
        assert_eq!(interp.transforms[0].name.name, "der");

        assert!(
            matches!(&interp.transforms[0].context, TransformContext::Static(ident) if ident.name == "acc")
        );
    }

    #[test]
    fn test_transform_with_dynamic_context() {
        let segments = parse_ok("{@count($n) card}");
        assert_eq!(segments.len(), 1);

        let interp = get_interpolation(&segments[0]);
        assert_eq!(interp.transforms.len(), 1);
        assert_eq!(interp.transforms[0].name.name, "count");
        assert!(
            matches!(&interp.transforms[0].context, TransformContext::Dynamic(ident) if ident.name == "n")
        );
        assert!(matches!(&interp.reference, Reference::Identifier(ident) if ident.name == "card"));
    }

    #[test]
    fn test_transform_with_both_contexts() {
        let segments = parse_ok("{@transform:lit($param) ref}");
        assert_eq!(segments.len(), 1);

        let interp = get_interpolation(&segments[0]);
        assert_eq!(interp.transforms.len(), 1);
        assert_eq!(interp.transforms[0].name.name, "transform");
        assert!(matches!(
            &interp.transforms[0].context,
            TransformContext::Both(s, d) if s.name == "lit" && d.name == "param"
        ));
    }

    // =========================================================================
    // Selectors
    // =========================================================================

    #[test]
    fn test_single_literal_selector() {
        let segments = parse_ok("{noun:case}");
        assert_eq!(segments.len(), 1);

        let interp = get_interpolation(&segments[0]);
        assert_eq!(interp.selectors.len(), 1);
        assert!(matches!(&interp.selectors[0], Selector::Literal(ident) if ident.name == "case"));
    }

    #[test]
    fn test_multiple_literal_selectors() {
        let segments = parse_ok("{noun:case:number}");
        assert_eq!(segments.len(), 1);

        let interp = get_interpolation(&segments[0]);
        assert_eq!(interp.selectors.len(), 2);
        assert_eq!(selector_name(&interp.selectors[0]), "case");
        assert_eq!(selector_name(&interp.selectors[1]), "number");
    }

    #[test]
    fn test_parameter_selector() {
        let segments = parse_ok("{card:$n}");
        assert_eq!(segments.len(), 1);

        let interp = get_interpolation(&segments[0]);
        assert_eq!(interp.selectors.len(), 1);
        assert!(matches!(&interp.selectors[0], Selector::Parameter(ident) if ident.name == "n"));
    }

    #[test]
    fn test_mixed_selectors() {
        let segments = parse_ok("{card:acc:$n}");
        assert_eq!(segments.len(), 1);

        let interp = get_interpolation(&segments[0]);
        assert_eq!(interp.selectors.len(), 2);
        assert!(matches!(&interp.selectors[0], Selector::Literal(ident) if ident.name == "acc"));
        assert!(matches!(&interp.selectors[1], Selector::Parameter(ident) if ident.name == "n"));
    }

    #[test]
    fn test_parameter_with_literal_selectors() {
        let segments = parse_ok("{$base:nom:one}");
        assert_eq!(segments.len(), 1);

        let interp = get_interpolation(&segments[0]);
        assert!(matches!(&interp.reference, Reference::Parameter(ident) if ident.name == "base"));
        assert_eq!(interp.selectors.len(), 2);
        assert_eq!(selector_name(&interp.selectors[0]), "nom");
        assert_eq!(selector_name(&interp.selectors[1]), "one");
    }

    // =========================================================================
    // Phrase calls
    // =========================================================================

    #[test]
    fn test_phrase_call_with_term_arg() {
        let segments = parse_ok("{foo(bar)}");
        assert_eq!(segments.len(), 1);

        let interp = get_interpolation(&segments[0]);
        match &interp.reference {
            Reference::Call { name, args } => {
                assert_eq!(name.name, "foo");
                assert_eq!(args.len(), 1);
                assert!(matches!(&args[0], Reference::Identifier(ident) if ident.name == "bar"));
            }
            _ => panic!("expected Call reference"),
        }
    }

    #[test]
    fn test_phrase_call_with_param_arg() {
        let segments = parse_ok("{foo($bar)}");
        assert_eq!(segments.len(), 1);

        let interp = get_interpolation(&segments[0]);
        match &interp.reference {
            Reference::Call { name, args } => {
                assert_eq!(name.name, "foo");
                assert_eq!(args.len(), 1);
                assert!(matches!(&args[0], Reference::Parameter(ident) if ident.name == "bar"));
            }
            _ => panic!("expected Call reference"),
        }
    }

    #[test]
    fn test_phrase_call_mixed_args() {
        let segments = parse_ok("{foo($a, term, $b)}");
        assert_eq!(segments.len(), 1);

        let interp = get_interpolation(&segments[0]);
        match &interp.reference {
            Reference::Call { name, args } => {
                assert_eq!(name.name, "foo");
                assert_eq!(args.len(), 3);
                assert!(matches!(&args[0], Reference::Parameter(ident) if ident.name == "a"));
                assert!(matches!(&args[1], Reference::Identifier(ident) if ident.name == "term"));
                assert!(matches!(&args[2], Reference::Parameter(ident) if ident.name == "b"));
            }
            _ => panic!("expected Call reference"),
        }
    }

    #[test]
    fn test_phrase_call_with_selectors() {
        let segments = parse_ok("{subtype($s):other}");
        assert_eq!(segments.len(), 1);

        let interp = get_interpolation(&segments[0]);
        match &interp.reference {
            Reference::Call { name, args } => {
                assert_eq!(name.name, "subtype");
                assert_eq!(args.len(), 1);
                assert!(matches!(&args[0], Reference::Parameter(ident) if ident.name == "s"));
            }
            _ => panic!("expected Call reference"),
        }
        assert_eq!(interp.selectors.len(), 1);
        assert!(matches!(&interp.selectors[0], Selector::Literal(ident) if ident.name == "other"));
    }

    #[test]
    fn test_nested_phrase_call() {
        let segments = parse_ok("{foo(bar(baz))}");
        assert_eq!(segments.len(), 1);

        let interp = get_interpolation(&segments[0]);
        match &interp.reference {
            Reference::Call { name, args } => {
                assert_eq!(name.name, "foo");
                assert_eq!(args.len(), 1);
                match &args[0] {
                    Reference::Call {
                        name: inner,
                        args: inner_args,
                    } => {
                        assert_eq!(inner.name, "bar");
                        assert_eq!(inner_args.len(), 1);
                        assert!(matches!(
                            &inner_args[0],
                            Reference::Identifier(ident) if ident.name == "baz"
                        ));
                    }
                    _ => panic!("expected nested Call"),
                }
            }
            _ => panic!("expected Call reference"),
        }
    }

    #[test]
    fn test_phrase_call_multiple_args() {
        let segments = parse_ok("{foo(a, b, c)}");
        assert_eq!(segments.len(), 1);

        let interp = get_interpolation(&segments[0]);
        match &interp.reference {
            Reference::Call { name, args } => {
                assert_eq!(name.name, "foo");
                assert_eq!(args.len(), 3);
            }
            _ => panic!("expected Call reference"),
        }
    }

    // =========================================================================
    // Error cases
    // =========================================================================

    #[test]
    fn test_error_unclosed_brace() {
        let err = parse_err("{name");
        assert!(err.to_string().contains("unclosed"));
    }

    #[test]
    fn test_error_empty_interpolation() {
        let err = parse_err("{}");
        assert!(err.to_string().contains("empty"));
    }

    #[test]
    fn test_error_unexpected_closing_brace() {
        let err = parse_err("name}");
        assert!(err.to_string().contains("unexpected"));
    }

    #[test]
    fn test_error_dollar_without_name() {
        let err = parse_err("{$}");
        assert!(err.to_string().contains("expected parameter name"));
    }

    #[test]
    fn test_error_expression_in_call_arg() {
        let err = parse_err("{f(card:one)}");
        assert!(
            err.to_string()
                .contains("expressions not supported as phrase call arguments")
        );
    }

    #[test]
    fn test_error_expression_in_call_arg_parameter_selector() {
        let err = parse_err("{f($n:one)}");
        assert!(
            err.to_string()
                .contains("expressions not supported as phrase call arguments")
        );
    }

    // =========================================================================
    // Complex cases
    // =========================================================================

    #[test]
    fn test_transform_and_selectors() {
        let segments = parse_ok("{@cap noun:case}");
        assert_eq!(segments.len(), 1);

        let interp = get_interpolation(&segments[0]);
        assert_eq!(interp.transforms.len(), 1);
        assert_eq!(interp.transforms[0].name.name, "cap");
        assert_eq!(interp.selectors.len(), 1);
        assert_eq!(selector_name(&interp.selectors[0]), "case");
    }

    #[test]
    fn test_full_v2_syntax() {
        // v2 realistic template: "Draw {$n} {@cap card:$n}."
        let segments = parse_ok("Draw {$n} {@cap card:$n}.");
        assert_eq!(segments.len(), 5);
        assert_eq!(get_literal(&segments[0]), "Draw ");
        assert_eq!(get_literal(&segments[4]), ".");

        let interp1 = get_interpolation(&segments[1]);
        assert!(matches!(&interp1.reference, Reference::Parameter(ident) if ident.name == "n"));

        assert_eq!(get_literal(&segments[2]), " ");

        let interp2 = get_interpolation(&segments[3]);
        assert_eq!(interp2.transforms.len(), 1);
        assert_eq!(interp2.transforms[0].name.name, "cap");
        assert!(matches!(&interp2.reference, Reference::Identifier(ident) if ident.name == "card"));
        assert_eq!(interp2.selectors.len(), 1);
        assert!(matches!(&interp2.selectors[0], Selector::Parameter(ident) if ident.name == "n"));
    }

    #[test]
    fn test_full_v1_compat_syntax() {
        // v1-style template (bare names are identifiers): "Draw {n} {@cap card:n}."
        let segments = parse_ok("Draw {n} {@cap card:n}.");
        assert_eq!(segments.len(), 5);

        let interp1 = get_interpolation(&segments[1]);
        assert!(matches!(&interp1.reference, Reference::Identifier(ident) if ident.name == "n"));

        let interp2 = get_interpolation(&segments[3]);
        assert_eq!(interp2.selectors.len(), 1);
        assert!(matches!(&interp2.selectors[0], Selector::Literal(ident) if ident.name == "n"));
    }
}
