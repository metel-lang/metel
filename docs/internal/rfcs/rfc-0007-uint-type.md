---
id: rfc-0007
title: "UInt Type"
date: '2026-05-21'
status: draft
---

## Summary

Add a 64-bit unsigned integer type `UInt` to complement the existing `Int` (64-bit signed). Covers the type itself, literal syntax, casting rules with `Int` and `Float`, overflow semantics, and whether `UInt` replaces `Int` as the array index type.

---

## Motivation

`Int` is sufficient for most arithmetic but awkward as an array index (negative indices are meaningless) and for bit-manipulation use cases. `UInt` is a natural addition once the type system is stable enough to handle the casting complexity.

Blocked until added: `Int ↔ UInt` casting via `as`, `TryFrom`/`Into` for fallible conversions, and `UInt` as the canonical array index type.

---

## Open Questions

- **Literal syntax**: suffix (`42u`? `42_u64`?) or inferred from context?
- **Overflow semantics**: wrapping, panicking, or saturating? Should this match RFC-0013's decision on `Int` overflow?
- **`as` casting**: `Int → UInt` truncates or panics on negative? `UInt → Int` truncates or panics on overflow?
- **Array indexing**: should `UInt` become the canonical index type, with `Int` requiring an explicit cast? Or keep `Int` for ergonomics?
- **`TryFrom`/`Into`**: does adding `UInt` motivate a fallible cast trait alongside `From`/`as`?

---

## Decision

**Outcome:** *(pending)*  
**Target:** *(blank until accepted)*

*(Decision rationale goes here when the RFC is evaluated.)*
