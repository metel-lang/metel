# ADR-0021: Path Normalization as a Dedicated Pre-Pass

**Status:** Accepted  
**Date:** 2026-05-28  
**Tracking issue:** #185

---

## Context

Multi-segment path expressions like `helper::answer()` or `root::parser::Token` appear in user code. Before the typechecker can resolve them, they must be rewritten to bare local names (`answer`, `Token`) using the scope information from `ResolvedNames`.

Two alternative insertion points were considered: the **parser** (earliest, at CST→AST time) and the **typechecker inference pass** (at the point of use).

---

## Decision

Normalization runs as a **dedicated pre-pass** (`path_normalizer::normalize`) between name resolution and typechecking. It:

1. Receives the `ModuleGraph` (mutable) and the immutable `ResolvedNames`.
2. Walks every module's AST in-place, rewriting `Expr::Path` nodes with module qualifiers to `Expr::ResolvedPath { resolved, original, span }`.
3. Returns `NormalizedModuleGraph` — a newtype wrapper that proves normalization ran.

---

## Alternatives Rejected

**In the parser:** The parser has no access to `ResolvedNames` (which requires a fully-loaded `ModuleGraph`). Wiring it in would couple module loading to parsing and prevent incremental loading. Rejected.

**In the typechecker inference pass:** Possible but undesirable — the inference pass would need to carry scope state alongside type state, and the construction pass would need the same scope state independently. Two injection sites create two opportunities for divergence. Rejected.

**In name resolution:** The resolver only computes scope metadata; mutating the AST there would violate the resolver's read-only contract and complicate testing. Rejected.

---

## Invariants

- `NormalizedModuleGraph` is the only input type accepted by `check_graph`. Passing a raw `ModuleGraph` is a compile-time error.
- After normalization, no module-qualified `Expr::Path` (where the first segment is a known module name) should remain. `Expr::ResolvedPath` carries the original segments for error messages.
- Single-segment `Expr::Ident` nodes and type-member paths (`Color::Red` where `Color` is a type, not a module) pass through unchanged.

---

## Consequences

- The typechecker only needs to handle `Expr::Ident` and `Expr::ResolvedPath` for name lookups; it never sees module-qualified multi-segment paths.
- Error messages for resolved paths display the original qualified form (e.g. `undefined name 'helper::answer'`) for user clarity.
- ADR-0020 (last-segment fallback hack) becomes removable once the CLI is migrated to the new pipeline (#179).
