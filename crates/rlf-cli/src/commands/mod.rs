//! CLI command implementations.

mod check;
mod eval;

pub use check::{run_check, CheckArgs};
pub use eval::{run_eval, EvalArgs};
