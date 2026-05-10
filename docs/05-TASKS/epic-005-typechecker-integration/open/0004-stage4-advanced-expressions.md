# Task 0004: Stage 4 — Advanced Expressions

**Status:** open  
**Epic:** epic-005-typechecker-integration  
**Component:** typechecker  
**Spec Link:** docs/01-SPEC/LANGUAGE-SPEC.md (§5, §6, §7)  
**Blocked By:** 0003  
**Decisions:** ADR-0002

## What

Extend the typechecker to the remaining expression and statement forms: closures,
pattern matching, method calls, field access, struct literals, loop constructs, and
the `Never` type. Also fix `let`/`mut` generalization (HM let-polymorphism, deferred
from Stage 1).

## Acceptance Criteria

- [ ] `Expr::Closure`: parameters and return type inferred; body inferred and unified
  with return; closure let-bound in a `let`/`mut` is generalized into `SchemeEnv`;
  Pass 2 constructs `TypedExpr::Closure`
- [ ] `Expr::Match`: scrutinee type inferred; each arm pattern unified with scrutinee
  type; all arm bodies must unify to a common result type; Pass 2 constructs
  `TypedExpr::Match`
- [ ] `Expr::MethodCall`: receiver type inferred; method looked up in `TypeRegistry`;
  arguments unified; result type is the method's return type; Pass 2 constructs
  `TypedExpr::MethodCall`
- [ ] `Expr::FieldAccess`: receiver type inferred; field looked up in `TypeRegistry`;
  result is the field type; Pass 2 constructs `TypedExpr::FieldAccess`
- [ ] `Expr::StructLiteral`: each field value unified with the declared field type;
  result is the struct type; Pass 2 constructs `TypedExpr::StructLiteral`
- [ ] `Stmt::For` / `Stmt::ForIn`: induction variable type inferred from range or
  collection element type; body inferred; Pass 2 constructs corresponding `TypedStmt`
- [ ] `Stmt::Loop`: body inferred; result type is `Never` unless a `break` exits with
  a value, in which case `break` expressions must unify
- [ ] `Type::Never` / `InferType::Never`: `Never` coerces to any type (subtype of
  everything); loops with no reachable `break` infer to `!`
- [ ] `let`/`mut` bindings holding closures are generalized (HM let-polymorphism):
  after solving, track `let`/`mut` in `pending` alongside `fun`; insert into
  `scheme_env`; Pass 2 looks them up from `scheme_env` with instantiation
- [ ] Stage 4 `.yolo` test programs pass through `check()` without error
- [ ] All Stage 1–3 tests still pass

## Notes

- `let`/`mut` generalization fix: track `let`/`mut` bindings in `pending` after
  `infer_expr`; after solving, generalize and insert into `scheme_env`; Pass 2 looks
  them up from `scheme_env` with instantiation instead of the concrete env. This
  requires knowing the binding expression is a closure — only closures need
  generalization; scalar `let` bindings are already monomorphic.
