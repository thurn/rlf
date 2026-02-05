//! Miette diagnostic wrapper for RLF parse errors.
//!
//! Note: This module has an exception for `unused_assignments` because miette
//! derive macros read struct fields in generated code that rustc cannot track.
#![allow(unused_assignments)]

use miette::{Diagnostic, NamedSource, SourceSpan};
use rlf::parser::ParseError;
use std::path::Path;
use thiserror::Error;

/// A miette-compatible diagnostic for RLF parse errors.
///
/// Note: Fields are read by miette derive macros, not directly by code.
#[derive(Debug, Error, Diagnostic)]
#[error("syntax error: {message}")]
#[diagnostic(code(rlf::syntax))]
pub struct RlfDiagnostic {
    #[source_code]
    src: NamedSource<String>,

    #[label("error here")]
    span: SourceSpan,

    message: String,

    #[help]
    help: Option<String>,
}

impl RlfDiagnostic {
    /// Create a diagnostic from a ParseError with source context.
    pub fn from_parse_error(path: &Path, content: &str, err: &ParseError) -> Self {
        let (line, column, message) = match err {
            ParseError::Syntax {
                line,
                column,
                message,
            } => (*line, *column, message.clone()),
            ParseError::UnexpectedEof { line, column } => {
                (*line, *column, "unexpected end of file".into())
            }
            ParseError::InvalidUtf8 => (1, 1, "invalid UTF-8".into()),
        };

        // Convert line:column to byte offset.
        // Sum of (line_length + 1) for lines before error line, plus column.
        let offset = content
            .lines()
            .take(line.saturating_sub(1))
            .map(|l| l.len() + 1)
            .sum::<usize>()
            + column.saturating_sub(1);

        // Clamp offset to content length to avoid miette panic on out-of-bounds
        let offset = offset.min(content.len());

        RlfDiagnostic {
            src: NamedSource::new(path.display().to_string(), content.to_string()),
            span: (offset, 1).into(),
            message,
            help: None,
        }
    }
}
