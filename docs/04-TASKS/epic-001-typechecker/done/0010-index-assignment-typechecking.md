# Task 0010: Index Assignment Typechecking

**Status:** done
**Epic:** epic-001-typechecker
**Component:** typechecker
**Spec Link:** `../01-SPEC/LANGUAGE-SPEC.md` §4 Statements — Assignment
**Blocked By:** none

## What

`Expr::Assign` with `AssignTarget::Index` hits the internal-error fallback:

```yoloscript
mut arr = [1, 2, 3];
arr[0] = 99;   // → internal error: "non-ident assignment target not yet supported"
```

The element-read path (`Expr::Index`) is already implemented and constraints the
object to `Array(T)` and the index to `Int`. The write path needs the same
constraints, plus a constraint that the assigned value unifies with `T`.

`AssignTarget::FieldAccess` is deferred — it requires impl block typechecking
which is not yet supported.

## Fix

In `infer_expr`, replace the catch-all `_` arm inside `Expr::Assign`'s target
match with a specific `AssignTarget::Index` arm:

```rust
AssignTarget::Index { object, index, span: target_span } => {
    let obj_ty  = infer_expr(object, ctx, fun_generalizations)?;
    let idx_ty  = infer_expr(index,  ctx, fun_generalizations)?;
    ctx.add_constraint(idx_ty, InferType::int(), target_span.clone());
    let elem_var = ctx.fresh_var();
    ctx.add_constraint(obj_ty, InferType::Array(Box::new(elem_var.clone())), target_span.clone());
    elem_var
}
```

This `elem_var` becomes `target_ty` for the existing op-dispatch (plain `=` or
compound operators), exactly as with `AssignTarget::Ident`.

Note: mutability checking on index targets is not enforced here. Tracing the
root identifier through an arbitrary index expression to check `is_mutable` is
complex and deferred. A future task can add it.

No Pass 2 changes needed — `construct_expr` for `Expr::Assign` already clones
`target` unchanged.

## Acceptance Criteria

- [x] `AssignTarget::Index` is handled in `infer_expr`, no longer hits internal error
- [x] Object type is constrained to `Array(T)` with a fresh element variable
- [x] Index type is constrained to `Int`
- [x] Assigned value is constrained to unify with the element type `T`
- [x] Compound operators (`+=`, etc.) work on array elements the same way as ident targets
- [x] Positive test: `mut arr[i] = val` with matching element type passes
- [x] Negative test: type mismatch on assigned value produces E0001
- [x] `AssignTarget::FieldAccess` continues to return the internal error (deferred)
- [x] All 131 prior tests still pass; new tests added (133+ total)
