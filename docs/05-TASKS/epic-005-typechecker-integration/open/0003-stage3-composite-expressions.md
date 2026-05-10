# Task 0003: Stage 3 — Composite Expressions

**Status:** open  
**Epic:** epic-005-typechecker-integration  
**Component:** typechecker  
**Spec Link:** docs/01-SPEC/LANGUAGE-SPEC.md (§5 Expressions)  
**Blocked By:** 0002  
**Decisions:** ADR-0002

## What

Extend inference and construction to composite expression forms: `Expr::Tuple`,
`Expr::Array`, `Expr::Call`, and `Expr::Index`. After this stage, function calls and
collection construction are fully typed.

## Acceptance Criteria

- [ ] `Expr::Tuple`: each element inferred; result type is `Tuple(t1, t2, ...)`;
  Pass 2 constructs `TypedExpr::Tuple`
- [ ] `Expr::Array`: all elements unified to a common element type; result is
  `Array(T)`; empty array literal uses a fresh type variable; Pass 2 constructs
  `TypedExpr::Array`
- [ ] `Expr::Call`: callee must be `Fun(params, ret)`; each argument unified with the
  corresponding parameter type; arity mismatch produces E0004; result type is `ret`;
  Pass 2 constructs `TypedExpr::Call`
- [ ] `Expr::Call` on a polymorphic function: scheme instantiated fresh per call site
  via `SchemeEnv`
- [ ] `Expr::Index`: operand must be `Array(T)`, index must be `Int`; result is `T`;
  Pass 2 constructs `TypedExpr::Index`
- [ ] Stage 3 `.yolo` test programs pass through `check()` without error
- [ ] All Stage 1–2 tests still pass
