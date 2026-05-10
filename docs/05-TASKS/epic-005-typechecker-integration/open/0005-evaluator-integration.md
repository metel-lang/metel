# Task 0005: Evaluator Integration

**Status:** open  
**Epic:** epic-005-typechecker-integration  
**Component:** evaluator  
**Spec Link:** docs/01-SPEC/LANGUAGE-SPEC.md  
**Blocked By:** 0004  
**Decisions:** none

## What

Wire the typechecker output into the evaluator so the full pipeline
`parse() → check() → evaluate()` runs end-to-end on a non-trivial program.

## Acceptance Criteria

- [ ] `typechecker::check()` output passes into `evaluator::evaluate()` without error
- [ ] Full pipeline `parse() → check() → evaluate()` works on a non-trivial program
- [ ] All previous tests still pass
