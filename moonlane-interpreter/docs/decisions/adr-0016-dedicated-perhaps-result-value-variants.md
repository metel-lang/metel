# ADR-0016: Dedicated Value::Perhaps and Value::Result Variants

**Status:** Superseded by ADR-0028  
**Sprint:** 7 (v0.4.1)  
**Issue:** #142

---

## Context

The evaluator's `Value` enum has a general `Value::Enum { name, variant, fields }` variant for user-defined enum types. Before v0.4.1, `Perhaps<T>` and `Result<T,E>` values at runtime were split:

- `nope` literal → `Value::Perhaps(None)` (dedicated variant)
- `Perhaps::Some { value: v }` struct literal → `Value::Enum { name: "Perhaps", variant: "Some", fields: {"value": v} }`
- `Result::Ok { value: v }` → `Value::Enum { name: "Result", variant: "Ok", fields: {"value": v} }`
- `Result::Err { error: e }` → `Value::Enum { name: "Result", variant: "Err", fields: {"error": e} }`

This split representation had two concrete problems:

1. **Double-dispatch everywhere**: `eval_for_in`, `PropagateError`, and `match_pattern` each needed two match arms for Perhaps — one for `Value::Perhaps(None)` and one for `Value::Enum { name: "Perhaps", .. }`.
2. **Latent pattern match bug**: `Pattern::Nope` only matched `Value::Perhaps(None)`. If `Perhaps::Nope {}` were ever constructed as a struct literal (it can be), the resulting `Value::Enum { name: "Perhaps", variant: "Nope", .. }` would silently fail to match `Pattern::Nope`.

## Decision

All `Perhaps` and `Result` values are now routed through dedicated variants:

- `Value::Perhaps(Option<Box<Value>>)` — `None` for `nope`/`Perhaps::Nope`; `Some(v)` for `Perhaps::Some { value: v }`
- `Value::Result(Result<Box<Value>, Box<Value>>)` — `Ok(v)` for `Result::Ok { value: v }`; `Err(e)` for `Result::Err { error: e }`

Construction is intercepted in `eval_untyped_struct_literal` and `eval_struct_literal` (the `TypedExpr::StructLiteral` arm): when `path[0]` is `"Perhaps"` or `"Result"`, the evaluator constructs the dedicated variant instead of `Value::Enum`. Unit-variant paths for `Perhaps::Nope` are also intercepted in the `Path` evaluation arms.

`Value::Enum` is no longer constructed for `Perhaps` or `Result` at any point.

## Consequences

- `eval_for_in` collapses from a two-branch match to a single `Value::Perhaps` arm.
- `PropagateError` (typed and untyped) becomes a single `Value::Result(Ok/Err)` match — no field-map lookup.
- `match_pattern` for `Pattern::EnumVariant` handles `Perhaps` and `Result` as special cases before falling through to the general `Value::Enum` arm.
- `call_function` and `call_function_mut_self` wrap `Signal::PropagateErr` errors in `Value::Result(Err(...))` on the way out.
- `format_value` renders both variants correctly.

## Constraints for future contributors

- Do not add new `Value::Enum { name: "Perhaps", .. }` or `Value::Enum { name: "Result", .. }` construction sites. Route all Perhaps/Result values through the dedicated variants.
- If a new construction path is added (e.g. a desugaring pass), it must intercept `Perhaps`/`Result` struct literals and produce `Value::Perhaps`/`Value::Result`.
- `Value::Enum` continues to be the correct representation for all user-defined enums.
