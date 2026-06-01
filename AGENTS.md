# Metel - Agent Guide

## Project

Metel is a statically typed, expression-oriented language. This repository contains the tree-walk interpreter and consumes the shared documentation repository as the `docs/` submodule.

The interpreter is the shipped runtime. Treat it as the current product surface, not as throwaway compiler scaffolding. The language specification is the contract the interpreter must satisfy.

The repository remote is Codeberg (`codeberg.org/metel-lang/metel`). Task tracking is in Plane, not GitHub Projects.

---

## Current Documentation Structure

`docs/` is the shared `metel-docs` submodule. Update it as a real submodule: make docs edits in the submodule, commit them there, then update the pointer in this repo.

| Location | Purpose |
|---|---|
| `docs/README.md` | Authoritative public/internal docs layout |
| `docs/public/getting-started/` | Intro, quickstart, and tutorials |
| `docs/public/reference/spec.md` | Language specification entry point |
| `docs/public/reference/spec/` | Spec sections: lexical, types, declarations, functions, expressions, modules, runtime, grammar |
| `docs/public/reference/error-codes.md` | Error code reference |
| `docs/public/release-notes/changelog.md` | Version changelog and release notes |
| `docs/internal/versioning.md` | Version numbering, RFC lifecycle, and doc conventions |
| `docs/internal/rfcs/active/` | RFCs under design or review |
| `docs/internal/rfcs/accepted/` | Accepted RFCs not yet shipped |
| `docs/internal/rfcs/implemented/` | Implemented/incorporated RFCs |
| `docs/reports/` | Design reports and longer-form research notes |
| `metel-interpreter/docs/architecture.md` | Interpreter pipeline and component boundaries |
| `metel-interpreter/docs/typechecker.md` | Typechecker theory and implementation notes |
| `metel-interpreter/docs/evaluator.md` | Runtime values, signals, environment, and evaluator notes |
| `metel-interpreter/docs/decisions/` | Architectural decision records |

Public docs no longer live at `docs/public/spec.md`, `docs/public/spec/`, or `docs/public/changelog.md`. Those paths are stale.

---

## Task Tracking: Plane

Plane is the source of truth for tasks, RFC tracking, sprint cycles, and version milestones.

Current Plane identifiers:

| Field | Value |
|---|---|
| Project identifier | `METEL` |
| Project ID | `ec7904a4-cd24-40bd-8089-19a5eb8875ab` |
| States | `Backlog`, `Todo`, `In Progress`, `Done`, `Cancelled` |
| Work item types | `Task`, `RFC`, `Epic` |
| Product modules | `Interpreter`, `Wiki`, `Compiler`, `Playground`, `LSP` |

Use Plane work item identifiers in user-facing references and commit messages, for example `METEL-57`. When a tool requires the UUID, use the project ID above.

Common Plane actions:

- Read a task: retrieve work item by project identifier `METEL` and sequence number.
- Search tasks: list work items with a query, label, milestone, state, cycle, or module filter.
- Start work: set the work item state to `In Progress`.
- Finish work: set the work item state to `Done` only after acceptance criteria and tests pass.
- Track dependencies: use work item relations (`blocked_by`, `blocking`, `relates_to`) rather than encoding dependency state in files.
- Version planning: use Plane milestones such as `v0.6.4`, `0.7.0`, `v0.8.0`.
- Sprint planning: use Plane cycles such as `Sprint 17 - Aspect Bounds`.

Do not rely on `.github/` automation or GitHub issue labels. This checkout no longer has `.github/` workflow or issue-template files.

---

## Sprint Workflow

Sprints are Plane cycles and repository branches. Sprint branches still use the `sprint/<N>` convention.

### Starting a Sprint

1. Create or confirm the Plane cycle (`Sprint N - Theme`).
2. Add planned Plane work items to the cycle with state `Todo`.
3. Create the branch from current `main`:

```bash
git checkout main
git pull --recurse-submodules
git checkout -b sprint/N
git push -u origin sprint/N
```

4. Keep all sprint code, docs pointer updates, and release-prep commits on `sprint/N`.

### During a Sprint

- Read the Plane work item before editing code.
- Move only actively worked items to `In Progress`.
- Keep commits on the sprint branch.
- Push after each logical unit of completed work.
- If public docs changed, commit in `docs/` first, then commit the updated submodule pointer in this repo.

### Closing a Sprint

Before opening a pull request from `sprint/N` to `main`, run the quality gate below. If any gate fails, fix it on the sprint branch and run the gate again.

1. **Tests** - `cargo test` from `metel-interpreter/` must pass with zero failures.
2. **Code quality** - review every file in `git diff main..HEAD --name-only` for stale code, dead branches, accidental `todo!()`, `unimplemented!()`, `unreachable!()`, and fallible `unwrap()`/`expect()` paths.
3. **Coverage** - every feature or fix needs a focused regression test:
   - Parser or grammar changes: parsing tests or typechecking tests.
   - Type system changes: typechecking tests in `tests/typechecking/sources/`.
   - Evaluator/runtime changes: evaluator tests in `tests/evaluator/sources/` or module semantics tests.
   - Module graph/name-resolution changes: `tests/module_loading/` or `tests/module_semantics/`.
4. **Spec accuracy** - every language-visible change is documented in `docs/public/reference/spec.md` and the linked spec section.
5. **Changelog** - version-visible work is recorded in `docs/public/release-notes/changelog.md`.
6. **RFC state** - accepted RFCs implemented by the sprint are moved or marked according to `docs/internal/versioning.md`; incorporated RFCs must not remain in the accepted queue.
7. **Internal docs** - update `metel-interpreter/docs/architecture.md`, `typechecker.md`, or `evaluator.md` when the corresponding pipeline, inference, construction, runtime, or builtin behavior changes.
8. **Decision records** - add a new ADR in `metel-interpreter/docs/decisions/` for non-obvious architectural decisions, reversals, or workarounds future contributors must know.
9. **Plane** - completed work items have satisfied acceptance criteria and are set to `Done`; deferred work is explicit in Plane, not hidden in comments.

After the gate passes, open a pull request from `sprint/N` to `main` on Codeberg. The pull request diff is the authoritative sprint deliverable.

---

## Task Workflow

### Before Starting a Task

1. Retrieve and read the full Plane work item, including acceptance criteria, dependencies, labels, milestone, cycle, and module.
2. Read every spec section the task touches. The spec entry point is `docs/public/reference/spec.md`.
3. Read relevant RFCs in `docs/internal/rfcs/` and ADRs in `metel-interpreter/docs/decisions/`.
4. Check dependency work items and confirm their implementation matches the contract this task depends on.
5. If the spec is missing or ambiguous, update the spec first. If the choice is non-obvious, write an ADR before implementation.
6. Move the Plane work item to `In Progress`.

### During Implementation

- Follow the spec exactly. If behavior is not in the spec, it does not exist yet.
- Do not implement undocumented behavior and plan to fix docs later.
- Keep scope tight. If required work falls outside the task, create or update a Plane work item and only proceed if it is a real blocker.
- Preserve user changes in the worktree. Never revert unrelated dirty files.
- Keep docs submodule changes and root-repo pointer changes distinct.

### Before Marking Done

1. All acceptance criteria are satisfied.
2. Relevant tests pass; for typechecker or inference changes, the full `cargo test` suite passes.
3. Spec, changelog, RFC, internal docs, and ADR updates are complete where required.
4. The work item is moved to `Done` in Plane.

---

## RFC Workflow

RFCs live in `docs/internal/rfcs/` and are tracked in Plane with work item type `RFC`.

Follow `docs/internal/versioning.md` for the lifecycle and frontmatter requirements. In practice, the folders are:

- `active/` - design work not yet accepted or not yet scheduled.
- `accepted/` - design accepted and assigned to a target version milestone.
- `implemented/` - implemented and shipped/incorporated.

Rules:

- The RFC document is the source of truth for design details.
- The Plane RFC item should summarize the topic and link to the RFC file; do not duplicate the whole RFC body in Plane.
- Accepted RFCs must have the relevant spec or internal architecture docs updated before implementation work begins.
- Implementation tasks should relate back to the RFC work item.
- When the target version ships, incorporated RFCs must be moved or marked according to `docs/internal/versioning.md`.

If an existing RFC's folder, frontmatter status, or `spec_status` contradicts `docs/internal/versioning.md`, stop and resolve the documentation workflow inconsistency before implementing against it.

---

## Commit Convention

Every commit related to a Plane task should reference the work item identifier:

```text
<type>(METEL-<number>): <description>
```

Types: `feat`, `fix`, `refactor`, `test`, `docs`.

Examples:

```text
feat(METEL-57): enforce function aspect bounds
docs(METEL-58): update aspect bound spec text
test(METEL-60): cover generic bound regressions
```

Commits not tied to a tracked item may omit the reference, for example `docs: point CLAUDE.md to AGENTS.md`.

When a commit is intended to close work after merge, include a body describing what changed and reference the work item:

```text
feat(METEL-57): enforce function aspect bounds

- Check call-site type arguments against declared bounds
- Seed bound methods during function body inference
- Add stage12 typechecking regressions

Completes METEL-57
```

During an active sprint, commit only on `sprint/N`, not directly on `main`.

---

## Spec Discipline

- The spec is the source of truth for language-visible behavior.
- The spec contains rules and syntax, not rationale, history, or open questions. Put rationale in RFCs or ADRs.
- New public behavior must be documented in `docs/public/reference/spec/`.
- Runtime builtins documented in `docs/public/reference/spec/runtime.md` must match what the interpreter registers.
- Version-visible changes must be reflected in `docs/public/release-notes/changelog.md`.
- Patch releases must not introduce spec changes; see `docs/internal/versioning.md`.

---

## Interpreter Architecture Invariants

The current interpreter pipeline is:

```text
.mln root file
  -> Module Loader
  -> Name Resolver
  -> Path Normalizer
  -> Type Checker
  -> Evaluator
```

Do not skip stages.

Important module-system invariants:

- `module_loader::load_root` produces a `ModuleGraph` in topological order.
- `name_resolver::resolve` owns import scopes, visibility, public surfaces, and re-exports.
- `path_normalizer::normalize` rewrites qualified paths before typechecking.
- `typechecker::check_graph` consumes the normalized graph plus resolved names and returns `TypedModuleGraph`.
- `evaluator::evaluate_graph` consumes `TypedModuleGraph`.
- Cross-module public APIs must be fully annotated; do not introduce cross-module type inference casually.

If a change alters these boundaries, update `metel-interpreter/docs/architecture.md` and consider an ADR.

---

## Type System Stability

The sensitive areas are `metel-interpreter/src/typeinference/` and `metel-interpreter/src/typechecker/`. Bugs here can produce silent wrong typing, not just crashes.

### Two-Pass Typechecker Boundary

The typechecker remains split into inference and construction:

- Pass 1 (`src/typechecker/inference.rs`): walk the AST, emit constraints, solve into substitutions, update inference context.
- Pass 2 (`src/typechecker/construction.rs`): read solved substitutions and build typed AST nodes.

Do not infer types in Pass 2. Do not build typed AST nodes in Pass 1. If a task seems to require that, stop and ask.

### Key Invariants

- `Substitution::compose` is ordered. Verify composition direction every time it is used.
- `Never` is a bottom type. Typechecking tests alone may not distinguish a diverging expression from a correctly typed runtime path; use evaluator tests for runtime behavior.
- Route conversions through `type_to_infer` where `Perhaps`/`Result` normalization matters.
- Distinguish formal `TypeVar`s from fresh `InferType::Var(TypeVar)` usage-site variables.
- Generic instantiation should follow the established `instantiate_scheme_for_call` pattern: fresh variables, initial substitution, unification against actuals, then extraction from the composed substitution.
- Imported schemes must seed both inference and construction paths for a module. If only one pass sees imports, the typechecker is wrong.
- Public module declarations that are consumed cross-module must have enough annotations to export concrete schemes.

### Before Finalizing Type System Changes

1. Run `cargo test` from `metel-interpreter/`.
2. Run or manually apply the `/review-typechecker` checklist.
3. For every new `unify` call, verify expected-vs-actual argument order and substitution composition direction.
4. For every `infer_type_to_type` call, verify all type variables are resolved and a useful span is available.
5. If `construct_block` expected-type threading changes, check every call site.
6. Add regression tests that would fail without the fix.

Stop and ask if:

- You need to touch inference and construction in a way that blurs their boundary.
- No existing pattern covers the new type-system behavior.
- A substitution-order change breaks an existing test.
- The task depends on a spec interpretation that is unclear.

---

## Decision Records

Create an ADR in `metel-interpreter/docs/decisions/` when:

- Multiple reasonable implementation options exist and the chosen tradeoff matters.
- The decision changes or reverses a previous ADR or RFC.
- A workaround or limitation would surprise a future contributor.
- A spec or architecture doc changes because implementation revealed a real constraint.

Do not create ADRs for routine implementation details that follow directly from the spec.

Accepted ADRs are not edited to reverse them. Add a new ADR that supersedes the old one.

When code intentionally encodes an ADR-backed invariant that may look wrong, add a concise comment with the reason and ADR number.

---

## Wiki and Public Docs Release Workflow

The public website consumes the same `metel-docs` content through the docs submodule.

When a task or release affects public documentation:

1. Update and commit `docs/` first.
2. Update this repo's `docs` submodule pointer on the sprint branch.
3. Update `metel-website` to point at the same docs commit.
4. For public releases, generate the versioned website snapshot if the release process requires it.
5. Publish only after the docs version and website pointer match.

Do not assume automatic publication unless the release workflow explicitly says it exists.

---

## When to Stop and Ask

Stop before proceeding when:

- A design decision has multiple plausible options with architectural consequences.
- The spec is ambiguous in a way that affects implementation.
- The task description contradicts current code, docs, or Plane state.
- A dependency is incomplete or wrong.
- Completing the task requires a scope expansion that could affect other work.
- You are about to make an irreversible or hard-to-reverse change.

When stopping, explain what you found, the options, and the recommended path.

---

## What Not to Do

- Do not implement behavior that is not in the spec.
- Do not let implementation and docs diverge.
- Do not add rationale or history to the spec.
- Do not use GitHub Projects, GitHub issue labels, or `.github/` workflows as the current process.
- Do not create new tracking documents for open work; use Plane.
- Do not mark a Plane work item `Done` with unchecked acceptance criteria.
- Do not commit sprint work directly to `main`.
