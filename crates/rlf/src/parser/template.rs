//! Template string parser using winnow.

use super::ast::*;
use super::error::ParseError;

/// Parse a template string into an AST.
pub fn parse_template(_input: &str) -> Result<Template, ParseError> {
    // Placeholder - will be implemented in Task 2
    Ok(Template { segments: vec![] })
}
