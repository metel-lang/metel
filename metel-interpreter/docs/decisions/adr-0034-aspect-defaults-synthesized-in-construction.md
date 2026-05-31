---
id: adr-0034
title: "Aspect Default Methods Synthesized During Construction"
date: '2026-05-31'
status: active
---

## Context

An aspect may provide a default body for some of its methods. An `impl` block may omit methods that have defaults, inheriting the aspect's implementation instead. The interpreter must make the inherited method available for dispatch.

Three plausible synthesis sites exist:

1. **Registry** — mark each method as required or optional when the aspect is registered; use this at impl-check time.
2. **Construction pass** — when a complete `impl` block is being typed, detect missing defaulted methods and synthesize fully-typed `TypedMethodDecl` nodes from the aspect's default bodies.
3. **Evaluator** — at method-call time, fall back to the aspect's default if no concrete impl is found.

## Decision

Default methods are synthesized during the **construction pass** (`typechecker/construction.rs`, `construct_impl_block`). When processing an `impl Aspect for Target` block, the construction pass:

1. Iterates all methods declared in the aspect's default-method registry (`TypeDefinitionRegistry::aspect_defaults`).
2. For each method absent from the `impl` block, retrieves the default body's untyped AST from the registry and calls `construct_fun_decl` with a `Self`-substituted signature.
3. Appends the synthesized `TypedMethodDecl` to the typed impl block as if the user had written it.

The evaluator never needs to know about defaults — it always dispatches to a concrete, fully-typed method body.

## Alternatives considered

1. **Evaluate defaults at call time** — rejected. This would require the evaluator to hold a reference to the untyped AST and re-run construction on every default-method call. The evaluator is already the most performance-critical path; adding late-bound construction there conflicts with the principle that evaluation operates on a fully-typed program (see ADR-0004).

2. **Synthesize in the inference pass** — rejected. Inference works with type variables and partial information; constructing a fully-typed method body there would require a premature full solve and duplicate the construction logic.

3. **Require all impl blocks to be complete (no defaults)** — the original design. Rejected in favour of usability; aspects without defaults force boilerplate at every impl site.

## Consequences

- `TypeDefinitionRegistry` holds a new `aspect_defaults: HashMap<String, Vec<(String, FunDecl)>>` field mapping aspect name → `[(method_name, default_body)]`.
- The inference pass must also register the default method's signature into the method env for the impl target so that call-site inference sees the method (see `typechecker/inference.rs`, `infer_impl_block_defaults`).
- Both the inference registration and the construction synthesis must run for each impl block that uses defaults; an impl method present in the user's source takes precedence and must suppress synthesis for that method.
- Required methods (those with no default body) are still checked and produce a missing-method error if absent from the impl block.
- Future contributors: if you add a new pass between construction and evaluation that processes method declarations, ensure it is aware that synthesized default methods appear in the typed AST alongside user-written ones — they are indistinguishable after synthesis.
