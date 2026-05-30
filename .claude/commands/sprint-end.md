# /sprint-end

Close a sprint: run the mandatory quality gate, then push `main` and create a tag.

**Arguments:** `$ARGUMENTS` — sprint number, e.g. `1`

**This skill will not push until every quality gate passes. See AGENTS.md § Quality Gate for the rationale.**

---

## Step 1 — Fetch sprint context

Read the sprint cycle in Plane to get the sprint goal, planned work items, and milestone:

```
mcp__plane__list_cycles  →  find Sprint $ARGUMENTS
mcp__plane__list_cycle_work_items  →  get all items in that cycle
```

Identify the **milestone** (e.g. `v0.6.4`) — every commit and Plane update during sprint-end must use this milestone.

Categorise all planned work items as completed (Done state) or carried over (not Done).

---

## Step 2 — Quality Gate (mandatory — do not skip)

Work through every gate in order. **If any gate fails, stop, report what failed, and do not proceed.**

### Gate 1: Test suite

```bash
cargo test
```

All tests must pass with zero failures. If any fail, fix them before continuing.

### Gate 2: Code quality

Inspect every file changed on the sprint branch relative to `main`:
```bash
git diff main..HEAD --name-only
```

For each changed Rust file, check:
- No stale `todo!()`, `unimplemented!()`, or `unreachable!()` without a tracking issue linked in a comment.
- No `unwrap()` or `expect()` on paths that can fail at runtime (parsing errors, env lookups on user input).
- No unused imports, dead match arms, or commented-out code left behind.
- Builtins are registered in **all** required places: `src/typechecker/registry.rs` (inference pass) **and** `src/typechecker/construction.rs` (construction pass). A builtin missing from construction.rs will typecheck but fail with "undefined name" at runtime.

Report each finding. If a `todo!()`/`unreachable!()` is intentional (e.g. a placeholder variant per an RFC), verify a tracking issue exists and note it.

### Gate 3: Test coverage

For every feature or fix introduced in the sprint, verify a test exists:

| Change type | Required test location |
|---|---|
| New builtin | `metel-interpreter/tests/typechecking/sources/stage*_*.mln` — positive and at least one negative (wrong arg type) |
| New grammar construct | Parsing test or typechecking test |
| New evaluator behaviour | Evaluator test in `metel-interpreter/tests/evaluator_tests.rs` or integration `.mln` file |
| Bug fix | A regression test that would have caught the original bug |
| New error code | A negative typechecking or evaluator test that triggers it |

List each sprint change and confirm its test. Flag any untested changes — either add a test or document in the PR body why it is untestable.

### Gate 4: Spec accuracy

For every language-visible change in the sprint, verify the spec reflects it:

1. Check `docs/public/spec/runtime.md` builtin table against `src/typechecker/registry.rs` — every `ctx.bind_poly(...)` call must have a matching row.
2. Check `docs/public/spec/` for each new grammar construct — it must be described in the appropriate section (expressions, declarations, etc.).
3. Check each RFC implemented this sprint — its frontmatter `status` must be `incorporated`.
4. Check `docs/public/changelog.md` — the current version milestone must have an entry listing the sprint's shipped features.

Report any spec/code divergence found.

### Gate 5: Spec completeness

Read `docs/public/spec.md` and every section it links to. Verify:
- No section refers to a feature that was removed or renamed this sprint without updating the reference.
- No `TODO`, `TBD`, or `(coming soon)` markers were introduced by sprint work.
- All cross-references between spec sections are still valid.

### Gate 6: Internal doc accuracy

For every component touched during the sprint, check:

| Component | Doc to verify |
|---|---|
| Evaluator (`src/evaluator/`) | `metel-interpreter/docs/evaluator.md` — Value variants, signals, builtins, known limitations |
| Typechecker (`src/typechecker/`) | `metel-interpreter/docs/typechecker.md` — passes, constraints, inference rules |
| Parser / grammar | `metel-interpreter/docs/architecture.md` — pipeline diagram still accurate |

Report any internal doc that is stale or missing.

### Gate 7: Architectural decision records

Review every commit on the sprint branch:
```bash
git log main..HEAD --oneline
```

For each commit, ask: did this change involve a non-obvious architectural decision? Use the criteria from AGENTS.md § Decision Records. Examples of what qualifies:
- A choice between two plausible designs with real trade-offs
- A deliberate deviation from a prior decision record or RFC
- A constraint or invariant that future contributors must know to avoid breaking the design
- A workaround for a language or library limitation that isn't obvious from the code

For each qualifying decision, verify a decision record exists in `metel-interpreter/docs/decisions/`. If any are missing, create them now — before proceeding.

List every qualifying decision found and whether a record exists or was created.

### Gate 8: ADR links in code

For every ADR written or referenced this sprint, check whether the code it governs carries an inline comment linking back to it. Use the criteria from AGENTS.md § Linking decisions to code:

- Code that looks wrong but is intentional (a workaround, a deliberate shortcut, a known limitation) **must** have a comment explaining the reason and citing the ADR.
- A load-bearing invariant the ADR documents **must** have a comment at the enforcement point.
- Routine code that simply implements a spec rule does **not** need a link.

```bash
grep -rn "ADR-\|adr-" metel-interpreter/src/
```

For each ADR written this sprint: read it, identify the specific code it governs, and verify the comment is present and informative (not just `// see ADR-NNNN`). Add comments where missing.

List each ADR, the file(s) it governs, and whether a comment was present or added.

---

## Step 3 — Fix findings before continuing

For every failing gate from Step 2: fix the issue, commit to the sprint branch, and re-run the relevant gate check. Do not proceed until all gates are green.

---

## Step 4 — Integration tests (language version sprints only)

**If this sprint ships a language version milestone** (i.e. the milestone is a version tag such as `v0.3`):

Write comprehensive integration tests that exercise the **complete feature set** of that version — not just features added this sprint. These tests must:
- Live in `metel-interpreter/tests/evaluator/sources/` as `int_NN_<name>.mln`
- Be self-asserting (`assert(...)`)
- Cover all combinations of new features interacting (generics + closures, structs + enums, etc.)
- Use idiomatic Metel: type annotations where expected, explicit braces where required

After writing the tests, run them:
```bash
cargo test int_
```

**Examine every failure and inconsistency found.** For each:
- If it is a bug: fix it, add a regression test, commit.
- If it exposes a spec ambiguity: note it and open a tracking issue.
- If it reveals a limitation that is out of scope for this version: document it in the relevant `docs/*.md` Known Limitations section and open a tracking issue for the next version.

Report a summary of: tests written, failures found, fixes made, issues opened.

Do not proceed to Step 5 until all integration tests pass.

---

## Step 5 — Bump the crate version in Cargo.toml

Read the milestone version (e.g. `v0.6.4`). Strip the leading `v` to get the semver string (e.g. `0.6.4`).

Update `metel-interpreter/Cargo.toml`:
```toml
version = "0.6.4"
```

Update `docs/public/changelog.md` — add a new entry for this version above the previous one.

Commit both changes together on the sprint branch.

The crate version and changelog must be updated before pushing.

---

## Step 6 — Mark carried-over items in Plane

For each work item that is still open and was planned for this sprint, move it to the backlog state via `mcp__plane__update_work_item`.

---

## Step 7 — Push main

Fast-forward `main` to the sprint branch:

```bash
git checkout main
git rebase sprint/$ARGUMENTS
git push origin main
```

---

## Step 8 — Hand off to user

Provide a sprint summary inline (not as a separate issue or PR):

```
## Sprint $ARGUMENTS — <theme> — CLOSED ✅

**Version:** v<X.Y.Z>
**Quality gate:** All 8 gates passed.

### Completed
- METEL-N: <title>
...

### Carried over
- METEL-N: <title> (reason)
...

### Spec / doc changes
- <file>: <what changed>
...

### ADRs written this sprint
- ADR-NNNN: <title> — governs <file(s)>
...

### Integration tests (if applicable)
N tests written, M failures found, K fixed.
```

Then instruct the user:

> **`main` is up to date. Create the release tag:**
> ```bash
> git tag -a v<X.Y.Z> -m "v<X.Y.Z>: <sprint theme>" && git push origin v<X.Y.Z>
> ```
> Then delete the sprint branch: `git push origin --delete sprint/$ARGUMENTS`

**The tag must be created on `main` after the rebase — never on the sprint branch.**
The tag name must match the version in `docs/public/changelog.md`.

---

## Notes

- Do not push `main` until every quality gate passes.
- Do not create the release tag — instruct the user to create it after verifying the push.
- A sprint with 0 completed items still produces a summary — record why in Completed.
- If spec ambiguities surfaced (visible in Gate 5 or Spec Notes), prompt the user to open a `/new-rfc`.
- All Plane state changes during sprint-end must use the sprint's milestone.
