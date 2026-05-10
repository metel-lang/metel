# Epic 005: Typechecker Integration

**Status:** in-progress  
**Started:** 2026-05-07

## Overview

Wire `InferContext` (from epic-001) into `typechecker::check()` to produce a real
`TypedProgram` from an untyped `Program`, and connect that output to the evaluator
so the full pipeline runs end-to-end.

Uses the two-pass design from ADR-0002: Pass 1 infers types via constraint
generation and solving, Pass 2 re-derives concrete types structurally and builds
`TypedAST` nodes. A pre-pass handles function hoisting for forward references and
mutual recursion.

```rust
let mut ctx = InferContext::new();
hoist_fun_decls(&program.decls, &mut ctx);
let (substitution, scheme_env) = infer(&program, &mut ctx)?;
let typed_program = construct(&program, &substitution, &scheme_env)?;
```

## Goals

1. **Stage 1** — Core expressions and let-bindings (done)
2. **Stage 2** — Control flow (`if`/`while`/`return` stmts, `if`-as-expression)
3. **Stage 3** — Composite expressions (tuple, array, call, index)
4. **Stage 4** — Advanced expressions (closure, match, method call, field access,
   struct literal, for/loop, Never type, let/mut generalization)
5. **Evaluator integration** — full pipeline end-to-end

## Permanently Out of Scope (this epic)

| Node | Reason | Where it lands |
|---|---|---|
| `FunDecl` with `GenericParam`s | Generics are out of scope | Epic 003 |
| `Expr::Cast` | Cast validation requires trait resolution | Epic 004 |
| `Expr::Path` | Needs a dedicated name-resolution pass | Module system (future) |

## Dependencies

- **Epic 001** (typechecker foundation): `InferContext`, `TypeScheme`, `Substitution`,
  `InferType`, typed AST nodes
- **Spec:** §3.2 (type inference), §4 (statements), §5 (expressions), §8 (Perhaps)

## Key Design Decisions

- [ADR-0001](../../06-DECISIONS/closed/ADR-0001-type-registry.md) — TypeRegistry
  structure: pre-built and injected into `InferContext`
- [ADR-0002](../../06-DECISIONS/closed/ADR-0002-inference-pass-structure.md) — Two-pass
  with re-derivation (Option C): Pass 1 returns `(Substitution, SchemeEnv)`;
  Pass 2 re-derives types structurally with no constraint emission

## Success Criteria

- [ ] All Stage 1–4 `.yolo` test programs pass through `check()` without error
- [ ] Type errors produce `TypeError` with correct error code and source span
- [ ] Full `parse() → check() → evaluate()` pipeline works on a non-trivial program
- [ ] All previous tests still pass
