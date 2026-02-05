//! Implementation of the `rlf check` command.

use std::path::PathBuf;

/// Arguments for the check command.
#[derive(Debug, clap::Args)]
pub struct CheckArgs {
    /// Files to check (.rlf)
    #[arg(required = true)]
    pub files: Vec<PathBuf>,

    /// Output as JSON
    #[arg(long)]
    pub json: bool,

    /// Check against source file for phrase coverage
    #[arg(long)]
    pub strict: Option<PathBuf>,
}

/// Run the check command.
pub fn run_check(_args: CheckArgs) -> miette::Result<i32> {
    // Placeholder implementation - will be completed in Task 2
    Ok(exitcode::OK)
}
