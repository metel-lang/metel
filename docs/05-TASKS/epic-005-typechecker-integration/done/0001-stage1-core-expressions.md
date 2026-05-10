# Task 0001: Stage 1 — Core Expressions and Let-Bindings

**Status:** done  
**Epic:** epic-005-typechecker-integration  
**Component:** typechecker  
**Spec Link:** docs/01-SPEC/LANGUAGE-SPEC.md#32-type-inference  
**Blocked By:** none  
**Decisions:** ADR-0001, ADR-0002

## What

Implement Pass 1 (inference) and Pass 2 (construction) for the core expression and
declaration forms: literals, identifiers, binary/unary operators, `let`/`mut`
bindings, and function declarations. Add error codes, wire the two-pass design into
`typechecker::check()`, and establish the negative test harness.

## Acceptance Criteria

- [x] `ErrorCode` enum added to `src/error/mod.rs`; `TypeError` carries a code;
  `type_error()` updated
- [x] `TypeExpr → InferType` conversion covers all annotation forms
- [x] `InferType → Type` conversion errors on unresolved variables (E0002)
- [x] Pre-pass registers all top-level function names for forward references
- [x] Literals infer to their concrete types; `nope` infers to `Perhaps<?t0>`
- [x] Identifiers resolve via `InferContext::lookup` (E0003 on miss)
- [x] `BinOp` emits correct constraints and produces correct result type
- [x] `UnaryOp` emits correct constraints
- [x] `Decl::Let` / `Decl::Mut`: annotation (if present) unified with inferred value type
- [x] `Decl::Fun`: parameters and return type unified, body inferred, scheme generalized
- [x] Stage 1 `.yolo` test programs pass through `check()` without error
- [x] Type mismatches produce `YoloscriptError::TypeError` with source span
- [x] `nope` with annotation passes through Pass 2 via `expected_ty` parameter on
  `construct_expr`

## Notes

- Pass 2 uses `expected_ty: Option<&Type>` on `construct_expr` to propagate annotation
  types into `nope` literal construction — necessary because `nope`'s type cannot be
  re-derived from the literal alone in a structural re-derivation pass.
- `let`/`mut` generalization (HM let-polymorphism for closures) is deferred to Stage 4.
  Unannotated lambda bindings used at two different types produce E0001 (loud failure).
- Negative test harness: per-line `// ERROR[EXXXX]` comments; harness checks error
  code and 1-based line number against the first annotation in the file.
- Error codes: E0001 (type mismatch), E0002 (annotation required), E0003 (undefined
  name), E0004 (arity mismatch), E0005 (invalid operand types).
