---
id: adr-0019
title: "Flat Module Merge for Typechecker in v0.5.0"
date: '2026-05-28'
status: active
---

## Context

The v0.5.0 module system (RFC-0030) introduces multi-file programs. The module loader builds a `ModuleGraph` (one `LoadedModule` per `.mln` file), each holding its own parsed `Program`. The typechecker (`typechecker::check`) takes a single `ast::Program`.

Two options for bridging them:
1. **Flat merge**: concatenate all `decls`, `imports`, and `exports` from every module into one `Program` before passing to the typechecker.
2. **Per-module check**: thread per-module scope into the typechecker so declarations from different modules are checked in their own namespace.

## Decision

Option 1 (flat merge) was chosen for v0.5.0, implemented in `module_loader::load_program`.

## Rationale

The per-module check requires significant changes to the typechecker's `InferContext`/`ConstructCtx` to carry a "current module" scope and to enforce cross-module visibility during inference. That work is a sprint of its own.

Flat merge allows the module loading, parsing, and name resolution infrastructure to land in v0.5.0 without rewriting the typechecker. The name resolver (`name_resolver::resolve`) is built and tested in isolation; integrating it into the typechecker pipeline is the v0.6.0 milestone.

## Consequences

- All declarations from all loaded modules are globally visible to the typechecker. Cross-module visibility is not enforced by the typechecker (a `pub` function and a private function are both callable from any module).
- The name resolver's per-module scopes and `pub_surface` map are not consulted during type checking.
- Qualified path resolution (`X::Y`) uses a last-segment fallback (`Y`) rather than a true per-module lookup.
- Tracking issue for full integration: the next sprint should wire `name_resolver::resolve` into the typechecker and remove the last-segment fallback.
