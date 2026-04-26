# Task 0002: Implement Type Inference Engine

**Status:** open  
**Epic:** epic-001-typechecker  
**Component:** typechecker, interpreter  
**Spec Link:** spec/Language Spec.md#Type-Inference-Rules  
**Blocked By:** 0001

## What

Implement a type inference engine that:

1. Infers types for unannotated expressions
2. Builds type constraints from the AST
3. Solves constraints to determine actual types
4. Reports conflicts and unresolvable types

Start with a simple approach (e.g., Hindley-Milner-style let polymorphism) without generics.

## Design Approach

1. **Constraint collection:** Walk the typed AST and emit type equations (e.g., `typeof(a) = typeof(b)`)
2. **Unification:** Solve constraints using unification algorithm
3. **Error reporting:** When constraints conflict, report what types were expected vs. actual

## Acceptance Criteria

- [ ] Inference engine infers types for basic literals correctly
- [ ] Inference handles binary operations (int, bool operations)
- [ ] Inference rejects operations with incompatible types
- [ ] Inference propagates types through let bindings
- [ ] Inference handles function definitions and calls
- [ ] Type error messages identify the conflict clearly
- [ ] All tests from task 0001 still pass
- [ ] New test suite covers inference cases

## Notes

- Consider a two-pass approach: constraint collection, then unification
- Don't worry about advanced features (traits, bounds, etc.) yet
- Performance is secondary; correctness and clarity first
