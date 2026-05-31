---
id: rfc-0035
title: "Aspect Bound Syntax (Revised)"
date: '2026-05-31'
status: draft
supersedes: rfc-0002
target:
---

## Summary

Supersedes RFC-0002. RFC-0002 was incorrectly marked as fully accepted and incorporated; in practice only single-bound syntax (`<T: Aspect>` and `where T: Aspect`) reached the spec. The multi-bound separator, anonymous type parameter syntax, `aspect` aliases, `type` aliases, and associated type constraint form were never incorporated. Several decisions from RFC-0002 have also changed. This RFC replaces RFC-0002 in full and defines the complete, authoritative design for aspect bound syntax.

---

## Motivation

Generic functions and types need a way to express constraints on type parameters. The current spec covers only the single-bound case and leaves the following unspecified:

- How to express multiple bounds on a single type parameter
- Whether `where` and inline bound forms are equivalent or one is preferred
- How to write a generic parameter position without naming the type parameter
- How to bundle a set of bounds under a reusable name
- How to constrain associated types

RFC-0002 proposed answers but was never fully reflected in the spec, and two of its decisions have since been revised.

---

## Proposal

### 1. Inline and `where` forms are equivalent

Both forms express identical constraints and are always interchangeable. Neither is deprecated or preferred by the language. Style guides or linters may recommend one over the other, but the spec treats them as semantically equivalent.

```metel
// Inline form
fun largest<T: Comparable>(a: T, b: T) -> T { ... }

// where form
fun largest<T>(a: T, b: T) -> T where T: Comparable { ... }
```

This reverses RFC-0002 decision #2, which restricted multiple bounds to `where`-only. Both forms now support any number of bounds.

### 2. Multiple bounds — `+` separator

Multiple bounds on a single type parameter are expressed with `+` in both inline and `where` forms:

```metel
// Inline multi-bound
fun display_and_clone<T: Display + Clone>(x: T) -> T { ... }

// where multi-bound
fun display_and_clone<T>(x: T) -> T
    where T: Display + Clone { ... }

// Multiple type params, multiple bounds
fun convert<T, U>(x: T, y: U) -> T
    where T: Display + Clone,
          U: Iterable<T> { ... }
```

`+` was chosen over `&` to avoid visual tension with the address-of operator (`&x`, `&mut x`) introduced in RFC-0001. `+` in a type constraint position is distinct from arithmetic because it appears after `:` inside a type parameter context.

### 3. Anonymous type parameters — open question

See **Open Questions §1**. The decision on syntax for anonymous (unnamed) type parameters is deferred. Explicit named type parameters (`<T: Aspect>`) are fully specified; anonymous forms are not yet available.

### 4. Associated type constraints — Swift-style primary associated types

Associated types are constrained by passing the concrete type as a type argument, matching the existing `Perhaps<T>` and `Result<T, E>` conventions:

```metel
fun process<T: Iterable<String>>(iter: T) { ... }
```

This is equivalent to Rust's `T: Iterator<Item = String>` but uses the same syntax as any other generic type instantiation. No special named-argument form is introduced.

### 5. `Self` in aspect definitions

`Self` inside an aspect definition refers to the concrete implementing type. Call sites use the bare aspect name with no type parameter repetition:

```metel
aspect Comparable {
    fun compare(self, other: Self) -> Int;
}

fun largest<T: Comparable>(a: T, b: T) -> T { ... }  // not T: Comparable<T>
```

### 6. Constraint aliases — `aspect` and `type`

Two alias forms are introduced for naming reusable bound combinations.

#### `aspect` aliases

An `aspect` alias names a bundle of aspect bounds. It can be used anywhere a single aspect bound appears, including inline and `where` positions:

```metel
aspect Sortable = Comparable + Display + Clone

fun sort<T: Sortable>(arr: T[]) -> T[] { ... }

fun display_sorted<T>(items: T[])
    where T: Sortable { ... }
```

The `+` separator is used in the alias definition — this is the only place `+` is mandatory (there is no `where`-style alternative for alias bodies).

`aspect` aliases are purely additive at the call site: `T: Sortable` is exactly equivalent to `T: Comparable + Display + Clone`. An `aspect` alias does not introduce a new aspect — it cannot have default method implementations or be `impl`-ed directly.

#### `type` aliases

`type` aliases name concrete types, generic instantiations, and function types. They are not bound-specific but are included here for completeness because they interact with the alias story:

```metel
type StringList = String[]
type Callback<T> = fun(T) -> Bool
type IntResult = Result<Int, String>
```

A `type` alias cannot be used as an aspect bound — it aliases a concrete type, not a constraint. The two alias forms are intentionally distinct:

| Form | Purpose | Usable as bound? |
|---|---|---|
| `aspect Foo = A + B` | Bundles aspect bounds | Yes |
| `type Foo = Bar<T>` | Names a type | No |

---

## Alternatives Considered

### `&` as the multi-bound separator

`T: Display & Clone` has precedent in TypeScript, Swift, and Scala. Rejected because `&` is the address-of operator in Metel (RFC-0001), and while the overlap is syntactically unambiguous, the visual tension was judged unnecessary when `+` is available and equally readable in a constraint context.

### `where`-only for multiple bounds (RFC-0002 decision #2)

Kotlin bans inline multi-bound — all multi-bound cases go in `where`. Rejected in this RFC: there is no compelling reason to force a `where` clause for a two-bound constraint on a single parameter. Treating both forms as equivalent removes an arbitrary restriction without introducing ambiguity.

### Comma-separated bounds in `where` (`where T: Display, T: Clone`)

Rejected in favour of `+`. Repeating the type parameter name per bound is verbose and inconsistent with the inline form. `+` allows `where T: Display + Clone` which mirrors the inline form exactly.

---

## Open Questions

### 1. Anonymous type parameter syntax

Rust uses `impl Aspect` at parameter position. In Metel, `impl` is already a keyword with two established uses: `impl Type { ... }` (inherent methods) and `impl Aspect for Type` (aspect implementation). Adding a third meaning — "anonymous type parameter implementing this aspect" — is a potential source of confusion.

**Option A: `impl Aspect` (keep Rust's form)**  
Accept the overloading. The three uses are syntactically distinguishable by context: `impl` followed by a type name and `{` is a block; `impl` followed by an aspect name and `for` is an implementation; `impl` followed by an aspect name in a parameter type position is an anonymous bound.

```metel
fun print_all(items: impl Display + Clone) { ... }
```

**Option B: `some Aspect` (Swift's keyword)**  
Introduce `some` as a keyword meaning "some type that implements this aspect." Reads naturally ("some Comparable"), is visually distinct from `impl`, and avoids overloading. Cost: a new keyword.

```metel
fun print_all(items: some Display + Clone) { ... }
```

**Option C: `any Aspect`**  
`any` reads as "any type implementing this aspect." Consistent with how programmers reason about generics ("I want any type that is Displayable"). Cost: a new keyword.

```metel
fun print_all(items: any Display + Clone) { ... }
```

**Option D: Require explicit named type parameters**  
Drop the anonymous form entirely. All generic parameters must be named (`<T: Aspect>`). This is the simplest option and removes a second syntax for the same concept. The ergonomic cost is minor for most functions.

```metel
fun print_all<T: Display + Clone>(items: T) { ... }  // no anonymous form
```

### 2. Async/concurrency bounds

If Metel gains async or threading support, the aspect system will need marker aspects (`Send`, `Sync`-like) and the pointer RFC will need corresponding bounds. Deferred until the concurrency model is designed.

---

## Timing Recommendation

This RFC should be accepted and incorporated into the spec before aspect bounds are implemented in the interpreter (v0.4.0 work). The anonymous type parameter question (§1) can be deferred to a follow-up RFC if needed — the named-parameter form is sufficient to unblock implementation.

---

## References

- Supersedes: `docs/internal/rfcs/rfc-0002-aspect-bound-syntax.md`
- RFC-0001: `docs/internal/rfcs/rfc-0001-pointer-syntax.md` (`&` operator — tension with `&` separator)
- Language spec: `docs/public/spec/declarations.md#aspects`, `docs/public/spec/types.md#generics`
- v0.4.0 issues: aspects and method dispatch

---

## Decision

*Pending.*
