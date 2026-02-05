use proc_macro::TokenStream;

mod input;
mod parse;
mod validate;

/// The rlf! macro for defining localized phrases.
///
/// Parses phrase definitions and generates typed Rust functions
/// with compile-time validation.
#[proc_macro]
pub fn rlf(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as input::MacroInput);

    // Validation (Plan 02)
    if let Err(e) = validate::validate(&input) {
        return e.to_compile_error().into();
    }

    // TODO: Code generation (Plan 03)

    // For now, just produce empty output so the crate compiles
    proc_macro2::TokenStream::new().into()
}
