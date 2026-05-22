---
id: rfc-0012
title: "Derived Traits"
date: '2026-05-21'
status: draft
---

## Summary

Define a mechanism for automatically generating trait implementations for structs and enums — the equivalent of Rust's `#[derive(...)]`. The primary use cases are `Eq`, `Ord`/`Comparable`, `Display`, `Clone`, and `Hash`.

---

## Motivation

Writing `impl Eq for Point { fun eq(self, other: Point) -> Bool { self.x == other.x && self.y == other.y } }` by hand for every struct is tedious and error-prone. A derive mechanism generates these implementations structurally (field-by-field for structs, variant-by-variant for enums).

Requires: the trait system (v0.2) and the operator overloading traits (RFC-0011) for `Eq`/`Ord`.

---

## Open Questions

- **Syntax**: Rust's `#[derive(Eq, Ord)]` attribute syntax is foreign to Gust's style. Options:
  - `derive Eq, Ord for Point { ... }` — keyword instead of attribute
  - `impl derive(Eq, Ord) for Point` — embedded in the impl block syntax
  - `struct Point derives Eq, Ord { ... }` — inline on the declaration
- **Derivable traits**: which traits can be auto-derived? Is the set fixed (compiler-known) or extensible by user-defined derive macros?
- **Derive macros**: does Gust want a macro/metaprogramming system, or is derive a closed set of compiler-known structural derivations?
- **Partial derivation**: what if a field's type doesn't implement the derived trait — compile error, or derive is simply unavailable?
- **`Display` vs `From` for string conversion** (#76): `print` currently only accepts `String`. When traits land, `print` should accept any type with a string representation. The question is which trait owns that conversion:
  - A `Display` trait (`fun to_string(self) -> String`) implemented by the source type — the natural direction for user-defined types.
  - `String` implementing `From<T>` for each printable type — consistent with the `from` pattern but puts the responsibility on `String`, which cannot know about user-defined types without open dispatch.
  These serve different purposes (`From`/`Into` for type conversion, `Display` for human-readable output) and should likely remain separate traits. Deriving `Display` here means generating a structural `to_string` implementation; it does not imply a `From<T> for String` impl. Resolve before finalising the `print` signature in the runtime spec.

---

## Decision

**Outcome:** *(pending)*  
**Target:** *(blank until accepted)*

*(Decision rationale goes here when the RFC is evaluated.)*
