# /gap-analysis

Analyse the planned sprint work items for description gaps, scope ambiguities, and missing tasks. Gather all needed user input in a single pass, then update Plane so the sprint can be executed without further clarification.

**Arguments:** `$ARGUMENTS` — sprint number, e.g. `16`

**The agent must complete the full analysis before asking the user anything. All questions are batched into one interaction, not asked one at a time.**

---

## Step 1 — Load sprint context

Call `mcp__plane__list_cycles` on the Metel project and find the cycle for Sprint `<N>`. Note the cycle UUID, sprint goal, and milestone.

Call `mcp__plane__list_cycle_work_items` (or `mcp__plane__list_work_items` with `cycle_ids`) to retrieve every work item in the cycle. For each item, fetch its full details via `mcp__plane__retrieve_work_item_by_identifier` — you need the description, labels, state, milestone, and any linked items.

Also read `CLAUDE.md` for the active milestone and any active epics, and read the corresponding spec sections in `docs/public/spec/` that are likely to be touched by the sprint theme.

---

## Step 2 — Analyse each work item

For every work item in the sprint, silently evaluate all of the following. Record every finding — you will present them all at once in Step 3.

### 2a. Description completeness
Ask yourself: could an agent implement this from the description alone, without asking any questions?

Flag the item if any of these are missing or vague:
- **What** specifically needs to change (which file, function, grammar rule, type, or behaviour)
- **Why** — the motivation or constraint driving the change
- **Acceptance criteria** — explicit, testable conditions for "done" (not just "it works")
- **Edge cases** — what should happen at boundaries (empty input, type mismatch, recursive structure, etc.)
- **Error behaviour** — what error code or message should be produced on invalid input

### 2b. Scope
Flag the item if it contains more than one independent concern that could fail or be deferred separately. Signs of over-scoping:
- The description contains "and also…" for unrelated behaviour
- Implementing it requires touching more than two unrelated modules
- It could be split into a spec change + an implementation without loss of coherence

### 2c. Spec and RFC alignment
For each work item:
- Is there a spec section in `docs/public/spec/` that governs this behaviour? If so, does the spec already describe the target behaviour, or does the spec need updating first?
- Does this require an RFC? If so, does the RFC exist in `docs/internal/rfcs/` and is its `status` either `draft` (needs acceptance) or `accepted`?
- Does this implement an already-accepted RFC? If so, note the RFC id.

Flag: missing spec section, RFC not yet accepted, RFC not yet written, or spec/RFC conflict.

### 2d. Dependencies
- Does this item depend on another work item in the sprint or in Backlog? Check the description for `METEL-N` references and "Depends on" sections.
- If a dependency exists, is it scheduled before this item?
- Does this item require a spec change that is not tracked as a separate work item?

### 2e. Test requirements
For the acceptance criteria to be verifiable, there must be a clear test strategy. Flag if:
- There are no acceptance criteria that map to a concrete test
- The item touches the typechecker or evaluator but has no negative-case test requirement stated
- The item changes a builtin but the spec table update is not mentioned

---

## Step 3 — Analyse the sprint as a whole

After analysing individual items, look at the sprint collectively:

### 3a. Sprint goal coverage
Re-read the sprint goal. List every concern implied by the goal. Check each implied concern against the work item list. Flag any implied concern that has no work item tracking it.

### 3b. Implementation order
Identify the natural implementation order given dependencies. Flag any ordering conflict (item A depends on item B, but B is not earlier in the dependency graph).

### 3c. Missing scaffolding
Flag if the sprint requires any of the following but has no work item for it:
- A spec section that does not yet exist
- A new error code or error variant
- A new AST node or grammar rule
- A test fixture or `.mln` test file that does not yet exist
- A new type or typed AST node

### 3d. Risk items
Flag any work item that:
- Touches `src/typeinference/mod.rs` or `src/typechecker/mod.rs` (high blast radius)
- Requires changing the grammar in `src/grammar.pest` (ripple effects to parser and AST)
- Has no prior art in the codebase (first instance of a pattern)

For each risk item, check whether an investigation or spike task should be added before the implementation task.

---

## Step 4 — Batch all questions

Do **not** ask questions one at a time. Compile every finding from Steps 2 and 3 into a single structured report presented to the user before making any changes.

Format the report as follows:

```
## Gap Analysis — Sprint <N>

### Work item gaps
For each flagged item:

**METEL-N — <title>**
- Gap type: [Description / Scope / Spec / RFC / Dependency / Test]
- Finding: <one sentence describing what is missing or ambiguous>
- Question: <the specific question whose answer fills the gap>

### Sprint-level gaps
For each sprint-level finding:

**[Coverage / Order / Scaffolding / Risk]**
- Finding: <what is missing or risky>
- Question: <the specific question or decision needed>

### Proposed new work items
For each gap large enough to warrant a new task:

**Proposed: <title>**
- Reason: <why this needs to be a separate work item>
- Suggested description: <draft description>
- Question: Should this be added to the sprint, deferred to backlog, or is it already covered?
```

Wait for the user to answer **all** questions before proceeding.

---

## Step 5 — Update work items in Plane

Using the user's answers, update every flagged work item via `mcp__plane__update_work_item`:
- Rewrite or extend the `description` field to incorporate the clarified requirements, explicit acceptance criteria, edge cases, and error behaviour.
- Keep the original intent — do not replace, extend.
- If an RFC reference was identified, add it to the description: `RFC: rfc-NNNN`.
- If a spec section was identified, add it: `Spec: docs/public/spec/<section>.md`.

---

## Step 6 — Create new work items

For each gap the user confirmed should be a new work item:
- Call `mcp__plane__create_work_item` with a complete description (not a stub — use the information gathered in Steps 2–4 to write it fully).
- Set `state_id` to the **Todo** state UUID (these are sprint items, not backlog).
- Assign the sprint milestone via `mcp__plane__add_work_items_to_milestone`.
- Add to the sprint cycle via `mcp__plane__add_work_items_to_cycle`.

For each gap the user deferred to backlog:
- Call `mcp__plane__create_work_item` with `state_id` set to **Backlog**.
- Do not add to the sprint cycle.

---

## Step 7 — Verify and report

After all updates are applied, call `mcp__plane__list_cycle_work_items` again and present the final sprint work item list:

```
## Sprint <N> — Ready for execution

**Goal:** <sprint goal>
**Milestone:** <milestone>

### Work items
- METEL-N — <title> [labels]
  Acceptance criteria: <1-line summary>
  ...

### Deferred to backlog
- METEL-N — <title> (reason: <why deferred>)
  ...

### Risk items requiring extra care
- METEL-N — <title>: <risk summary>
```

Remind the user: the sprint is now ready. Run `/start-issue METEL-<N>` to begin work on the first item.

---

## Notes

- The goal of this skill is **zero surprises during implementation**. Every question that could arise mid-sprint should be answered here.
- Do not update Plane until after the user has answered all questions in Step 4. Do not make partial updates.
- If the user's answers reveal that a work item is out of scope for this sprint entirely, move it to Backlog via `mcp__plane__update_work_item` (state → Backlog) and remove it from the cycle — do not leave it as a Todo item that will not be worked.
- If the sprint goal itself is unclear after analysis, surface that as the first question — a sprint with an unclear goal cannot be gap-analysed.
- The Metel project identifier is `METEL`; use `mcp__plane__retrieve_work_item_by_identifier` with `METEL-<N>` to resolve sequence IDs to UUIDs when needed.
