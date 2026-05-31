---
id: adr-0033
title: "String Interpolation Lowered in the Parser"
date: '2026-05-31'
status: active
---

## Context

String interpolation (`"hello, ${name}!"`) needs to be lowered to a sequence of string-concatenation operations. Two plausible lowering sites exist:

1. **Parser** — produces `"hello, " + name.to_string() + "!"` as an untyped `Expr::BinOp(Plus, …)` tree before any type information is available.
2. **path_normalizer pre-pass** — lowering happens after parsing, alongside `?` desugaring, but still before type inference.

The `?` operator was deliberately **not** lowered in the parser (see ADR-0030) because it requires the enclosing function's declared return type, which is not available during parsing.

## Decision

String interpolation is lowered in the parser (`parser/mod.rs`, `parse_string_interpolation`). Each `${expr}` hole is recursively parsed as a full expression and then wrapped in a `.to_string()` method-call node. The segments are concatenated left-to-right via `BinOp(Plus, …)` nodes. All synthetic nodes carry the span of the interpolated string literal.

This produces a plain `Expr` tree — no new AST variant is needed. After the parser returns, every downstream pass (name resolution, normalization, inference, construction, evaluator) handles interpolation transparently as ordinary concatenation.

## Alternatives considered

1. **Lower in path_normalizer** — rejected. Interpolation lowering requires no type context, so moving it to the pre-pass would add complexity with no benefit. Parser-level lowering keeps the pre-pass focused on type-context-dependent rewrites (`?` desugaring).

2. **Dedicated `Expr::Interpolation` AST node** — rejected. Would require every pass (inference, construction, evaluator) to handle a new variant. The AST stays smaller and more stable when interpolation is erased before inference.

3. **Lower during type inference** — rejected. Inference is the wrong place for syntax sugar; it processes typed constraints, not surface syntax.

## Consequences

- No `Expr::Interpolation` variant exists or should ever be added.
- Interpolation errors (non-Display hole types) are caught by the typechecker's existing check for `to_string()` availability — no special-case error handling needed.
- Nested string literals inside `${…}` require careful scanner bookkeeping in the parser to track quote nesting; this is handled by `scan_interpolation_body` (see METEL-82).
- Future changes to interpolation semantics must be made in `parse_string_interpolation` only — inference, construction, and the evaluator are unaffected.
