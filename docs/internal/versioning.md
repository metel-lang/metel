---
id: versioning
title: "Versioning Model"
type: guide
created_date: '2026-05-21'
---

# Moonlane Versioning Model

This document is the authority on version numbering, the RFC lifecycle, and documentation conventions. All other guides defer to it on these topics.

---

## Version Numbering

All Moonlane releases — language spec and interpreter — share a single three-digit version number: `v<major>.<minor>.<patch>`.

| Segment | When to increment | Examples |
|---|---|---|
| **major** | Breaking changes to existing programs | `v0.x → v1.0` |
| **minor** | New language features; spec changes incorporated from accepted RFCs | `v0.4.0 → v0.5.0` |
| **patch** | Interpreter-only changes — bug fixes, refactors, performance — with **no spec changes** | `v0.4.0 → v0.4.1` |

**Rule:** `patch > 0` always means the spec is unchanged from the `.0` release of that minor version. A patch release never adds, removes, or alters any language-visible behaviour.

### Pre-1.0 era

Versions before `v1.0` cover the active development period. Minor versions may introduce significant new capabilities (generics, aspects, concurrency, the memory model). Breaking changes before `v1.0` are possible but must be called out explicitly in the CHANGELOG.

### Historical note

Versions v0.1 through v0.4 were tagged with two-digit identifiers (`v0.3`, `v0.4`) before this scheme was adopted. They are treated as equivalent to `v0.1.0`–`v0.4.0`. New releases use three digits.

---

## The Spec as a Living Document

`docs/public/spec.md` is the entry point for the language specification. It links to focused sub-files in `docs/public/spec/`. The spec describes the full language including features planned for future versions. Version snapshots are captured as **git tags**, not separate document files.

### Version tags

When a version is released, a single git tag is applied:

| Tag | Meaning |
|---|---|
| `vX.Y.0` | First release of spec version X.Y (spec + interpreter) |
| `vX.Y.Z` (Z > 0) | Patch release — interpreter only, spec unchanged |

**A tagged spec version is immutable.** If a spec error is discovered after tagging, it is documented as errata in the next version's CHANGELOG. Tags are never amended.

### Annotation style

Spec sections are annotated to indicate which version introduced or changed a feature:

| Situation | Annotation |
|---|---|
| Feature added in a specific version | `> *Since vX.Y.Z.*` |
| Existing feature changed in a version | `> *Changed in vX.Y.Z: description.*` |
| Feature planned for a future version | `> **vX.Y.Z feature.** description...` |

---

## RFC Lifecycle

RFCs are the mechanism for proposing language changes. An RFC must be accepted and assigned a target version before implementation begins.

### States

| State | Meaning |
|---|---|
| `draft` | Being written; not yet ready for review |
| `under-review` | Ready for evaluation; set manually by the author |
| `accepted` | Design decided; `target: vX.Y.0` assigned; implementation may begin |
| `rejected` | Will not be implemented; reason recorded in `## Decision` |
| `deferred` | Not rejected, but not scheduled for any version |
| `incorporated` | Implemented and shipped in the target version |

### Frontmatter fields

```yaml
---
id: rfc-NNNN
title: "..."
date: 'YYYY-MM-DD'
status: draft          # one of the states above
---
```

The target version is **not** stored in the RFC frontmatter. It lives in exactly one place: the GitHub issue milestone. The `## Decision` section records it in prose (`**Target:** vX.Y.0`) as a human-readable audit trail, but the milestone is the authoritative field.

### Acceptance process

1. Author sets `status: under-review` when the RFC is ready for evaluation.
2. Discussion happens in the linked GitHub issue.
3. The project owner records the outcome in a `## Decision` section at the bottom of the RFC file.
4. **If accepted**: set `status: accepted`, assign the RFC's GitHub issue to the target version milestone, and record `**Target:** vX.Y.0` in the `## Decision` section.
5. **If rejected or deferred**: set status accordingly; record the reason in `## Decision`.

Once the RFC's target version ships (git tag applied), set `status: incorporated`.

### Decision section format

```markdown
## Decision

**Outcome:** Accepted / Rejected / Deferred  
**Target:** vX.Y.0 *(if accepted)*

Brief rationale — why this design was chosen (or not), what alternatives were considered, and any constraints that drove the decision.
```

---

## GitHub Milestone Structure

| Milestone type | Examples | Purpose |
|---|---|---|
| **Version** | `v0.4.0`, `v0.5.0`, `v1.0.0` | Release planning — what ships in which version |

Implementation issues are assigned to the **version milestone** they target. Use label-based filtering (`--label "generics"`) for area-level CLI queries.

---

## Changelog

Version entries live in `docs/public/changelog.md`. Each entry lists RFCs incorporated, features added, breaking changes (if any), and whether it includes spec changes.

Patch releases (`vX.Y.Z` with Z > 0) get a short entry listing only the interpreter changes — no spec section needed.

---

## References

- Project vision and dual-mode commitment: `docs/internal/vision.md`
- Language spec: `docs/public/spec.md`
- Changelog: `docs/public/changelog.md`
