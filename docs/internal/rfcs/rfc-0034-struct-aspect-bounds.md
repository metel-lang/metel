---
id: rfc-0034
title: "Aspect Bounds on Struct and Enum Generic Parameters"
date: '2026-05-30'
status: draft
---

## Summary

Define the semantics of aspect bounds on struct and enum generic type parameters — what a bound means, when it is checked, how it propagates into impl blocks, and whether it is inherited by functions that receive the bounded type.

---

## Motivation

RFC-0002 accepted aspect bounds on function type parameters (`fun foo<T: Comparable>(x: T)`). Structs and enums also have generic params, and the grammar already permits bounds there (`struct Bag<T: Printable>`). However, the spec does not define what such a bound means or when it is enforced. Without a decision, the typechecker cannot implement it, and the spec is ambiguous.

---

## The Questions

### Q1 — When is the bound checked?

Three models are possible:

**Model A — Instantiation time.** The bound is checked when a value of the struct is constructed:

```metel
struct Bag<T: Printable> { item: T }

let b = Bag { item: 5 };    // ERROR: Int does not implement Printable
```

The bound gates construction. Any caller that constructs a `Bag` must satisfy `T: Printable`.

**Model B — Use-site only (propagation model).** The bound is a propagation hint. Any function that receives a `Bag<T>` automatically has `T: Printable` in scope without re-declaring it. Construction is not checked — the bound is not about preventing construction but about advertising that the struct's methods may call Printable methods on T.

```metel
struct Bag<T: Printable> { item: T }

fun print_bag(b: Bag<T>) {          // no explicit T: Printable needed
    b.item.print();                 // OK — T: Printable inherited from Bag
}
```

**Model C — Explicit re-declaration required (Rust model).** The bound on the struct is advisory; every function and impl block that wishes to use the constraint must re-declare it:

```metel
struct Bag<T: Printable> { item: T }

fun print_bag<T: Printable>(b: Bag<T>) {    // must repeat bound explicitly
    b.item.print();                          // OK
}

fun get_item<T>(b: Bag<T>) -> T {           // T not bounded here
    b.item.print();                          // ERROR: T not constrained
}
```

Rust uses Model C for structs but enforces the bound at construction in certain contexts.

---

### Q2 — How do impl blocks interact with struct bounds?

When `struct Bag<T: Printable>`, there are three options for how `impl` blocks work:

**Option i — Implicit inheritance.** An `impl Bag<T>` block automatically inherits `T: Printable` without re-declaration:

```metel
impl Bag<T> {                       // T: Printable inherited automatically
    fun show(self) {
        self.item.print();          // OK
    }
}
```

**Option ii — Explicit re-declaration required (Rust model).** The impl block must re-state the bound:

```metel
impl<T: Printable> Bag<T> {         // must repeat T: Printable
    fun show(self) {
        self.item.print();          // OK
    }
}
```

**Option iii — Impl blocks are unconstrained; methods opt-in via where.** The `impl` header is unconstrained; individual methods that need the bound declare it:

```metel
impl Bag<T> {
    fun show(self) where T: Printable {
        self.item.print();          // OK
    }
    fun size(self) -> Int { 1 }    // no bound needed
}
```

---

### Q3 — Interaction with generic functions

If `struct Bag<T: Printable>` and a function receives `Bag<T>`, does the caller need to re-declare `T: Printable`?

```metel
// Option A: inherited (no re-declaration)
fun foo(b: Bag<T>) { b.item.print(); }

// Option B: re-declaration required
fun foo<T: Printable>(b: Bag<T>) { b.item.print(); }

// Option C: doesn't matter — Bag<T> is always Bag<PrintableT> so T is always Printable
fun foo(b: Bag<T>) { ... }   // T is constrained by the type of b
```

---

### Q4 — Enum bounds

Does the same model apply to enums?

```metel
enum Opt<T: Comparable> { Some { value: T }, None }
```

Enums follow the same generic param mechanism as structs, so the decision for structs should apply uniformly unless there is a reason to differentiate.

---

### Q5 — Empty bounds as documentation only

Should it be legal to write a bound on a struct generic param purely as documentation — with no enforcement? This would let authors signal intent without the typechecker blocking construction:

```metel
struct Bag<T: Printable> { ... }    // advisory — not enforced at construction
```

This is Model B taken to an extreme. It degrades the bound to a comment-like annotation, which is likely confusing and should be rejected.

---

## Design Options

### Option 1 — Rust model (Model C + Option ii + re-declaration required)

Bounds on struct params are enforced at:
- Construction: `Bag { item: x }` requires T: Printable
- impl blocks: must re-declare bound — `impl<T: Printable> Bag<T>`
- Functions: must re-declare — `fun foo<T: Printable>(b: Bag<T>)`

**Pros:** Well-understood semantics. Explicit — every constraint is visible at its use site.  
**Cons:** Verbose. Bound on the struct type is somewhat redundant — every use site repeats it. Adds a new `impl<T: Bound> Type<T>` syntax to the grammar that does not currently exist.

---

### Option 2 — Propagation model (Model A + Option i + implicit inheritance)

Bound is checked at construction. All `impl` blocks and functions automatically inherit the bound from the struct definition.

```metel
struct Bag<T: Printable> { item: T }

// Construction gated:
let b = Bag { item: 5 };            // ERROR: Int not Printable

// impl block inherits bound automatically:
impl Bag<T> {
    fun show(self) { self.item.print(); }   // OK
}

// Functions inheriting from parameter types:
fun print_bag(b: Bag<T>) {
    b.item.print();                 // OK — T: Printable flows from Bag<T>
}
```

**Pros:** No repetition. Bound is stated once. The struct type itself carries the constraint into all usage sites.  
**Cons:** Requires the typechecker to propagate constraints from struct definitions into caller scopes — a more complex inference mechanism. May surprise users when a bound appears "for free" without explicit declaration.

---

### Option 3 — Construction-only (Method B from Q1 + unconstrained impl/functions)

Bound is checked only at construction. It does not propagate into impl blocks or functions. Methods that want to use the bound must declare it explicitly in a `where` clause on the method (Option iii above).

```metel
struct Bag<T: Printable> { item: T }

let b = Bag { item: 5 };            // ERROR: Int not Printable

impl Bag<T> {
    fun show(self) where T: Printable { self.item.print(); }
    fun size(self) -> Int { 1 }
}
```

**Pros:** Simple rule — bound gates construction, nothing else. No propagation complexity. Methods opt in explicitly.  
**Cons:** If every method needs the bound, every method re-declares it. Splits the definition (struct has `T: Printable`) from the usage (method `where T: Printable`).

---

## Open Questions

1. **Which model for construction checking?** Options 1 and 3 gate construction; Option 2 also gates construction but adds propagation.
2. **Should impl blocks re-declare bounds or inherit them?** This affects whether the grammar needs `impl<T: Bound> Type<T>` syntax.
3. **Should generic functions that receive a bounded struct inherit the bound, require re-declaration, or something else?**
4. **Do enum bounds follow the exact same rules as struct bounds?**
5. **Are bounds on struct params enforced even in generic contexts?** E.g., `fun make<T>(x: T) -> Bag<T>` — does the caller need `T: Printable`, or is this only checked when T is concretely resolved?

---

## Recommendation

TBD — pending discussion.

The simplest model consistent with Metel's existing design (spec-first, static dispatch, explicit bounds in RFC-0002) is **Option 3**: bounds on struct params gate construction, do not propagate, and methods that need the bound declare it explicitly in a `where` clause. This is the least novel mechanism — it composes cleanly with RFC-0002's where clause design and avoids adding implicit constraint propagation to the typechecker.

Option 2 (propagation) is more ergonomic but adds meaningful typechecker complexity and should only be chosen if the ergonomic benefit is judged worth it.

---

## References

- RFC-0002: `docs/internal/rfcs/rfc-0002-aspect-bound-syntax.md` — bounds on function type params
- Spec: `docs/public/spec/declarations.md#aspects`
- AST: `src/ast/mod.rs` — `GenericParam`, `StructDecl`, `EnumDecl`
