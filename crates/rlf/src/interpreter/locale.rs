//! Locale management for RLF translations.
//!
//! The Locale struct provides the user-facing API for managing language selection,
//! loading translations, and accessing phrases.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use bon::Builder;

use crate::interpreter::error::LoadError;
use crate::interpreter::registry::PhraseRegistry;
use crate::interpreter::transforms::TransformRegistry;
use crate::interpreter::{EvalContext, EvalError, eval_phrase_def, eval_template};
use crate::parser::{ParseError, parse_file, parse_template};
use crate::types::{Phrase, Value};

/// User-facing locale management for RLF translations.
///
/// Locale owns per-language phrase registries and a shared transform registry.
/// This design provides:
/// - Language-scoped phrase storage (each language has its own registry)
/// - Shared transforms across all languages
/// - Clean replacement semantics (loading same language replaces all phrases)
///
/// # Example
///
/// ```
/// use rlf::Locale;
///
/// let mut locale = Locale::builder()
///     .language("en")
///     .build();
///
/// // Load translations (would normally read from file)
/// locale.load_translations_str("en", r#"hello = "Hello!";"#).unwrap();
///
/// assert_eq!(locale.language(), "en");
/// ```
#[derive(Builder)]
#[builder(on(String, into))]
pub struct Locale {
    /// Current language code (e.g., "en", "ru", "de").
    #[builder(default = "en".to_string())]
    language: String,

    /// Optional fallback language for missing phrases.
    /// If set, phrases not found in the primary language will be looked up here.
    /// Default is None (no fallback - missing phrases return error).
    fallback_language: Option<String>,

    /// Per-language phrase registries.
    /// Each language has its own PhraseRegistry, enabling:
    /// - Clean "replace" semantics when reloading a language
    /// - Language-scoped phrase lookup
    /// - Independent phrase storage per language
    #[builder(skip)]
    registries: HashMap<String, PhraseRegistry>,

    /// Shared transform registry for all languages.
    /// Transforms (like UPPERCASE, lowercase) are language-independent.
    #[builder(skip)]
    transforms: TransformRegistry,

    /// File paths for hot-reload support: language -> PathBuf.
    /// Only populated for file-loaded translations, not string-loaded.
    #[builder(skip)]
    loaded_paths: HashMap<String, PathBuf>,
}

impl Default for Locale {
    fn default() -> Self {
        Locale::builder().build()
    }
}

impl Locale {
    /// Create a new Locale with default settings (English, no fallback).
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new Locale with the specified language.
    pub fn with_language(language: impl Into<String>) -> Self {
        Locale::builder().language(language.into()).build()
    }

    // =========================================================================
    // Language Management
    // =========================================================================

    /// Get the current language code.
    pub fn language(&self) -> &str {
        &self.language
    }

    /// Change the current language.
    ///
    /// This does not reload translations - the new language must already have
    /// translations loaded via `load_translations` or `load_translations_str`.
    pub fn set_language(&mut self, language: impl Into<String>) {
        self.language = language.into();
    }

    // =========================================================================
    // Registry Access
    // =========================================================================

    /// Get the phrase registry for a specific language (read-only).
    ///
    /// Returns None if no translations have been loaded for that language.
    pub fn registry_for(&self, language: &str) -> Option<&PhraseRegistry> {
        self.registries.get(language)
    }

    /// Get the phrase registry for the current language (read-only).
    pub fn registry(&self) -> Option<&PhraseRegistry> {
        self.registries.get(&self.language)
    }

    /// Get the shared transform registry (read-only).
    pub fn transforms(&self) -> &TransformRegistry {
        &self.transforms
    }

    /// Get the shared transform registry (mutable) for registering custom transforms.
    pub fn transforms_mut(&mut self) -> &mut TransformRegistry {
        &mut self.transforms
    }

    /// Get or create the phrase registry for a language (mutable).
    fn registry_for_mut(&mut self, language: &str) -> &mut PhraseRegistry {
        self.registries.entry(language.to_string()).or_default()
    }

    /// Clear all phrases for a specific language.
    ///
    /// This is called internally before loading to implement "replace" semantics.
    fn clear_language(&mut self, language: &str) {
        self.registries.remove(language);
    }

    // =========================================================================
    // Translation Loading
    // =========================================================================

    /// Load translations from a file for a specific language.
    ///
    /// The file path is stored for later `reload_translations()` support.
    /// Loading the same language twice **replaces** all previous phrases for that language.
    ///
    /// # Example
    ///
    /// ```ignore
    /// locale.load_translations("ru", "assets/localization/ru.rlf")?;
    /// ```
    pub fn load_translations(
        &mut self,
        language: &str,
        path: impl AsRef<Path>,
    ) -> Result<usize, LoadError> {
        let path = path.as_ref();

        // Read file content
        let content = fs::read_to_string(path).map_err(|e| LoadError::Io {
            path: path.to_path_buf(),
            source: e,
        })?;

        // Clear existing phrases for this language (replace semantics)
        self.clear_language(language);

        // Load via internal method, which handles parsing
        let count = self.load_translations_str_internal(language, &content, Some(path))?;

        // Store path for reload support
        self.loaded_paths
            .insert(language.to_string(), path.to_path_buf());

        Ok(count)
    }

    /// Load translations from a string for a specific language.
    ///
    /// Translations loaded this way cannot be reloaded via `reload_translations()`.
    /// Loading the same language twice **replaces** all previous phrases for that language.
    ///
    /// # Example
    ///
    /// ```
    /// use rlf::Locale;
    ///
    /// let mut locale = Locale::new();
    /// let count = locale.load_translations_str("en", r#"
    ///     hello = "Hello!";
    ///     card = { one: "card", other: "cards" };
    /// "#).unwrap();
    /// assert_eq!(count, 2);
    /// ```
    pub fn load_translations_str(
        &mut self,
        language: &str,
        content: &str,
    ) -> Result<usize, LoadError> {
        // Remove from loaded_paths since this is string-loaded
        self.loaded_paths.remove(language);

        // Clear existing phrases for this language (replace semantics)
        self.clear_language(language);

        self.load_translations_str_internal(language, content, None)
    }

    /// Hot-reload translations from the original file path.
    ///
    /// Returns an error if the translations were loaded from a string
    /// (via `load_translations_str`) rather than a file.
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Initial load
    /// locale.load_translations("ru", "assets/localization/ru.rlf")?;
    ///
    /// // Later, after file is modified
    /// locale.reload_translations("ru")?;
    /// ```
    pub fn reload_translations(&mut self, language: &str) -> Result<usize, LoadError> {
        let path =
            self.loaded_paths
                .get(language)
                .cloned()
                .ok_or_else(|| LoadError::NoPathForReload {
                    language: language.to_string(),
                })?;

        self.load_translations(language, path)
    }

    /// Internal loading implementation.
    fn load_translations_str_internal(
        &mut self,
        language: &str,
        content: &str,
        path: Option<&Path>,
    ) -> Result<usize, LoadError> {
        // Parse the content
        let definitions = parse_file(content).map_err(|e| {
            let default_path = PathBuf::from(format!("<{language}>"));
            let path_buf = path.map(Path::to_path_buf).unwrap_or(default_path);

            match e {
                ParseError::Syntax {
                    line,
                    column,
                    message,
                } => LoadError::Parse {
                    path: path_buf,
                    line,
                    column,
                    message,
                },
                ParseError::UnexpectedEof { line, column } => LoadError::Parse {
                    path: path_buf,
                    line,
                    column,
                    message: "unexpected end of file".to_string(),
                },
                ParseError::InvalidUtf8 => LoadError::Parse {
                    path: path_buf,
                    line: 0,
                    column: 0,
                    message: "invalid UTF-8".to_string(),
                },
            }
        })?;

        // Get or create registry for this language
        let registry = self.registry_for_mut(language);

        // Insert phrases (registry handles collision detection)
        let count = definitions.len();
        for def in definitions {
            registry.insert(def).map_err(|e| {
                let default_path = PathBuf::from(format!("<{language}>"));
                LoadError::Parse {
                    path: path.map(Path::to_path_buf).unwrap_or(default_path),
                    line: 0,
                    column: 0,
                    message: format!("{e}"),
                }
            })?;
        }

        Ok(count)
    }

    // =========================================================================
    // Phrase Evaluation (with fallback support)
    // =========================================================================

    /// Get a parameterless phrase.
    ///
    /// If the phrase is not found in the current language and a fallback
    /// language is configured, the fallback is tried.
    pub fn get_phrase(&self, name: &str) -> Result<Phrase, EvalError> {
        // Try primary language
        match self.get_phrase_for_language(&self.language, name) {
            Ok(phrase) => Ok(phrase),
            Err(e) => {
                // Try fallback if configured
                if let Some(fallback) = &self.fallback_language
                    && fallback != &self.language
                {
                    return self.get_phrase_for_language(fallback, name);
                }
                Err(e)
            }
        }
    }

    /// Get a parameterless phrase for a specific language.
    fn get_phrase_for_language(&self, language: &str, name: &str) -> Result<Phrase, EvalError> {
        let registry = self
            .registries
            .get(language)
            .ok_or_else(|| EvalError::PhraseNotFound {
                name: name.to_string(),
            })?;

        let def = registry
            .get(name)
            .ok_or_else(|| EvalError::PhraseNotFound {
                name: name.to_string(),
            })?;

        if !def.parameters.is_empty() {
            return Err(EvalError::ArgumentCount {
                phrase: name.to_string(),
                expected: def.parameters.len(),
                got: 0,
            });
        }

        let params = HashMap::new();
        let mut ctx = EvalContext::new(&params);
        ctx.push_call(name)?;
        let result = eval_phrase_def(def, &mut ctx, registry, &self.transforms, language)?;
        ctx.pop_call();
        Ok(result)
    }

    /// Call a phrase with arguments.
    ///
    /// If the phrase is not found in the current language and a fallback
    /// language is configured, the fallback is tried.
    pub fn call_phrase(&self, name: &str, args: &[Value]) -> Result<Phrase, EvalError> {
        // Try primary language
        match self.call_phrase_for_language(&self.language, name, args) {
            Ok(phrase) => Ok(phrase),
            Err(e) => {
                // Try fallback if configured
                if let Some(fallback) = &self.fallback_language
                    && fallback != &self.language
                {
                    return self.call_phrase_for_language(fallback, name, args);
                }
                Err(e)
            }
        }
    }

    /// Call a phrase for a specific language.
    fn call_phrase_for_language(
        &self,
        language: &str,
        name: &str,
        args: &[Value],
    ) -> Result<Phrase, EvalError> {
        let registry = self
            .registries
            .get(language)
            .ok_or_else(|| EvalError::PhraseNotFound {
                name: name.to_string(),
            })?;

        let def = registry
            .get(name)
            .ok_or_else(|| EvalError::PhraseNotFound {
                name: name.to_string(),
            })?;

        // Check argument count
        if def.parameters.len() != args.len() {
            return Err(EvalError::ArgumentCount {
                phrase: name.to_string(),
                expected: def.parameters.len(),
                got: args.len(),
            });
        }

        // Build param map
        let params: HashMap<String, Value> = def
            .parameters
            .iter()
            .zip(args.iter())
            .map(|(name, value)| (name.clone(), value.clone()))
            .collect();

        let mut ctx = EvalContext::new(&params);
        ctx.push_call(name)?;
        let result = eval_phrase_def(def, &mut ctx, registry, &self.transforms, language)?;
        ctx.pop_call();
        Ok(result)
    }

    /// Evaluate a template string with parameters.
    ///
    /// Uses the current language for plural rules.
    pub fn eval_str(
        &self,
        template_str: &str,
        params: HashMap<String, Value>,
    ) -> Result<Phrase, EvalError> {
        let registry =
            self.registries
                .get(&self.language)
                .ok_or_else(|| EvalError::PhraseNotFound {
                    name: format!("no translations loaded for language '{}'", self.language),
                })?;

        let template = parse_template(template_str).map_err(|e| EvalError::PhraseNotFound {
            name: format!("parse error: {e}"),
        })?;
        let mut ctx = EvalContext::new(&params);
        let text = eval_template(
            &template,
            &mut ctx,
            registry,
            &self.transforms,
            &self.language,
        )?;
        Ok(Phrase::builder().text(text).build())
    }
}
