# Task 0002: Stage 2 — Control Flow Statements

**Status:** open  
**Epic:** epic-005-typechecker-integration  
**Component:** typechecker  
**Spec Link:** docs/01-SPEC/LANGUAGE-SPEC.md (§4 Statements, §5 Expressions)  
**Blocked By:** 0001  
**Decisions:** ADR-0002

## What

Extend the two-pass typechecker to handle control flow: `Stmt::If`, `Stmt::While`,
`Stmt::Return`, and `Expr::If` (if-as-expression). After this stage the typechecker
handles all branching and looping constructs that don't require pattern matching or
closures.

## Acceptance Criteria

- [ ] `Stmt::If`: condition constrained to `Bool`; then/else blocks inferred; Pass 2
  constructs `TypedStmt::If`
- [ ] `Stmt::While`: condition constrained to `Bool`; body inferred; Pass 2 constructs
  `TypedStmt::While`
- [ ] `Stmt::Return`: return expression unified with `current_return_type`; bare
  `return` treated as `Unit`; error if used outside a function
- [ ] `Expr::If`: both branches must unify to a common type; that type is the result;
  condition constrained to `Bool`; Pass 2 constructs `TypedExpr::If`
- [ ] Stage 2 `.yolo` test programs (`phase8_stage2_*.yolo`) pass through `check()`
  without error
- [ ] Type errors in conditions or branches produce `TypeError` with correct error code
  and span
- [ ] All Stage 1 tests still pass
