//! Implementation of the `rlf eval` command.

use rlf::{Locale, Value};
use serde::Serialize;
use std::collections::HashMap;
use std::fs::read_to_string;
use std::path::PathBuf;

/// Arguments for the eval command.
#[derive(Debug, clap::Args)]
pub struct EvalArgs {
    /// Language code for evaluation (e.g., en, de, ru)
    #[arg(long, required = true)]
    pub lang: String,

    /// Template string to evaluate
    #[arg(long, required = true)]
    pub template: String,

    /// File with phrase definitions (.rlf)
    #[arg(long)]
    pub phrases: Option<PathBuf>,

    /// Parameters in name=value format (repeatable)
    #[arg(short = 'p', long = "param", value_parser = parse_key_val)]
    pub params: Vec<(String, String)>,

    /// Output as JSON
    #[arg(long)]
    pub json: bool,
}

/// JSON output for eval results.
#[derive(Serialize)]
pub struct EvalResult {
    pub result: String,
}

/// Parse a key=value parameter string.
fn parse_key_val(s: &str) -> Result<(String, String), String> {
    let pos = s
        .find('=')
        .ok_or_else(|| format!("invalid parameter format '{}': expected name=value", s))?;
    Ok((s[..pos].to_string(), s[pos + 1..].to_string()))
}

/// Run the eval command.
pub fn run_eval(args: EvalArgs) -> miette::Result<i32> {
    // Create locale with specified language
    let mut locale = Locale::with_language(&args.lang);

    // Load phrases file if provided
    if let Some(phrases_path) = &args.phrases {
        let content = read_to_string(phrases_path).map_err(|e| {
            miette::miette!("Cannot read phrases file {}: {}", phrases_path.display(), e)
        })?;
        locale
            .load_translations_str(&args.lang, &content)
            .map_err(|e| miette::miette!("Failed to parse phrases file: {}", e))?;
    } else {
        // Load empty phrases so eval_str works for plain templates
        locale
            .load_translations_str(&args.lang, "")
            .map_err(|e| miette::miette!("Failed to initialize locale: {}", e))?;
    }

    // Convert parameters to HashMap<String, Value>
    let params: HashMap<String, Value> = args
        .params
        .into_iter()
        .map(|(k, v)| {
            // Try parsing as i64 first, fall back to String
            let value = if let Ok(n) = v.parse::<i64>() {
                Value::from(n)
            } else {
                Value::from(v)
            };
            (k, value)
        })
        .collect();

    // Evaluate the template
    match locale.eval_str(&args.template, params) {
        Ok(result) => {
            if args.json {
                let output = EvalResult {
                    result: result.to_string(),
                };
                println!(
                    "{}",
                    serde_json::to_string_pretty(&output)
                        .expect("JSON serialization should not fail")
                );
            } else {
                println!("{}", result);
            }
            Ok(exitcode::OK)
        }
        Err(e) => {
            if args.json {
                let output = serde_json::json!({
                    "error": e.to_string()
                });
                eprintln!(
                    "{}",
                    serde_json::to_string_pretty(&output)
                        .expect("JSON serialization should not fail")
                );
            } else {
                eprintln!("Evaluation error: {}", e);
            }
            Ok(exitcode::DATAERR)
        }
    }
}
