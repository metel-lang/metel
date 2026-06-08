# ADR-0037 — Elaboration boundary: post-inference pass with SymbolId-keyed dispatch

**Status:** Accepted  
**Sprint:** 20 (v0.8.1)  
**Tasks:** METEL-123, METEL-151, METEL-152, METEL-154

---

## Context

After the typechecker produces a `TypedModuleGraph`, every method call site carries a
`MethodDispatch::Dynamic` placeholder and every `impl` block is identified only by its
aspect's string name.  The evaluator historically resolved method dispatch at runtime by
scanning `RuntimeAspectImpl` entries keyed on `aspect_name: String`.

This created two problems:

1. **Cross-module name collision** — two aspects from different modules that both declare a
   method with the same name (e.g. two unrelated `Display` aspects) would produce
   non-deterministic dispatch when both were visible in a module.

2. **Hidden re-resolution at runtime** — dispatch targets that were fully known after
   typechecking were recomputed on every call by string comparison, requiring the
   evaluator to carry knowledge of aspect structure that the typechecker had already
   resolved.

---

## Decision

A dedicated **elaboration pass** runs between the typechecker and the evaluator.  It is
implemented in `src/elaborator/mod.rs` and is the only stage that writes `MethodDispatch`
values.

### Boundary invariant

`evaluate_graph` accepts `ElaboratedModuleGraph` (a newtype wrapping `TypedModuleGraph`),
not `TypedModuleGraph` directly.  The Rust type system enforces that elaboration has run;
calling `evaluate_graph` without prior elaboration is a compile error.

### What elaboration does

1. Builds a dispatch map `HashMap<(type_name, method_name), SymbolId>` from every
   `TypedDecl::Impl` block that has an `aspect_name`.  The key is the *receiver type name*
   plus the method name — not the method name alone — so two aspects with the same method
   name on different types are stored under distinct keys.

2. Resolves the `SymbolId` by looking up the aspect's declaring module in
   `TypeDefinitionRegistry::aspect_declaring_module` and then querying
   `ResolvedNames::symbols`.

3. Walks every expression node and upgrades `MethodDispatch::Dynamic` to either
   `MethodDispatch::Inherent` (no dispatch-map entry) or
   `MethodDispatch::Aspect { aspect_id }` (entry found).

### SymbolId-keyed runtime dispatch

`RuntimeAspectImpl` carries `aspect_id: Option<SymbolId>`.  `RuntimeRegistry` exposes
`get_aspect_method_by_id(type_name, aspect_id, method_name)` which matches on `aspect_id`
first and falls back to string-name search for builtins that pre-date the elaboration pass
and are registered without a `SymbolId`.

`TypedImplBlock::aspect_id` is populated during the typechecker's Pass 2 (`construct_impl_decl`)
using the `symbols` table threaded from `check_graph`.  This keeps the SymbolId assignment
in one place (the name resolver) and avoids a second intern table.

---

## Consequences

### Positive

- Method dispatch for elaborated call sites is O(1): `aspect_id` comparison replaces a
  linear scan over string-named aspect impls.
- Two aspects with the same name from different modules are guaranteed to dispatch
  independently; the dispatch map key is `(receiver_type, method_name)`, not `method_name`
  alone.
- The `ElaboratedModuleGraph` newtype documents the pipeline contract at the type level.
- The elaboration pass is a pure transformation with no side effects; it can be run,
  inspected, and unit-tested independently of the evaluator.

### Negative / trade-offs

- An extra pipeline stage means one more pass over the typed AST at startup.  For the
  tree-walk interpreter this cost is negligible.
- `ConstructCtx` now carries `symbols: Option<&HashMap<(Vec<String>, String), SymbolId>>`.
  The single-module path (`check` / `check_with_ctx`) passes `None`, leaving `aspect_id`
  unpopulated.  The evaluator's string fallback covers this case, but it means the
  single-module pipeline has weaker dispatch guarantees than the multi-module pipeline.

### Long-term constraint

The elaborated form is a plausible future lowering boundary for a compiler backend:
`MethodDispatch::Aspect { aspect_id }` is a direct reference to a vtable slot once aspects
become proper trait objects.  The `ElaboratedModuleGraph` boundary should be preserved as
the compiler IR intake point; stages upstream of it (typechecker, name resolver) should not
be coupled to backend-specific representations.

---

## Alternatives considered

**Lazy resolution in the evaluator** — resolve dispatch on first call and cache the result.
Rejected: this still requires the evaluator to carry aspect-registry knowledge and does not
fix the cross-module name collision for programs with two active aspects of the same name.

**String-keyed dispatch with a module-qualified name** — store `"module::AspectName"` instead
of `"AspectName"` in `RuntimeAspectImpl`.  Rejected: requires all registration sites to be
updated and still produces a stringly-typed interface that degrades at module boundaries.

**Inline dispatch during typechecking** — populate `MethodDispatch` at construction time.
Rejected: the typechecker's construction pass does not have a complete picture of which impl
blocks exist (modules are processed in topological order and a dependency's impl blocks may
not yet be in the dispatch map).  A separate post-graph pass over `TypedModuleGraph` is the
correct moment.
