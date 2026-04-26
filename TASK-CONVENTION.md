# Task Management for Yolang

Two-level task organization: **Epics** (major features/milestones) contain **Tasks** (units of work). Folder structure reflects task state for easy navigation.

## Quick Reference

**Create an epic:** Make folder `docs/Yolang/tasks/epic-NNN-slug/` with subfolders `open/`, `in-progress/`, `done/`, `blocked/`

**Create a task:** Copy `docs/Yolang/tasks/epic-NNN-slug/0000-template.md` → save to appropriate status folder → rename to `NNNN-slug.md`

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
docs/Yolang/tasks/
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
**Spec Link:**   spec/Language Spec.md#Section-Name (or Backlog item)
**Blocked By:**  task IDs or "none"

## What
What needs doing and why.

## Acceptance Criteria
- [ ] Testable outcome 1
- [ ] Testable outcome 2
- [ ] No regressions

## Notes
(Optional) Progress and discoveries
```

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

## Epics

### Epic 001: Typechecker and Typed AST
**Status:** open

Build a complete type-checking system with an AST that carries type information throughout evaluation. This epic establishes the foundation for type safety and enables better error messages.

**Goals:**
- Implement a typed AST representation
- Build a type inference engine
- Create a type checker that validates programs before execution
- Support basic types (int, bool, string, lists)

**Current Tasks:**
- `open/0001-typed-ast-nodes.md` — Define AST nodes with type annotations
- `open/0002-type-inference.md` — Implement type inference rules
- `open/0003-type-checker.md` — Build type validation pass
- `open/0004-basic-types.md` — Support int, bool, string, list types

**Blocked/Future:**
- Generics and parametric polymorphism (depends on type-checker)
- Type aliases and custom types

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

- `docs/Yolang/tasks/README.md` — More details
- `docs/Yolang/tasks/epic-001-typechecker/EPIC.md` — Typechecker epic details
- `docs/Yolang/tasks/0000-template.md` — Task template
