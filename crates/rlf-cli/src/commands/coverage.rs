//! Coverage command implementation.

use std::collections::HashSet;
use std::fs::read_to_string;
use std::path::{Path, PathBuf};

use clap::Args;
use miette::{miette, IntoDiagnostic, Result};
use rlf::parser::parse_file;
use serde::Serialize;

use crate::output::table::{format_coverage_table, LanguageCoverage};
use crate::output::RlfDiagnostic;

/// Arguments for the coverage command.
#[derive(Debug, Args)]
pub struct CoverageArgs {
    /// Source language file (e.g., en.rlf).
    #[arg(long)]
    pub source: PathBuf,

    /// Languages to check coverage for (comma-separated).
    #[arg(long, value_delimiter = ',')]
    pub lang: Vec<String>,

    /// Directory containing translation files. Defaults to source file directory.
    #[arg(long)]
    pub translations: Option<PathBuf>,

    /// Exit with non-zero code if any translation is incomplete.
    #[arg(long)]
    pub strict: bool,

    /// Output results as JSON.
    #[arg(long)]
    pub json: bool,
}

/// JSON output format for coverage data.
#[derive(Debug, Serialize)]
struct CoverageJson {
    language: String,
    translated: usize,
    total: usize,
    missing: Vec<String>,
}

/// Run the coverage command.
pub fn run_coverage(args: CoverageArgs) -> Result<i32> {
    // Parse source file to get phrase names
    let source_content = read_to_string(&args.source)
        .into_diagnostic()
        .map_err(|e| miette!("Failed to read source file {:?}: {}", args.source, e))?;

    let source_phrases = match parse_file(&source_content) {
        Ok(phrases) => phrases,
        Err(e) => {
            let diagnostic = RlfDiagnostic::from_parse_error(&args.source, &source_content, &e);
            return Err(diagnostic.into());
        }
    };

    let source_names: HashSet<String> = source_phrases.iter().map(|p| p.name.clone()).collect();
    let source_count = source_names.len();

    // Determine base directory for translation files
    let base_dir = args
        .translations
        .clone()
        .or_else(|| args.source.parent().map(Path::to_path_buf))
        .unwrap_or_else(|| PathBuf::from("."));

    // Collect coverage data for each language
    let mut coverage_data: Vec<LanguageCoverage> = Vec::new();

    for lang in &args.lang {
        let lang_file = base_dir.join(format!("{}.rlf", lang));

        let (translated_names, missing): (HashSet<String>, Vec<String>) = if lang_file.exists() {
            let lang_content = read_to_string(&lang_file)
                .into_diagnostic()
                .map_err(|e| miette!("Failed to read translation file {:?}: {}", lang_file, e))?;

            match parse_file(&lang_content) {
                Ok(phrases) => {
                    let translated: HashSet<String> =
                        phrases.iter().map(|p| p.name.clone()).collect();
                    let missing: Vec<String> = source_names
                        .iter()
                        .filter(|name| !translated.contains(*name))
                        .cloned()
                        .collect();
                    (translated, missing)
                }
                Err(e) => {
                    let diagnostic = RlfDiagnostic::from_parse_error(&lang_file, &lang_content, &e);
                    return Err(diagnostic.into());
                }
            }
        } else {
            // File doesn't exist - all phrases are missing
            (HashSet::new(), source_names.iter().cloned().collect())
        };

        let translated_count = source_names.intersection(&translated_names).count();

        coverage_data.push(LanguageCoverage {
            language: lang.clone(),
            translated: translated_count,
            missing,
        });
    }

    // Check if any translation is incomplete
    let any_incomplete = coverage_data.iter().any(|c| !c.missing.is_empty());

    // Output results
    if args.json {
        let json_data: Vec<CoverageJson> = coverage_data
            .iter()
            .map(|c| CoverageJson {
                language: c.language.clone(),
                translated: c.translated,
                total: source_count,
                missing: c.missing.clone(),
            })
            .collect();

        let json_output = serde_json::to_string_pretty(&json_data).into_diagnostic()?;
        println!("{}", json_output);
    } else {
        // Print ASCII table
        let table = format_coverage_table(source_count, &coverage_data);
        println!("{}", table);

        // Print missing phrases per language
        for lang_coverage in &coverage_data {
            if !lang_coverage.missing.is_empty() {
                println!("\nMissing in {}:", lang_coverage.language);
                for name in &lang_coverage.missing {
                    println!("  - {}", name);
                }
            }
        }
    }

    // Determine exit code
    if args.strict && any_incomplete {
        Ok(exitcode::DATAERR)
    } else {
        Ok(exitcode::OK)
    }
}
