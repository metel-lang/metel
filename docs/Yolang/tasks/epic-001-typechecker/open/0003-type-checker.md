# Task 0003: Build Type Checking Pass

**Status:** open  
**Epic:** epic-001-typechecker  
**Component:** typechecker  
**Spec Link:** spec/Language Spec.md#Type-System  
**Blocked By:** 0002

## What

Create a type checking pass that:

1. Takes a fully-typed AST (after inference from 0002)
2. Validates that all operations are type-safe
3. Reports detailed errors for type mismatches
4. Ensures no `Unknown` types remain

This is separate from inference: inference *determines* types, checking *validates* them.

## Validation Rules

For each operation:
- Binary ops: both operands must be compatible types
- Function calls: argument types must match parameter types
- Assignments: assigned value type must match declared type
- Comparisons: both sides must be comparable

## Acceptance Criteria

- [ ] Type checker validates all operations in typed AST
- [ ] Type checker rejects operations with incompatible types
- [ ] Type checker ensures all expressions have resolved types
- [ ] Error messages are clear and include context (line/column)
- [ ] Type checker handles nested expressions correctly
- [ ] Integration with evaluator works (typed AST passes through checker to evaluator)
- [ ] All previous tests still pass
- [ ] New test suite covers type checking validation

## Notes

- Type checking should happen after inference completes
- Consider separating type checker into a distinct module: `typechecker.rs` or `type_check/mod.rs`
- Error recovery: report multiple type errors in one pass rather than stopping at first
