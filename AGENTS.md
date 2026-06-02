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
| `docs/internal/rfcs/0-draft/` | Draft RFCs being written |
| `docs/internal/rfcs/1-under-review/` | RFCs ready for evaluation |
| `docs/internal/rfcs/2-accepted/` | Accepted RFCs assigned to a target version |
| `docs/internal/rfcs/3-implemented/` | RFCs implemented and shipped |
| `docs/internal/rfcs/4-superseded/` | RFCs replaced by later RFCs |
| `docs/internal/rfcs/5-refused/` | RFCs refused with a recorded decision |
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
| Task states | `Backlog`, `Todo`, `In Progress`, `Done`, `Cancelled` |
| RFC Status values | `draft`, `under-review`, `accepted`, `implemented`, `superseded`, `refused` |
| Work item types | `Task`, `RFC`, `Epic` |
| RFC work item type ID | `6b00ca94-017d-45e2-81eb-f7b6bed6ac89` |
| RFC Status property ID | `4d858d79-066b-4948-b1bd-7f166b7cd024` |
| Product modules | `Interpreter`, `Wiki`, `Compiler`, `Playground`, `LSP` |

Use Plane work item identifiers in user-facing references and commit messages, for example `METEL-57`. When a tool requires the UUID, use the project ID above.

Common Plane actions:

- Read a task: retrieve work item by project identifier `METEL` and sequence number.
- Search tasks: list work items with a query, label, milestone, state, cycle, or module filter.
- Start task work: set the task work item state to `In Progress`.
- Finish task work: set the task work item state to `Done` only after acceptance criteria and tests pass.
- Move RFC work: use the custom `RFC` work item type and set its `RFC Status` custom property to the state represented by the RFC file directory.
- Track dependencies: use work item relations (`blocked_by`, `blocking`, `relates_to`) rather than encoding dependency state in files.
- Version planning: use Plane milestones such as `v0.6.4`, `0.7.0`, `v0.8.0`.
- Sprint planning: use Plane cycles such as `Sprint 17 - Aspect Bounds`.

Do not rely on `.github/` automation or GitHub issue labels. This checkout no longer has `.github/` workflow or issue-template files.

### Plane RFC API Steps

Use the Plane MCP tools for ordinary work item reads, creation, updates, links, and relations. Use the Plane REST API directly for RFC custom property values.

Reason: the MCP work item tools expose standard work item fields but do not expose a custom-property-value upsert tool. The MCP property listing path can also fail response validation on Plane properties with nullable fields. Direct API calls use Plane's official custom property endpoints and are the reliable way to keep `RFC Status` and `RFC Number` synchronized with the RFC file.

Use `https://api.plane.so/api/v1/workspaces/vladastos` with header `x-api-key: $PLANE_API_KEY`. Never write the API key into a tracked file.

When creating a new RFC:

1. Create the RFC document in the directory matching its lifecycle state, for example `docs/internal/rfcs/0-draft/rfc-0042-let-mut-bindings.md`.
2. Create a Plane work item with type `RFC` using `type_id: 6b00ca94-017d-45e2-81eb-f7b6bed6ac89`. Put the work item on the normal task board state that best matches the planning workflow, usually `Backlog` for a draft RFC.
3. Do not set the Plane `Doc Path` property. RFC paths include the lifecycle directory and therefore change whenever an RFC changes state. Derive the current path by scanning `docs/internal/rfcs/*/` for the RFC frontmatter `id`.
4. Do not add direct Codeberg file links for stateful RFC paths unless a stable redirect or generated RFC index URL exists. Direct file links include the lifecycle directory and become stale on every state change.
5. Query RFC work item properties if IDs need confirmation:

```bash
curl -sS \
  -H "x-api-key: $PLANE_API_KEY" \
  "https://api.plane.so/api/v1/workspaces/vladastos/projects/ec7904a4-cd24-40bd-8089-19a5eb8875ab/work-item-types/6b00ca94-017d-45e2-81eb-f7b6bed6ac89/work-item-properties/"
```

6. Upsert custom property values with:

```bash
curl -sS -X POST \
  -H "x-api-key: $PLANE_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"value":"<value-or-option-id>"}' \
  "https://api.plane.so/api/v1/workspaces/vladastos/projects/ec7904a4-cd24-40bd-8089-19a5eb8875ab/work-items/<work_item_id>/work-item-properties/<property_id>/values/"
```

Known RFC custom properties:

| Property | Property ID | Value |
|---|---|---|
| `RFC Number` | `71ccf8fa-9926-4216-8786-7f70e6eaec87` | decimal number, for example `42` |
| `RFC Status` | `4d858d79-066b-4948-b1bd-7f166b7cd024` | option ID from the table below |

Do not use the Plane `Doc Path` property for RFCs. The current path is derived from `RFC Number` plus the RFC file's frontmatter `id`.

Known `RFC Status` option IDs:

| Status | Option ID |
|---|---|
| `draft` | `bd6dc307-d1de-4363-a2ca-c99964258c49` |
| `under-review` | `de40e974-2b7e-4d7f-a070-165c28ee40f0` |
| `accepted` | `9fa2ac30-e5b0-4965-b83c-82bc92415218` |
| `implemented` | `e90b9ab6-b177-48bf-a356-fc1da3be2c89` |
| `superseded` | `63606434-211c-41ae-8f1e-7544b2d43d19` |
| `refused` | `d0f15e96-e5f0-4cc3-8c86-b876c8ae9f0c` |

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
6. **RFC state** - RFC files are in the directory for their current state, frontmatter agrees with that directory, and Plane `RFC` work items have the matching `RFC Status` custom property.
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

Follow `docs/internal/versioning.md` for the lifecycle and frontmatter requirements. An RFC has exactly one of these states, represented primarily by its directory:

- `0-draft/` - `draft`
- `1-under-review/` - `under-review`
- `2-accepted/` - `accepted`
- `3-implemented/` - `implemented`
- `4-superseded/` - `superseded`
- `5-refused/` - `refused`

Rules:

- The RFC document is the source of truth for design details.
- The directory is the source of truth for the RFC's lifecycle state. The RFC frontmatter and the Plane `RFC Status` custom property must reflect that directory exactly.
- The Plane item must use the custom `RFC` work item type, summarize the topic, link to the RFC file, and set `RFC Status` to the same state as the RFC directory; do not duplicate the whole RFC body in Plane.
- Accepted RFCs must have the relevant spec or internal architecture docs updated before implementation work begins.
- Implementation tasks should relate back to the RFC work item.
- When the target version ships, accepted RFCs that shipped must be moved to `3-implemented/` and their Plane `RFC Status` custom property must be set to `implemented`.

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
