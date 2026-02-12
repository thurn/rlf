//! Evaluation context for tracking state during recursive evaluation.

use std::collections::{HashMap, HashSet};
use std::mem;

use crate::interpreter::EvalError;
use crate::interpreter::error::EvalWarning;
use crate::types::Value;

/// Evaluation context carrying state through recursive evaluation.
///
/// The context tracks:
/// - Parameters available during evaluation
/// - Call stack for cycle detection
/// - Recursion depth for limiting deep recursion
/// - Optional string context for format variant selection
/// - Runtime warnings collected during evaluation
/// - `:from` context set for suppressing false-positive lint warnings
pub struct EvalContext<'a> {
    /// Parameters available during evaluation.
    params: &'a HashMap<String, Value>,
    /// Call stack for cycle detection (phrase names).
    call_stack: Vec<String>,
    /// Current recursion depth.
    depth: usize,
    /// Maximum allowed depth (default 64).
    max_depth: usize,
    /// Optional string context for selecting format-specific variants.
    ///
    /// When set, variant phrases prefer the variant matching this context
    /// as their default text. For example, with `string_context = "card_text"`,
    /// a phrase with variants `{ interface: "X", card_text: "<b>X</b>" }`
    /// produces `"<b>X</b>"` as its default text.
    string_context: Option<String>,
    /// Runtime warnings collected during evaluation.
    warnings: Vec<EvalWarning>,
    /// Parameters currently bound in a `:from` iteration context.
    ///
    /// When evaluating inside a `:from($p)` body, `$p` is in this set.
    /// Bare references to parameters in this set should not trigger
    /// the "missing selector" lint because `:from` binds the correct
    /// variant automatically.
    from_context: HashSet<String>,
}

impl<'a> EvalContext<'a> {
    /// Create new context with parameters.
    pub fn new(params: &'a HashMap<String, Value>) -> Self {
        Self {
            params,
            call_stack: Vec::new(),
            depth: 0,
            max_depth: 64,
            string_context: None,
            warnings: Vec::new(),
            from_context: HashSet::new(),
        }
    }

    /// Create context with custom max depth.
    pub fn with_max_depth(params: &'a HashMap<String, Value>, max_depth: usize) -> Self {
        Self {
            params,
            call_stack: Vec::new(),
            depth: 0,
            max_depth,
            string_context: None,
            warnings: Vec::new(),
            from_context: HashSet::new(),
        }
    }

    /// Create context with a string context for format variant selection.
    pub fn with_string_context(
        params: &'a HashMap<String, Value>,
        string_context: Option<String>,
    ) -> Self {
        Self {
            params,
            call_stack: Vec::new(),
            depth: 0,
            max_depth: 64,
            string_context,
            warnings: Vec::new(),
            from_context: HashSet::new(),
        }
    }

    /// Get a parameter value.
    pub fn get_param(&self, name: &str) -> Option<&Value> {
        self.params.get(name)
    }

    /// Check if a name is in the current call stack (cycle detection).
    pub fn is_in_call_stack(&self, name: &str) -> bool {
        self.call_stack.iter().any(|n| n == name)
    }

    /// Push a phrase call onto the stack.
    ///
    /// Returns error if:
    /// - Maximum depth exceeded
    /// - Cycle detected (name already in call stack)
    pub fn push_call(&mut self, name: &str) -> Result<(), EvalError> {
        if self.depth >= self.max_depth {
            return Err(EvalError::MaxDepthExceeded);
        }
        if self.is_in_call_stack(name) {
            let mut chain = self.call_stack.clone();
            chain.push(name.to_string());
            return Err(EvalError::CyclicReference { chain });
        }
        self.call_stack.push(name.to_string());
        self.depth += 1;
        Ok(())
    }

    /// Pop a phrase call from the stack.
    pub fn pop_call(&mut self) {
        self.call_stack.pop();
        if self.depth > 0 {
            self.depth -= 1;
        }
    }

    /// Get current recursion depth.
    pub fn depth(&self) -> usize {
        self.depth
    }

    /// Get the call stack for error reporting.
    pub fn call_stack(&self) -> &[String] {
        &self.call_stack
    }

    /// Get the current string context, if any.
    pub fn string_context(&self) -> Option<&str> {
        self.string_context.as_deref()
    }

    /// Add a runtime warning to this context.
    pub fn add_warning(&mut self, warning: EvalWarning) {
        if !self.warnings.contains(&warning) {
            self.warnings.push(warning);
        }
    }

    /// Drain all collected warnings from this context.
    pub fn take_warnings(&mut self) -> Vec<EvalWarning> {
        mem::take(&mut self.warnings)
    }

    /// Get a reference to collected warnings.
    pub fn warnings(&self) -> &[EvalWarning] {
        &self.warnings
    }

    /// Merge warnings from a child context into this context.
    pub fn merge_warnings_from(&mut self, child: &mut EvalContext<'_>) {
        for warning in child.take_warnings() {
            self.add_warning(warning);
        }
    }

    /// Add a parameter to the `:from` context set.
    pub fn add_from_context(&mut self, param: &str) {
        self.from_context.insert(param.to_string());
    }

    /// Check if a parameter is in the `:from` context set.
    pub fn is_in_from_context(&self, param: &str) -> bool {
        self.from_context.contains(param)
    }
}
