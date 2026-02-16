/// Formats RLF definition files and definition bodies within Rust files.
///
/// Uses text-level processing rather than AST parsing because `.rs` files
/// contain Rust escape sequences (e.g., `\u{25CF}`) that the RLF parser
/// would misinterpret, and comments need to be preserved.

/// Formats a complete `.rlf` file, wrapping lines to `max_width` characters.
pub fn format_file(input: &str, max_width: usize) -> String {
    format_definitions(input, "", max_width)
}

/// Formats definition body text with a given base indent, wrapping lines to
/// `max_width` characters.
pub fn format_definitions(input: &str, base_indent: &str, max_width: usize) -> String {
    let chunks = split_into_chunks(input, base_indent);
    let mut result = String::new();
    for chunk in &chunks {
        match chunk {
            Chunk::BlankLine => result.push('\n'),
            Chunk::Comment(line) => {
                result.push_str(line);
                result.push('\n');
            }
            Chunk::Definition(lines) => {
                let formatted = format_definition(lines, base_indent, max_width);
                result.push_str(&formatted);
                result.push('\n');
            }
        }
    }
    // Remove trailing newlines, then add exactly one
    let trimmed = result.trim_end_matches('\n');
    if trimmed.is_empty() {
        return String::new();
    }
    format!("{trimmed}\n")
}

enum Chunk {
    BlankLine,
    Comment(String),
    Definition(Vec<String>),
}

/// Splits input into chunks: blank lines, comment lines, and definition lines
/// (accumulated until a line ends with `;` outside strings/braces).
fn split_into_chunks(input: &str, base_indent: &str) -> Vec<Chunk> {
    let mut chunks = Vec::new();
    let mut def_lines: Vec<String> = Vec::new();

    for line in input.lines() {
        let trimmed = line.trim();

        if !def_lines.is_empty() {
            // We're accumulating a multi-line definition
            def_lines.push(line.to_string());
            if definition_complete(&def_lines) {
                chunks.push(Chunk::Definition(def_lines.clone()));
                def_lines.clear();
            }
            continue;
        }

        if trimmed.is_empty() {
            chunks.push(Chunk::BlankLine);
        } else if trimmed.starts_with("//") {
            // Preserve comment with proper indentation
            let comment = if base_indent.is_empty() {
                trimmed.to_string()
            } else {
                format!("{base_indent}{trimmed}")
            };
            chunks.push(Chunk::Comment(comment));
        } else {
            def_lines.push(line.to_string());
            if definition_complete(&def_lines) {
                chunks.push(Chunk::Definition(def_lines.clone()));
                def_lines.clear();
            }
        }
    }

    // Flush any remaining definition lines
    if !def_lines.is_empty() {
        chunks.push(Chunk::Definition(def_lines));
    }

    chunks
}

/// Returns true if accumulated lines form a complete definition (ends with `;`
/// outside strings and balanced braces).
fn definition_complete(lines: &[String]) -> bool {
    let joined: String = lines.iter().map(|l| l.trim()).collect::<Vec<_>>().join(" ");
    let trimmed = joined.trim_end();
    if !trimmed.ends_with(';') {
        return false;
    }
    // Check brace balance outside of strings
    let mut depth = 0i32;
    let mut in_string = false;
    let mut chars = trimmed.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\\' && in_string {
            chars.next(); // skip escaped char
            continue;
        }
        if ch == '"' {
            in_string = !in_string;
            continue;
        }
        if !in_string {
            match ch {
                '{' => depth += 1,
                '}' => depth -= 1,
                _ => {}
            }
        }
    }
    depth == 0
}

/// Formats a single definition (which may span multiple input lines).
fn format_definition(lines: &[String], base_indent: &str, max_width: usize) -> String {
    // Join into a single logical line, normalizing whitespace
    let joined = join_definition_lines(lines);

    // Fix tag spacing: `:tag{` → `:tag {`
    let fixed = fix_tag_spacing(&joined);

    // Normalize trailing commas before `}` for single-line form
    let normalized = normalize_trailing_commas(&fixed);

    // Try single-line first
    let single_line = format!("{base_indent}{normalized}");
    if single_line.len() <= max_width {
        return single_line;
    }

    // Try expanding blocks
    if let Some(result) = try_expand_blocks(&fixed, base_indent, max_width) {
        return result;
    }

    // Fall back to breaking after `=` and annotations
    if let Some(result) = try_break_at_equals(&fixed, base_indent, max_width) {
        return result;
    }

    // If nothing works, return the single line (it has an unavoidably long string)
    single_line
}

/// Joins multi-line definition text into a single logical line.
fn join_definition_lines(lines: &[String]) -> String {
    let mut parts = Vec::new();
    for line in lines {
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            parts.push(trimmed);
        }
    }
    parts.join(" ")
}

/// Fixes tag spacing: `:tag{` → `:tag {` but not inside strings.
fn fix_tag_spacing(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let chars: Vec<char> = input.chars().collect();
    let len = chars.len();
    let mut i = 0;
    let mut in_string = false;

    while i < len {
        let ch = chars[i];

        if ch == '\\' && in_string && i + 1 < len {
            result.push(ch);
            result.push(chars[i + 1]);
            i += 2;
            continue;
        }

        if ch == '"' {
            in_string = !in_string;
            result.push(ch);
            i += 1;
            continue;
        }

        if !in_string && ch == ':' && i + 1 < len && chars[i + 1].is_alphanumeric() {
            // Found `:tag` — scan for the tag name and check what follows
            result.push(ch);
            i += 1;
            let tag_start = i;
            while i < len && (chars[i].is_alphanumeric() || chars[i] == '_') {
                result.push(chars[i]);
                i += 1;
            }
            // If next char is `{` with no space, add space
            if i < len && chars[i] == '{' && i > tag_start {
                result.push(' ');
            }
            continue;
        }

        result.push(ch);
        i += 1;
    }

    result
}

/// Removes trailing commas before `}` outside of strings for single-line
/// formatting.
fn normalize_trailing_commas(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let chars: Vec<char> = input.chars().collect();
    let len = chars.len();
    let mut i = 0;
    let mut in_string = false;

    while i < len {
        let ch = chars[i];

        if ch == '\\' && in_string && i + 1 < len {
            result.push(ch);
            result.push(chars[i + 1]);
            i += 2;
            continue;
        }

        if ch == '"' {
            in_string = !in_string;
            result.push(ch);
            i += 1;
            continue;
        }

        if !in_string && ch == ',' {
            // Look ahead past whitespace for `}`
            let mut j = i + 1;
            while j < len && chars[j] == ' ' {
                j += 1;
            }
            if j < len && chars[j] == '}' {
                // Skip the comma (and any spaces between comma and `}`)
                // but add a single space before `}`
                result.push(' ');
                i = j;
                continue;
            }
        }

        result.push(ch);
        i += 1;
    }

    result
}

/// Tries to expand a block-style definition to multi-line.
fn try_expand_blocks(definition: &str, base_indent: &str, max_width: usize) -> Option<String> {
    // Find the outermost structural `{` and `}`
    let (prefix, block_content, suffix) = find_outermost_block(definition)?;

    let indent = format!("{base_indent}    ");

    // Split block content by structural commas
    let entries = split_by_structural_commas(&block_content);
    if entries.is_empty() {
        return None;
    }

    // Format entries, potentially expanding nested blocks
    let mut formatted_entries = Vec::new();
    for entry in &entries {
        let trimmed = entry.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Check if this entry itself contains a nested block that needs expansion
        let entry_line = format!("{indent}{trimmed},");
        if entry_line.len() > max_width {
            if let Some(expanded) = try_expand_nested_entry(trimmed, &indent, max_width) {
                formatted_entries.push(expanded);
                continue;
            }
        }
        formatted_entries.push(format!("{indent}{trimmed},"));
    }

    let mut result = format!("{base_indent}{prefix} {{\n");
    for entry in &formatted_entries {
        result.push_str(entry);
        result.push('\n');
    }
    result.push_str(&format!("{base_indent}}}{suffix}"));

    Some(result)
}

/// Tries to expand a nested block within a variant entry.
fn try_expand_nested_entry(entry: &str, indent: &str, max_width: usize) -> Option<String> {
    // Look for a nested block like `key: :match($p) { ... }`
    let (entry_prefix, block_content, entry_suffix) = find_outermost_block(entry)?;

    let nested_indent = format!("{indent}    ");
    let nested_entries = split_by_structural_commas(&block_content);
    if nested_entries.is_empty() {
        return None;
    }

    // Try single-line nested block first
    let single_line = format!("{indent}{entry},");
    if single_line.len() <= max_width {
        return Some(single_line);
    }

    let mut result = format!("{indent}{entry_prefix} {{\n");
    for nested in &nested_entries {
        let trimmed = nested.trim();
        if !trimmed.is_empty() {
            result.push_str(&format!("{nested_indent}{trimmed},\n"));
        }
    }
    result.push_str(&format!("{indent}}}{entry_suffix},"));

    // Check if the result actually fits better
    let max_line = result.lines().map(str::len).max().unwrap_or(0);
    if max_line <= max_width {
        Some(result)
    } else {
        // Even expanded doesn't fit, return it anyway as it's still better
        Some(result)
    }
}

/// Finds the outermost structural `{` `}` block in a definition.
/// Returns (prefix, block_content, suffix) or None.
fn find_outermost_block(input: &str) -> Option<(String, String, String)> {
    let chars: Vec<char> = input.chars().collect();
    let len = chars.len();
    let mut in_string = false;
    let mut i = 0;
    let mut block_start = None;

    // Find the first structural `{`
    while i < len {
        let ch = chars[i];
        if ch == '\\' && in_string && i + 1 < len {
            i += 2;
            continue;
        }
        if ch == '"' {
            in_string = !in_string;
            i += 1;
            continue;
        }
        if !in_string && ch == '{' {
            block_start = Some(i);
            break;
        }
        i += 1;
    }

    let start = block_start?;

    // Find matching `}`
    let mut depth = 0i32;
    in_string = false;
    let mut end = None;
    let mut j = start;
    while j < len {
        let ch = chars[j];
        if ch == '\\' && in_string && j + 1 < len {
            j += 2;
            continue;
        }
        if ch == '"' {
            in_string = !in_string;
            j += 1;
            continue;
        }
        if !in_string {
            if ch == '{' {
                depth += 1;
            } else if ch == '}' {
                depth -= 1;
                if depth == 0 {
                    end = Some(j);
                    break;
                }
            }
        }
        j += 1;
    }

    let end = end?;

    let prefix: String = chars[..start]
        .iter()
        .collect::<String>()
        .trim_end()
        .to_string();
    let content: String = chars[start + 1..end].iter().collect();
    let suffix: String = chars[end + 1..].iter().collect();

    Some((prefix, content, suffix))
}

/// Splits content by commas at depth 0, respecting strings and nested braces.
fn split_by_structural_commas(content: &str) -> Vec<String> {
    let mut entries = Vec::new();
    let mut current = String::new();
    let mut depth = 0i32;
    let mut in_string = false;
    let chars: Vec<char> = content.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        let ch = chars[i];

        if ch == '\\' && in_string && i + 1 < len {
            current.push(ch);
            current.push(chars[i + 1]);
            i += 2;
            continue;
        }

        if ch == '"' {
            in_string = !in_string;
            current.push(ch);
            i += 1;
            continue;
        }

        if !in_string {
            if ch == '{' {
                depth += 1;
                current.push(ch);
            } else if ch == '}' {
                depth -= 1;
                current.push(ch);
            } else if ch == ',' && depth == 0 {
                entries.push(current.trim().to_string());
                current = String::new();
            } else {
                current.push(ch);
            }
        } else {
            current.push(ch);
        }
        i += 1;
    }

    let remaining = current.trim().to_string();
    if !remaining.is_empty() {
        entries.push(remaining);
    }

    entries
}

/// Tries to break a definition at the `=` sign when it has no block body.
fn try_break_at_equals(definition: &str, base_indent: &str, max_width: usize) -> Option<String> {
    // Find `=` outside strings
    let chars: Vec<char> = definition.chars().collect();
    let mut in_string = false;
    let mut eq_pos = None;
    let mut skip_next = false;

    for (i, &ch) in chars.iter().enumerate() {
        if skip_next {
            skip_next = false;
            continue;
        }
        if ch == '\\' && in_string {
            skip_next = true;
            continue;
        }
        if ch == '"' {
            in_string = !in_string;
            continue;
        }
        if !in_string && ch == '=' {
            eq_pos = Some(i);
            break;
        }
    }

    let eq_pos = eq_pos?;
    let lhs: String = chars[..eq_pos]
        .iter()
        .collect::<String>()
        .trim_end()
        .to_string();
    let rhs: String = chars[eq_pos + 1..]
        .iter()
        .collect::<String>()
        .trim()
        .to_string();

    // Check if RHS has annotation prefixes like `:from($t)`, `:match($n)`
    // Try to keep annotations on the first line
    let (annotations, body) = split_annotations(&rhs);

    if annotations.is_empty() {
        let indent = format!("{base_indent}    ");
        let result = format!("{base_indent}{lhs} =\n{indent}{rhs}");
        let max_line = result.lines().map(str::len).max().unwrap_or(0);
        if max_line <= max_width {
            return Some(result);
        }
    } else {
        let indent = format!("{base_indent}    ");
        let first_line = format!("{base_indent}{lhs} = {annotations}");
        if first_line.len() <= max_width {
            let result = format!("{first_line}\n{indent}{body}");
            let max_line = result.lines().map(str::len).max().unwrap_or(0);
            if max_line <= max_width {
                return Some(result);
            }
        }

        // Fall back to break after `=`
        let result = format!("{base_indent}{lhs} =\n{indent}{annotations}\n{indent}{body}");
        let max_line = result.lines().map(str::len).max().unwrap_or(0);
        if max_line <= max_width {
            return Some(result);
        }
    }

    None
}

/// Splits RHS into annotation prefixes (`:tag`, `:from($x)`, `:match($x)`)
/// and the remaining body.
fn split_annotations(rhs: &str) -> (String, String) {
    let trimmed = rhs.trim();
    let mut annotations = String::new();
    let mut remaining = trimmed;

    loop {
        let r = remaining.trim_start();
        if !r.starts_with(':') {
            break;
        }

        // Scan past the tag name
        let chars: Vec<char> = r.chars().collect();
        let mut i = 1; // skip ':'
        while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
            i += 1;
        }

        // Check for parenthesized arguments
        if i < chars.len() && chars[i] == '(' {
            let mut depth = 0;
            while i < chars.len() {
                if chars[i] == '(' {
                    depth += 1;
                } else if chars[i] == ')' {
                    depth -= 1;
                    if depth == 0 {
                        i += 1;
                        break;
                    }
                }
                i += 1;
            }
        }

        let annotation: String = chars[..i].iter().collect();
        if !annotations.is_empty() {
            annotations.push(' ');
        }
        annotations.push_str(&annotation);
        remaining = &r[i..];

        // If next is a block `{`, it's part of the body not annotations
        let next = remaining.trim_start();
        if next.starts_with('{') || next.starts_with('"') {
            break;
        }
    }

    (annotations, remaining.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_definition() {
        let input = "name = \"value\";";
        assert_eq!(format_file(input, 100), "name = \"value\";\n");
    }

    #[test]
    fn test_comment_preserved() {
        let input = "// This is a comment\nname = \"value\";";
        assert_eq!(
            format_file(input, 100),
            "// This is a comment\nname = \"value\";\n"
        );
    }

    #[test]
    fn test_blank_lines_preserved() {
        let input = "a = \"1\";\n\n\nb = \"2\";";
        assert_eq!(format_file(input, 100), "a = \"1\";\n\n\nb = \"2\";\n");
    }

    #[test]
    fn test_tag_spacing_fix() {
        let input = "card = :a{ one: \"card\", other: \"cards\" };";
        assert_eq!(
            format_file(input, 100),
            "card = :a { one: \"card\", other: \"cards\" };\n"
        );
    }

    #[test]
    fn test_tag_spacing_not_in_strings() {
        let input = "name = \"text :a{not a tag}\";";
        assert_eq!(format_file(input, 100), "name = \"text :a{not a tag}\";\n");
    }

    #[test]
    fn test_already_spaced_tag() {
        let input = "card = :a { one: \"card\", other: \"cards\" };";
        assert_eq!(
            format_file(input, 100),
            "card = :a { one: \"card\", other: \"cards\" };\n"
        );
    }

    #[test]
    fn test_expand_long_block() {
        let input = "agent = :masc :anim { nom: \"A\", *acc: \"B\", gen: \"C\", ins: \"D\", inf: \"E\", nom_pl: \"F\", one: \"G\", other: \"H\" };";
        let result = format_file(input, 80);
        assert!(result.contains("{\n"));
        assert!(result.contains("    nom: \"A\",\n"));
        assert!(result.ends_with("};\n"));
    }

    #[test]
    fn test_short_block_stays_single_line() {
        let input = "card = :a { one: \"card\", other: \"cards\" };";
        assert_eq!(
            format_file(input, 100),
            "card = :a { one: \"card\", other: \"cards\" };\n"
        );
    }

    #[test]
    fn test_multiline_input_joined() {
        let input = "cards($n) = :match($n) {\n    1: \"a card\",\n    *other: \"{$n} cards\",\n};";
        let result = format_file(input, 200);
        // Should join to single line when it fits
        assert_eq!(
            result,
            "cards($n) = :match($n) { 1: \"a card\", *other: \"{$n} cards\" };\n"
        );
    }

    #[test]
    fn test_multiline_block_preserved_when_long() {
        let input = "cards($n) = :match($n) { 1: \"a card\", *other: \"{$n} cards\" };";
        let result = format_file(input, 40);
        assert!(result.contains("{\n"));
    }

    #[test]
    fn test_base_indent() {
        let input = "name = \"value\";";
        assert_eq!(
            format_definitions(input, "    ", 100),
            "    name = \"value\";\n"
        );
    }

    #[test]
    fn test_base_indent_block_expansion() {
        let input = "agent = :masc :anim { nom: \"A\", *acc: \"B\", gen: \"C\", ins: \"D\", inf: \"E\", nom_pl: \"F\", one: \"G\", other: \"H\" };";
        let result = format_definitions(input, "    ", 80);
        assert!(result.starts_with("    agent"));
        assert!(result.contains("        nom: \"A\",\n"));
    }

    #[test]
    fn test_format_file_idempotent() {
        let input = "card = :a { one: \"card\", other: \"cards\" };\n\nagent = :an { one: \"Agent\", other: \"Agents\" };\n";
        let result = format_file(input, 100);
        let result2 = format_file(&result, 100);
        assert_eq!(result, result2);
    }

    #[test]
    fn test_trailing_comma_in_multiline_block() {
        let input = "x = :tag { a: \"1\", b: \"2\" };";
        let result = format_file(input, 20);
        // Each entry should have trailing comma in multi-line
        for line in result.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("a:") || trimmed.starts_with("b:") {
                assert!(
                    trimmed.ends_with(','),
                    "Line should end with comma: {trimmed}"
                );
            }
        }
    }

    #[test]
    fn test_split_annotations() {
        let (ann, body) = split_annotations(":from($s) :match($n) { 1: \"x\" }");
        assert_eq!(ann, ":from($s) :match($n)");
        assert_eq!(body, "{ 1: \"x\" }");
    }

    #[test]
    fn test_multiple_tags_before_block() {
        let input = "card = :fem :inan { nom: \"a\", *acc: \"b\" };";
        let result = format_file(input, 100);
        assert_eq!(result, "card = :fem :inan { nom: \"a\", *acc: \"b\" };\n");
    }
}
