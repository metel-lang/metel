---
id: adr-0036
title: "Explicit Receiver Dispatch for &self and &mut self Methods"
date: '2026-06-02'
status: active
---

## Context

Before RFC-0044, all method calls used a single `call_function` path that cloned `self` into the call frame. Mutations to `self` inside a `mut self` method were returned via `call_function_mut_self` (a special convention that returns `(Signal, updated_self)`) and written back by the caller. This worked for iterators but was ad-hoc and did not generalise to `&self` / `&mut self` as first-class receiver kinds.

## Decision

RFC-0044 introduces three explicit receiver kinds in the AST and typechecker:

| Receiver syntax | `ReceiverKind` | Behaviour |
|---|---|---|
| `self` (value) | `Value` | Existing path — self is cloned into the frame |
| `&self` | `Ref` | Self is not cloned; method receives a read-only view (currently equivalent to value for the tree-walk interpreter) |
| `&mut self` | `MutRef` | Self's binding `Rc` is looked up in the caller's environment; the method frame captures it by shared pointer; mutations inside the method are visible immediately through the `Rc<RefCell<Value>>` |

`&mut self` dispatch is implemented in `evaluator/call.rs`. The caller resolves the receiver to its `Rc<RefCell<Value>>` via `env.get_rc`, passes it into the call frame as a binding named `self`, and after the call reads the (possibly mutated) value back out of the same `Rc`. No writeback is needed — the `Rc` is shared.

## Consequences

- Nested field mutation through `&mut self` (e.g. `self.inner.x = 1`) works correctly because the `Rc` for `self` is held open across the call.
- `call_function_mut_self` is retained only for the `for-in` iterator path where `Iterable::next` takes value `self`. If `Iterable` is redefined with `&mut self`, that function can be deleted.
- The `call_function_ref_self` / `call_function_mut_ref_self` helpers in `call.rs` are the canonical dispatch points; do not add receiver-kind logic elsewhere.
- This design is specific to the tree-walk interpreter. A compiled backend would model `&mut self` as a mutable reference argument rather than a shared `Rc`.
