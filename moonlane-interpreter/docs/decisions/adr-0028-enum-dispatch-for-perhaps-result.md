---
id: adr-0028
title: "Unified Value::Enum Dispatch for Perhaps and Result"
date: '2026-05-29'
status: active
supersedes: adr-0016
---

## Context

ADR-0016 introduced dedicated `Value::Perhaps(Option<Box<Value>>)` and `Value::Result(Result<Box<Value>, Box<Value>>)` variants to eliminate a split representation (some constructors producing `Value::Enum`, others `Value::Perhaps(None)`) and to simplify dispatch in `eval_for_in`, `PropagateError`, and `match_pattern`.

By v0.6.2, user-defined enums and built-in enums (`Perhaps`, `Result`) follow the same `Value::Enum { name, variant, fields }` shape throughout the AST and type system. Keeping two additional dedicated variants creates asymmetry: every new code path that handles enums must remember to add special-case arms for `Perhaps` and `Result`, even though the logic is identical to the general `Value::Enum` path.

The original motivating problems from ADR-0016 no longer apply:
- The `nope`/`None` literal now consistently produces `Value::Enum { name: "Perhaps", variant: "None", fields: {} }` — the split construction that caused the latent pattern bug is gone.
- `PropagateError` and `Iterable::next` use field-map lookup (`fields["value"]`, `fields["error"]`) rather than destructuring dedicated variants, so the per-path special-casing is not needed.

## Decision

Remove `Value::Perhaps` and `Value::Result` from the `Value` enum. All `Perhaps` and `Result` values are represented as `Value::Enum { name, variant, fields }`, identical to user-defined enum values.

Construction sites (`eval_struct_literal`, `eval_untyped_struct_literal`, `None` literal handling) no longer intercept `Perhaps`/`Result` paths — they fall through to the general `Value::Enum` construction arm.

`PropagateError` (`?`) matches `Value::Enum { name: "Result", variant: "Ok"/"Err", .. }` and reads `fields["value"]` / `fields["error"]`.

`Iterable::next` matches `Value::Enum { name: "Perhaps", variant: "Some"/"None", .. }` and reads `fields["value"]`.

`call_function` / `call_function_mut_self` wrap `Signal::PropagateErr` errors as `Value::Enum { name: "Result", variant: "Err", fields: {"error": e} }`.

`match_pattern` for `Pattern::EnumVariant` uses a single general `Value::Enum` arm; no special cases for `Perhaps`/`Result`.

`format_value` (display.rs) uses guard arms to render `Perhaps`/`Result` in the familiar `Some(v)` / `None` / `Ok(v)` / `Err(e)` format.

## Consequences

- All enum code paths — built-in and user-defined — are handled by the same `Value::Enum` arm. New language features that handle enums generically do not need to enumerate special cases.
- `PropagateError` and `Iterable::next` use field-map lookups rather than variant destructuring. Performance is equivalent (HashMap with a short fixed key).
- `display.rs` retains explicit guard arms for `Perhaps`/`Result` display to preserve the user-visible `Some(v)` / `None` / `Ok(v)` / `Err(e)` output format. These guards are presentational only and do not affect evaluation semantics.

## Constraints for future contributors

- `Value::Perhaps` and `Value::Result` no longer exist. Do not reintroduce them.
- All `Perhaps`/`Result` construction must produce `Value::Enum { name: "Perhaps"/"Result", .. }`.
- The display guards in `display.rs` are the only place where `Perhaps`/`Result` are singled out by name. Any new display logic should mirror this pattern.
