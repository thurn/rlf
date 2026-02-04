//! Parse error types for RLF.

use thiserror::Error;

/// An error that occurred during parsing.
#[derive(Debug, Error)]
pub enum ParseError {
    /// A syntax error with location information.
    #[error("syntax error at {line}:{column}: {message}")]
    Syntax {
        line: usize,
        column: usize,
        message: String,
    },

    /// Unexpected end of input.
    #[error("unexpected end of input at {line}:{column}")]
    UnexpectedEof { line: usize, column: usize },

    /// Invalid UTF-8 in input.
    #[error("invalid UTF-8 in input")]
    InvalidUtf8,
}
