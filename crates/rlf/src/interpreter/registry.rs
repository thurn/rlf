//! Phrase registry for storing and looking up phrase definitions.

use std::cell::RefCell;
use std::collections::HashMap;

use crate::interpreter::transforms::TransformRegistry;
use crate::interpreter::{EvalContext, EvalError, eval_phrase_def, eval_template};
use crate::parser::ast::{PhraseDefinition, Template};
use crate::parser::{ParseError, parse_file, parse_template};
use crate::types::{Phrase, PhraseId, Value};

/// A registry for storing and looking up phrase definitions.
///
/// The registry supports lookup by both name (string) and by PhraseId (hash).
/// This enables efficient runtime lookups while maintaining human-readable names.
#[derive(Debug, Default)]
pub struct PhraseRegistry {
    /// Phrases indexed by name.
    phrases: HashMap<String, PhraseDefinition>,
    /// Maps PhraseId hash to phrase name for id-based lookup.
    id_to_name: HashMap<u64, String>,
    /// Cache of parsed template ASTs for `eval_str()`.
    ///
    /// Uses `RefCell` for interior mutability so `eval_str` can remain `&self`.
    template_cache: RefCell<HashMap<String, Template>>,
}

impl PhraseRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Get a phrase definition by name.
    pub fn get(&self, name: &str) -> Option<&PhraseDefinition> {
        self.phrases.get(name)
    }

    /// Get a phrase definition by PhraseId hash.
    pub fn get_by_id(&self, id: u64) -> Option<&PhraseDefinition> {
        self.id_to_name
            .get(&id)
            .and_then(|name| self.phrases.get(name))
    }

    /// Insert a phrase definition into the registry.
    ///
    /// Returns an error if a hash collision is detected (different name but same hash).
    pub fn insert(&mut self, def: PhraseDefinition) -> Result<(), EvalError> {
        let name = def.name.clone();
        let id = PhraseId::from_name(&name);
        let hash = id.as_u64();

        // Check for hash collision (different name but same hash)
        if let Some(existing_name) = self.id_to_name.get(&hash)
            && existing_name != &name
        {
            // This is a hash collision - different names producing same hash
            // This should be extremely rare with 64-bit FNV-1a but we handle it
            return Err(EvalError::PhraseNotFound {
                name: format!(
                    "hash collision: '{}' and '{}' produce same hash",
                    existing_name, name
                ),
            });
        }

        self.id_to_name.insert(hash, name.clone());
        self.phrases.insert(name, def);
        Ok(())
    }

    /// Load phrases from a string containing .rlf format.
    ///
    /// Returns the number of phrases loaded.
    ///
    /// # Example
    ///
    /// ```
    /// use rlf::PhraseRegistry;
    ///
    /// let mut registry = PhraseRegistry::new();
    /// let count = registry.load_phrases(r#"
    ///     hello = "Hello, world!";
    ///     card = { one: "card", other: "cards" };
    /// "#).unwrap();
    /// assert_eq!(count, 2);
    /// ```
    pub fn load_phrases(&mut self, content: &str) -> Result<usize, ParseError> {
        let definitions = parse_file(content)?;
        let count = definitions.len();
        for def in definitions {
            // insert handles collision detection
            self.insert(def).map_err(|e| ParseError::Syntax {
                line: 0,
                column: 0,
                message: format!("{e}"),
            })?;
        }
        Ok(count)
    }

    // =========================================================================
    // Public Evaluation API
    // =========================================================================

    /// Evaluate a template string with parameters.
    ///
    /// This is the main entry point for runtime template evaluation. The template
    /// string is parsed and evaluated with the given parameters in the specified
    /// language context. Parsed template ASTs are cached so repeated calls with
    /// the same template string skip parsing.
    ///
    /// # Arguments
    ///
    /// * `template_str` - A template string (e.g., "Draw {n} {card:n}.")
    /// * `lang` - Language code for plural rules (e.g., "en", "ru")
    /// * `params` - Parameters available during evaluation
    ///
    /// # Example
    ///
    /// ```
    /// use rlf::{PhraseRegistry, Value};
    /// use std::collections::HashMap;
    ///
    /// let mut registry = PhraseRegistry::new();
    /// registry.load_phrases(r#"card = { one: "card", other: "cards" };"#).unwrap();
    ///
    /// let params: HashMap<String, Value> = [("n".to_string(), Value::from(3))].into_iter().collect();
    /// let result = registry.eval_str("Draw {n} {card:n}.", "en", params).unwrap();
    /// assert_eq!(result.to_string(), "Draw 3 cards.");
    /// ```
    pub fn eval_str(
        &self,
        template_str: &str,
        lang: &str,
        params: HashMap<String, Value>,
    ) -> Result<Phrase, EvalError> {
        let template = self.cached_template(template_str)?;
        let transform_registry = TransformRegistry::new();
        let mut ctx = EvalContext::new(&params);
        let text = eval_template(&template, &mut ctx, self, &transform_registry, lang)?;
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

    /// Call a phrase by name with positional arguments.
    ///
    /// The phrase is looked up by name, the arguments are matched to parameters,
    /// and the phrase is evaluated in the specified language context.
    ///
    /// # Arguments
    ///
    /// * `lang` - Language code for plural rules
    /// * `name` - Name of the phrase to call
    /// * `args` - Positional arguments (must match parameter count)
    ///
    /// # Example
    ///
    /// ```
    /// use rlf::{PhraseRegistry, Value};
    ///
    /// let mut registry = PhraseRegistry::new();
    /// registry.load_phrases(r#"greet(name) = "Hello, {name}!";"#).unwrap();
    ///
    /// let result = registry.call_phrase("en", "greet", &[Value::from("World")]).unwrap();
    /// assert_eq!(result.to_string(), "Hello, World!");
    /// ```
    pub fn call_phrase(&self, lang: &str, name: &str, args: &[Value]) -> Result<Phrase, EvalError> {
        let def = self.get(name).ok_or_else(|| EvalError::PhraseNotFound {
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

        let transform_registry = TransformRegistry::new();
        let mut ctx = EvalContext::new(&params);
        ctx.push_call(name)?;
        let result = eval_phrase_def(def, &mut ctx, self, &transform_registry, lang)?;
        ctx.pop_call();
        Ok(result)
    }

    /// Get a parameterless phrase as a Phrase value.
    ///
    /// The phrase is looked up by name and evaluated. It must not have any
    /// parameters defined.
    ///
    /// # Arguments
    ///
    /// * `lang` - Language code for plural rules
    /// * `name` - Name of the phrase to get
    ///
    /// # Example
    ///
    /// ```
    /// use rlf::PhraseRegistry;
    ///
    /// let mut registry = PhraseRegistry::new();
    /// registry.load_phrases(r#"hello = "Hello, world!";"#).unwrap();
    ///
    /// let result = registry.get_phrase("en", "hello").unwrap();
    /// assert_eq!(result.to_string(), "Hello, world!");
    /// ```
    pub fn get_phrase(&self, lang: &str, name: &str) -> Result<Phrase, EvalError> {
        let def = self.get(name).ok_or_else(|| EvalError::PhraseNotFound {
            name: name.to_string(),
        })?;

        if !def.parameters.is_empty() {
            return Err(EvalError::ArgumentCount {
                phrase: name.to_string(),
                expected: def.parameters.len(),
                got: 0,
            });
        }

        let transform_registry = TransformRegistry::new();
        let params = HashMap::new();
        let mut ctx = EvalContext::new(&params);
        ctx.push_call(name)?;
        let result = eval_phrase_def(def, &mut ctx, self, &transform_registry, lang)?;
        ctx.pop_call();
        Ok(result)
    }

    /// Call a phrase by PhraseId with arguments.
    ///
    /// Like `call_phrase`, but looks up the phrase by its PhraseId hash.
    ///
    /// # Arguments
    ///
    /// * `id` - PhraseId hash of the phrase
    /// * `lang` - Language code for plural rules
    /// * `args` - Positional arguments
    pub fn call_phrase_by_id(
        &self,
        id: u64,
        lang: &str,
        args: &[Value],
    ) -> Result<Phrase, EvalError> {
        let name = self
            .id_to_name
            .get(&id)
            .ok_or(EvalError::PhraseNotFoundById { id })?;
        self.call_phrase(lang, name, args)
    }

    /// Get a phrase by PhraseId (parameterless only).
    ///
    /// Like `get_phrase`, but looks up the phrase by its PhraseId hash.
    ///
    /// # Arguments
    ///
    /// * `id` - PhraseId hash of the phrase
    /// * `lang` - Language code for plural rules
    pub fn get_phrase_by_id(&self, id: u64, lang: &str) -> Result<Phrase, EvalError> {
        let name = self
            .id_to_name
            .get(&id)
            .ok_or(EvalError::PhraseNotFoundById { id })?;
        self.get_phrase(lang, name)
    }

    /// Get the parameter count for a phrase by id.
    ///
    /// Returns 0 if the phrase is not found.
    pub fn phrase_parameter_count(&self, id: u64) -> usize {
        self.id_to_name
            .get(&id)
            .and_then(|name| self.get(name))
            .map(|def| def.parameters.len())
            .unwrap_or(0)
    }

    /// Look up the phrase name for a PhraseId hash.
    ///
    /// Returns None if no phrase with that hash is registered.
    pub fn name_for_id(&self, id: u64) -> Option<&str> {
        self.id_to_name.get(&id).map(String::as_str)
    }
}
