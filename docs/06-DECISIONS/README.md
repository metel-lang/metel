# Architecture Decision Records

This folder tracks **decisions** — choices that are non-obvious, have meaningful
alternatives, and whose rationale would otherwise be lost in task comments or
conversation history.

## When to write an ADR

Write one when:
- A question has two or more real options with genuine tradeoffs
- The wrong choice would require significant rework to reverse
- The rationale will not be obvious from the code or spec alone

Do NOT write one for:
- Implementation details with a clear single answer
- Choices that are trivially reversible
- Things already documented in the spec or architecture docs

## Folder structure

```
06-DECISIONS/
├── open/      # proposed — decision not yet made
├── closed/    # accepted | rejected | superseded
└── ADR-0000-template.md
```

## Workflow

```
1. Question arises in a task or discussion
   ↓
2. Create ADR in open/ with status "proposed" — document context and options
   ↓
3. Decision is made — update status to "accepted", fill in Decision and Consequences
   ↓
4. Move file from open/ to closed/
   ↓
5. Reference the ADR from the relevant task(s) and remove the open question
   ↓
6. If the decision is later reversed — mark "superseded by ADR-NNN", create new ADR in open/
```

## Naming

Files are named `ADR-NNNN-short-slug.md`, e.g. `ADR-0005-error-reporting.md`.
Numbers are assigned sequentially and never reused. The slug is lowercase-hyphenated.

## Referencing from tasks

Add a `**Decisions:**` field to the task header listing any ADRs that govern the
task's design. Use the path reflecting the ADR's current folder:

- Open: `**Decisions:** [ADR-NNNN](../../06-DECISIONS/open/ADR-NNNN-slug.md)`
- Closed: `**Decisions:** [ADR-NNNN](../../06-DECISIONS/closed/ADR-NNNN-slug.md)`

## Index

| ADR | Title | Status |
|-----|-------|--------|
| [ADR-0001](./closed/ADR-0001-type-registry.md) | TypeRegistry Structure and Location | accepted |
| [ADR-0002](./closed/ADR-0002-inference-pass-structure.md) | Inference Pass Structure | accepted |
| [ADR-0003](./closed/ADR-0003-v0.1-feature-set.md) | v0.1 Feature Set Scope | accepted |
| [ADR-0004](./closed/ADR-0004-interpreter-architecture.md) | Interpreter Architecture | accepted |
