/// Type inference module for Yolang.
///
/// This module is being built incrementally with comprehensive tests.
/// See tasks in docs/Yolang/tasks/epic-001-typechecker/ for the step-by-step breakdown.
///
/// Current status: Foundation phase (type variables)

use crate::ast::Span;
use crate::types::Type;
use crate::error::YolangError;
use std::collections::HashMap;

// ── Phase 1: Type Variables ───────────────────────────────────────────────────

/// A type variable representing an unknown type during inference.
/// Each type variable has a unique ID.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TypeVar(pub u32);

impl std::fmt::Display for TypeVar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "?t{}", self.0)
    }
}

/// Counter for generating fresh type variables.
pub struct TypeVarGenerator {
    counter: u32,
}

impl TypeVarGenerator {
    /// Create a new type variable generator.
    pub fn new() -> Self {
        TypeVarGenerator { counter: 0 }
    }

    /// Generate a fresh type variable.
    pub fn fresh(&mut self) -> TypeVar {
        let var = TypeVar(self.counter);
        self.counter += 1;
        var
    }

    /// Get the current counter state (for testing).
    pub fn counter(&self) -> u32 {
        self.counter
    }
}

impl Default for TypeVarGenerator {
    fn default() -> Self {
        Self::new()
    }
}

// TODO: Add remaining components in subsequent tasks:
// - Phase 2: Unification algorithm
// - Phase 3: Constraints
// - Phase 4: Substitution
// - Phase 5: Type schemes
// - Phase 6: Inference context
