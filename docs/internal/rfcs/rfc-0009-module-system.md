---
id: rfc-0009
title: "Module System"
date: '2026-05-21'
status: draft
---

## Summary

Design the module system: how source files map to modules, how names are imported and exported, the `use` keyword semantics, visibility (`pub`), and re-exports (`pub use`). This is the largest deferred feature — it blocks the standard library, multi-file programs, and all visibility control.

---

## Motivation

All v0.1 programs are single-file. Adding a module system unlocks:

- Multi-file programs and code organisation
- A standard library (math, string, io, collections)
- Visibility control — `pub` to export, private by default
- Re-exports for public API shaping

The `use` keyword is already a reserved word in the grammar.

---

## Open Questions

- **File-to-module mapping**: one file = one module (Go/Rust style)? Directory = module? Explicit `mod` declarations?
- **`use` syntax**: `use path::to::Name;` (Rust-style)? `use "path/to/file"` (Go-style)? Glob imports?
- **Visibility default**: private by default, `pub` to export (Rust-style) — or public by default, explicit hiding?
- **`pub use` re-exports**: are they needed at the same time as the module system, or can they be deferred?
- **Circular imports**: allowed (resolved lazily) or a compile error?
- **Standard library path**: is the stdlib a special root (e.g. `use std::math`) or a normal module?
- **Single-file compatibility**: are existing v0.1 single-file programs valid without any `use` or `mod` declarations?

---

## Decision

**Outcome:** *(pending)*  
**Target:** *(blank until accepted)*

*(Decision rationale goes here when the RFC is evaluated.)*
