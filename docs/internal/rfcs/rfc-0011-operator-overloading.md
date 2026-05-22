---
id: rfc-0011
title: "Operator Overloading Traits"
date: '2026-05-21'
status: draft
---

## Summary

Define the set of traits that map to built-in operators (`+`, `-`, `*`, `/`, `%`, `==`, `<`, etc.), their method signatures, and how the compiler desugars operator expressions into trait method calls.

---

## Motivation

Currently, operators are hardcoded for primitive types. User-defined types cannot participate in arithmetic, comparison, or equality expressions. Operator overloading via traits (Rust/Haskell style) makes user types first-class in expressions without adding new syntax.

Additive safety: this feature can be added without breaking any existing v0.x programs.

Requires: the trait system (v0.2).

---

## Open Questions

- **Trait names**: `Add`, `Sub`, `Mul`, `Div`, `Rem` (Rust-style)? Something else?
- **Return type**: `Add` returns `Self`? Or an associated `Output` type (allows `Vec + Vec = Vec` and `Vec + &Vec = Vec`)?
- **Comparison traits**: `Eq` (equality, `==`/`!=`) and `Comparable` / `Ord` (ordering, `<`/`<=`/`>`/`>=`) — are these unified or separate?
- **`Display` / `ToString`**: does string interpolation (RFC-0010) depend on a `Display` trait here? If so, they need to be designed together.
- **Compound assignment** (`+=`, `-=`, etc.): separate `AddAssign` traits or derived from `Add`?
- **Negation** (`-x`, `!x`): `Neg` and `Not` traits?

---

## Decision

**Outcome:** *(pending)*  
**Target:** *(blank until accepted)*

*(Decision rationale goes here when the RFC is evaluated.)*
