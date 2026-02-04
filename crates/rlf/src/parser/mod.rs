//! RLF template and file parser.
//!
//! This module provides parsing for RLF template strings and `.rlf` files.
//! The parser produces an AST that can be used for interpretation, code generation,
//! or external tooling.

pub mod ast;
pub mod error;
mod file;
mod template;

pub use ast::*;
pub use error::ParseError;
pub use file::parse_file;
pub use template::parse_template;
