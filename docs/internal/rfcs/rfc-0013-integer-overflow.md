---
id: rfc-0013
title: "Integer Overflow Behaviour"
date: '2026-05-21'
status: draft
---

## Summary

Define what happens when an integer arithmetic operation overflows the bounds of `Int` (or `UInt`, when added). Options are wrapping, panicking, or saturating — potentially with a debug/release split.

---

## Motivation

The current interpreter uses Rust's `i64` arithmetic which wraps on overflow in release builds and panics in debug builds. This behaviour is inherited from the implementation, not specified by the language. Defining it explicitly matters for:

- Predictable program behaviour across implementations (interpreter vs future compiler)
- Communicating the contract to users writing arithmetic-heavy code

---

## Open Questions

- **Policy options**:
  - **Wrapping always**: simple, predictable, but silent bugs
  - **Panic always**: safe, but expensive to check every operation
  - **Debug panics / release wrapping**: Rust's approach — best of both but adds build-mode complexity
  - **Saturating**: clamps to min/max instead of wrapping; unusual for integer semantics
- **Configurability**: should the behaviour be a language-level guarantee or a runtime/compiler flag?
- **`UInt` alignment**: if RFC-0007 is accepted, `UInt` overflow policy should be decided here too.
- **Explicit wrapping operations**: should there be `wrapping_add` / `checked_add` functions regardless of default policy?

---

## Decision

**Outcome:** *(pending)*  
**Target:** *(blank until accepted)*

*(Decision rationale goes here when the RFC is evaluated.)*
