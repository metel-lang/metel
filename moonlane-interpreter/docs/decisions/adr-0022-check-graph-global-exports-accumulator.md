# ADR-0022: `check_graph` Uses a `GlobalExports` Accumulator

**Status:** Accepted  
**Date:** 2026-05-28  
**Tracking issues:** #172, #183

---

## Context

`check_graph` must typecheck each module against only the names it explicitly imported from already-checked modules. The typechecker's two passes (inference and construction) must both see imported function schemes — the inference pass to emit correct constraints, and the construction pass to build typed call expressions.

---

## Decision

`check_graph` maintains a `GlobalExports` map (`HashMap<ModulePath, ModuleExports>`) that is populated incrementally as modules are checked in topological order:

1. After checking module M, its `pub`-declared function schemes are filtered (via `filter_pub_schemes`) and inserted into `GlobalExports[M.module_path]`.
2. Before checking module N, `build_import_schemes` reads `GlobalExports` to assemble `imported_schemes: SchemeEnv` — the set of name→scheme bindings N can see from its imports.
3. `check_impl(program, imported_schemes, type_context)` seeds `imported_schemes` into the `InferContext` (via `ctx.bind_poly`) **and** merges them into the `scheme_env` passed to the construction pass.

The `type_context` accumulator (a `Vec<Decl>`) handles cross-module struct/enum type definitions: it is extended with each module's struct/enum/impl/aspect decls after checking, and passed to `check_impl` so the type registry knows about imported types.

---

## Critical Invariant

**`imported_schemes` must be seeded into BOTH the inference pass (via `ctx.bind_poly`) and the construction pass (via `scheme_env.entry(...).or_insert`).** These are independent lookups:

- The inference pass uses `InferContext::lookup` (checks `poly_env`).
- The construction pass uses `ConstructCtx::scheme_env` for polymorphic callees, and `ConstructCtx::env` (derived from `scheme_env`) for monomorphic ones.

If `imported_schemes` is seeded into inference but not construction, the inference pass succeeds but the construction pass fails with "undefined name". This is the bug fixed during sprint 10.

---

## Alternatives Rejected

**Two-pass graph:** Collect all exports first, then typecheck all modules. This would allow any module to reference any other module's names regardless of declaration order. Rejected because: (a) it doesn't model actual import semantics, (b) it would make circular-import detection harder, (c) forward references across modules violate the no-cross-module-inference invariant.

**Lazy / on-demand typechecking:** Typecheck module B when module A first references a name from B. Rejected as too complex for v0.6.0 scope (RFC-0031 option C).

---

## Consequences

- Processing order matters: `graph.modules()` must return modules in topological order (dependencies before dependents). This is guaranteed by `module_loader::load_root` via DFS post-order.
- `StdPrelude` is seeded into `GlobalExports[["std", "core"]]` before the loop, so `std::` imports resolve without a real std file.
- When #179 lands and the flat merge is removed, `GlobalExports` becomes the sole mechanism for cross-module name sharing.
