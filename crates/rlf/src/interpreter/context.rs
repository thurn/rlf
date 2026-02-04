//! Evaluation context for tracking state during recursive evaluation.

use std::collections::HashMap;

use crate::interpreter::EvalError;
use crate::types::Value;

/// Evaluation context carrying state through recursive evaluation.
///
/// The context tracks:
/// - Parameters available during evaluation
/// - Call stack for cycle detection
/// - Recursion depth for limiting deep recursion
pub struct EvalContext<'a> {
    /// Parameters available during evaluation.
    params: &'a HashMap<String, Value>,
    /// Call stack for cycle detection (phrase names).
    call_stack: Vec<String>,
    /// Current recursion depth.
    depth: usize,
    /// Maximum allowed depth (default 64).
    max_depth: usize,
}

impl<'a> EvalContext<'a> {
    /// Create new context with parameters.
    pub fn new(params: &'a HashMap<String, Value>) -> Self {
        Self {
            params,
            call_stack: Vec::new(),
            depth: 0,
            max_depth: 64,
        }
    }

    /// Create context with custom max depth.
    pub fn with_max_depth(params: &'a HashMap<String, Value>, max_depth: usize) -> Self {
        Self {
            params,
            call_stack: Vec::new(),
            depth: 0,
            max_depth,
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
}
