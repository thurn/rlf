//! RLF CLI entry point.
//!
//! Provides command-line tools for working with RLF localization files:
//! - `rlf check` - Validate .rlf file syntax

mod commands;
mod output;

use std::process::exit;

use clap::{Parser, Subcommand, ValueEnum};
use commands::{run_check, run_coverage, run_eval, CheckArgs, CoverageArgs, EvalArgs};

/// RLF localization file tools.
#[derive(Debug, Parser)]
#[command(name = "rlf")]
#[command(about = "RLF localization file tools", long_about = None)]
#[command(version)]
pub struct Cli {
    /// Color output control
    #[arg(long, value_enum, default_value_t = ColorWhen::Auto, global = true)]
    pub color: ColorWhen,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Commands,
}

/// When to use colored output.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ColorWhen {
    Auto,
    Always,
    Never,
}

/// CLI subcommands.
#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Check .rlf file syntax
    Check(CheckArgs),
    /// Report translation coverage across languages
    Coverage(CoverageArgs),
    /// Evaluate an RLF template string
    Eval(EvalArgs),
}

/// Set up color output based on user preference.
fn setup_colors(color_when: ColorWhen) {
    match color_when {
        ColorWhen::Auto => {
            // owo-colors automatically checks TTY, NO_COLOR, FORCE_COLOR
        }
        ColorWhen::Always => {
            owo_colors::set_override(true);
        }
        ColorWhen::Never => {
            owo_colors::set_override(false);
        }
    }
}

fn main() -> miette::Result<()> {
    let cli = Cli::parse();
    setup_colors(cli.color);

    miette::set_hook(Box::new(|_| {
        Box::new(
            miette::MietteHandlerOpts::new()
                .terminal_links(true)
                .unicode(true)
                .context_lines(2)
                .build(),
        )
    }))?;

    let result = match cli.command {
        Commands::Check(args) => run_check(args),
        Commands::Coverage(args) => run_coverage(args),
        Commands::Eval(args) => run_eval(args),
    };

    match result {
        Ok(code) => exit(code),
        Err(e) => {
            eprintln!("{:?}", e);
            exit(exitcode::SOFTWARE);
        }
    }
}
