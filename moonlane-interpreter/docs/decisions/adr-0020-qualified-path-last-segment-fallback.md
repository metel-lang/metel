---
id: adr-0020
title: "Qualified Path Resolution via Last-Segment Fallback"
date: '2026-05-28'
status: active
---

## Context

RFC-0030 allows fully-qualified paths in expression and type position, e.g. `helper::answer()` or `-> helper::Token`. The typechecker (both inference and construction passes) and the evaluator need to resolve these paths to concrete types or runtime values.

True per-module resolution would require the typechecker to know which declarations live in which module. In v0.5.0 all modules are merged into a flat namespace (see ADR-0019).

## Decision

When a path `[seg_0, seg_1, ..., seg_n]` cannot be resolved as a type/method chain (e.g. `TypeName::VariantOrMethod`), both the typechecker and evaluator fall back to looking up only the last segment (`seg_n`) as a plain name in the current environment.

Specifically:
- `Expr::Path(segments)` in the inference pass: after type::member checks fail, `ctx.lookup(last_segment)`.
- `Expr::Path(segments)` in the construction pass: after method_env miss, `ctx.lookup(last_segment)` then `scheme_env.get(last_segment)`.
- `TypeExpr::Named(name, _)` in conversions: `bare_type_name(name)` strips the `::` prefix, leaving just the final component.
- `TypedExpr::Path(segments)` in the evaluator: `env.get(last_segment)` fallback after the full joined key misses.

## Rationale

The flat merge means all module declarations are registered under their bare names in the typechecker's environment. The module-qualified spelling (`helper::answer`) is not registered — only `answer` is. The last-segment fallback bridges the two representations without requiring a separate symbol table.

## Consequences

- A path `a::b` and a bare name `b` are interchangeable in the typechecker and evaluator. No shadowing protection exists between modules.
- An ambiguous case (two modules both export `answer`) will resolve to whichever `answer` the flat merge registered last — silently.
- This fallback must be removed when the name resolver is wired into the typechecker (next sprint). At that point, qualified paths must resolve via the module scope, not via last-segment lookup.
