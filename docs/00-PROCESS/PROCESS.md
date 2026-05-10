# Yoloscript Documentation Process

## The three documents

There are exactly three kinds of documentation, each with a strict and non-overlapping role.

---

### 1. `docs/01-SPEC/LANGUAGE-SPEC.md` — the source of truth

This is the only place where the language is described. It is authoritative: if something is in the spec, it is part of the language. If it is not in the spec, it does not exist yet.

The spec answers: **"How does the language work?"**

When a feature is designed and accepted, it is written into the spec. The spec does not contain rationale, history, or open questions.

---

### 2. `docs/01-SPEC/BACKLOG.md` — what is not in the spec yet

The backlog tracks every open design question and every deferred feature. It is the only place that records "what is missing."

The backlog answers: **"What still needs to be designed or implemented?"**

Each item has a status:

- `open` — not yet designed; needs a decision before it can be written into the spec
- `deferred` — consciously excluded from the current version; a reason is given
- `in-progress` — actively being discussed or drafted

When a backlog item is resolved, its entry is removed from the backlog and the feature is written into the spec. The spec is the proof it's done.

---

### 3. `docs/06-DECISIONS/` — why a non-obvious choice was made

Decision records capture the reasoning behind choices that were not obvious or that had meaningful alternatives. They are written once and never modified (superseded decisions get a new record).

Decision records answer: **"Why did we choose X over Y?"**

A decision record does **not** re-describe what the feature does — it points to the spec for that. It records context, alternatives considered, and the reasoning for the choice.

Only write one when:
- Multiple reasonable options existed and the choice was non-trivial
- The rationale will be useful when the decision is revisited later
- A previous decision is being reversed
- Implementation revealed something that forced a spec change (see below)

---

## Incorporating implementation

The interpreter and compiler are not just consumers of the spec — they are feedback sources for it. Implementation routinely reveals ambiguities, wrong assumptions, and missing details that pure design work misses. The process accounts for this explicitly.

### The spec has a readiness level

Every section of the spec is implicitly at one of three levels:

| Level | Meaning |
|-------|---------|
| **Designed** | Written in the spec; not yet implemented |
| **Interpreter-validated** | Implemented and tested in the interpreter; spec confirmed accurate |
| **Compiler-validated** | Implemented in the compiler; any lower-level implications (layout, codegen) resolved |

Nothing moves to a higher level without going through the lower one first. The compiler does not implement a feature until the interpreter has validated it. This means the spec only needs to be complete enough for the current implementation phase — not the next one.

### How implementation feeds back into the spec

Implementation work produces two kinds of findings:

**Ambiguity** — the spec does not say what happens in some case. Resolution: decide and update the spec immediately before continuing. If the decision is non-obvious, write a decision record. Do not implement a behaviour that is not in the spec.

**Incorrectness** — the spec says something that turns out to be wrong, contradictory, or impractical to implement. Resolution: same as reversing a decision — write a new decision record explaining what was wrong and why, then update the spec. The interpreter or compiler implementation follows the updated spec, not the old one.

In both cases, the spec is updated before or alongside the implementation, never after. An implementation that diverges from the spec is always a bug, either in the implementation or in the spec — never an intentional divergence.

### Version tagging in the spec

When a section has been interpreter-validated, it is tagged at the top of that section:

```
> ✓ Interpreter-validated (v0.1)
```

When compiler-validated:

```
> ✓ Compiler-validated (v0.1)
```

Untagged sections are designed but not yet validated. This makes it immediately clear which parts of the spec can be trusted as ground truth versus which parts are still theoretical.

---

## Workflow for a change

### Adding or modifying a feature (spec-first)

1. If the design is open, add an item to the backlog (`open`).
2. Discuss and decide. If the decision is non-obvious, write a decision record.
3. Write the feature into `Language Spec.md`.
4. Remove the resolved item from the backlog.
5. Implement against the spec. If ambiguities surface, resolve them in the spec first (step 3 again), then implement.

### Deferring a feature

1. Add it to the backlog with status `deferred` and a one-line reason.
2. No other files change.

### Reversing a decision

1. Write a new decision record that supersedes the old one.
2. Update `Language Spec.md` to reflect the new behaviour.
3. The old decision record is not deleted — its status is updated to `Superseded by NNNN`.

### Handling an implementation finding

1. Stop. Do not work around the issue in the implementation.
2. Determine whether this is an ambiguity or an incorrectness in the spec.
3. Make the decision (write a decision record if non-obvious).
4. Update the spec.
5. Continue the implementation against the updated spec.

---

## What not to do

- Do not implement behaviour that is not in the spec. If it is not in the spec, add it to the spec first.
- Do not let the implementation diverge from the spec and "fix the docs later." Later never comes.
- Do not mirror spec content in decision records. Link to the spec section instead.
- Do not add rationale or history to the spec. That belongs in a decision record.
- Do not create additional tracking documents. All open work lives in the backlog.
- Do not skip levels — the compiler should not implement a feature the interpreter has not yet validated.
