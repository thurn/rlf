use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;

mod codegen;
mod input;
mod parse;
mod validate;

/// The rlf! macro for defining localized phrases.
///
/// Parses phrase definitions and generates typed Rust functions
/// with compile-time validation.
///
/// # Generated Code
///
/// For each phrase, the macro generates:
/// - A function with the phrase name that returns `::rlf::Phrase`
/// - A constant in `phrase_ids` module with SCREAMING_CASE name
///
/// Additionally generates:
/// - `SOURCE_PHRASES` const with embedded phrase definitions
/// - `register_source_phrases()` function to load phrases into a locale
///
/// # Example
///
/// ```ignore
/// rlf! {
///     card = { one: "card", other: "cards" };
///     draw(n) = "Draw {n} {card:n}.";
/// }
///
/// // Generated: pub fn card(locale: &Locale) -> Phrase
/// // Generated: pub fn draw(locale: &Locale, n: impl Into<Value>) -> Phrase
/// // Generated: pub mod phrase_ids { pub const CARD: PhraseId = ...; ... }
/// ```
#[proc_macro]
pub fn rlf(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as input::MacroInput);

    match expand(input) {
        Ok(tokens) => tokens.into(),
        Err(e) => e.into_compile_error().into(),
    }
}

fn expand(input: input::MacroInput) -> syn::Result<TokenStream2> {
    // Step 1: Validate
    validate::validate(&input)?;

    // Step 2: Generate code
    Ok(codegen::codegen(&input))
}
