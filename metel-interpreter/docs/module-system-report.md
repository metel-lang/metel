# Metel Module System: Implementation Report

**Date:** 2026-05-29  
**Covers:** v0.5.0 – v0.6.2 (Sprints 9–13)  
**Purpose:** Full technical account of what was implemented, how it works, what is missing, and what technical debt exists. Written to allow informed design decisions without requiring a code reading session.

---

## Table of Contents

1. [The Pipeline](#1-the-pipeline)
2. [Pass 1 — Module Loading](#2-pass-1--module-loading)
3. [Pass 2 — Name Resolution](#3-pass-2--name-resolution)
4. [Pass 3 — Path Normalization](#4-pass-3--path-normalization)
5. [Pass 4 — Typechecking](#5-pass-4--typechecking)
6. [Pass 5 — Evaluation](#6-pass-5--evaluation)
7. [What Is Missing](#7-what-is-missing)
8. [Technical Debt](#8-technical-debt)
9. [Summary Table](#9-summary-table)

---

## 1. The Pipeline

The module system runs in five sequential passes. Each pass's output type is the only accepted input to the next — there are no shortcuts or bypass paths.

```
Source files
    │
    ▼
module_loader::load_root()          → ModuleGraph
    │   DFS file discovery, cycle detection, path assignment
    ▼
name_resolver::resolve()            → ResolvedNames
    │   Scope building, pub_surface, import/re-export resolution
    ▼
path_normalizer::normalize()        → NormalizedModuleGraph
    │   AST rewriting: qualified paths → ResolvedPath nodes
    ▼
typechecker::check_graph()          → TypedModuleGraph
    │   Per-module type inference + construction, GlobalExports accumulator
    ▼
evaluator::evaluate_graph()         → Result<(), Error>
    │   Per-module environments, imported_names seeding, 3-pass eval
    ▼
Output / runtime error
```

The pipeline is orchestrated in `src/main.rs` (the `run` function, lines 44–66). There is no partial execution — all five passes run for every program, single-file or multi-file.

---

## 2. Pass 1 — Module Loading

**File:** `src/module_loader.rs` (255 lines)

### What it does

Discovers and parses every `.mln` file reachable from the root via `import` declarations. Uses depth-first recursion with:

- A `HashSet<PathBuf>` visited set to deduplicate (each file loaded at most once).
- A `Vec<PathBuf>` call stack to detect circular imports and report the full chain.

For every file it produces a `LoadedModule`:

```rust
pub struct LoadedModule {
    pub module_path: Vec<String>,  // e.g. ["parser", "ast"]
    pub file_path:   PathBuf,
    pub program:     Program,      // untyped AST, not yet name-resolved
}
```

All modules are collected into a `ModuleGraph`:

```rust
pub struct ModuleGraph {
    pub root: PathBuf,
    pub modules: Vec<LoadedModule>,
}
```

### Module path assignment

Module paths are hierarchical absolute paths from the project root. The file-to-path mapping is direct: `::` in an import becomes `/` in the filesystem, and the resulting path segments become the `module_path`. For example:

- `import parser::ast::Ast;` → file `parser/ast.mln` → `module_path: ["parser", "ast"]`

Path roots are resolved by `child_module_path` (lines 101–113):

| PathRoot | Result |
|---|---|
| `Root` | `[]` (empty — project root) |
| `Self_` | parent module's path |
| `Super` | parent module's path minus last segment |
| `Name(n)` | parent module's path + `[n]` |

`std::` prefixed imports return `None` from the resolver — they are virtual modules with no corresponding file and are skipped during loading.

### Ordering

The output `Vec<LoadedModule>` is in DFS traversal order: dependencies are visited before dependents. This DFS order is the topological order that every downstream pass relies on. If a module is imported by multiple others, it appears once (the first visit) and subsequent references find it already in the visited set.

### What the loader does NOT do

- It does not validate visibility — any file can `import` any other.
- It does not resolve name references or check that imported names exist.
- It does not understand `std::core` contents.
- It does not handle diamond dependencies specially — DFS order handles them correctly by visiting each file exactly once.

---

## 3. Pass 2 — Name Resolution

**File:** `src/name_resolver.rs` (722 lines)

### What it does

Takes the `ModuleGraph` and produces `ResolvedNames` — the scope metadata read by every subsequent pass:

```rust
pub struct ResolvedNames {
    pub scopes:      HashMap<Vec<String>, ModuleScope>,
    pub pub_surface: HashMap<Vec<String>, HashSet<String>>,
}
```

`pub_surface` maps each module's path to the set of names it publicly exports. `scopes` maps each module's path to its full import scope:

```rust
pub struct ModuleScope {
    pub module_path: Vec<String>,
    pub explicit:    HashMap<String, ImportBinding>,     // name → where it came from
    pub globs:       Vec<(GlobTier, Vec<String>)>,      // ordered glob sources
    pub re_exports:  HashMap<String, ImportBinding>,     // names this module re-exports
}

pub struct ImportBinding {
    pub source_module: Vec<String>,   // which module owns the name
    pub source_name:   String,        // canonical name in that module
    pub kind:          BindingKind,   // Item (value/type) or Module (handle)
}
```

### Three-pass structure

The resolver runs three passes over the full module graph before returning:

**Pass 1** — Collect locally-declared `pub` names from each module's AST into `pub_surface`. Only names with the `pub` keyword are included.

**Pass 2** — Process `export` declarations. Re-exports extend `pub_surface` further: `export path::Name` pulls a name from a submodule and makes it part of this module's public API. Re-exporting a private item is a T0009 error caught here.

**Pass 3** — Resolve `import` declarations using the finalized `pub_surface`. Each import is recorded in the module's `ModuleScope` as either:
- An explicit `ImportBinding` (single name, alias, or group import), or
- A glob entry in `globs` (from `import path::*`).

After these three passes, `std::core` is injected into every module's `globs` at `GlobTier::Std`.

### Import priority and conflict rules

```
GlobTier::Std  (lowest)   ← std::core auto-import
GlobTier::User            ← explicit import path::*
Explicit binding          ← import path::Name (or as Alias)
```

- Explicit binding always wins over any glob.
- `User` glob silently wins over `Std` glob for the same name (no error).
- Two `User` globs that both export the same referenced name → T0011 error.
- Two explicit bindings to the same local name → error.

### Absolute path computation

The function `absolute_base` (lines 237–257) converts `PathRoot` variants to absolute module paths during import resolution. It must agree exactly with `module_loader::child_module_path` — both functions implement the same table:

| PathRoot | Result |
|---|---|
| `Root` | `[]` |
| `Self_` | current module path |
| `Super` | current module path minus last segment |
| `Name(n)` | current module path + `[n]` |

These two functions are independent implementations of the same logic. If they diverge, `GlobalExports` lookups in the typechecker break silently.

### What the name resolver does NOT do

- It does not resolve types or check type compatibility.
- It does not enforce visibility at usage sites — only at `export` and `import` declaration sites.
- It does not understand `std::core`'s contents (names, types, schemes). Those are the typechecker's responsibility.
- It records bindings for all imports regardless of whether the name is actually public; the typechecker enforces visibility during import scheme building.

---

## 4. Pass 3 — Path Normalization

**File:** `src/path_normalizer.rs` (341 lines)

### What it does

Rewrites every module-qualified expression path in the AST before the typechecker sees it. The typechecker only handles single-segment `Expr::Ident` and `Expr::ResolvedPath` — it must never receive a multi-segment path like `std::core::Perhaps::Some`.

The normalizer walks the full AST of every module and processes `Expr::Path` nodes with more than one segment. For each, it calls `try_resolve_path` which:

1. Checks if the first segment is a path root keyword (`root`, `self`, `super`).
2. Checks if the first segment is a loaded module name or a glob-import prefix.
3. Looks up the final name segment in the module's explicit imports or glob sources.
4. Returns the local alias name (e.g. `Perhaps` from `std::core::Perhaps`).

The rewritten node becomes `Expr::ResolvedPath { resolved, original, span }` where `resolved` is the local name and `original` preserves the source for error messages.

Struct literal paths (e.g. `std::core::Perhaps::Some { value: x }`) are handled by `try_normalize_struct_path`, which strips the module prefix and returns only the type name + variant (e.g. `["Perhaps", "Some"]`).

### The newtype guarantee

The output is `NormalizedModuleGraph`, a newtype wrapping `ModuleGraph`:

```rust
pub struct NormalizedModuleGraph(pub(crate) ModuleGraph);
```

`check_graph` accepts only `NormalizedModuleGraph`. This is a compile-time guarantee that normalization cannot be skipped.

### What normalization does NOT do

- It does not validate that referenced names are public or exist — paths it cannot resolve are left unchanged, and the typechecker will report T0003 or T0009.
- It does not resolve type expressions, only expression paths. Type annotations in the AST (e.g. `Perhaps<Int>`) use a separate `TypeExpr` node and are handled differently by the typechecker.

---

## 5. Pass 4 — Typechecking

**File:** `src/typechecker/mod.rs` (512 lines); implementation in `inference.rs`, `construction.rs`, `registry.rs`, `conversions.rs`

### Overview

`check_graph` processes modules in topological order. It maintains a `GlobalExports` accumulator that grows as each module is successfully checked. This accumulator is the mechanism by which type information flows from dependency modules to the modules that import them.

```rust
struct GlobalExports {
    modules: HashMap<ModulePath, ModuleExports>,
}
struct ModuleExports {
    pub_schemes: SchemeEnv,   // name → TypeScheme for every pub name
}
```

`std::core` is pre-seeded into `GlobalExports` before any module is processed:

```rust
global_exports.insert(
    vec!["std".to_string(), "core".to_string()],
    ModuleExports { pub_schemes: StdPrelude::default().schemes().clone() },
);
```

### Per-module processing: six steps

For each module, `check_graph` runs six operations in order:

---

**Step 1 — `check_pub_annotations`** (lines 111–155)

Every `pub fun` must have explicit return type and parameter type annotations. Fails with T0010 if any annotation is missing. This runs before inference because unannotated `pub` functions cannot safely export a scheme to other modules.

---

**Step 2 — `build_import_schemes`** (lines 252–346)

Builds the `SchemeEnv` of names this module is allowed to see from other modules. This is the set of names the typechecker will treat as known during inference of this module's declarations.

The process:

1. Process glob imports in tier order: `Std` then `User`. For each glob source module, pull all names from `GlobalExports[source_module].pub_schemes`.
2. Process explicit imports. For each, look up the name in `GlobalExports[source_module]`. If not found, scan the source module's raw `program.decls` to decide between T0009 (name exists but is private) and T0003 (name does not exist at all).
3. T0011 fires if two same-tier glob imports both export a name that is actually referenced.

The result is a flat `SchemeEnv` — a `HashMap<String, TypeScheme>` — containing every name visible to this module from its imports.

---

**Step 3 — `check_impl`** (lines 442–512)

Runs Hindley-Milner type inference and typed AST construction on this module's declarations.

The `imported_schemes` from Step 2 must be seeded into **both**:
- The inference pass, via `ctx.bind_poly(name, scheme)` for each imported name.
- The construction pass, via `scheme_env.entry(name).or_insert(scheme)` for each imported name.

**This is the single most critical invariant in the module system.** If either pass doesn't see an imported name, behaviour is silently wrong: inference may leave type variables unresolved; construction may produce incorrect typed AST nodes. The invariant is enforced only by code convention — the two seeding calls sit near each other in `check_impl` with no type-level guarantee.

The type registry is built from `type_context` (type declarations from already-checked dependency modules) plus the current module's own type declarations. This allows the typechecker to resolve struct/enum types from dependencies.

After inference and construction, the function returns the module's `scheme_env` — all names the module defines, not just public ones.

---

**Step 4 — `filter_pub_schemes`** (lines 396–425)

Takes the module's full `scheme_env` and filters it to only publicly accessible names:

- Locally declared names that have the `pub` keyword.
- Names re-exported via `export` declarations (pulled from their source modules in `GlobalExports`).

---

**Step 5 — `GlobalExports.insert`**

Adds the filtered `pub_schemes` for this module to `GlobalExports`. From this point forward, subsequent modules can import from this one.

---

**Step 6 — `imported_names` population** (lines 201–232)

Extracts `TypedModule::imported_names` for the evaluator:

```rust
pub imported_names: HashMap<String, (Vec<String>, String)>
// local_name → (source_module_path, canonical_name)
```

Glob imports are processed Std then User tier. Explicit `Item` bindings override globs. `std::core` glob entries are skipped — those names come from `register_builtins` in the evaluator, not from a real module environment.

---

### The TypedModule structure

The output of `check_graph` is a `TypedModuleGraph` containing one `TypedModule` per input module:

```rust
pub struct TypedModule {
    pub module_path:    Vec<String>,
    pub decls:          Vec<TypedDecl>,
    pub import_aliases: HashMap<String, String>,              // alias → canonical name
    pub imported_names: HashMap<String, (Vec<String>, String)>, // local → (module, canonical)
}
```

`import_aliases` handles `import path::Name as Alias` — the evaluator uses this to register both names. `imported_names` is the cross-module seeding table.

---

## 6. Pass 5 — Evaluation

**File:** `src/evaluator/mod.rs` (900+ lines); `evaluate_graph` at lines 231–266

### What it does

`evaluate_graph` processes modules in topological order (same order as `check_graph`), building one `Environment` per module and storing them in an accumulator:

```rust
let mut module_envs: HashMap<Vec<String>, Environment> = HashMap::new();
```

For each module:

1. **Create a fresh `Environment`** and call `register_builtins` — this seeds all `std::core` names (builtins, Perhaps, Result, Display, Iterable, From implementations).

2. **Seed imported names** from already-built dependency environments, using `module.imported_names`:

   ```rust
   for (local_name, (source_module, canonical_name)) in &module.imported_names {
       if let Some(src_env) = module_envs.get(source_module) {
           if let Some(val) = src_env.get(canonical_name) {
               env.define(local_name, val);
           }
       }
   }
   ```

   Because modules are in topological order, all dependency environments are already built when a module is processed.

3. **Run `run_passes`** — the three-pass evaluation of this module's declarations.

4. **Store the environment** in `module_envs` keyed by `module_path`.

After all modules, `run_main` looks up `main()` in the root module's environment (the last entry, per topological order) and executes it.

### The three-pass evaluation (`run_passes`)

Within a single module, `run_passes` runs three sub-passes to handle mutual recursion:

**Pass 1a — Placeholder bindings.** For every `fun` declaration, insert a placeholder `Value::Closure` in the environment. The placeholder exists so subsequent closures can capture the name.

**Pass 1b — Real closures.** Replace placeholders with real `Value::Closure` values. By this point, all function names are in scope, so closures can capture each other correctly (mutual recursion within one module works).

**Alias registration.** After closures exist, bind aliased import names (`import X as Y` means `Y` also resolves to the same value as the canonical name).

**Pass 2.** Evaluate top-level `let`, `mut`, and statement declarations in source order.

### What the evaluator does NOT do

- It does not re-check types — it trusts the typed AST completely.
- It does not enforce module boundaries at runtime — once a value is in an environment, it behaves like any other value.
- Cross-module mutual recursion is not supported: Pass 1a and Pass 1b run per-module, not globally across all modules first (see §7.1).

---

## 7. What Is Missing

### 7.1 Cross-module mutual recursion (#189)

`run_passes` runs all three passes for one module before moving to the next. This is correct for the normal case (module B calls module A's functions — A is fully evaluated first, B's closures capture A's real values).

What it cannot handle: if function `foo` in module A calls function `bar` in module B, and `bar` calls `foo` — true circular mutual recursion across module boundaries. Topological sort prevents circular imports, so this scenario requires a diamond or similar structure where A and B are peers that both import a third module C. In that case, when A is being evaluated, B doesn't exist yet in `module_envs`, so A's closures cannot capture B's functions.

This has no test case and no current user-facing consequence because the pattern requires specific multi-module structure that none of the test programs use. The fix requires running Pass 1a for all modules before Pass 1b for any module.

### 7.2 `Type::Perhaps` and `Type::Result` remain dedicated aliases (#214, #150)

In `src/types/mod.rs`:

```rust
pub enum Type {
    Int, Float, Bool, Str, Unit, Never,
    Tuple(Vec<Type>),
    Array(Box<Type>),
    Fun(Vec<Type>, Box<Type>),
    Named(String, Vec<Type>),
    Perhaps(Box<Type>),          // ← should be Named("Perhaps", [T])
    Result(Box<Type>, Box<Type>), // ← should be Named("Result", [T, E])
}
```

Sprint 13 removed `Value::Perhaps` and `Value::Result` from the evaluator (they now use `Value::Enum`). The typechecker-level equivalents were not removed. There are 14 uses of `Type::Perhaps` / `Type::Result` in `construction.rs` and `conversions.rs` that handle them separately from `Type::Named`. Removing them requires #214 (desugar `?`) to land first, since `PropagateError` construction explicitly matches `Type::Result`.

### 7.3 `?` operator is still a special AST node (#214)

`Expr::PropagateError` and `TypedExpr::PropagateError` are dedicated AST nodes. The inference pass has special-case logic including a **mid-inference partial solve** (the only place in the entire inference pass where `ctx.solve()` is called mid-flight) to detect whether From coercion is needed. The construction pass emits a `coercion: Option<Box<TypedExpr>>` field. The evaluator has dedicated arms for it in both the typed and untyped evaluation paths.

The desugaring to a match expression (planned in #214) will eliminate all of this, but the From coercion case needs careful handling — see §8.5.

### 7.4 Field-level visibility and field-level mutability (#158)

All struct fields are implicitly public. There is no syntax or enforcement for private fields. The spec section for modules documents item-level visibility (`pub fun`, `pub struct`) but field-level visibility and field-level mutability are not yet implemented.

**RFC-0032** (Field-Level Visibility, `docs/internal/rfcs/rfc-0032-field-level-visibility.md`) proposes making fields module-private by default and requiring explicit `pub` on each field to expose it. This is a breaking change for existing `pub struct` definitions. Deferred to a sprint following v0.6.3.

**RFC-0033** (Field-Level Mutability, `docs/internal/rfcs/rfc-0033-field-level-mutability.md`) proposes a `let` annotation on fields to mark them permanently immutable after construction, independent of the binding's mutability. Non-breaking and additive. Intended to ship in the same version as RFC-0032 since the two compose at the field annotation level (`pub let field: Type`). Deferred to the same sprint.

### 7.5 Primitives have no `std::core` paths (#150, partial)

`Int`, `Float`, `Bool`, `String` exist as `Type::Int`, `Type::Float`, `Type::Bool`, `Type::Str` in the type system — dedicated enum variants, not `Type::Named("Int", [])`. They are registered into the scheme environment by short name only. `std::core::Int` does not resolve. The #150 acceptance criteria about "core primitive names resolve through `std::core` paths" is unmet and is a larger project than the Perhaps/Result alias removal.

### 7.6 Single-file pipeline may bypass the normalizer

`path_normalizer::normalize` is called explicitly in the multi-file pipeline. Whether single-file programs (which may call `evaluator::evaluate` directly rather than `evaluate_graph`) run through the normalizer has not been audited. If they don't, any pre-pass added to `path_normalizer.rs` (such as the `?` desugaring from #214) will not apply to single-file programs, and behaviour will diverge between the two execution modes.

---

## 8. Technical Debt

### 8.1 `evaluator.md` Known Limitations are stale

The Known Limitations section in `metel-interpreter/docs/evaluator.md` still documents two problems that were fixed in sprint 13:

> **Flat module environment:** `std::core` builtins can be shadowed by user names in any module. Tracked as issue #189.

> **Declaration name collisions across modules:** Second declaration silently overwrites first. Evaluator emits warning but does not hard-error.

Both were fixed by #210 (per-module isolated environments). The warning code was removed from the evaluator; `evaluate_graph` no longer flattens modules. Anyone reading this documentation believes the module system is broken in ways it no longer is. The Known Limitations section needs to be rewritten to reflect the current state.

### 8.2 `sprint-12-gap-analysis.md` is untracked

`metel-interpreter/docs/sprint-12-gap-analysis.md` has never been committed to the repository. It contains the full pre-implementation gap analysis from sprint 12, documenting why issues #205, #206, #189 were created and the decisions made during planning. It is useful historical context. It is addressed as Gap 5 in sprint 14's issue #216.

### 8.3 Dual path computation is fragile and untested

Two independent functions implement the same logic — "compute absolute module path from a `PathRoot`":

- `module_loader::child_module_path` (used during file loading)
- `name_resolver::absolute_base` (used during import resolution)

If these two functions diverge, `GlobalExports` lookups fail silently: the typechecker uses a key that doesn't match what the loader assigned, causing T0009 or T0003 errors on valid imports. ADR-0023 documents that this bug existed once and was fixed. There is no test asserting the two functions agree, and no shared implementation. A future change to one function that doesn't update the other will produce cryptic, hard-to-diagnose errors.

**Recommended fix:** Extract a single `resolve_path_root(root: &PathRoot, current: &[String]) -> Vec<String>` function into a shared location (`src/module_paths.rs` or similar) and have both functions call it.

### 8.4 The dual-registration invariant has no enforcement

The critical invariant — `imported_schemes` must be seeded into both inference (`ctx.bind_poly`) and construction (`scheme_env.entry().or_insert()`) — is documented in ADR-0022 but enforced only by code convention. The two calls sit adjacent in `check_impl` with a comment, but there is no type-level guarantee, no assertion, and no test that exercises a missing-one-side case.

If someone adds a new code path that creates an `InferContext` without seeding imported schemes (for example, a future incremental-check optimisation), programs will typecheck incorrectly with no immediate error. The failure mode is type errors in downstream modules or, worse, silently wrong typed AST.

**Recommended fix:** Make `InferContext::new` accept `&SchemeEnv` and seed the imported schemes during construction, making it impossible to create an inference context without them.

### 8.5 Mid-inference partial solve in PropagateError

The `?` operator inference (lines 669–714 of `inference.rs`) calls `ctx.solve()` mid-way through type inference to determine if `E1 != E2` (whether From coercion is needed). This is the only site in the entire codebase where a partial solve occurs before constraint collection is complete. It can interact with constraints added after the `?` expression in non-obvious ways.

This also means the current `?` implementation already ships partial From coercion support (cross-type `?` coercion, tracked in #13), which was never explicitly planned as shipped. The desugaring in #214 must either preserve this behaviour or explicitly document its regression.

### 8.6 T0009 detection requires an O(n) scan on raw AST

In `build_import_schemes`, when an explicit import is not found in `GlobalExports`, the typechecker scans the source module's entire `program.decls` (raw untyped AST) to distinguish T0009 (private name) from T0003 (absent name). This is quadratic for modules with many imports and many declarations. More importantly, it accesses the pre-normalization AST while the typechecker is supposed to operate only on `NormalizedModuleGraph`. This is a back-channel dependency that the newtype guarantee was meant to prevent.

### 8.7 `type_context` accumulator is an approximation

In `check_graph`, struct/enum/aspect/impl declarations from already-checked modules are collected into `type_context: Vec<Decl>` and passed to `build_registry` for subsequent modules. This allows module B to see module A's struct types during type inference.

However, `type_context` is populated from raw `Decl` (untyped AST), not from resolved type information. `build_registry` re-processes these declarations from scratch for each module. If a struct in module A references a type from a third module C, and module B hasn't imported C, `build_registry` may fail to resolve that type when building B's registry. This has not caused problems in current tests (which use flat, non-nested cross-module types) but is a latent correctness issue for complex multi-module programs with deep type dependencies.

### 8.8 StdPrelude and `register_builtins` can silently diverge

In `check_graph`'s `imported_names` population, `std::core` glob entries are skipped because `register_builtins` in the evaluator handles them:

```rust
// std::core names are always registered via builtins, so skipping the
// Std glob here is safe
```

This assumes `StdPrelude::schemes()` (used by the typechecker) and `register_builtins` (used by the evaluator) always contain the same names. There is no assertion enforcing this. If someone adds a builtin to `StdPrelude` and forgets `register_builtins`, or vice versa, the name passes typechecking but fails at runtime with "undefined name" — or is available at runtime but the typechecker rejects it. ADR-0027 established `StdPrelude` as the single source of truth, but the evaluator's `register_builtins` is still separately maintained code.

**Recommended fix:** Generate the evaluator's builtin registration from `StdPrelude` at startup, or add a test that asserts `StdPrelude::schemes().keys()` equals the set of names registered by `register_builtins`.

---

## 9. Summary Table

### Implemented and working

| Feature | File | Notes |
|---|---|---|
| File loading + cycle detection | `module_loader.rs` | DFS with visited set + call stack |
| Hierarchical module paths | `module_loader.rs` | ADR-0023 |
| Three-pass scope building | `name_resolver.rs` | pub_surface + explicit + globs |
| Glob import tier priority | `name_resolver.rs` | ADR-0026; Std < User < Explicit |
| Re-export propagation | `name_resolver.rs` | `export path::Name` |
| Path normalization pre-pass | `path_normalizer.rs` | ADR-0021; newtype guarantee |
| T0010 pub annotation enforcement | `typechecker/mod.rs` | Before inference |
| T0009 / T0003 / T0011 errors | `typechecker/mod.rs` | In `build_import_schemes` |
| GlobalExports accumulator | `typechecker/mod.rs` | ADR-0022 |
| Per-module typechecking | `typechecker/mod.rs` | Topological order |
| `std::core` virtual module | `name_resolver.rs`, `registry.rs` | ADR-0027 |
| Per-module runtime environments | `evaluator/mod.rs` | ADR-0029; sprint 13 |
| Cross-module imported_names seeding | `check_graph` + `evaluate_graph` | Sprint 13 (#210) |
| Mutual recursion within a module | `evaluator/mod.rs` | Pass 1a/1b tie-the-knot |

### Missing

| Feature | Issue | Blocker |
|---|---|---|
| Cross-module mutual recursion | #189 | Needs global 1a-before-1b pass ordering |
| `?` desugaring to match pre-pass | #214 | — |
| `Type::Perhaps`/`Type::Result` as `Named` | #150 | Needs #214 first |
| Field-level visibility | #158, RFC-0032 | Deferred post-v0.6.3; breaking change |
| Field-level mutability | RFC-0033 | Deferred with RFC-0032; non-breaking |
| `std::core` paths for primitives | #150 (partial) | Requires type system changes |
| Single-file pipeline audit for pre-pass | #216 Gap 3 | Before #214 implementation |

### Technical debt

| Debt | Severity | Location |
|---|---|---|
| `evaluator.md` Known Limitations stale | High — misleads readers | `docs/evaluator.md` |
| Dual path computation (`child_module_path` vs `absolute_base`) | High — silent divergence risk | `module_loader.rs` + `name_resolver.rs` |
| Dual-registration invariant unenforced | High — silent wrong-typecheck risk | `check_impl` in `typechecker/mod.rs` |
| StdPrelude / `register_builtins` can diverge | Medium — runtime "undefined name" | `registry.rs` + `evaluator/builtins.rs` |
| Mid-inference partial solve in `?` | Medium — obscure interaction risk | `inference.rs:669–714` |
| T0009 detection scans raw AST | Low-medium — quadratic + back-channel | `build_import_schemes` |
| `type_context` accumulator is approximate | Low — latent for complex programs | `check_graph` |
| `sprint-12-gap-analysis.md` untracked | Low — lost documentation | `docs/` |
