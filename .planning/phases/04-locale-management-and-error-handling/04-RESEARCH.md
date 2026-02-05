# Phase 4: Locale Management and Error Handling - Research

**Researched:** 2026-02-04
**Domain:** User-facing Locale API with translation loading, language management, and error handling
**Confidence:** HIGH

## Summary

This phase implements the user-facing `Locale` struct that wraps the existing interpreter infrastructure (PhraseRegistry, TransformRegistry) into a cohesive API for language management. The implementation provides translation loading from files and strings, language switching, hot-reloading capabilities, and comprehensive error types with helpful error messages.

The standard approach follows the established project patterns: `bon::Builder` derive for builder pattern (per CONTEXT.md decision), `thiserror::Error` derive for error types (already in project), and internal storage using `HashMap` for path-to-language mappings. The `strsim` crate provides "did you mean" suggestions for `MissingVariant` errors using Levenshtein distance.

Key constraints from CONTEXT.md define the API shape: builder pattern for construction, owned interpreter (not borrowed), mutable language change via `&mut self`, replace-not-merge semantics for translation loading, configurable single-step fallback (disabled by default), and `LoadError` that includes file path with line/column information.

**Primary recommendation:** Implement `Locale` as a wrapper struct with `bon::Builder` derive, store loaded paths in `HashMap<String, PathBuf>` for reload support, add `strsim` for "did you mean" suggestions in `MissingVariant` errors, and create `LoadError` as a new error type distinct from `EvalError`.

## Standard Stack

The established libraries/tools for this domain:

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| bon | 3.8 | Builder pattern derive | Already in project, established pattern |
| thiserror | 2.0 | Error type derive | Already in project, standard for library errors |
| strsim | 0.11 | "Did you mean" suggestions | De facto standard for edit distance in Rust |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| std::fs | stable | File I/O for load_translations | Reading translation files |
| std::path | stable | PathBuf for file path storage | Storing paths for reload_translations |
| std::collections::HashMap | stable | Language -> path mapping | Internal reload support |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| strsim | textdistance | textdistance has more algorithms but strsim is more widely used and simpler |
| HashMap for paths | BTreeMap | BTreeMap provides ordering but not needed here |
| Owned interpreter | Borrowed (&'a) | Borrowed would require lifetime parameters throughout; owned is simpler per CONTEXT.md |

**Installation:**
```bash
cargo add strsim@0.11
```

## Architecture Patterns

### Recommended Project Structure
```
crates/rlf/src/
  interpreter/
    mod.rs           # Add Locale export
    error.rs         # Extend with LoadError
    registry.rs      # Existing PhraseRegistry
    locale.rs        # NEW: Locale struct and builder
```

### Pattern 1: Locale Struct with Builder
**What:** `Locale` struct owns PhraseRegistry and TransformRegistry, built via `#[derive(Builder)]`.
**When to use:** All user-facing locale management.
**Example:**
```rust
// Source: CONTEXT.md decisions - builder pattern, owned interpreter
use bon::Builder;

#[derive(Builder)]
pub struct Locale {
    /// Current language code
    #[builder(default = "en".to_string())]
    language: String,

    /// Optional fallback language (single step only)
    #[builder(default)]
    fallback_language: Option<String>,

    /// Phrase storage (internal)
    #[builder(skip)]
    registry: PhraseRegistry,

    /// Transform registry (internal)
    #[builder(skip)]
    transforms: TransformRegistry,

    /// Paths for hot-reload support: language -> PathBuf
    #[builder(skip)]
    loaded_paths: HashMap<String, PathBuf>,
}

// Usage per CONTEXT.md:
let locale = Locale::builder()
    .language("ru")
    .fallback_language("en")
    .build();
```

### Pattern 2: Translation Loading with Path Tracking
**What:** `load_translations` reads file and stores path for later `reload_translations`.
**When to use:** File-based translation loading.
**Example:**
```rust
// Source: CONTEXT.md - loading same language twice replaces, reload remembers path
impl Locale {
    pub fn load_translations(
        &mut self,
        language: &str,
        path: impl AsRef<Path>,
    ) -> Result<usize, LoadError> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path).map_err(|e| LoadError::Io {
            path: path.to_path_buf(),
            source: e,
        })?;

        // Replace existing phrases for this language
        self.clear_language(language);

        let count = self.load_translations_str(language, &content)?;

        // Store path for reload support
        self.loaded_paths.insert(language.to_string(), path.to_path_buf());

        Ok(count)
    }

    pub fn reload_translations(&mut self, language: &str) -> Result<usize, LoadError> {
        let path = self.loaded_paths.get(language).cloned().ok_or_else(|| {
            LoadError::NoPathForReload {
                language: language.to_string(),
            }
        })?;

        self.load_translations(language, path)
    }
}
```

### Pattern 3: LoadError with File Context
**What:** `LoadError` includes PathBuf, line, column for parse failures.
**When to use:** All translation loading errors.
**Example:**
```rust
// Source: CONTEXT.md - LoadError includes original file path
use thiserror::Error;
use std::path::PathBuf;

#[derive(Debug, Error)]
pub enum LoadError {
    #[error("failed to read {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("{path}:{line}:{column}: {message}")]
    Parse {
        path: PathBuf,
        line: usize,
        column: usize,
        message: String,
    },

    #[error("cannot reload '{language}': loaded from string, not file")]
    NoPathForReload {
        language: String,
    },
}
```

### Pattern 4: "Did You Mean" Suggestions
**What:** Use `strsim::levenshtein` to suggest similar variant keys.
**When to use:** `MissingVariant` error enhancement.
**Example:**
```rust
// Source: strsim crate docs, CONTEXT.md - "did you mean" suggestions
use strsim::levenshtein;

fn suggest_similar(target: &str, available: &[String], max_suggestions: usize) -> Vec<String> {
    let mut scored: Vec<_> = available
        .iter()
        .map(|s| (s.clone(), levenshtein(target, s)))
        .filter(|(_, dist)| *dist <= 3) // Only suggest if reasonably close
        .collect();

    scored.sort_by_key(|(_, dist)| *dist);
    scored.truncate(max_suggestions);
    scored.into_iter().map(|(s, _)| s).collect()
}

// Enhanced MissingVariant error:
#[derive(Debug, Error)]
pub enum EvalError {
    #[error("missing variant '{key}' in phrase '{phrase}', available: {available}{suggestions}",
        available = available.join(", "),
        suggestions = format_suggestions(suggestions)
    )]
    MissingVariant {
        phrase: String,
        key: String,
        available: Vec<String>,
        suggestions: Vec<String>, // NEW: "did you mean" suggestions
    },
    // ... other variants
}

fn format_suggestions(suggestions: &[String]) -> String {
    if suggestions.is_empty() {
        String::new()
    } else {
        format!("; did you mean: {}?", suggestions.join(", "))
    }
}
```

### Pattern 5: Language Fallback (Configurable, Off by Default)
**What:** Try fallback language if phrase not found in primary.
**When to use:** When `fallback_language` is configured.
**Example:**
```rust
// Source: CONTEXT.md - configurable fallback, off by default, single step only
impl Locale {
    pub fn get_phrase(&self, name: &str) -> Result<Phrase, EvalError> {
        // Try primary language first
        match self.registry.get_phrase(&self.language, name) {
            Ok(phrase) => Ok(phrase),
            Err(e) => {
                // Try fallback if configured
                if let Some(fallback) = &self.fallback_language {
                    self.registry.get_phrase(fallback, name)
                } else {
                    Err(e) // No fallback - return original error
                }
            }
        }
    }
}
```

### Anti-Patterns to Avoid
- **Silent fallback by default:** CONTEXT.md explicitly says no fallback by default. Missing translations must error.
- **Merging translations on reload:** CONTEXT.md says loading same language twice replaces, not merges.
- **Fallback chains:** Only single-step fallback is supported. Don't implement `ru -> en -> root`.
- **Storing paths for string-loaded content:** If loaded via `load_translations_str`, reload should error.

## Don't Hand-Roll

Problems that look simple but have existing solutions:

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Edit distance for suggestions | Custom Levenshtein | `strsim::levenshtein` | Tested, optimized, handles edge cases |
| Error Display formatting | Manual `impl Display` | `thiserror::Error` derive | Less boilerplate, consistent formatting |
| Builder pattern | Manual builder struct | `bon::Builder` derive | Typestate pattern ensures required fields |

**Key insight:** The "did you mean" feature seems simple but requires careful threshold tuning. Using `strsim` with a max distance of 2-3 provides good suggestions without noise from unrelated strings.

## Common Pitfalls

### Pitfall 1: Forgetting to Clear Before Replace
**What goes wrong:** Loading translations for same language twice accumulates phrases instead of replacing.
**Why it happens:** Not clearing existing phrases for the language before loading new ones.
**How to avoid:** Implement `clear_language(lang)` that removes all phrases for a language from registry, call before loading.
**Warning signs:** Phrase count keeps growing on reload.

### Pitfall 2: Reload Without Path
**What goes wrong:** `reload_translations("en")` called but English was loaded via `load_translations_str`.
**Why it happens:** String loading doesn't store a path to reload from.
**How to avoid:** Return `LoadError::NoPathForReload` when attempting to reload string-loaded content.
**Warning signs:** Panic or silent failure on reload.

### Pitfall 3: Fallback Masking Development Errors
**What goes wrong:** Missing Russian translation returns English silently, noticed only in production.
**Why it happens:** Fallback enabled during development masks incomplete translations.
**How to avoid:** Default to no fallback. Fallback is opt-in for production edge cases only.
**Warning signs:** Tests pass but production users see wrong language.

### Pitfall 4: LoadError Without Context
**What goes wrong:** Error says "parse error at line 5" but doesn't say which file.
**Why it happens:** ParseError from parser doesn't include file path.
**How to avoid:** LoadError wraps ParseError and adds path context.
**Warning signs:** Users can't find error location in multi-file translation setups.

### Pitfall 5: Suggestion Noise
**What goes wrong:** "Did you mean" suggests unrelated strings (e.g., "one" suggests "other" which is distance 3).
**Why it happens:** Too lenient threshold for edit distance.
**How to avoid:** Use max distance of 2 for short strings, 3 for longer ones. Filter suggestions that don't share first character.
**Warning signs:** Suggestions are unhelpful or confusing.

## Code Examples

Verified patterns from official sources and established project patterns:

### Complete Locale Struct
```rust
// Source: Synthesized from CONTEXT.md decisions and bon/thiserror docs
use bon::Builder;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::interpreter::{EvalContext, EvalError, PhraseRegistry, TransformRegistry};
use crate::types::{Phrase, Value};

/// User-facing locale management for RLF translations.
#[derive(Builder)]
pub struct Locale {
    /// Current language code (e.g., "en", "ru", "de")
    #[builder(default = "en".to_string())]
    language: String,

    /// Optional fallback language for missing phrases
    #[builder(default)]
    fallback_language: Option<String>,

    /// Internal phrase storage
    #[builder(skip)]
    registry: PhraseRegistry,

    /// Internal transform storage
    #[builder(skip)]
    transforms: TransformRegistry,

    /// File paths for hot-reload support
    #[builder(skip)]
    loaded_paths: HashMap<String, PathBuf>,
}

impl Locale {
    /// Get current language code.
    pub fn language(&self) -> &str {
        &self.language
    }

    /// Change current language.
    pub fn set_language(&mut self, language: impl Into<String>) {
        self.language = language.into();
    }

    /// Access the underlying interpreter (read-only).
    pub fn interpreter(&self) -> &PhraseRegistry {
        &self.registry
    }

    /// Access the underlying interpreter (mutable, for loading).
    pub fn interpreter_mut(&mut self) -> &mut PhraseRegistry {
        &mut self.registry
    }
}
```

### Load Error Implementation
```rust
// Source: thiserror 2.0 docs, CONTEXT.md error requirements
use thiserror::Error;
use std::path::PathBuf;

/// Errors that occur during translation loading.
#[derive(Debug, Error)]
pub enum LoadError {
    /// File I/O error
    #[error("failed to read '{path}': {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// Parse error with location
    #[error("{path}:{line}:{column}: {message}")]
    Parse {
        path: PathBuf,
        line: usize,
        column: usize,
        message: String,
    },

    /// Attempted reload on string-loaded content
    #[error("cannot reload '{language}': was loaded from string, not file")]
    NoPathForReload {
        language: String,
    },
}
```

### Enhanced MissingVariant with Suggestions
```rust
// Source: strsim 0.11 docs, CONTEXT.md error requirements
use strsim::levenshtein;

/// Compute "did you mean" suggestions for a key.
pub fn compute_suggestions(target: &str, available: &[String]) -> Vec<String> {
    let max_distance = if target.len() <= 3 { 1 } else { 2 };

    let mut scored: Vec<_> = available
        .iter()
        .filter_map(|candidate| {
            let dist = levenshtein(target, candidate);
            if dist <= max_distance && dist > 0 {
                Some((candidate.clone(), dist))
            } else {
                None
            }
        })
        .collect();

    scored.sort_by_key(|(_, dist)| *dist);
    scored.truncate(3); // Max 3 suggestions
    scored.into_iter().map(|(s, _)| s).collect()
}
```

### Translation Loading Implementation
```rust
// Source: CONTEXT.md loading decisions
impl Locale {
    /// Load translations from a file.
    ///
    /// Loading the same language twice replaces previous phrases.
    /// The path is stored for `reload_translations()` support.
    pub fn load_translations(
        &mut self,
        language: &str,
        path: impl AsRef<Path>,
    ) -> Result<usize, LoadError> {
        let path = path.as_ref();

        // Read file
        let content = std::fs::read_to_string(path).map_err(|e| LoadError::Io {
            path: path.to_path_buf(),
            source: e,
        })?;

        // Load and track path
        let count = self.load_translations_str_internal(language, &content, Some(path))?;
        self.loaded_paths.insert(language.to_string(), path.to_path_buf());

        Ok(count)
    }

    /// Load translations from a string.
    ///
    /// Cannot be reloaded via `reload_translations()`.
    pub fn load_translations_str(
        &mut self,
        language: &str,
        content: &str,
    ) -> Result<usize, LoadError> {
        self.load_translations_str_internal(language, content, None)
    }

    /// Hot-reload translations from original file path.
    pub fn reload_translations(&mut self, language: &str) -> Result<usize, LoadError> {
        let path = self.loaded_paths.get(language).cloned().ok_or_else(|| {
            LoadError::NoPathForReload {
                language: language.to_string(),
            }
        })?;

        self.load_translations(language, path)
    }

    fn load_translations_str_internal(
        &mut self,
        language: &str,
        content: &str,
        path: Option<&Path>,
    ) -> Result<usize, LoadError> {
        // Parse content
        let definitions = crate::parser::parse_file(content).map_err(|e| match e {
            crate::parser::ParseError::Syntax { line, column, message } => LoadError::Parse {
                path: path.map(|p| p.to_path_buf()).unwrap_or_default(),
                line,
                column,
                message,
            },
            crate::parser::ParseError::UnexpectedEof { line, column } => LoadError::Parse {
                path: path.map(|p| p.to_path_buf()).unwrap_or_default(),
                line,
                column,
                message: "unexpected end of file".to_string(),
            },
            crate::parser::ParseError::InvalidUtf8 => LoadError::Parse {
                path: path.map(|p| p.to_path_buf()).unwrap_or_default(),
                line: 0,
                column: 0,
                message: "invalid UTF-8".to_string(),
            },
        })?;

        // Insert phrases
        let count = definitions.len();
        for def in definitions {
            self.registry.insert(def).map_err(|e| LoadError::Parse {
                path: path.map(|p| p.to_path_buf()).unwrap_or_default(),
                line: 0,
                column: 0,
                message: format!("{}", e),
            })?;
        }

        Ok(count)
    }
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Manual builder structs | `bon::Builder` derive | bon 3.x (2024) | Typestate pattern ensures compile-time safety |
| Manual Display for errors | `thiserror::Error` derive | thiserror 1.x stable | Less boilerplate, source chaining |
| No suggestion support | `strsim` for edit distance | N/A | Better error messages |

**Deprecated/outdated:**
- `derive_builder` crate: `bon` is more ergonomic and actively maintained
- Custom error formatting: Use `thiserror` consistently throughout project

## Open Questions

Things that couldn't be fully resolved:

1. **Per-Language Registry vs Single Registry**
   - What we know: CONTEXT.md doesn't specify internal storage strategy
   - What's unclear: Should each language have its own PhraseRegistry or share one with language prefixing?
   - Recommendation: Single PhraseRegistry per Locale. Language is passed to evaluation methods, not stored per-phrase. This matches existing API (`get_phrase(&self, lang: &str, name: &str)`).

2. **Cache Parsed Templates on Load**
   - What we know: CONTEXT.md marks this as Claude's discretion
   - What's unclear: Parse once on load, or parse on each evaluation?
   - Recommendation: Parse on load (current behavior via `PhraseRegistry::load_phrases`). Templates are already parsed into AST on load. No change needed.

3. **Thread Safety for Locale**
   - What we know: Not specified in CONTEXT.md
   - What's unclear: Should Locale be `Send + Sync`?
   - Recommendation: Start without explicit thread safety. `Locale` contains `HashMap` which is not `Sync` by default. Users can wrap in `Arc<Mutex<>>` if needed. Document this.

## Sources

### Primary (HIGH confidence)
- [thiserror 2.0 docs.rs](https://docs.rs/thiserror/2.0.18/thiserror/) - Error derive macros
- [bon crate docs](https://bon-rs.com/reference/builder) - Builder pattern with `#[builder(default)]`
- [strsim 0.11 docs.rs](https://docs.rs/strsim/0.11.1/strsim/) - Levenshtein distance API
- `.planning/phases/04-locale-management-and-error-handling/04-CONTEXT.md` - User decisions constraining implementation

### Secondary (MEDIUM confidence)
- [GreptimeDB Error Handling](https://greptime.com/blogs/2024-05-07-error-rust) - Best practices for library errors
- [WebSearch: Rust "did you mean" suggestions](https://medium.com/@ben.lafferty/did-you-mean-rust-75ca22f536b0) - Algorithm selection guidance

### Tertiary (LOW confidence)
- None - all findings verified with official documentation

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - All libraries already in project or well-established
- Architecture: HIGH - Patterns derived from CONTEXT.md decisions and existing codebase
- Pitfalls: HIGH - Based on common localization library issues and explicit CONTEXT.md constraints

**Research date:** 2026-02-04
**Valid until:** 2026-03-04 (30 days - stable domain, well-specified by context decisions)
