---
id: rfc-0033
title: "Field-Level Mutability"
date: '2026-05-30'
status: draft
target:
---

## Summary

Add a `let` annotation on struct fields to mark them as permanently immutable after construction. Unannotated fields retain the current behaviour: they are mutable when the containing binding is `mut`. This is a **non-breaking, additive** change.

---

## Motivation

Metel's current mutability model is binding-level: a `mut` variable gives write access to every field of the struct it holds; a `let` variable gives access to none. There is no way to express that some fields are part of a value's *identity* (should never change after construction) while others are *state* (expected to change during the value's lifetime).

### The core problem

```metel
struct Connection {
    host:    String,   // identity — should never change
    port:    Int,      // identity — should never change
    retries: Int,      // state — changes as reconnects happen
}
```

Today, declaring `mut conn: Connection` to allow `conn.retries += 1` also silently allows `conn.host = "evil.example"` — there is no way to prohibit the latter. The programmer must rely on convention, not enforcement.

This matters most for:

- **Identity fields** — values that define what a thing *is* (`id`, `host`, `name`). Mutating them would be a semantic error, not just a style issue.
- **Post-construction invariants** — fields computed once at construction time from other fields. Allowing reassignment would silently violate the invariant.
- **Public API contracts** — with RFC-0032 field-level visibility, a `pub` field is now part of a module's public API. Making it `pub let` signals "you can read this, and I guarantee it never changes" — a stronger contract than "you can read this."

### Cross-language precedent

| Language | Per-field immutability | Keyword / mechanism |
|---|---|---|
| **Kotlin** | Yes — per property | `val` (immutable) vs `var` (mutable) |
| **Swift** | Yes — on class properties | `let` vs `var` |
| **C#** | Yes — on fields | `readonly` |
| **Java** | Yes — on fields | `final` |
| **OCaml** | Yes — on record fields | `mutable` (opt-in mutability; default immutable) |
| **Rust** | No — binding-level only | `Cell<T>` / `RefCell<T>` for interior mutability |
| **Go** | No | n/a |

The most directly relevant models for Metel:

- **Kotlin** (`val`/`var`) — closest in spirit; per-property, same keywords as variable declarations, default is mutable (`var`).
- **OCaml** (`mutable`) — default immutable fields, opt-in mutability. Clean but the opposite default from Metel's current model.
- **C# `readonly`** — field frozen after constructor exits; exactly the semantics proposed here.

Metel uses `let`/`mut` for variable declarations. Applying `let` to fields makes field-level immutability immediately readable for anyone who knows the language — no new vocabulary.

---

## Proposal

### Syntax

A field prefixed with `let` is immutable after the struct is fully constructed:

```metel
struct Connection {
    let host:    String,   // immutable — cannot be reassigned after construction
    let port:    Int,       // immutable
        retries: Int,       // mutable — follows the binding
}
```

Plain fields (no annotation) behave exactly as today.

`let` fields may appear in any order alongside plain fields. Mixing is valid and common.

### Semantics

**Assignment after construction is a compile error**, regardless of the mutability of the containing binding:

```metel
mut conn = Connection { host: "db.local", port: 5432, retries: 0 };

conn.retries += 1;   // OK — plain field, binding is mut
conn.host = "other"; // ERROR: field `host` is declared `let` and cannot be reassigned
conn.port = 443;     // ERROR: field `port` is declared `let` and cannot be reassigned
```

**`mut self` methods cannot reassign `let` fields:**

```metel
impl Connection {
    fun reconnect(mut self) {
        self.retries += 1;        // OK
        self.host = "fallback";   // ERROR: field `host` is `let`
    }
}
```

**Construction is the only assignment point.** All fields — `let` and plain — are assigned exactly once in the struct literal:

```metel
let conn = Connection { host: "db.local", port: 5432, retries: 0 };
```

There is no `init`-block or deferred assignment syntax. `let` fields must be provided in the struct literal at construction time; they cannot be left uninitialised and filled in later.

### Interaction with binding mutability

Binding mutability and field mutability are orthogonal. The table of valid operations:

| Binding | Field annotation | Can read? | Can mutate? |
|---|---|---|---|
| `let` binding | plain field | Yes | No — binding is immutable |
| `let` binding | `let` field | Yes | No — both prohibit it |
| `mut` binding | plain field | Yes | **Yes** |
| `mut` binding | `let` field | Yes | **No — field annotation wins** |

A `let` field is always immutable. A plain field defers to the binding.

### Interaction with RFC-0032 (field-level visibility)

`let` and `pub` are independent annotations that compose freely:

```metel
pub struct Token {
    pub let kind: TokenKind,   // readable by anyone, never mutable
    pub let span: Span,        // readable by anyone, never mutable
        let id:   Int,         // module-private (RFC-0032), never mutable
            pos:  Int,         // module-private, mutable if binding is mut
}
```

The annotation order is `pub let field: Type`. `pub` comes first (consistent with `pub fun`, `pub struct`). `let` comes second.

This gives a clean 2×2 at the field level:

|  | plain | `let` |
|---|---|---|
| *(no pub)* | module-private, binding-mutable | module-private, immutable |
| `pub` | externally readable, binding-mutable | externally readable, immutable |

### Interaction with `linear struct`

`let` fields on a `linear struct` follow the same rules. A linear value can only be *consumed* (moved), not mutated. `let` fields are consistent with this — consumption is not mutation. The rule is simply that `let` fields cannot be reassigned; linear consumption does not constitute reassignment.

### `linear enum` and enum struct variants

`let` annotations are valid on struct-variant fields:

```metel
enum Event {
    Connect { let host: String, let port: Int },
    Disconnect { reason: String },
}
```

Tuple-variant fields are positional and cannot be individually annotated, consistent with RFC-0032's decision for tuple variants. Tuple variant fields follow binding mutability only.

### Update syntax (`..`)

Struct update syntax (if adopted in a future RFC) must forbid updating `let` fields:

```metel
let conn2 = Connection { retries: 0, ..conn };   // OK — only plain fields updated
let conn3 = Connection { host: "x", ..conn };    // ERROR — host is `let`
```

This is a forward-compatibility note; struct update syntax is not yet part of the language.

### Grammar impact

```
field-decl ::= 'pub'? 'let'? identifier ':' type ','
```

`pub let` is the only valid combined form. `let pub` is a parse error.

---

## Alternatives Considered

### A — `mut` on fields, immutable by default (OCaml model)

Fields are immutable by default; `mut` opts into mutability:

```metel
struct Connection {
    host:      String,     // immutable by default
    port:      Int,        // immutable by default
    mut retries: Int,      // explicitly mutable
}
```

**Pros:** Immutability is the default — the safer choice. Consistent with the philosophy that mutation should be explicit.

**Cons:** **Breaking change** — all existing field assignments from `mut` bindings would break. Every struct with any mutable field needs to be updated. Also adds the word `mut` in a new position, conflicting with the established meaning of `mut` on bindings (which makes all fields mutable). Seeing `mut` on a field and `mut` on a binding — both governing mutability but in completely different ways — creates conceptual confusion.

**Verdict:** Rejected. The breaking cost is high and the keyword reuse is confusing. The proposed `let` opt-in model achieves the same expressiveness without breaking anything.

### B — `readonly` keyword on fields

```metel
struct Token {
    readonly kind: TokenKind,
    readonly span: Span,
}
```

**Pros:** Explicit, unambiguous, familiar from C#.

**Cons:** Adds new vocabulary when `let` already exists and carries the right meaning in Metel. "Readonly" is also more verbose. Not in keeping with Metel's terse, expression-oriented style.

**Verdict:** Rejected in favour of `let`, which is already established language vocabulary.

### C — Immutable struct types (`frozen struct`)

Mark an entire struct as immutable rather than individual fields:

```metel
frozen struct Point { x: Float, y: Float }
```

**Pros:** Simple; no per-field decision.

**Cons:** All-or-nothing. Cannot express a struct with a mix of identity fields and state fields (the `Connection` example above). Does not compose with RFC-0032 at the field level.

**Verdict:** Potentially useful as a shorthand for structs where *all* fields should be `let`, but does not cover the general case. Could be layered on top of this RFC later as sugar for `struct T { let f1: T1, let f2: T2 }`.

### D — Interior mutability wrappers (Rust's `Cell<T>` model)

No per-field syntax. Fields that need to be mutable despite an immutable binding use a wrapper type that encapsulates the mutation:

```metel
struct Connection {
    host:    String,
    port:    Int,
    retries: Cell<Int>,   // hypothetical
}
```

**Pros:** No new syntax; mutation semantics are purely type-level.

**Cons:** Requires a `Cell<T>` type in `std::core`, adding runtime overhead for a purely compile-time concept. Ergonomically unpleasant for common cases. Adds complexity to the type system before the simpler syntactic approach is tried.

**Verdict:** Deferred. Interior mutability wrappers may be needed for advanced use cases (shared mutable state across references) but should not be the *primary* mechanism for a simple "this field never changes" constraint.

---

## Open Questions

### OQ-1 — Interaction with future struct update syntax

If struct update syntax (`Connection { retries: 0, ..conn }`) is added later, should updating a `let` field be:
- **A compile error** (proposed above — consistent with "never reassigned after construction").
- **Allowed**, on the grounds that update syntax constructs a *new* value, not mutating the original.

This depends on how struct update syntax is defined. If it is pure construction (produces a new value), allowing `let` field overrides is semantically sound. If it is defined as "copy then patch," it conflicts with `let`.

**Recommendation:** Resolve this when struct update syntax is designed. Suggest treating update syntax as new construction — `let` fields can appear in the update, but only if they produce a new value, not patch an existing one.

### OQ-2 — Error code

Private field access already has an error code proposed in RFC-0032 (T0013). Mutating a `let` field needs its own distinct error code. Recommendation: T0014 — "assignment to `let` field `{name}`."

### OQ-3 — Relationship to future `const` fields

If Metel later adds compile-time constants at the struct level (e.g. associated constants), `let` fields and `const` fields would overlap in meaning but differ in timing: `let` is a runtime immutability guarantee; `const` would be a compile-time constant. They should not share the same keyword. This RFC does not block a future `const` field design.

### OQ-4 — Linting: suggest `let` for never-reassigned fields

Should the compiler/linter suggest adding `let` to plain fields that are never mutated in any `mut self` method in the same module? This would be a quality-of-life lint, not a hard error. Out of scope for the spec but worth noting for tooling.

---

## Timing Recommendation

This RFC is a natural companion to RFC-0032 (Field-Level Visibility). The two RFCs compose at the field annotation level (`pub let field: Type`) and should ideally be implemented in the same release to avoid a two-phase grammar change. If RFC-0032 is accepted first and implemented without `let`, the grammar extension is still backwards-compatible — `let` is a new optional annotation, not a change to existing syntax.

No existing feature blocks this RFC. It can be targeted to the same version as RFC-0032.

---

## References

- RFC-0032: Field-Level Visibility — `docs/internal/rfcs/rfc-0032-field-level-visibility.md`
- Language spec: `docs/public/spec/declarations.md` — Structs section
- Kotlin visibility modifiers: [kotlinlang.org/docs/visibility-modifiers.html](https://kotlinlang.org/docs/visibility-modifiers.html)
- C# readonly fields: [learn.microsoft.com — readonly keyword](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/keywords/readonly)
- OCaml mutable record fields: [ocaml.org/manual — record expressions](https://ocaml.org/manual/5.1/expr.html)
