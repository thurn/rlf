//! Code generation for the rlf! macro.
//!
//! Transforms validated MacroInput into Rust code that provides:
//! - Typed functions for each phrase
//! - SOURCE_PHRASES const with embedded phrase definitions
//! - register_source_phrases() function for loading
//! - phrase_ids module with PhraseId constants

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::input::{
    Interpolation, MacroInput, PhraseBody, PhraseDefinition, Reference, Segment, Selector,
    Template, TransformContext, TransformRef,
};

/// Main code generation entry point.
///
/// Produces TokenStream containing:
/// - Phrase functions with typed parameters
/// - SOURCE_PHRASES const
/// - register_source_phrases() function
/// - phrase_ids module
pub fn codegen(input: &MacroInput) -> TokenStream {
    let functions = generate_functions(input);
    let source_phrases = generate_source_phrases(input);
    let phrase_ids_module = generate_phrase_ids_module(input);

    quote! {
        #functions
        #source_phrases
        #phrase_ids_module
    }
}

// =============================================================================
// Phrase Function Generation
// =============================================================================

/// Generate all phrase functions.
fn generate_functions(input: &MacroInput) -> TokenStream {
    let functions: Vec<TokenStream> = input.phrases.iter().map(generate_function).collect();

    quote! {
        #(#functions)*
    }
}

/// Generate a single phrase function.
///
/// With `global-locale`: no `locale` parameter, uses global locale.
/// Without: `pub fn name(locale: &::rlf::Locale, ...) -> ::rlf::Phrase`
fn generate_function(phrase: &PhraseDefinition) -> TokenStream {
    if cfg!(feature = "global-locale") {
        generate_function_global(phrase)
    } else {
        generate_function_explicit(phrase)
    }
}

/// Generate a phrase function that takes an explicit `locale` parameter.
fn generate_function_explicit(phrase: &PhraseDefinition) -> TokenStream {
    let fn_name = format_ident!("{}", phrase.name.name);
    let phrase_name = &phrase.name.name;
    let doc = format!("Returns the \"{}\" phrase.", phrase_name);

    if phrase.parameters.is_empty() {
        quote! {
            #[doc = #doc]
            pub fn #fn_name(locale: &::rlf::Locale) -> ::rlf::Phrase {
                locale.get_phrase(#phrase_name)
                    .expect(concat!("phrase '", #phrase_name, "' should exist"))
            }
        }
    } else {
        let param_names: Vec<_> = phrase
            .parameters
            .iter()
            .map(|p| format_ident!("{}", p.name))
            .collect();

        let param_decls: Vec<TokenStream> = param_names
            .iter()
            .map(|name| quote! { #name: impl Into<::rlf::Value> })
            .collect();

        let param_conversions: Vec<TokenStream> = param_names
            .iter()
            .map(|name| quote! { #name.into() })
            .collect();

        quote! {
            #[doc = #doc]
            pub fn #fn_name(locale: &::rlf::Locale, #(#param_decls),*) -> ::rlf::Phrase {
                locale.call_phrase(#phrase_name, &[#(#param_conversions),*])
                    .expect(concat!("phrase '", #phrase_name, "' should exist"))
            }
        }
    }
}

/// Generate a phrase function that uses the global locale (no `locale` parameter).
fn generate_function_global(phrase: &PhraseDefinition) -> TokenStream {
    let fn_name = format_ident!("{}", phrase.name.name);
    let phrase_name = &phrase.name.name;
    let doc = format!("Returns the \"{}\" phrase.", phrase_name);

    if phrase.parameters.is_empty() {
        quote! {
            #[doc = #doc]
            pub fn #fn_name() -> ::rlf::Phrase {
                __RLF_REGISTER.call_once(|| {
                    ::rlf::with_locale_mut(|locale| {
                        locale.load_translations_str("en", SOURCE_PHRASES)
                            .expect("source phrases should parse successfully");
                    });
                });
                ::rlf::with_locale(|locale| {
                    locale.get_phrase(#phrase_name)
                        .expect(concat!("phrase '", #phrase_name, "' should exist"))
                })
            }
        }
    } else {
        let param_names: Vec<_> = phrase
            .parameters
            .iter()
            .map(|p| format_ident!("{}", p.name))
            .collect();

        let param_decls: Vec<TokenStream> = param_names
            .iter()
            .map(|name| quote! { #name: impl Into<::rlf::Value> })
            .collect();

        let param_conversions: Vec<TokenStream> = param_names
            .iter()
            .map(|name| quote! { #name.into() })
            .collect();

        quote! {
            #[doc = #doc]
            pub fn #fn_name(#(#param_decls),*) -> ::rlf::Phrase {
                __RLF_REGISTER.call_once(|| {
                    ::rlf::with_locale_mut(|locale| {
                        locale.load_translations_str("en", SOURCE_PHRASES)
                            .expect("source phrases should parse successfully");
                    });
                });
                ::rlf::with_locale(|locale| {
                    locale.call_phrase(#phrase_name, &[#(#param_conversions),*])
                        .expect(concat!("phrase '", #phrase_name, "' should exist"))
                })
            }
        }
    }
}

// =============================================================================
// SOURCE_PHRASES and register_source_phrases Generation
// =============================================================================

/// Generate SOURCE_PHRASES const and register_source_phrases function.
fn generate_source_phrases(input: &MacroInput) -> TokenStream {
    let source = reconstruct_source(input);

    if cfg!(feature = "global-locale") {
        quote! {
            /// Source language phrases embedded as data.
            const SOURCE_PHRASES: &str = #source;

            static __RLF_REGISTER: ::std::sync::Once = ::std::sync::Once::new();

            /// Registers source language phrases with the global locale.
            ///
            /// This is called automatically on first use of any phrase function,
            /// but can be called explicitly to ensure registration is complete.
            pub fn register_source_phrases() {
                __RLF_REGISTER.call_once(|| {
                    ::rlf::with_locale_mut(|locale| {
                        locale.load_translations_str("en", SOURCE_PHRASES)
                            .expect("source phrases should parse successfully");
                    });
                });
            }
        }
    } else {
        quote! {
            /// Source language phrases embedded as data.
            /// Parsed by the interpreter at runtime.
            const SOURCE_PHRASES: &str = #source;

            /// Registers source language phrases with the locale.
            /// Call once at startup before using phrase functions.
            ///
            /// # Example
            ///
            /// ```ignore
            /// let mut locale = Locale::new();
            /// register_source_phrases(&mut locale);
            /// ```
            pub fn register_source_phrases(locale: &mut ::rlf::Locale) {
                locale.load_translations_str("en", SOURCE_PHRASES)
                    .expect("source phrases should parse successfully");
            }
        }
    }
}

/// Reconstruct RLF source from MacroInput.
///
/// This recreates the phrase definitions in v2 RLF syntax for the interpreter.
/// The file format expects: `name($params)? = tags? from? body ;`
fn reconstruct_source(input: &MacroInput) -> String {
    let mut lines = Vec::new();

    for phrase in &input.phrases {
        let mut line = String::new();

        // Name
        line.push_str(&phrase.name.name);

        // Parameters (v2: $-prefixed)
        if !phrase.parameters.is_empty() {
            line.push('(');
            let params: Vec<_> = phrase
                .parameters
                .iter()
                .map(|p| format!("${}", p.name))
                .collect();
            line.push_str(&params.join(", "));
            line.push(')');
        }

        line.push_str(" = ");

        // Tags (after = sign, before body)
        for tag in &phrase.tags {
            line.push(':');
            line.push_str(&tag.name);
            line.push(' ');
        }

        // :from modifier (v2: $-prefixed parameter)
        if let Some(ref from) = phrase.from_param {
            line.push_str(":from($");
            line.push_str(&from.name);
            line.push_str(") ");
        }

        // Body
        match &phrase.body {
            PhraseBody::Simple(template) => {
                line.push('"');
                line.push_str(&reconstruct_template(template));
                line.push('"');
            }
            PhraseBody::Variants(variants) => {
                line.push_str("{ ");
                let variant_strs: Vec<String> = variants
                    .iter()
                    .map(|v| {
                        let keys: Vec<_> = v.keys.iter().map(|k| k.name.as_str()).collect();
                        format!(
                            "{}: \"{}\"",
                            keys.join(", "),
                            reconstruct_template(&v.template)
                        )
                    })
                    .collect();
                line.push_str(&variant_strs.join(", "));
                line.push_str(" }");
            }
        }

        line.push(';');
        lines.push(line);
    }

    lines.join("\n")
}

/// Reconstruct a template string from Template AST.
fn reconstruct_template(template: &Template) -> String {
    let mut result = String::new();

    for segment in &template.segments {
        match segment {
            Segment::Literal(text) => {
                // In v2, only braces need escaping in template text.
                // `@`, `:`, and `$` are literal outside interpolations.
                let escaped = text
                    .replace('{', "{{")
                    .replace('}', "}}")
                    .replace('\\', "\\\\")
                    .replace('"', "\\\"");
                result.push_str(&escaped);
            }
            Segment::Interpolation(interp) => {
                result.push_str(&reconstruct_interpolation(interp));
            }
        }
    }

    result
}

/// Reconstruct an interpolation from Interpolation AST.
fn reconstruct_interpolation(interp: &Interpolation) -> String {
    let mut result = String::from("{");

    // Transforms
    for transform in &interp.transforms {
        result.push_str(&reconstruct_transform(transform));
    }

    // Reference (v2: $-prefix for parameters)
    result.push_str(&reconstruct_reference(&interp.reference));

    // Selectors (v2: already typed as Literal/Parameter)
    for selector in &interp.selectors {
        result.push(':');
        match selector {
            Selector::Literal(ident) => result.push_str(&ident.name),
            Selector::Parameter(ident) => {
                result.push('$');
                result.push_str(&ident.name);
            }
        }
    }

    result.push('}');
    result
}

/// Reconstruct a transform reference.
fn reconstruct_transform(transform: &TransformRef) -> String {
    let mut result = String::from("@");
    result.push_str(&transform.name.name);

    // Handle context: static (:literal), dynamic ($param), or both
    match &transform.context {
        TransformContext::None => {}
        TransformContext::Static(ident) => {
            result.push(':');
            result.push_str(&ident.name);
        }
        TransformContext::Dynamic(ident) => {
            result.push_str("($");
            result.push_str(&ident.name);
            result.push(')');
        }
        TransformContext::Both(static_ident, dynamic_ident) => {
            result.push(':');
            result.push_str(&static_ident.name);
            result.push_str("($");
            result.push_str(&dynamic_ident.name);
            result.push(')');
        }
    }

    result.push(' ');
    result
}

/// Reconstruct a reference (identifier, parameter, call, or literal).
fn reconstruct_reference(reference: &Reference) -> String {
    match reference {
        Reference::Identifier(ident) => ident.name.clone(),
        Reference::Parameter(ident) => format!("${}", ident.name),
        Reference::Call { name, args } => {
            let arg_strs: Vec<String> = args.iter().map(reconstruct_reference).collect();
            format!("{}({})", name.name, arg_strs.join(", "))
        }
        Reference::NumberLiteral(n, _) => n.to_string(),
        Reference::StringLiteral(s, _) => {
            let escaped = s.replace('\\', "\\\\").replace('"', "\\\"");
            format!("\"{escaped}\"")
        }
    }
}

// =============================================================================
// phrase_ids Module Generation
// =============================================================================

/// Generate the phrase_ids module with PhraseId constants.
fn generate_phrase_ids_module(input: &MacroInput) -> TokenStream {
    let constants: Vec<TokenStream> = input
        .phrases
        .iter()
        .map(generate_phrase_id_constant)
        .collect();

    quote! {
        /// PhraseId constants for all defined phrases.
        ///
        /// Use these for serializable phrase references:
        /// ```ignore
        /// use crate::strings::phrase_ids;
        ///
        /// let id = phrase_ids::CARD;
        /// let phrase = id.resolve(&locale)?;
        /// ```
        pub mod phrase_ids {
            #(#constants)*
        }
    }
}

/// Generate a single PhraseId constant.
fn generate_phrase_id_constant(phrase: &PhraseDefinition) -> TokenStream {
    let phrase_name = &phrase.name.name;

    // Convert to SCREAMING_CASE
    let const_name = format_ident!("{}", to_screaming_case(phrase_name));

    // Doc comment with parameter info if applicable
    let doc = if phrase.parameters.is_empty() {
        format!("ID for the \"{}\" phrase.", phrase_name)
    } else {
        let param_names: Vec<_> = phrase.parameters.iter().map(|p| p.name.as_str()).collect();
        format!(
            "ID for the \"{}\" phrase. Call with {} argument(s) ({}).",
            phrase_name,
            phrase.parameters.len(),
            param_names.join(", ")
        )
    };

    quote! {
        #[doc = #doc]
        pub const #const_name: ::rlf::PhraseId = ::rlf::PhraseId::from_name(#phrase_name);
    }
}

/// Convert snake_case to SCREAMING_CASE.
///
/// Examples:
/// - "card" -> "CARD"
/// - "fire_elemental" -> "FIRE_ELEMENTAL"
/// - "drawCards" -> "DRAWCARDS"
fn to_screaming_case(s: &str) -> String {
    s.to_uppercase()
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
    // to_screaming_case tests
    // =========================================================================

    #[test]
    fn test_screaming_case_lowercase() {
        assert_eq!(to_screaming_case("hello"), "HELLO");
    }

    #[test]
    fn test_screaming_case_snake() {
        assert_eq!(to_screaming_case("fire_elemental"), "FIRE_ELEMENTAL");
    }

    #[test]
    fn test_screaming_case_already_upper() {
        assert_eq!(to_screaming_case("HELLO"), "HELLO");
    }

    #[test]
    fn test_screaming_case_mixed() {
        assert_eq!(to_screaming_case("drawCards"), "DRAWCARDS");
    }

    #[test]
    fn test_screaming_case_with_numbers() {
        assert_eq!(to_screaming_case("item1"), "ITEM1");
    }

    // =========================================================================
    // reconstruct_template tests
    // =========================================================================

    #[test]
    fn test_reconstruct_literal_escapes_braces() {
        let input = parse_input(parse_quote! {
            test = "{{literal}}";
        });
        let source = reconstruct_source(&input);
        // The reconstruction should escape braces
        assert!(source.contains("{{"));
        assert!(source.contains("}}"));
    }

    #[test]
    fn test_reconstruct_literal_escapes_quotes() {
        let input = parse_input(parse_quote! {
            test = "say \"hello\"";
        });
        let source = reconstruct_source(&input);
        assert!(source.contains("\\\""));
    }

    #[test]
    fn test_reconstruct_parameter_interpolation() {
        let input = parse_input(parse_quote! {
            greet($name) = "Hello, {$name}!";
        });
        let source = reconstruct_source(&input);
        assert!(
            source.contains("{$name}"),
            "v2: parameters should have $ prefix, got: {source}"
        );
    }

    #[test]
    fn test_reconstruct_transform() {
        let input = parse_input(parse_quote! {
            hello = "hello";
            greeting = "{@cap hello}";
        });
        let source = reconstruct_source(&input);
        assert!(source.contains("@cap"));
    }

    #[test]
    fn test_reconstruct_parameter_selectors() {
        let input = parse_input(parse_quote! {
            card = { one: "card", other: "cards" };
            draw($n) = "Draw {card:$n}.";
        });
        let source = reconstruct_source(&input);
        assert!(
            source.contains("{card:$n}"),
            "v2: parameter selectors should have $ prefix, got: {source}"
        );
    }

    #[test]
    fn test_reconstruct_dynamic_context() {
        let input = parse_input(parse_quote! {
            card = "card";
            draw($n) = "抽{@count($n) card}";
        });
        let source = reconstruct_source(&input);
        assert!(
            source.contains("@count($n)"),
            "dynamic context should use () syntax, got: {source}"
        );
    }

    #[test]
    fn test_reconstruct_static_context() {
        let input = parse_input(parse_quote! {
            karte = "Karte";
            destroy = "Zerstöre {@der:acc karte}.";
        });
        let source = reconstruct_source(&input);
        assert!(
            source.contains("@der:acc"),
            "static context should use : syntax, got: {source}"
        );
    }

    #[test]
    fn test_reconstruct_both_contexts() {
        let input = parse_input(parse_quote! {
            ref_term = "ref";
            test($param) = "{@transform:lit($param) ref_term}";
        });
        let source = reconstruct_source(&input);
        assert!(
            source.contains("@transform:lit($param)"),
            "both contexts should combine : and () syntax, got: {source}"
        );
    }

    // =========================================================================
    // reconstruct_source tests
    // =========================================================================

    #[test]
    fn test_reconstruct_simple_phrase() {
        let input = parse_input(parse_quote! {
            hello = "world";
        });
        let source = reconstruct_source(&input);
        assert!(source.contains("hello = \"world\""));
    }

    #[test]
    fn test_reconstruct_phrase_with_params() {
        let input = parse_input(parse_quote! {
            greet($name, $title) = "Hello, {$title} {$name}!";
        });
        let source = reconstruct_source(&input);
        assert!(
            source.contains("greet($name, $title)"),
            "v2: parameter declarations should have $ prefix, got: {source}"
        );
    }

    #[test]
    fn test_reconstruct_phrase_with_variants() {
        let input = parse_input(parse_quote! {
            card = { one: "card", other: "cards" };
        });
        let source = reconstruct_source(&input);
        assert!(source.contains("card = {"));
        assert!(source.contains("one: \"card\""));
        assert!(source.contains("other: \"cards\""));
    }

    #[test]
    fn test_reconstruct_phrase_with_tags() {
        let input = parse_input(parse_quote! {
            item = :masc "item";
        });
        let source = reconstruct_source(&input);
        // Tags should come after = sign in the file format
        assert!(source.contains("item = :masc \"item\""));
    }

    #[test]
    fn test_reconstruct_phrase_with_from_modifier() {
        let input = parse_input(parse_quote! {
            subtype($s) = :from($s) "<b>{$s}</b>";
        });
        let source = reconstruct_source(&input);
        assert!(
            source.contains(":from($s)"),
            "v2: :from parameter should have $ prefix, got: {source}",
        );
        assert!(
            source.contains("subtype($s)"),
            "v2: parameter declaration should have $ prefix, got: {source}"
        );
    }

    #[test]
    fn test_reconstruct_phrase_with_tags_and_from() {
        let input = parse_input(parse_quote! {
            subtype($s) = :an :from($s) "<b>{$s}</b>";
        });
        let source = reconstruct_source(&input);
        assert!(
            source.contains(":an"),
            "source should contain :an tag, got: {source}",
        );
        assert!(
            source.contains(":from($s)"),
            "v2: :from parameter should have $ prefix, got: {source}",
        );
    }

    #[test]
    fn test_reconstruct_multiple_phrases() {
        let input = parse_input(parse_quote! {
            a = "first";
            b = "second";
            c = "third";
        });
        let source = reconstruct_source(&input);
        assert!(source.contains("a = \"first\""));
        assert!(source.contains("b = \"second\""));
        assert!(source.contains("c = \"third\""));
    }

    #[test]
    fn test_reconstruct_literal_at_sign_in_text() {
        let input = parse_input(parse_quote! {
            test = "user@example.com";
        });
        let source = reconstruct_source(&input);
        // v2: @ is literal in text — preserved as-is in round-trip
        assert!(
            source.contains("user@example.com"),
            "v2: @ should be literal in text, got: {source}"
        );
    }

    #[test]
    fn test_reconstruct_literal_colon_in_text() {
        let input = parse_input(parse_quote! {
            test = "Ratio 1:2.";
        });
        let source = reconstruct_source(&input);
        // v2: : is literal in text — preserved as-is in round-trip
        assert!(
            source.contains("Ratio 1:2."),
            "v2: : should be literal in text, got: {source}"
        );
    }

    // =========================================================================
    // codegen integration tests
    // =========================================================================

    #[test]
    fn test_codegen_produces_tokens() {
        let input = parse_input(parse_quote! {
            hello = "Hello, world!";
        });
        let tokens = codegen(&input);
        let output = tokens.to_string();

        // Should produce function
        assert!(output.contains("fn hello"));
        // Should produce SOURCE_PHRASES
        assert!(output.contains("SOURCE_PHRASES"));
        // Should produce register_source_phrases
        assert!(output.contains("register_source_phrases"));
        // Should produce phrase_ids module
        assert!(output.contains("mod phrase_ids"));
        // Should produce HELLO constant
        assert!(output.contains("HELLO"));
    }

    #[test]
    fn test_codegen_parameterized_phrase() {
        let input = parse_input(parse_quote! {
            greet($name) = "Hello, {$name}!";
        });
        let tokens = codegen(&input);
        let output = tokens.to_string();

        // Function should have parameter
        assert!(output.contains("fn greet"));
        assert!(output.contains("name"));
        assert!(output.contains("impl Into"));
    }
}
