//! RLF interpreter for evaluating phrases.
//!
//! This module provides the evaluation engine that takes parsed templates
//! and produces formatted strings. It resolves phrase calls, applies variant
//! selection based on parameters, and substitutes values.

mod context;
mod error;
mod plural;
mod registry;
mod transforms;

pub use context::EvalContext;
pub use error::EvalError;
pub use plural::plural_category;
pub use registry::PhraseRegistry;
pub use transforms::TransformRegistry;
