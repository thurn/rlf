//! Locale management for RLF translations.
//!
//! The Locale struct provides the user-facing API for managing language selection,
//! loading translations, and accessing phrases.

use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use bon::Builder;

use crate::interpreter::error::{LoadError, LoadWarning};
use crate::interpreter::registry::PhraseRegistry;
use crate::interpreter::transforms::TransformRegistry;
use crate::interpreter::{EvalContext, EvalError, eval_phrase_def, eval_template};
use crate::parser::ast::Template;
use crate::parser::{ParseError, parse_file, parse_template};
use crate::types::{Phrase, Value};

/// User-facing locale management for RLF translations.
///
/// Locale owns per-language phrase registries and a shared transform registry.
/// Missing translations are errors, not silently papered over with fallback
/// behavior. This design provides:
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

    /// Optional string context for format variant selection.
    ///
    /// When set, variant phrases prefer the variant matching this context
    /// as their default text. For example, with `string_context = "card_text"`,
    /// a phrase `{ interface: "X", card_text: "<b>X</b>" }` produces
    /// `"<b>X</b>"` as its default text.
    string_context: Option<String>,

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

    /// Cache of parsed template ASTs for `eval_str()`.
    ///
    /// Uses `RefCell` for interior mutability so `eval_str` can remain `&self`.
    /// Templates are keyed by their source string and reused across calls.
    #[builder(skip)]
    template_cache: RefCell<HashMap<String, Template>>,
}

impl Default for Locale {
    fn default() -> Self {
        Locale::builder().build()
    }
}

impl Locale {
    /// Create a new Locale with default settings (English).
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

    /// Get the current string context, if any.
    pub fn string_context(&self) -> Option<&str> {
        self.string_context.as_deref()
    }

    /// Set the string context for format variant selection.
    ///
    /// When set, variant phrases prefer the variant matching this context
    /// as their default text. Pass `None` to clear the context.
    ///
    /// # Example
    ///
    /// ```
    /// use rlf::Locale;
    ///
    /// let mut locale = Locale::new();
    /// locale.load_translations_str("en", r#"
    ///     energy = { interface: "E", card_text: "<b>E</b>" };
    /// "#).unwrap();
    ///
    /// // Without context: default text is first variant
    /// let phrase = locale.get_phrase("energy").unwrap();
    /// assert_eq!(phrase.to_string(), "E");
    ///
    /// // With context: default text matches the context variant
    /// locale.set_string_context(Some("card_text"));
    /// let phrase = locale.get_phrase("energy").unwrap();
    /// assert_eq!(phrase.to_string(), "<b>E</b>");
    /// ```
    pub fn set_string_context(&mut self, context: Option<impl Into<String>>) {
        self.string_context = context.map(Into::into);
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

    /// Validate translations for a target language against a source language.
    ///
    /// Checks for:
    /// - Phrases in the target language that do not exist in the source language
    /// - Phrases with a different parameter count than the source
    ///
    /// Both the source and target languages must already be loaded. Returns an
    /// empty vector if no warnings are found or if either language is not loaded.
    ///
    /// # Example
    ///
    /// ```
    /// use rlf::{Locale, LoadWarning};
    ///
    /// let mut locale = Locale::new();
    /// locale.load_translations_str("en", r#"hello = "Hello!";"#).unwrap();
    /// locale.load_translations_str("ru", r#"
    ///     hello = "Привет!";
    ///     extra = "Лишнее";
    /// "#).unwrap();
    ///
    /// let warnings = locale.validate_translations("en", "ru");
    /// assert_eq!(warnings.len(), 1); // "extra" not in source
    /// ```
    pub fn validate_translations(
        &self,
        source_language: &str,
        target_language: &str,
    ) -> Vec<LoadWarning> {
        let mut warnings = Vec::new();

        let Some(source_registry) = self.registries.get(source_language) else {
            return warnings;
        };
        let Some(target_registry) = self.registries.get(target_language) else {
            return warnings;
        };

        let mut target_names: Vec<&str> = target_registry.phrase_names().collect();
        target_names.sort();

        for name in target_names {
            if let Some(source_def) = source_registry.get(name) {
                let target_def = target_registry
                    .get(name)
                    .expect("name came from this registry");
                if source_def.parameters.len() != target_def.parameters.len() {
                    warnings.push(LoadWarning::ParameterCountMismatch {
                        name: name.to_string(),
                        language: target_language.to_string(),
                        source_count: source_def.parameters.len(),
                        translation_count: target_def.parameters.len(),
                    });
                }
            } else {
                warnings.push(LoadWarning::UnknownPhrase {
                    name: name.to_string(),
                    language: target_language.to_string(),
                });
            }
        }

        warnings
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
    // Phrase Evaluation
    // =========================================================================

    /// Get a parameterless phrase in the current language.
    ///
    /// Returns an error if the phrase is not found. Missing translations are
    /// treated as errors to be caught during development or by CI tooling.
    pub fn get_phrase(&self, name: &str) -> Result<Phrase, EvalError> {
        let registry =
            self.registries
                .get(&self.language)
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
        let mut ctx = EvalContext::with_string_context(&params, self.string_context.clone());
        ctx.push_call(name)?;
        let result = eval_phrase_def(def, &mut ctx, registry, &self.transforms, &self.language)?;
        ctx.pop_call();
        Ok(result)
    }

    /// Call a phrase with arguments in the current language.
    ///
    /// Returns an error if the phrase is not found. Missing translations are
    /// treated as errors to be caught during development or by CI tooling.
    pub fn call_phrase(&self, name: &str, args: &[Value]) -> Result<Phrase, EvalError> {
        let registry =
            self.registries
                .get(&self.language)
                .ok_or_else(|| EvalError::PhraseNotFound {
                    name: name.to_string(),
                })?;

        let def = registry
            .get(name)
            .ok_or_else(|| EvalError::PhraseNotFound {
                name: name.to_string(),
            })?;

        if def.parameters.len() != args.len() {
            return Err(EvalError::ArgumentCount {
                phrase: name.to_string(),
                expected: def.parameters.len(),
                got: args.len(),
            });
        }

        let params: HashMap<String, Value> = def
            .parameters
            .iter()
            .zip(args.iter())
            .map(|(name, value)| (name.clone(), value.clone()))
            .collect();

        let mut ctx = EvalContext::with_string_context(&params, self.string_context.clone());
        ctx.push_call(name)?;
        let result = eval_phrase_def(def, &mut ctx, registry, &self.transforms, &self.language)?;
        ctx.pop_call();
        Ok(result)
    }

    /// Get a parameterless phrase by PhraseId in the current language.
    ///
    /// Returns the full Phrase with text, variants, and tags.
    pub fn get_phrase_by_id(&self, id: u64) -> Result<Phrase, EvalError> {
        let registry = self
            .registries
            .get(&self.language)
            .ok_or(EvalError::PhraseNotFoundById { id })?;

        let name = registry
            .name_for_id(id)
            .ok_or(EvalError::PhraseNotFoundById { id })?;

        self.get_phrase(name)
    }

    /// Call a phrase by PhraseId with arguments in the current language.
    ///
    /// Returns the evaluated Phrase.
    pub fn call_phrase_by_id(&self, id: u64, args: &[Value]) -> Result<Phrase, EvalError> {
        let registry = self
            .registries
            .get(&self.language)
            .ok_or(EvalError::PhraseNotFoundById { id })?;

        let name = registry
            .name_for_id(id)
            .ok_or(EvalError::PhraseNotFoundById { id })?;

        self.call_phrase(name, args)
    }

    /// Look up the phrase name for a PhraseId hash in the current language.
    ///
    /// Returns None if no phrase with that hash is registered.
    pub fn name_for_id(&self, id: u64) -> Option<&str> {
        self.registries
            .get(&self.language)
            .and_then(|registry| registry.name_for_id(id))
    }

    /// Get the parameter count for a phrase by PhraseId in the current language.
    ///
    /// Returns 0 if the phrase is not found.
    pub fn phrase_parameter_count(&self, id: u64) -> usize {
        self.registries
            .get(&self.language)
            .map(|registry| registry.phrase_parameter_count(id))
            .unwrap_or(0)
    }

    /// Evaluate a template string with parameters.
    ///
    /// Uses the current language for plural rules. Parsed template ASTs are
    /// cached so repeated calls with the same template string skip parsing.
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

        let template = self.cached_template(template_str)?;
        let mut ctx = EvalContext::with_string_context(&params, self.string_context.clone());
        let text = eval_template(
            &template,
            &mut ctx,
            registry,
            &self.transforms,
            &self.language,
        )?;
        Ok(Phrase::builder().text(text).build())
    }

    /// Clear the template cache.
    ///
    /// Call this if you need to free memory used by cached template ASTs.
    pub fn clear_template_cache(&self) {
        self.template_cache.borrow_mut().clear();
    }

    /// Return the number of cached template ASTs.
    pub fn template_cache_len(&self) -> usize {
        self.template_cache.borrow().len()
    }

    /// Look up or parse and cache a template string.
    fn cached_template(&self, template_str: &str) -> Result<Template, EvalError> {
        {
            let cache = self.template_cache.borrow();
            if let Some(template) = cache.get(template_str) {
                return Ok(template.clone());
            }
        }
        let template = parse_template(template_str).map_err(|e| EvalError::PhraseNotFound {
            name: format!("parse error: {e}"),
        })?;
        self.template_cache
            .borrow_mut()
            .insert(template_str.to_string(), template.clone());
        Ok(template)
    }
}
