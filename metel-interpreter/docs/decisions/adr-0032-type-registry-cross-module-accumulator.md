# ADR-0032 — TypeDefinitionRegistry as the cross-module type accumulator

**Date:** 2026-05-30
**Status:** Accepted

## Context

`check_graph` processes modules in topological order. Each module's inference pass needs to know the resolved struct/enum/method/aspect definitions from every module that has already been processed (its transitive dependencies), so that field references and method calls across modules resolve correctly.

The original design (pre-METEL-3) accumulated raw `Decl` AST nodes in a `Vec<Decl>` (`type_context`) and prepended them to a synthetic `Program` passed to `build_registry` before each module was processed. This re-processed already-resolved declarations from scratch on every module, and — more critically — re-ran `build_registry` on unresolved `Decl` nodes, which could fail to resolve cross-module type references in struct field types (e.g. module B imports a struct from A whose field type is defined in C, which B doesn't import).

## Decision

After `check_impl` finishes for a module, consume `InferContext` via `into_registry()` to extract its `TypeDefinitionRegistry`. Pass the accumulated registry as `base_registry: &TypeDefinitionRegistry` to each subsequent module's `check_impl`. Inside `check_impl`, merge it into the freshly-built registry via `merge_from` before inference begins.

This ensures subsequent modules start with already-resolved type data rather than raw AST nodes.

## Consequences

- `check_impl` signature changes from `type_context: &[Decl]` to `base_registry: &TypeDefinitionRegistry` and returns a third value: `TypeDefinitionRegistry`.
- `TypeDefinitionRegistry` must implement `merge_from`, which copies entries without overwriting locally-defined names (local definitions shadow transitive ones).
- `into_registry` consumes `InferContext`, so it must be called after both inference and construction passes complete.
- The `merge_from` order matters: call `reg.merge_from(base_registry)` *after* `build_registry(program)` so locally-defined types shadow any same-named dependency types.

## Invariant

`merge_from` uses `or_insert_with` for all maps — it never overwrites existing entries. Reversing this (inserting `base_registry` entries first and letting the module's own entries shadow them) would work the same way, but the current order (`build_registry` first, then `merge_from`) is more efficient because the local entries are inserted once and never compared against.
