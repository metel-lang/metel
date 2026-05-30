# Sprint 12 Pre-Implementation Gap Analysis

**Reconstructed from git history and issue context — original file was never committed (see §8.2 of the module system implementation report).**

**Date:** ~2025 (exact date unknown)
**Sprint:** 12 (targets v0.6.1)
**Purpose:** Gaps discovered during Sprint 12 planning that shaped the implementation plan and led to new issues being created.

---

## Context

Sprint 12 planned to land the std::core virtual module system — specifically `std::core` auto-import (#201), declaring core types and aspects in `std::core` (#202), and unifying the type definition registry (#133). During planning, several design gaps were uncovered that required new issues or changed the execution order.

---

## Gap 1 — `std::core` name registration needed a unified registry

The typechecker and evaluator each registered builtin names independently with no single source of truth. Adding `std::core` types (`Perhaps`, `Result`) as first-class declared types (#202) required both the typechecker and evaluator to agree on what names exist and how their impls are keyed.

**Resolution:** Issue #133 (TypeDefinitionRegistry) was already on the backlog. It became a Sprint 12 prerequisite rather than a follow-on. `StdPrelude` was introduced as the shared registration point — all typechecker and evaluator builtin registration routes through it.

---

## Gap 2 — Built-in aspect dispatch remained special-cased after #133

After #133 unified the registry for user-defined types and aspects, the built-in dispatch for `Display`, `Iterable`, and `From` remained split across `construction.rs` and `evaluator/call.rs` with hardcoded strings. With `Perhaps` and `Result` moving to general enum representation (#202), the special-casing surface grew.

**Resolution:** The full built-in dispatch unification was too large to bundle with #133 in Sprint 12. It was split into a separate issue (#209) and deferred post-Sprint 12. The `Value::Perhaps`/`Value::Result` migration (evaluator side) was similarly split into #205 and deferred.

---

## Gap 3 — Per-module runtime environments were not ready (#189)

`evaluate_graph` at the start of Sprint 12 flattened all modules into a single environment. A user-defined name in any module could silently overwrite a builtin. This became more visible as `std::core` was about to inject names into every module's scope.

**Resolution:** The full fix (#189 — per-module runtime contexts) required `TypedModule` to carry `imported_names` (the cross-module binding map), which was not yet implemented. #189 was broken into two steps: #210 (extend `TypedModule` with import bindings; isolated module initialization) as a Sprint 13 prerequisite, with #189 (full per-module context) following after. Sprint 12 proceeded without the runtime fix; the risk was accepted as a known limitation.

---

## Gap 4 — `std::core` auto-import conflicted with user glob imports

The plan for `std::core` auto-import (#201) was to inject `std::core::*` into every module's glob scope. T0011 (glob/glob name conflict) fired whenever two globs exported the same name — this meant any module that also used `import other_module::*` where `other_module` exported a name that also existed in `std::core` (e.g. `print`) would get a spurious error.

A special-case `if one_glob_is_std_core { suppress }` was considered and rejected as an ad-hoc workaround that would recur for every future layered import source.

**Resolution:** Issue #206 (glob import tiers: `Std` / `User`) was created and scheduled into Sprint 12 as a prerequisite for #201. T0011 was redefined to fire only within same-tier globs; cross-tier conflicts resolve deterministically (Explicit > User > Std). #201 was blocked until #206 landed.

---

## Gap 5 — `Value::Perhaps` and `Value::Result` migration was larger than expected

#202 declared `Perhaps` and `Result` as first-class types in `std::core`. For consistency, the evaluator's dedicated `Value::Perhaps` and `Value::Result` variants needed to be replaced with the general `Value::Enum` representation. The migration touched `mod.rs`, `call.rs`, `pattern.rs`, `display.rs`, and all test files that matched against these variants.

**Resolution:** The migration (#205) was deferred out of Sprint 12 to avoid blocking the core std::core work. Sprint 12 shipped `Perhaps` and `Result` as declared types (#202) while the evaluator still used the dedicated variants. #205 (evaluator-side migration) and its sequel #209 (dispatch unification) landed in Sprint 13.

---

## Dependency order (as executed)

```
Sprint 12:
  #133  TypeDefinitionRegistry    → prerequisite for #202
  #206  Glob import tiers         → prerequisite for #201
  #201  std::core auto-import     → after #206
  #202  Declare core types        → after #133 and #201
  #203  Sprint 12 close

Sprint 13 (deferred from Sprint 12):
  #210  TypedModule import bindings + isolated module init   → prerequisite for #189
  #205  Value::Perhaps/Result migration                      → after #205 Part 1 done
  #209  Built-in dispatch unification                        → after #205
  #189  Per-module runtime contexts                          → after #210 (partially; full fix ongoing)
  #211  Sprint 13 close
```
