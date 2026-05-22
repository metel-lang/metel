---
id: rfc-0015
title: "Unwrap Syntax"
date: '2026-05-21'
status: draft
---

## Summary

Decide the final surface syntax for unwrapping `Perhaps<T>` and `Result<T, E>`: keep `.yolo()` as a method call, or introduce a `yolo` keyword expression.

---

## Motivation

The spec currently describes `.yolo()` as a method call on `Perhaps<T>` and `Result<T, E>` that unwraps the value or panics. This form is simple and consistent with method call syntax, but it requires generic method support in the typechecker (the method must work on both `Perhaps<T>` and `Result<T, E>`).

An alternative is a `yolo` keyword expression — a special form handled by the compiler/typechecker directly, analogous to how `?` is handled rather than being a trait method call. This is simpler to typecheck but adds a keyword.

---

## Open Questions

- **Method form (`.yolo()`)**: requires the typechecker to support generic methods returning `T` from `Perhaps<T>` or `Result<T, E>`. Is this a derived instance of a general `Unwrap` trait, or a hardcoded special case?
- **Keyword form (`yolo expr`)**: a unary keyword expression. Simpler typechecking, distinct syntax for an inherently unsafe operation. Does it read well? `yolo list[0]`, `yolo parse_int(s)`?
- **Both forms**: is there value in supporting both? (Likely not — pick one for consistency.)
- **Panic message**: should `.yolo()` / `yolo` accept an optional message argument for a better panic message? E.g. `.yolo("user must exist")`?

---

## Decision

**Outcome:** *(pending)*  
**Target:** *(blank until accepted)*

*(Decision rationale goes here when the RFC is evaluated.)*
