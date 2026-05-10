# Task Management for Yoloscript

Two-level task organization: **Epics** (major features/milestones) contain **Tasks** (units of work). Folder structure reflects task state for easy navigation.

## Quick Reference

**Create an epic:** Make folder `docs/05-TASKS/epic-NNN-slug/` with subfolders `open/`, `in-progress/`, `done/`, `blocked/`

**Create a task:** Copy `docs/05-TASKS/epic-NNN-slug/0000-template.md` → save to appropriate status folder → rename to `NNNN-slug.md`

**Move task:** Change status by moving file to corresponding subfolder (or update `**Status:**` field for quick reference)

**When done:** Check acceptance criteria → move to `done/` subfolder → update spec if needed

## What Are Epics?

An **epic** is a major language feature, subsystem milestone, or architectural component. Each epic:
- Gets its own folder: `epic-NNN-slug` (e.g., `epic-001-typechecker`)
- Contains related tasks organized by status
- Has a high-level `EPIC.md` describing scope, goals, and dependencies
- Typically spans multiple milestones or weeks of work

**Example structure:**
```
docs/05-TASKS/
├── epic-001-typechecker/
│   ├── EPIC.md                          # Epic description and goals
│   ├── open/
│   │   └── 0001-typed-ast-nodes.md
│   ├── in-progress/
│   │   └── 0002-type-inference.md
│   ├── done/
│   │   └── 0003-basic-type-checking.md
│   └── blocked/
│       └── 0004-generics.md             # blocked by 0002
│
├── epic-002-error-recovery/
│   ├── EPIC.md
│   ├── open/
│   └── done/
```

## Task Fields

```markdown
# Task NNNN: Brief Title

**Status:**      open | in-progress | done | blocked
**Epic:**        epic-001-typechecker
**Component:**   interpreter | repl | parser | typechecker | evaluator | error-handling | spec
**Spec Link:**   01-SPEC/LANGUAGE-SPEC.md#Section-Name (or Backlog item)
**Blocked By:**  task IDs or "none"
**Decisions:**   none | ADR-NNNN, ADR-NNNN (links to docs/06-DECISIONS/)

## What
What needs doing and why.

## Acceptance Criteria
- [ ] Testable outcome 1
- [ ] Testable outcome 2
- [ ] No regressions

## Notes
(Optional) Progress and discoveries
```

## Open Questions and Decisions

Open questions in tasks are temporary placeholders for unresolved design choices.
Once a question is resolved:

1. **If the answer was obvious** (one clear option, no real tradeoff) — absorb it
   directly into the task's Architecture or Notes section, then remove the question.

2. **If the answer involved real alternatives** (two or more options with genuine
   tradeoffs, or a choice whose rationale would be non-obvious from the code) —
   create an ADR in `docs/06-DECISIONS/`, add it to the task's `**Decisions:**`
   field, then remove the open question.

See `docs/06-DECISIONS/README.md` for the full ADR workflow.

## Rules

1. **Every task links to a spec** (or a backlog item if not yet speced)
2. **Every task belongs to an epic** (via `**Epic:**` field)
3. **Components:** Connect tasks to the subsystems they affect
   - `interpreter` — overall interpreter
   - `repl` — interactive shell
   - `parser` — parsing and grammar
   - `typechecker` — type inference/checking
   - `evaluator` — runtime execution
   - `error-handling` — error messages and recovery
   - `spec` — spec work only
4. **Status is honest:** If you haven't touched a task in days, mark it `blocked` with reason
5. **Acceptance criteria are testable:** Not "improve error messages" but "error reports include X and Y"
6. **Folder structure mirrors status:** File location reflects true status (move file when status changes)

## Workflow

```
1. Create or pick epic
   ↓
2. Create task in epic's open/ folder with status "open"
   ↓
3. Start work → move to in-progress/ → set status "in-progress"
   ↓
4. If stuck → move to blocked/ → set status "blocked" (with reason)
   ↓
5. Finish → check criteria, move to done/, update spec, set status "done"
```

## See Also

- `docs/05-TASKS/README.md` — More details
- `docs/05-TASKS/0000-template.md` — Task template
- `docs/06-DECISIONS/README.md` — ADR process and index
- `docs/06-DECISIONS/ADR-0000-template.md` — ADR template
- `docs/06-DECISIONS/open/` — proposed ADRs (decision pending)
- `docs/06-DECISIONS/closed/` — accepted, rejected, or superseded ADRs
