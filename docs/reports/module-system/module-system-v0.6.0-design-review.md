# Module System v0.6.0 — Design Review

**Date:** 2026-05-28
**Sprint:** v0.6.0
**RFC:** RFC-0031

This document records a pre-implementation design review of the v0.6.0 module semantics sprint. It identifies gaps in the planned work, design weaknesses, and future pain-points, with resolutions and tracking issues for each finding.

---

## Findings Summary

| # | Category | Title | Severity | Tracking |
|---|---|---|---|---|
| 1 | Gap | `ResolvedNames` not wired into the pipeline | Critical | #190 |
| 2 | Gap | `ModuleExports` cannot distinguish T0009 from T0003 | High | #191 |
| 3 | Gap | ScopedEnv seeding order for local decls vs. imports unspecified | Medium | #173 updated |
| 4 | Gap | `StdPrelude::default()` scope undefined | Medium | #188 updated |
| 5 | Weakness | Declaration name collisions across non-importing modules | Medium | #192 |
| 6 | Weakness | T0011 disambiguation hint is one-sided | Low | #177 updated |
| 7 | Weakness | No type-level guarantee that normalization has run | Low | #185 updated |
| 8 | Pain-point | `StdPrelude` and real std modules will eventually diverge | Future | #193 |
| 9 | Pain-point | `ScopedEnv` memory growth with many glob imports | Future | noted |

---

## Finding 1 — `ResolvedNames` not wired into the pipeline (Critical)

### Problem

`name_resolver::resolve(graph)` takes the whole `ModuleGraph` and returns a single `ResolvedNames` struct containing a `scopes: HashMap<Vec<String>, ModuleScope>` map. This struct is not stored in `ModuleGraph` or `LoadedModule` and is currently unused in the main pipeline (only called in tests).

Both the path normalizer (#185) and the scope builder (#173) depend on per-module `ModuleScope` being available. The pipeline `load_root → normalize → check_graph` implicitly assumes `ResolvedNames` is already in the graph, but nothing puts it there.

### Resolution

The pipeline must explicitly become:

```
load_root → resolve → normalize → check_graph → evaluate_graph
```

`resolve()` is called on the completed `ModuleGraph` immediately after loading. Its output — a `ResolvedNames` with per-module `ModuleScope` — is either:

- Attached to `ModuleGraph` as a field: `pub resolved: Option<ResolvedNames>`
- Or passed alongside the graph as a separate parameter to `normalize` and `check_graph`

The first option is simpler and keeps the graph self-contained. Every entry point to the pipeline (CLI, tests, REPL) must call `resolve()` before proceeding. Forgetting it should be made a compile-time error if possible — if `normalize` takes `(ModuleGraph, ResolvedNames)` as separate parameters, the caller is forced to provide both explicitly.

**Tracking:** Issue #190

---

## Finding 2 — `ModuleExports` cannot distinguish T0009 from T0003 (High)

### Problem

The proposed `ModuleExports` only stores `pub_schemes` and `pub_types`. When the scope builder looks up a name from a dependency module and finds it absent from `pub_schemes`, it cannot distinguish:

- The name does not exist in the module → `T0003` (undefined name)
- The name exists but is private → `T0009` (private item)

Without the distinction, T0009 is effectively unreachable. Every private-item access produces the misleading T0003 error.

### Resolution

When a name is absent from `pub_schemes`, look it up directly in the source module's `program.decls` inside the `NormalizedModuleGraph`. `check_graph` already has the full graph in scope — no extra data structure is required:

```rust
if source_exports.pub_schemes.contains_key(name) {
    // resolved
} else {
    let is_private = graph.modules.iter()
        .find(|m| m.module_path == *source_path)
        .map(|m| m.program.decls.iter().any(|d| decl_name(d) == name))
        .unwrap_or(false);
    return Err(if is_private { T0009 } else { T0003 });
}
```

`ModuleExports` stays pure — it holds only public type information with no redundant name sets to keep in sync. The lookup is O(declarations in module) and only occurs on error paths.

**Tracking:** Issue #191

---

## Finding 3 — ScopedEnv seeding order for local decls vs. imports unspecified (Medium)

### Problem

The `ScopedEnv` conflict table in RFC-0031 and #173 covers every import-vs-import combination, but says nothing about the interaction between a module's own declarations and its imports. RFC-0030 states that local declarations take precedence over imports. If the scope builder seeds imports first and local decls second, this is enforced correctly. If it seeds local decls first and imports second, imports silently overwrite local names — a bug with no obvious symptom.

### Resolution

The seeding order is:

1. Imported names (explicit, then globs) — added first, establishing the imported scope
2. Local declarations — added after, silently winning over any imported name with the same identifier

This mirrors the "local always shadows import" rule in RFC-0030. Added to #173 acceptance criteria.

---

## Finding 4 — `StdPrelude::default()` scope undefined (Medium)

### Problem

`#188` says `StdPrelude::default()` covers "at minimum: `std::core::{Int, Float, Bool, String}`." However `Perhaps<T>`, `Result<T, E>`, and the built-in collection types (`Array`, `Tuple`) are currently handled by special-cased logic in `construction.rs` and `conversions.rs`, not as externally resolved names. If a user writes `import std::core::Perhaps`, it is not clear whether `StdPrelude::default()` covers it or whether it remains intrinsic.

### Resolution

Draw an explicit boundary in #188:

- **`StdPrelude::default()` covers:** all names that a user might write in an `import std::core::*` statement and expect to resolve — `Int`, `Float`, `Bool`, `String`, `Perhaps`, `Result`.
- **Intrinsics remain intrinsic:** `Array<T>`, tuples, and `Never` are not importable by name in v0.6.0; they are produced by the parser and type system directly and do not need a `StdPrelude` entry.

The boundary must be documented in #188 so it is not rediscovered during implementation.

---

## Finding 5 — Declaration name collisions across non-importing modules (Medium)

### Problem

Two modules can each declare `fun tokenize()` without importing each other. The per-module typechecker is correct — each module sees only its own scope, so no conflict is detected. But `evaluate_graph` concatenates both modules' `TypedDecl` lists into one flat environment. The second `tokenize` silently overwrites the first at runtime. This can produce wrong program behaviour with no error or warning.

This is a consequence of the flat evaluator (tracked in #189 for v0.7.0) but it should be documented as a known limitation, and considered for an explicit runtime-error guard.

### Resolution

Two actions:

1. Document this as a known limitation in `metel-interpreter/docs/evaluator.md` — "In v0.6.0, two modules declaring the same top-level name produce undefined behaviour at runtime if both are reachable. The typechecker does not detect this; it will be resolved when per-module runtime environments are introduced (#189)."
2. Add a best-effort detection pass in `evaluate_graph`: before building the flat environment, scan for duplicate declaration names across modules and emit a runtime warning (not a hard error, since the typechecker approved the program). Hard error requires per-module runtime scope (#189).

**Tracking:** Issue #192

---

## Finding 6 — T0011 disambiguation hint is one-sided (Low)

### Problem

The T0011 error note reads: `use an explicit import to disambiguate: import parser::Token`. This assumes the user wants `parser`'s version. They may want `lexer`'s.

### Resolution

List both options:

```
note: use an explicit import to disambiguate:
  `import parser::Token`  or  `import lexer::Token`
```

Updated in #177.

---

## Finding 7 — No type-level guarantee that normalization has run (Low)

### Problem

`Expr::Path` is the pre-normalization form; `Expr::ResolvedPath` is the post-normalization form. Nothing in the type system prevents a pass from receiving an un-normalized AST. A future pass added in the wrong order — or a test that constructs an AST without normalizing — will silently mishandle qualified paths.

### Resolution

Two mitigations:

1. `normalize()` consumes the `ModuleGraph` and returns a new `NormalizedModuleGraph` newtype that wraps `ModuleGraph`. `check_graph` accepts `NormalizedModuleGraph`, not `ModuleGraph`. This makes it a compile-time error to call `check_graph` without normalizing first.
2. If the newtype approach is too heavy for v0.6.0, add a `debug_assert!` at the start of `check_graph` that panics if any `Expr::Path` with multiple segments is found in the input.

The newtype is the correct long-term solution. Updated in #185.

---

## Finding 8 — `StdPrelude` and real std modules will eventually diverge (Future)

### Problem

`StdPrelude::default()` is a hardcoded register of built-in type schemes. When the standard library is implemented as real `.mln` files (a future sprint), both sources of `std::core::Int` will exist simultaneously. One must become authoritative and the other deleted. The longer `StdPrelude` accumulates entries, the more friction this transition creates.

### Resolution

Create a tracking issue now so the StdPrelude is not forgotten. When the real `std/core.mln` is introduced, `StdPrelude::default()` entries for the types it defines must be removed in the same PR.

**Tracking:** Issue #193

---

## Finding 9 — `ScopedEnv` memory growth with many glob imports (Future)

### Problem

A module using `import a::*; import b::*; import c::*;` materialises the full pub surface of all three modules into its `ScopedEnv`. All imported names — including unused ones — are carried through the inference pass. For a large standard library or third-party module, this could be significant.

### Resolution

No action in v0.6.0. Note it as a future optimisation: lazy scope resolution (look up names in `GlobalExports` on demand rather than pre-materialising them). Revisit when the standard library is large enough for this to matter.
