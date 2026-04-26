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

The three core epics form the complete interpreter:

### Epic 001: Typechecker and Typed AST
**Status:** open  
**Depends On:** None (foundation)

Build a complete type-checking system with an AST that carries type information throughout evaluation.

**Goals:**
- Implement a typed AST representation
- Build a type inference engine
- Create a type checker that validates programs before execution
- Support basic types (int, float, bool, string, array, unit, tuple)

**Tasks:** 4 tasks (0001-0004)
- `0001` — Typed AST node design and implementation
- `0002` — Type inference engine
- `0003` — Type checker validation pass
- `0004` — Basic type system implementation

---

### Epic 002: Evaluator
**Status:** open  
**Depends On:** Epic 001 (Typechecker)

Implement the runtime engine that executes fully typed programs, transforming TypedAST into running code.

**Goals:**
- Evaluate all expression types
- Execute statements and control flow
- Handle function definitions and calls
- Support closures and first-class functions
- Implement built-in functions

**Tasks:** 4 tasks (0001-0004)
- `0001` — Value representation and basic evaluation
- `0002` — Expression evaluation (all 20 variants)
- `0003` — Control flow and statement execution
- `0004` — Function calls and closures

---

### Epic 003: Generics and Monomorphization
**Status:** open  
**Depends On:** Epic 002 (Evaluator)

Add generic type support and compile-time specialization through monomorphization.

**Goals:**
- Support type parameters on functions and types
- Implement type variable unification
- Handle generic instantiation (explicit and implicit)
- Specialize generics at compile time
- Support recursive and nested generics

**Tasks:** 5 tasks (0005-0009)
- `0005` — Type variables and constraint system
- `0006` — Generic type instantiation
- `0007` — Generic function type checking
- `0008` — Generic struct and enum type checking
- `0009` — Monomorphization engine

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

## Current Epics

1. **Epic 001:** `docs/Yolang/tasks/epic-001-typechecker/EPIC.md`
2. **Epic 002:** `docs/Yolang/tasks/epic-002-evaluator/EPIC.md`
3. **Epic 003:** `docs/Yolang/tasks/epic-002-generics/EPIC.md` (folder naming note: still named epic-002-generics, but is conceptually Epic 003)

## See Also

- `docs/Yolang/tasks/README.md` — More details
- `docs/Yolang/tasks/0000-template.md` — Task template
