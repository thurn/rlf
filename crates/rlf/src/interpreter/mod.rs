//! RLF interpreter for evaluating phrases.
//!
//! This module provides the evaluation engine that takes parsed templates
//! and produces formatted strings. It resolves phrase calls, applies variant
//! selection based on parameters, and substitutes values.

mod context;
mod error;
mod evaluator;
pub mod lint;
mod locale;
mod plural;
mod registry;
mod transforms;

pub use context::EvalContext;
pub use error::{EvalError, EvalWarning, LoadError, LoadWarning, compute_suggestions};
pub use evaluator::{eval_phrase_def, eval_template};
pub use lint::{lint_definitions, run_lints};
pub use locale::Locale;
pub use plural::plural_category;
pub use registry::PhraseRegistry;
pub use transforms::{TransformKind, TransformRegistry};
