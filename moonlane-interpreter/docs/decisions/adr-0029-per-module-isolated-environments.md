---
id: adr-0029
title: "Per-Module Isolated Environments in evaluate_graph"
date: '2026-05-29'
status: active
supersedes: adr-0019
---

## Context

ADR-0019 chose a flat-merge strategy for v0.5.0: all module declarations were concatenated into a single `TypedProgram` before evaluation, giving every declaration global visibility at runtime. This was explicitly deferred — per-module environments were tracked as issue #189/#210.

The flat merge had a concrete correctness problem: if two modules declared a top-level function with the same name (e.g. both export `new`), the later module's definition silently shadowed the earlier one. The evaluator emitted a best-effort warning but could not enforce isolation. With the `std::core` virtual module introduced in v0.6.1, the flat merge also prevented modules from having locally scoped overrides of standard names.

The `check_graph` typechecker pass already populates per-module scope information in `ResolvedNames`. With the addition of `imported_names` to `TypedModule` (a `HashMap<local_name, (source_module, canonical_name)>` populated in `check_graph`), the evaluator has a precise, typechecker-validated table of which names each module needs from which dependency.

## Decision

`evaluate_graph` allocates a fresh `Environment` per module, seeds it with builtins, then consults the module's `imported_names` table to copy values from already-evaluated dependency environments (modules are processed in topological order, so dependencies are always ready). The module's own declarations are then evaluated into this isolated environment via `run_passes`.

`main()` is called only from the root module's environment (the last entry in topological order).

The flat-merge path (`evaluate_with_aliases` / the concatenation loop in the old `evaluate_graph`) is removed entirely.

`check_graph` is extended to populate `TypedModule::imported_names`: for each module scope in `ResolvedNames`, it records explicit `Item` bindings and glob-imported names (Std tier then User tier, mirroring `build_import_schemes` priority), skipping `std::core` glob entries because those are already seeded by `register_builtins`.

## Consequences

- Top-level name collisions across modules are no longer silently resolved by shadowing. Each module's environment contains only its own declarations plus explicitly imported names.
- `register_builtins` is called once per module, not once globally. This is correct: builtins must be present in every module environment, and they are cheap to register (function pointers, no heap allocation beyond the env HashMap entries).
- The `module_envs: HashMap<Vec<String>, Environment>` accumulator holds one environment per module for the duration of `evaluate_graph`. For programs with many modules, this is proportional to program size and is not a leak — all environments are dropped at function return.
- Import aliases (`import X as Y`) remain handled separately via `import_aliases` on `TypedModule`, which `run_passes` processes as before.
- `std::core` glob imports are excluded from `imported_names` because their values come from `register_builtins`, not from an evaluated module environment. Attempting to look them up from `module_envs` would fail (there is no `["std", "core"]` entry).

## Constraints for future contributors

- The `imported_names` table is the single source of truth for which cross-module names an environment should contain. Do not add ad-hoc name-seeding logic in `evaluate_graph` — extend `check_graph`'s population pass instead.
- `register_builtins` must be called before seeding `imported_names`, so that built-in values are present when a dependency module's environment is consulted.
- The root module is identified as `graph.modules.last()` — this relies on the topological order guarantee from the module loader. If that order changes, `evaluate_graph` must be updated accordingly.
