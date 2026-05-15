# Task 0009: Diverging Statements Produce Never Type in Block Inference

**Status:** done
**Epic:** epic-001-typechecker
**Component:** typechecker
**Spec Link:** `../01-SPEC/LANGUAGE-SPEC.md` §5 Expressions — Never type (`!`)
**Blocked By:** none
**Decisions:** none

## What

`infer_block` returns `InferType::unit()` when a block has no tail expression,
regardless of whether the last statement diverges. This causes a spurious type
error for any function whose only exit path is `return`, `break`, or `continue`:

```yoloscript
fun abs(x: Int) -> Int {
    if (x < 0) {
        return 0 - x;
    }
    return x;
    // block tail = None → Unit; constraint Unit == Int → E0001
}
```

The root cause: `return`, `break`, and `continue` are diverging statements —
they never fall through. A block whose last statement diverges has type `!`
(Never), which unifies with any type. `infer_block` must honour this.

## Fix

Change `infer_stmt` to return `InferType` instead of `()`:

- `Stmt::Return`, `Stmt::Break`, `Stmt::Continue` → return `InferType::never()`
- All other statements → return `InferType::unit()`

Update `infer_decl` (the caller of `infer_stmt`) to thread this value through.

Update `infer_block` to use the last processed item's divergence type when
there is no tail expression:

```rust
let ty = match &block.tail {
    Some(tail) => infer_expr(tail, ctx, fun_generalizations)?,
    None       => last_stmt_ty,   // Never if last stmt diverged, Unit otherwise
};
```

No changes are required in Pass 2 (`construct_*`): the constraint check is
complete after Pass 1, and typed blocks with `tail: None` are valid — the
evaluator handles divergence via the `TypedStmt::Return` / `TypedStmt::Break`
nodes.

## Acceptance Criteria

- [x] `infer_stmt` returns `InferType` (`Never` for Return/Break/Continue, `Unit` for others)
- [x] `infer_decl` propagates the divergence signal from `Stmt` variants
- [x] `infer_block` uses the last statement's type when `tail` is absent
- [x] Positive test: function whose only exit is `return` type-checks correctly (`stage4_02_return_diverges.yolo`: `double`, `sign`)
- [x] Positive test: function with multiple `return` paths and no tail type-checks correctly (`sign`, `abs_val`, `maybe_skip`)
- [x] All 131 tests pass (130 prior + 1 new)
