//! Table formatting utilities for CLI output.

use comfy_table::{presets, ContentArrangement, Table};

/// Coverage data for a single language.
pub struct LanguageCoverage {
    /// Language code (e.g., "es", "fr").
    pub language: String,
    /// Number of phrases translated.
    pub translated: usize,
    /// Names of missing phrases.
    pub missing: Vec<String>,
}

/// Format coverage data as an ASCII table.
pub fn format_coverage_table(source_count: usize, coverage: &[LanguageCoverage]) -> Table {
    let mut table = Table::new();
    table.load_preset(presets::UTF8_BORDERS_ONLY);
    table.set_content_arrangement(ContentArrangement::Dynamic);
    table.set_header(vec!["Language", "Coverage", "Missing"]);

    for lang in coverage {
        table.add_row(vec![
            lang.language.clone(),
            format!("{}/{}", lang.translated, source_count),
            lang.missing.len().to_string(),
        ]);
    }

    table
}
