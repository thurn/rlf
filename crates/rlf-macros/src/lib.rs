use proc_macro::TokenStream;

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

    // Validation (Plan 02)
    if let Err(e) = validate::validate(&input) {
        return e.to_compile_error().into();
    }

    // Code generation (Plan 03)
    codegen::codegen(&input).into()
}
