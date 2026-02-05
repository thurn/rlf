// Placeholder for Parse implementations - will be implemented in Task 3
use crate::input::MacroInput;
use syn::parse::{Parse, ParseStream};

impl Parse for MacroInput {
    fn parse(_input: ParseStream) -> syn::Result<Self> {
        Ok(MacroInput)
    }
}
