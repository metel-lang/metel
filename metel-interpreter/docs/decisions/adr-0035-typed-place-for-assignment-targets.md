---
id: adr-0035
title: "TypedPlace Replaces AssignTarget in the Typed AST"
date: '2026-06-02'
status: active
---

## Context

`TypedExpr::Assign` previously carried a raw `AssignTarget` (an untyped AST node) for the assignment target. The evaluator had to evaluate sub-expressions inside the target — particularly index expressions — using `eval_untyped_index`, which only handled integer literals and bare identifiers. Any computed index such as `arr[i + 1] = v` emitted an internal error at runtime despite passing the typechecker.

## Decision

Introduce `TypedPlace` in `typed_ast/mod.rs` as a recursive lvalue representation where index sub-expressions are fully typed `TypedExpr` nodes:

```
TypedPlace::Ident(String, Span)
TypedPlace::Deref { object: Box<TypedExpr>, span }
TypedPlace::Field { object: Box<TypedPlace>, field: String, span }
TypedPlace::Index { object: Box<TypedPlace>, index: Box<TypedExpr>, span }
```

`TypedExpr::Assign.target` changes from `AssignTarget` to `TypedPlace`. The typechecker's construction pass converts `AssignTarget` → `TypedPlace` via `assign_target_to_typed_place`, running `construct_expr` on index sub-expressions. The evaluator's `Assign` branch matches `TypedPlace` variants and uses `eval_expr` for index expressions.

## Consequences

- Computed index assignment (`arr[i + 1] = v`, `s.data[offset * 2] = v`) works correctly.
- The untyped helpers `eval_untyped_index` and `eval_untyped_lvalue_value` are no longer called from the typed evaluator path.
- `AssignTarget` is still used by the untyped evaluator path (`eval_untyped_expr`) and the inference pass, which operate directly on the AST.
- Field chains containing an index step (`arr[i].field = v`) are not yet supported for mutation — `extract_typed_place_field_path` returns an error if it encounters a non-Ident/Field step. This is the same limitation as before; the new representation makes the boundary explicit.
