//! CLI command implementations.

mod check;
mod coverage;
mod eval;

pub use check::{run_check, CheckArgs};
pub use coverage::{run_coverage, CoverageArgs};
pub use eval::{run_eval, EvalArgs};
