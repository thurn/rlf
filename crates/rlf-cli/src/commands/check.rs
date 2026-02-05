//! Implementation of the `rlf check` command.

use crate::output::RlfDiagnostic;
use rlf::parser::{parse_file, ParseError};
use serde::Serialize;
use std::collections::HashSet;
use std::fs::read_to_string;
use std::path::PathBuf;

/// Maximum number of errors to display per file before truncating.
const MAX_ERRORS_PER_FILE: usize = 10;

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

/// JSON output for check results.
#[derive(Serialize)]
pub struct CheckResult {
    pub files: Vec<FileResult>,
    pub total_errors: usize,
}

/// JSON output for a single file's check result.
#[derive(Serialize)]
pub struct FileResult {
    pub path: String,
    pub status: FileStatus,
    pub errors: Vec<ErrorInfo>,
}

/// Status of a file check.
#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FileStatus {
    Ok,
    Error,
    NotFound,
}

/// JSON output for a single error.
#[derive(Serialize)]
pub struct ErrorInfo {
    pub line: usize,
    pub column: usize,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggestion: Option<String>,
}

/// Run the check command.
pub fn run_check(args: CheckArgs) -> miette::Result<i32> {
    // If --strict mode, parse source file first to get phrase names
    let source_phrases = if let Some(source_path) = &args.strict {
        let content = read_to_string(source_path).map_err(|e| {
            miette::miette!("Cannot read source file {}: {}", source_path.display(), e)
        })?;
        let defs =
            parse_file(&content).map_err(|e| miette::miette!("Source file parse error: {}", e))?;
        Some(
            defs.into_iter()
                .map(|d| d.name)
                .collect::<HashSet<String>>(),
        )
    } else {
        None
    };

    let mut total_errors = 0;
    let mut results = Vec::new();

    for path in &args.files {
        let (file_result, errors) = check_file(path, source_phrases.as_ref(), args.json);
        total_errors += errors;
        results.push(file_result);
    }

    if args.json {
        let output = CheckResult {
            files: results,
            total_errors,
        };
        println!(
            "{}",
            serde_json::to_string_pretty(&output).expect("JSON serialization should not fail")
        );
    } else if total_errors == 0 {
        let file_count = args.files.len();
        if file_count == 1 {
            // Single file - already printed OK
        } else {
            println!("Checked {} files, no errors", file_count);
        }
    }

    Ok(if total_errors > 0 {
        exitcode::DATAERR
    } else {
        exitcode::OK
    })
}

/// Check a single file and return results.
fn check_file(
    path: &PathBuf,
    source_phrases: Option<&HashSet<String>>,
    json_mode: bool,
) -> (FileResult, usize) {
    let content = match read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            if !json_mode {
                eprintln!("{}: cannot read file: {}", path.display(), e);
            }
            return (
                FileResult {
                    path: path.display().to_string(),
                    status: FileStatus::NotFound,
                    errors: vec![ErrorInfo {
                        line: 0,
                        column: 0,
                        message: format!("cannot read file: {}", e),
                        suggestion: None,
                    }],
                },
                1,
            );
        }
    };

    match parse_file(&content) {
        Ok(defs) => {
            // Check for missing phrases if --strict mode
            if let Some(source_names) = source_phrases {
                let target_names: HashSet<String> = defs.iter().map(|d| d.name.clone()).collect();
                let missing: Vec<_> = source_names
                    .iter()
                    .filter(|n| !target_names.contains(*n))
                    .cloned()
                    .collect();

                if !missing.is_empty() {
                    let error_count = missing.len();
                    if !json_mode {
                        eprintln!(
                            "{}: missing {} phrases from source:",
                            path.display(),
                            error_count
                        );
                        for (i, name) in missing.iter().enumerate() {
                            if i < MAX_ERRORS_PER_FILE {
                                eprintln!("  - {}", name);
                            }
                        }
                        if missing.len() > MAX_ERRORS_PER_FILE {
                            eprintln!("  ... and {} more", missing.len() - MAX_ERRORS_PER_FILE);
                        }
                    }
                    return (
                        FileResult {
                            path: path.display().to_string(),
                            status: FileStatus::Error,
                            errors: missing
                                .into_iter()
                                .map(|name| ErrorInfo {
                                    line: 0,
                                    column: 0,
                                    message: format!("missing phrase: {}", name),
                                    suggestion: None,
                                })
                                .collect(),
                        },
                        error_count,
                    );
                }
            }

            // File parsed successfully
            if !json_mode {
                println!("{}: OK", path.display());
            }
            (
                FileResult {
                    path: path.display().to_string(),
                    status: FileStatus::Ok,
                    errors: vec![],
                },
                0,
            )
        }
        Err(e) => {
            let (line, column, message) = extract_error_info(&e);

            if !json_mode {
                let diagnostic = RlfDiagnostic::from_parse_error(path, &content, &e);
                eprintln!("{:?}", miette::Report::new(diagnostic));
            }

            (
                FileResult {
                    path: path.display().to_string(),
                    status: FileStatus::Error,
                    errors: vec![ErrorInfo {
                        line,
                        column,
                        message,
                        suggestion: None,
                    }],
                },
                1,
            )
        }
    }
}

/// Extract line, column, and message from a ParseError.
fn extract_error_info(err: &ParseError) -> (usize, usize, String) {
    match err {
        ParseError::Syntax {
            line,
            column,
            message,
        } => (*line, *column, message.clone()),
        ParseError::UnexpectedEof { line, column } => {
            (*line, *column, "unexpected end of file".into())
        }
        ParseError::InvalidUtf8 => (1, 1, "invalid UTF-8".into()),
    }
}
