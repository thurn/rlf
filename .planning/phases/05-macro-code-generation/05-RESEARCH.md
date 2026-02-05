# Phase 5: Macro Code Generation - Research

**Researched:** 2026-02-04
**Domain:** Rust procedural macros, code generation, compile-time validation
**Confidence:** HIGH

## Summary

This phase implements the `rlf!` procedural macro that parses phrase definitions and generates typed Rust functions with compile-time validation. The macro transforms inline RLF syntax into functions that delegate to the existing interpreter infrastructure from Phases 1-4.

The standard approach for Rust procedural macros involves a pipeline architecture: parse TokenStream to AST with `syn`, validate and analyze, then generate code with `quote`. Proc-macro crates must be separate from the library crate. The existing RLF parser AST (`PhraseDefinition`, `Template`, etc.) can be reused for validation, but macro parsing requires a separate `syn`-based parser for TokenStream input.

Key challenges include span preservation for error messages, cycle detection at compile time, and the split between compile-time validation (literal references) and runtime validation (parameter references). The macro will embed source phrases as a const string and generate thin wrapper functions that call the interpreter.

**Primary recommendation:** Create a separate `rlf-macros` crate using syn 2.0, quote 1.0, and proc-macro2 1.0. Implement a 4-stage pipeline (parse, analyze, validate, codegen) with the validation stage reusing patterns from the interpreter's error module.

## Standard Stack

The established libraries/tools for this domain:

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| syn | 2.0 | Parse TokenStream to AST | De facto standard for proc-macro parsing, maintained by dtolnay |
| quote | 1.0 | Generate Rust code from AST | Quasi-quoting with interpolation, pairs with syn |
| proc-macro2 | 1.0 | TokenStream wrapper for testing | Enables unit testing of proc-macro code |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| strsim | 0.11 | Levenshtein distance | Already a dependency, reuse for typo suggestions |
| trybuild | 1.0 | Compile-fail tests | Test error messages and compile-time validation |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| syn | venial | Lighter weight but less feature complete, syn is more established |
| proc-macro-error | syn::Error | proc-macro-error adds convenience but syn::Error is sufficient and simpler |

**Installation:**
```toml
# In rlf-macros/Cargo.toml
[dependencies]
syn = { version = "2.0", features = ["full", "parsing", "printing"] }
quote = "1.0"
proc-macro2 = "1.0"
strsim = "0.11"  # For typo suggestions

[dev-dependencies]
trybuild = "1.0"

[lib]
proc-macro = true
```

## Architecture Patterns

### Recommended Project Structure
```
crates/
├── rlf/               # Existing library crate (interpreter, types, parser)
│   └── src/
│       └── lib.rs     # Re-exports types for generated code
├── rlf-macros/        # New proc-macro crate
│   ├── src/
│   │   ├── lib.rs     # Macro entry point (#[proc_macro])
│   │   ├── parse.rs   # TokenStream -> MacroInput parsing
│   │   ├── validate.rs # Compile-time validation
│   │   ├── codegen.rs # Code generation with quote!
│   │   └── error.rs   # Compile error handling
│   └── tests/
│       ├── pass/      # Should-compile test cases
│       └── fail/      # Should-fail test cases with .stderr
└── rlf-tests/         # Existing integration tests
```

### Pattern 1: Pipeline Architecture
**What:** Separate macro logic into distinct stages: parse -> analyze -> validate -> codegen
**When to use:** Always for proc-macros with complex validation
**Example:**
```rust
// Source: Ferrous Systems blog on testing proc-macros
#[proc_macro]
pub fn rlf(input: TokenStream) -> TokenStream {
    // 1. Parse: TokenStream -> MacroInput
    let input = parse_macro_input!(input as MacroInput);

    // 2. Validate: Check references, detect cycles
    if let Err(e) = validate(&input) {
        return e.into_compile_error().into();
    }

    // 3. Codegen: MacroInput -> TokenStream
    codegen(&input).into()
}
```

### Pattern 2: syn::Error for Compile Errors
**What:** Use syn::Error::new_spanned() for errors with source location
**When to use:** All validation errors to preserve span information
**Example:**
```rust
// Source: https://docs.rs/syn/latest/syn/struct.Error.html
fn validate_reference(name: &Ident, phrases: &HashSet<String>) -> syn::Result<()> {
    if !phrases.contains(&name.to_string()) {
        return Err(syn::Error::new_spanned(
            name,
            format!("unknown phrase '{}'", name),
        ));
    }
    Ok(())
}
```

### Pattern 3: Spanned Identifiers
**What:** Wrap parsed identifiers with their spans for error reporting
**When to use:** All user-provided names that may need error messages
**Example:**
```rust
// Source: APPENDIX_RUST_INTEGRATION.md
struct SpannedIdent {
    name: String,
    span: Span,
}

impl Parse for SpannedIdent {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident: Ident = input.parse()?;
        Ok(SpannedIdent {
            name: ident.to_string(),
            span: ident.span(),
        })
    }
}
```

### Pattern 4: Quote Interpolation
**What:** Use quote! macro with #var interpolation for code generation
**When to use:** Generating all Rust code output
**Example:**
```rust
// Source: https://docs.rs/quote/latest/quote/macro.quote.html
fn generate_phrase_function(name: &str, params: &[String]) -> TokenStream2 {
    let fn_name = format_ident!("{}", name);
    let param_idents: Vec<_> = params.iter().map(|p| format_ident!("{}", p)).collect();

    quote! {
        /// Returns the #name phrase.
        pub fn #fn_name(locale: &Locale, #(#param_idents: impl Into<Value>),*) -> Phrase {
            locale.call_phrase(#name, &[#(#param_idents.into()),*])
                .expect(concat!("phrase '", #name, "' should exist"))
        }
    }
}
```

### Anti-Patterns to Avoid
- **Panicking in proc-macros:** Use syn::Error::into_compile_error() instead of panic!()
- **Using proc_macro::TokenStream internally:** Use proc_macro2::TokenStream for testability
- **Reimplementing parsers:** Reuse syn's parsing infrastructure with custom Parse impls
- **Monolithic macro function:** Split into stages for testability and maintainability

## Don't Hand-Roll

Problems that look simple but have existing solutions:

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| TokenStream parsing | Custom tokenizer | syn with Parse trait | Handles all Rust syntax edge cases, spans |
| Code generation | String concatenation | quote! macro | Proper tokenization, span handling, hygiene |
| Levenshtein distance | Custom algorithm | strsim crate | Already a dependency, well-tested |
| Compile-fail testing | Manual verification | trybuild | Automated .stderr comparison |
| Identifier case conversion | Manual formatting | heck crate (or format_ident!) | snake_case to SCREAMING_CASE |

**Key insight:** Proc-macro infrastructure is mature and well-tested. The syn/quote ecosystem handles edge cases (Unicode, nested macros, span tracking) that are easy to get wrong.

## Common Pitfalls

### Pitfall 1: Span Loss in Error Messages
**What goes wrong:** Errors point to wrong location or say "call site"
**Why it happens:** Using Span::call_site() instead of preserving input spans
**How to avoid:** Store spans alongside parsed data, use new_spanned()
**Warning signs:** Error messages say "at macro invocation" instead of specific location

### Pitfall 2: Testing Difficulties
**What goes wrong:** Can't unit test macro logic, only integration test
**Why it happens:** Using proc_macro::TokenStream which can't exist outside proc-macro context
**How to avoid:** Use proc_macro2::TokenStream internally, convert at entry/exit
**Warning signs:** All tests are in a separate crate, can't test individual stages

### Pitfall 3: Cycle Detection Complexity
**What goes wrong:** Stack overflow or infinite loop during validation
**Why it happens:** Naive DFS without visited set
**How to avoid:** Build dependency graph, use standard cycle detection (DFS with colors or topological sort)
**Warning signs:** Large phrase files cause compilation hangs

### Pitfall 4: Hygiene Issues
**What goes wrong:** Generated code conflicts with user's imports
**Why it happens:** Using short paths like `Value` instead of fully qualified paths
**How to avoid:** Use absolute paths in generated code: `::rlf::Value`, `::rlf::Locale`
**Warning signs:** Users report "cannot find type" errors despite correct macro usage

### Pitfall 5: Confusing Literal vs Parameter Validation
**What goes wrong:** Errors reported for parameter references that can only be validated at runtime
**Why it happens:** Not distinguishing between literal (known at compile time) and parameter (runtime) references
**How to avoid:** Track which names are parameters, only validate literal references at compile time
**Warning signs:** False positive errors on valid code like `draw(n) = "{card:n}"`

### Pitfall 6: Case Sensitivity in Name Conversion
**What goes wrong:** PhraseId constant FIRE_ELEMENTAL doesn't match phrase fire_elemental
**Why it happens:** Case conversion changes the string used for hashing
**How to avoid:** Constants use display name for doc, original name for hash: `PhraseId::from_name("fire_elemental")`
**Warning signs:** PhraseId lookups fail at runtime despite generated constants

## Code Examples

Verified patterns from official sources:

### Macro Entry Point
```rust
// Source: https://doc.rust-lang.org/reference/procedural-macros.html
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use syn::parse_macro_input;

#[proc_macro]
pub fn rlf(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as MacroInput);

    match expand(input) {
        Ok(tokens) => tokens.into(),
        Err(e) => e.into_compile_error().into(),
    }
}

fn expand(input: MacroInput) -> syn::Result<TokenStream2> {
    // Validation
    validate(&input)?;

    // Code generation
    Ok(codegen(&input))
}
```

### Custom Parse Implementation
```rust
// Source: https://docs.rs/syn/latest/syn/parse/index.html
use syn::{parse::{Parse, ParseStream}, Ident, Token, LitStr, braced};
use syn::punctuated::Punctuated;

struct PhraseDefinition {
    name: Ident,
    params: Vec<Ident>,
    body: PhraseBody,
}

impl Parse for PhraseDefinition {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name: Ident = input.parse()?;

        // Optional parameters: (p1, p2)
        let params = if input.peek(syn::token::Paren) {
            let content;
            syn::parenthesized!(content in input);
            Punctuated::<Ident, Token![,]>::parse_terminated(&content)?
                .into_iter().collect()
        } else {
            Vec::new()
        };

        input.parse::<Token![=]>()?;
        let body = input.parse()?;
        input.parse::<Token![;]>()?;

        Ok(PhraseDefinition { name, params, body })
    }
}
```

### Error with Span
```rust
// Source: https://docs.rs/syn/latest/syn/struct.Error.html
fn check_undefined_phrase(
    reference: &SpannedIdent,
    defined: &HashSet<String>,
    available: &[String],
) -> syn::Result<()> {
    if !defined.contains(&reference.name) {
        let suggestions = compute_suggestions(&reference.name, available);
        let mut msg = format!("unknown phrase '{}'", reference.name);
        if !suggestions.is_empty() {
            msg.push_str(&format!("\nhelp: did you mean '{}'?", suggestions[0]));
        }
        return Err(syn::Error::new(reference.span, msg));
    }
    Ok(())
}
```

### Cycle Detection
```rust
// Source: Standard graph algorithm (not library-specific)
use std::collections::{HashMap, HashSet};

enum Color { White, Gray, Black }

fn detect_cycles(
    phrases: &HashMap<String, Vec<String>>, // name -> references
) -> Result<(), Vec<String>> {
    let mut colors: HashMap<&str, Color> = phrases.keys()
        .map(|k| (k.as_str(), Color::White))
        .collect();

    for name in phrases.keys() {
        if matches!(colors.get(name.as_str()), Some(Color::White)) {
            let mut path = Vec::new();
            if let Some(cycle) = dfs_cycle(name, phrases, &mut colors, &mut path) {
                return Err(cycle);
            }
        }
    }
    Ok(())
}

fn dfs_cycle<'a>(
    name: &'a str,
    phrases: &'a HashMap<String, Vec<String>>,
    colors: &mut HashMap<&'a str, Color>,
    path: &mut Vec<&'a str>,
) -> Option<Vec<String>> {
    colors.insert(name, Color::Gray);
    path.push(name);

    if let Some(refs) = phrases.get(name) {
        for r in refs {
            match colors.get(r.as_str()) {
                Some(Color::Gray) => {
                    // Found cycle - extract cycle from path
                    let cycle_start = path.iter().position(|&n| n == r).unwrap();
                    let mut cycle: Vec<String> = path[cycle_start..].iter()
                        .map(|s| s.to_string()).collect();
                    cycle.push(r.to_string());
                    return Some(cycle);
                }
                Some(Color::White) | None => {
                    if let Some(cycle) = dfs_cycle(r, phrases, colors, path) {
                        return Some(cycle);
                    }
                }
                Some(Color::Black) => {}
            }
        }
    }

    colors.insert(name, Color::Black);
    path.pop();
    None
}
```

### Generated Function Pattern
```rust
// Source: APPENDIX_RUST_INTEGRATION.md
// Generated code for: card = { one: "card", other: "cards" };
/// Returns the "card" phrase.
pub fn card(locale: &::rlf::Locale) -> ::rlf::Phrase {
    locale.get_phrase("card")
        .expect("phrase 'card' should exist")
}

// Generated code for: draw(n) = "Draw {n} {card:n}.";
/// Evaluates the "draw" phrase.
pub fn draw(locale: &::rlf::Locale, n: impl Into<::rlf::Value>) -> ::rlf::Phrase {
    locale.call_phrase("draw", &[n.into()])
        .expect("phrase 'draw' should exist")
}
```

### SOURCE_PHRASES Embedding
```rust
// Source: APPENDIX_RUST_INTEGRATION.md
const SOURCE_PHRASES: &str = r#"
card = { one: "card", other: "cards" };
draw(n) = "Draw {n} {card:n}.";
"#;

/// Registers source language phrases with the locale. Call once at startup.
pub fn register_source_phrases(locale: &mut ::rlf::Locale) {
    locale.load_translations_str("en", SOURCE_PHRASES)
        .expect("source phrases should parse successfully");
}
```

### PhraseId Constants Module
```rust
// Source: APPENDIX_RUST_INTEGRATION.md
pub mod phrase_ids {
    use ::rlf::PhraseId;

    /// ID for the "card" phrase.
    pub const CARD: PhraseId = PhraseId::from_name("card");

    /// ID for the "draw" phrase. Call with 1 argument (n).
    pub const DRAW: PhraseId = PhraseId::from_name("draw");
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| syn 1.x | syn 2.0 | 2023 | Simpler API, better error handling |
| proc-macro-error crate | syn::Error directly | 2023 | Fewer dependencies, simpler |
| String-based codegen | quote! macro | Long-standing | Type-safe, span-preserving |

**Deprecated/outdated:**
- proc-macro-error crate: While still functional, syn 2.0's native Error type is now preferred
- proc-macro-hack: No longer needed, proc-macro-crate features work in stable Rust

## Open Questions

Things that couldn't be fully resolved:

1. **SOURCE_PHRASES format: verbatim vs normalized**
   - What we know: The appendix shows embedding as-is; normalization could reduce size
   - What's unclear: Whether whitespace/formatting matters for any edge cases
   - Recommendation: Use verbatim (Claude's discretion per CONTEXT.md) - simpler and preserves user intent

2. **Re-export strategy for generated code**
   - What we know: Generated code needs Locale, Phrase, Value, PhraseId types
   - What's unclear: Whether to re-export from macro module or require direct rlf import
   - Recommendation: Generate `use ::rlf::{Locale, Phrase, Value, PhraseId};` in output, users import rlf directly

## Sources

### Primary (HIGH confidence)
- [Rust Reference: Procedural macros](https://doc.rust-lang.org/reference/procedural-macros.html) - Official documentation
- [syn 2.0 Documentation](https://docs.rs/syn/latest/syn/) - Parsing library docs
- [quote Documentation](https://docs.rs/quote/latest/quote/) - Code generation docs
- APPENDIX_RUST_INTEGRATION.md - Project-specific design decisions

### Secondary (MEDIUM confidence)
- [Ferrous Systems: Testing Proc Macros](https://ferrous-systems.com/blog/testing-proc-macros/) - Pipeline architecture pattern
- [dtolnay/syn GitHub](https://github.com/dtolnay/syn) - Examples and best practices

### Tertiary (LOW confidence)
- Web search results for "proc-macro best practices 2026" - General community patterns

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - syn/quote/proc-macro2 are the established standard
- Architecture: HIGH - Pipeline pattern is well-documented and widely used
- Pitfalls: HIGH - Based on official documentation and established patterns
- Validation logic: MEDIUM - Cycle detection algorithm is standard, integration with existing code needs verification

**Research date:** 2026-02-04
**Valid until:** 60 days (proc-macro ecosystem is stable)
