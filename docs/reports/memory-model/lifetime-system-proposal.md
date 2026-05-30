# Lifetime System Proposal — Design Exploration

**Date:** 2026-05-27
**Status:** Exploratory — not normative
**Related reports:** `docs/reports/memory-model-programs.md`
**Related RFCs:** RFC-0028, RFC-0025, RFC-0024 (superseded), RFC-0003, RFC-0006

---

## Purpose

This report proposes adding a complete lifetime system to Metel's memory model. It defines what "lifetime" means against the backdrop of a language that already has three memory management mechanisms — RC heap, region allocation, and linear types — rather than the single ownership-based model that lifetime systems are typically designed around.

The report is structured in four parts:

1. **The lifetime system** — syntax, what it tracks, elision rules
2. **What it enables** — program examples of the gaps it closes relative to the current model
3. **What remains** — residual limitations and persistent differences from Rust even with a full lifetime system
4. **Interaction proposals** — concrete designs for the three critical intersections: lifetimes with regions, lifetimes with linear types, and lifetimes with the RC heap

This is design material, not a spec change. It extends the analysis from `docs/reports/memory-model-programs.md` Parts 5–6.

---

## Part 1 — The Lifetime System

### 1.1 What lifetimes track

A lifetime names a *live range* — the span of code during which a borrowed reference is guaranteed to remain valid. In a stack-first language, this is straightforward: a reference to a local variable is valid until the variable's frame is popped. In Metel the picture is more complex because the language has three memory systems with three different validity models:

| Memory system | Value validity governed by |
|---|---|
| RC heap (`*T`, `*mut T`) | Reference count — valid while any `*T` handle exists |
| Region scope (`~T`) | Scope boundary — valid until `Region::scope` returns |
| Linear bindings (`!T`) | Linear checker — valid until consumed exactly once |

Lifetime annotations apply to **borrowed read references (`@T`)** and **region-internal pointers (`~T`)**. They do not apply to RC pointers (`*T`) or unique pointers (`unique *T`) — those types already have their own validity guarantees through reference counting and linearity respectively. This is a deliberate narrowing of scope relative to Rust, and its implications are explored in Part 3.

### 1.2 Syntax

```
lifetime          ::= '\'' identifier            -- 'a, 'r, 'scope, 'static
read-ref-type     ::= '@' lifetime T             -- @'a T: read reference valid for 'a
region-ptr-type   ::= '*' lifetime T             -- *'r T: region-internal pointer
type-params       ::= '<' (lifetime | type) '>'  -- Foo<'a, T>
fun-lifetime      ::= 'fun' '<' lifetime, ... '>' '(' ... ')' -> ...
```

The existing `@T` without a lifetime annotation is shorthand for `@'_ T` — an anonymous lifetime that the compiler fills in via elision. The elision rules are:

1. **Single-input rule:** if a function has exactly one `@T` parameter and returns `@T`, the return lifetime equals the input lifetime.
2. **Self-like receiver rule:** if one `@T` parameter is the receiver (self-position), output lifetime equals that parameter's lifetime.
3. **Explicit otherwise:** when two or more `@T` inputs are present and the return borrows from one of them, the lifetime must be written explicitly.

`'static` is the only predefined lifetime — it means valid for the entire program duration. Values that are never freed (program-level globals, values leaked intentionally) satisfy `'static`.

### 1.3 Lifetime bounds and ordering

Lifetimes can be ordered with outlives constraints: `'a: 'b` means lifetime `'a` lives at least as long as `'b`. The compiler infers most of these; they surface in signatures only when a returned reference's validity depends on two input lifetimes with an ordering relationship.

```metel
// 'a: 'b — the container lives at least as long as the key
fun lookup<'a, 'b, K, V>(map: @'a Map<K, V>, key: @'b K) -> Perhaps<@'a V> where 'a: 'b {
    ...
}
```

Lifetime parameters on structs make the struct's validity dependent on the referenced lifetime:

```metel
struct Iter<'a, T> {
    data:  @'a T[],
    index: Int,
}
// An Iter<'a, T> cannot outlive 'a — enforced at all use sites
```

---

## Part 2 — What It Enables

### 2.1 Zero-copy borrowed views

**Current state.** Functions that take `@T` cannot return `@T` — the read reference is expression-scoped and cannot escape the call. Any function that needs to return a view into its input must return an owned copy.

**With lifetimes.** The return type carries the input's lifetime, making the zero-copy view pattern expressible at the function boundary.

```metel
// Single-input elision: lifetime is inferred — no annotation required
fun first_word(input: @String) -> @String {
    match string_find(input, " ") {
        nope                       => input,
        Perhaps::Some { value: i } => string_view(input, 0, i),
    }
}

// Two inputs: explicit annotation required to resolve which one is returned
fun longest<'a>(x: @'a String, y: @'a String) -> @'a String {
    if string_len(x) >= string_len(y) { x } else { y }
}

// Caller: no allocation
let sentence = "hello world";
let word = first_word(@sentence);   // word is @'scope String — borrows from sentence
println(word);
// sentence goes out of scope here; word is already dead — lifetime enforced statically
```

The same pattern applies to any borrowed projection: returning a field reference, an element of an array, a slice into a buffer.

```metel
fun name_of<'a>(person: @'a Person) -> @'a String { @person.name }
fun head<'a, T>(arr: @'a T[]) -> Perhaps<@'a T> {
    if array_len(arr) == 0 { nope }
    else { Perhaps::Some { value: @arr[0] } }
}
```

**What this enables.** Lexers, parsers, and string-processing pipelines that today must allocate a new `String` per token can work on views into the original source buffer. The allocation model becomes opt-in rather than mandatory.

---

### 2.2 Borrowed-reference structs

**Current state.** `@T` cannot be stored in a struct field. Structs that logically hold a reference to external data must instead hold an owned copy, or use a raw pointer (`*T`) restricted to a region scope.

**With lifetimes.** A struct parameterized by a lifetime can hold `@'a T` fields. The lifetime parameter prevents the struct from outliving the referenced value.

```metel
struct Parser<'a> {
    source: @'a String,
    pos:    Int,
}

fun make_parser<'a>(src: @'a String) -> Parser<'a> {
    Parser { source: src, pos: 0 }
}

fun peek<'a>(p: @Parser<'a>) -> Perhaps<Char> {
    if p.pos >= string_len(p.source) { nope }
    else { Perhaps::Some { value: string_char_at(p.source, p.pos) } }
}

fun advance<'a>(p: Parser<'a>) -> Parser<'a> {
    Parser { source: p.source, pos: p.pos + 1 }
}

fun main() {
    let src = "fun main() {}";
    let p = make_parser(@src);     // Parser<'scope>: borrows src
    let p = advance(p);
    match peek(@p) {
        nope                        => println("empty"),
        Perhaps::Some { value: c }  => println(char_to_string(c)),
    }
    // p is dropped here; src is still live — lifetime is sound
}
```

The key rule: `Parser<'a>` is not `Send` (it contains `@'a String` which borrows from somewhere), so it cannot cross fiber boundaries or escape the scope of `'a`. This is enforced by the lifetime constraint, not by a separate `Send` check.

---

### 2.3 Reference-yielding iterators

**Current state.** The `for` loop yields owned values. Without storable `@T`, there is no way to build an iterator type that yields read references into a collection.

**With lifetimes.** An iterator struct can hold a `@'a T[]` and yield `@'a T` elements — references into the original array with the same lifetime.

```metel
struct ArrayIter<'a, T> {
    data:  @'a T[],
    index: Int,
}

fun array_iter<'a, T>(arr: @'a T[]) -> ArrayIter<'a, T> {
    ArrayIter { data: arr, index: 0 }
}

fun iter_next<'a, T>(it: ArrayIter<'a, T>) -> (Perhaps<@'a T>, ArrayIter<'a, T>) {
    if it.index >= array_len(it.data) {
        (nope, it)
    } else {
        let elem = @it.data[it.index];
        (Perhaps::Some { value: elem }, ArrayIter { data: it.data, index: it.index + 1 })
    }
}

fun main() {
    let names: String[] = ["alpha", "beta", "gamma"];
    mut it = array_iter(@names);

    loop {
        let (result, next_it) = iter_next(it);
        it = next_it;
        match result {
            nope                       => break,
            Perhaps::Some { value: s } => println(s),   // s is @'scope String — no copy
        }
    }
}
```

The lifetime `'a` on `ArrayIter<'a, T>` ensures the iterator cannot outlive the array it borrows from. `println(s)` receives a read reference into `names` — no allocation.

---

### 2.4 More precise region scope exit

**Current state.** `Region::scope` uses `Send` as the return type constraint. Since `*T` is not `Send`, any type that directly or transitively contains a raw pointer — including heap-backed `unique *T` — is rejected at scope exit, even when the pointer points to RC-heap memory that will not be freed when the region exits.

**With lifetimes.** The region scope introduces a named lifetime `'r` for the scope boundary. Region-internal pointers have type `*'r T` — a pointer tied to lifetime `'r`. The scope exit constraint becomes `RegionFree<'r>` — "contains no `*'r T` for the current `'r`" — rather than the broader "contains no `*T` at all."

```metel
// After the change: unique *T from the RC heap is allowed to escape

fun alloc_node() -> unique *Node {
    Box::alloc(Node { id: 0, label: "root", edges: [] })   // RC heap — not region-internal
}

let result: unique *Node = Region::scope(fun() {
    // Inside: *'r GraphNode — region-internal, not RegionFree
    let scratch: (*'r GraphNode)[] = build_graph(...);

    // alloc_node() returns unique *Node — no 'r tag — RegionFree, allowed to escape
    alloc_node()
    // scratch is freed; result is not
});
```

The distinction between `*'r T` and `*T` (untagged, heap-backed) is the mechanism. `*'r T` is the region-internal type that the compiler produces for allocations inside `Region::scope`. `*T` outside a region scope is always heap-backed and implicitly `'static`-tagged.

---

### 2.5 Lifetime-parameterized closures

**Current state.** Closures cannot capture by borrow. Capture is by copy (for `Send` types) or by linear move (for `!T`). A closure that wants read-only access to an external value without owning it must receive that value as an explicit argument on every call.

**With lifetimes.** A closure type can be parameterized by a lifetime, expressing "this closure is valid for `'a` and its captures borrow from `'a`." The compiler infers the lifetime from the captured bindings.

```metel
// lookup borrows 'table' for its entire lifetime — no copy, no move
let table: Map<String, Int> = build_lookup_table();
let lookup: fun(@String) -> Perhaps<Int> = fun<'a>(key: @'a String) {
    map_get(@table, key)
    // captures @table with lifetime 'scope — table must outlive the closure
};

let a = lookup(@"alpha");
let b = lookup(@"beta");
// table still live here — closure's borrow is sound
```

The closure type `fun(@String) -> Perhaps<Int>` carries an implicit lifetime for the capture. The compiler ensures `table` cannot be moved or consumed while `lookup` is live.

---

## Part 3 — What Remains Different from Rust

A complete lifetime system closes most of the expressiveness gaps from `memory-model-programs.md` Part 5. But three structural differences from Rust persist regardless.

### 3.1 No `@mut T` — mutation is still consume-and-return

Rust's `&mut T` is an *exclusive* mutable reference: the borrow checker proves that no other reference (`&T` or `&mut T`) exists to the same value during the mutable borrow. This exclusivity proof is what makes safe in-place mutation possible without data races.

Metel's RFC-0028 explicitly defers `@mut T`. Without it, lifetimes alone do not enable in-place mutation through references — they only make read references storable and returnable. Patterns that require `&mut T` in Rust (mutable iterators, in-place sorting, mutation through a borrowed cursor) still require consume-and-return in Metel.

```metel
// Desired: mutate through a borrowed cursor without consuming it
fun increment<'a>(counter: @'a mut Int) {   // @mut T — NOT YET VALID
    *counter += 1;
}

// Actual: consume-and-return
fun increment(counter: Int) -> Int { counter + 1 }
let counter = increment(counter);
```

Adding `@mut T` after lifetimes are in place would require an exclusivity checker — a borrow checker in all but name. If `@mut T` is ever added, it should be designed as an extension of the lifetime system, not a separate mechanism.

### 3.2 Linear types and lifetimes are two separate ownership mechanisms

Rust has one mechanism for enforcing single ownership: move semantics with lifetime-tracked borrows. Metel with lifetimes would have two:

- **Linear types (`!T`):** explicitly opt-in, enforced by the `LinearEnv` pass, ownership tracked by the "consumed / not consumed" invariant
- **Lifetimes (`@'a T`):** apply to borrowed views, enforced by the lifetime checker, ownership tracked by "valid for `'a` / expired"

For non-linear values, lifetimes are the only mechanism. For linear values, both mechanisms apply simultaneously: the linear checker enforces exactly-once consumption, and the lifetime checker enforces that `@'a` borrows don't outlive the owned value.

The result is expressive but additive in complexity. A programmer working with a linear value that is also borrowed must reason about two independent invariants. The interaction between the two is defined in §4.2.

### 3.3 RC heap and lifetimes: a constrained relationship

In Rust, references always borrow from a stack-owned value with a definite scope. The borrowed value has a fixed stack frame that determines when it is freed.

In Metel, the default allocation is the RC heap. An RC value has no fixed stack frame — it lives as long as any `*T` handle exists. A lifetime attached to a `@T` borrow from an RC value expresses "this borrow is valid for `'a`", but the *underlying* condition is that the `*T` handle used to produce the borrow is live for `'a`. If a different `*T` clone to the same value drops during `'a`, the RC value is still live — the reference count ensures it. But if the `*T` handle used to produce the borrow is itself dropped, the borrow expires.

This creates an asymmetry: with RC-heap values, the lifetime of a borrow is bounded by the scope of the specific handle used to produce it, not by any "owner" in the Rust sense. This is sound but weaker than Rust's model — it is harder to express "this reference is valid as long as anyone holds the value" because "anyone" is not a single named scope.

Furthermore, `*mut T` allows aliased mutation. A `@'a T` borrow produced from `*T` can be invalidated by a concurrent write through a `*mut T` alias to the same object. Without `@mut T` (§3.1), the lifetime system cannot enforce exclusivity over RC values. The safe rule is: `@'a T` can only be formed from `*T` (immutable pointer), never from `*mut T`. The programmer must downgrade `*mut T` to `*T` before borrowing, and ensure no `*mut T` write occurs during `'a`.

### 3.4 Variance rules

Lifetime-parameterized types require variance: whether a `Foo<'long>` can be used where `Foo<'short>` is expected (covariance), or vice versa (contravariance), or neither (invariance). Rust resolves variance automatically from how the lifetime appears in the type definition (return position = covariant, argument position = contravariant, both = invariant). Metel would need the same rules, which add compiler complexity and produce error messages that are notoriously difficult for newcomers to interpret.

### 3.5 Higher-ranked lifetimes

Some Rust patterns require lifetimes that are universally quantified over all possible lifetimes — `for<'a> Fn(&'a T)`, meaning "a closure that works for any lifetime `'a`." These arise in callbacks, trait objects, and function pointers that accept borrowed arguments. Metel's existing function type syntax has no equivalent quantification. This is a gap that would only become visible once lifetime-parameterized function types are in use and callers need to pass callbacks.

---

## Part 4 — Interaction Proposals

### 4.1 Lifetimes × Regions

**The core correspondence.** A `Region::scope` call naturally introduces a lifetime: the duration of the scope. All allocations made inside the scope are valid for exactly that lifetime and no longer. The lifetime system makes this correspondence explicit and precise.

**Proposal.** `Region::scope` introduces an implicit lifetime parameter `'r` bound to the scope boundary. The compiler tags all allocations made inside the scope with `'r`, producing type `*'r T` rather than the unqualified `*T`. Outside the scope, `*'r T` is unconstructible — `'r` has ended.

```metel
// Conceptual desugaring of Region::scope
fun scope<'r, T: RegionFree<'r>>(f: fun<'r>() -> T) -> T

// Inside 'f', the compiler produces *'r T for region allocations:
let node: *'r GraphNode = region_alloc(GraphNode { ... });  // tagged *'r T, not *T
```

The scope exit constraint changes from `Send` (contains no `*T`) to `RegionFree<'r>` (contains no `*'r T` for the current `'r`). Heap-backed `*T`, `unique *T`, `String`, `Int`, and all value types satisfy `RegionFree<'r>` regardless of `r` — they carry no `'r` tag. Only `*'r T` and types that contain `*'r T` fail the check.

**`RegionFree` as an aspect bound.** `RegionFree<'r>` is a marker aspect auto-derived for all types that do not contain `*'r T` fields. The derivation rules mirror `Send`:

```metel
// Auto-derived: T: RegionFree<'r> if all fields are RegionFree<'r>
// Explicit negative: *'r T is never RegionFree<'r>
// Explicit positive: Int, String, Bool, *T (heap-backed) are always RegionFree<'r>
```

**Named regions.** In simple programs, one region scope is active at a time, so `'r` is unambiguous. For nested scopes, each `Region::scope` call introduces a distinct lifetime:

```metel
Region::scope(fun() {                      // introduces 'r1
    let outer: *'r1 Node = region_alloc(...);
    Region::scope(fun() {                  // introduces 'r2
        let inner: *'r2 Node = region_alloc(...);
        // *'r2 Node: RegionFree<'r2>? No. RegionFree<'r1>? Yes (not tagged 'r1)
        // Can return *'r1 Node from inner scope? No — *'r1 is not RegionFree<'r2>
        // Wait: *'r1 Node is not tagged 'r2, so it IS RegionFree<'r2>
        // This correctly models: a pointer into the outer region is safe to return
        // from the inner scope (the outer region is still live)
        outer
    });
    // inner region freed; outer still live
})
```

This correctly captures the outlives relationship: a pointer into `'r1` is valid to return from `'r2` because `'r1: 'r2` (outer region outlives inner). The lifetime system makes this provable rather than relying on programmer discipline.

---

### 4.2 Lifetimes × Linear Types

**The relationship.** A linear value (`!T`) is owned exclusively by one binding at a time. A lifetime system adds the ability to *borrow* from a linear value — to hold `@'a T` read references that temporarily share access without consuming. The linear checker and the lifetime checker must agree on the state of the binding during the borrow.

**Proposal: borrow state in the linear environment.** Extend the `LinearEnv` with a third state alongside `Unconsumed` and `Consumed`:

```
LinearEnv state for linear binding x:
  Unconsumed              — live, no active borrows
  Borrowed('a)            — live, one or more @'a borrows active
  Consumed(location)      — consumed, cannot be used
```

The transition rules:

- `@'a x` where `x: !T` in `Unconsumed` state → transitions `x` to `Borrowed('a)`; produces `@'a T`
- Attempting to consume `x` (pass to a function, destructure, `drop`) while `x` is in `Borrowed('a)` → compile error: "cannot consume `x` — it is borrowed for `'a`"
- When lifetime `'a` ends (its scope exits) → all bindings in `Borrowed('a)` transition back to `Unconsumed`
- Attempting to form a second `@'b x` while `x` is in `Borrowed('a)` with `'b: 'a` → allowed (nested borrows are fine if `'b` does not outlive `'a`); the binding stays in `Borrowed` state

```metel
let conn: !Connection = Connection::new(fd);

// Borrow for inspection — conn transitions to Borrowed('scope)
let port = connection_port(@conn);
let addr = connection_addr(@conn);

// conn is Borrowed — cannot consume here
// conn.close();  ← COMPILE ERROR: conn is borrowed

// All borrows (@conn above) expired at their expression scope
// conn transitions back to Unconsumed
conn.close();   // OK — consuming conn here
```

**Storable borrows of linear values.** With lifetimes, a `@'a T` borrow of a linear value can be stored in a struct or returned from a function, as long as the struct or return type carries `'a` and the borrow does not outlive the linear binding.

```metel
struct ConnView<'a> {
    conn: @'a Connection,
}

fun view_conn<'a>(c: @'a Connection) -> ConnView<'a> {
    ConnView { conn: c }   // stores the borrow — valid for 'a
}

let conn: !Connection = Connection::new(fd);
let view = view_conn(@conn);   // conn is Borrowed('scope)
let port = connection_port(view.conn);
// view goes out of scope here → 'scope for view ends → conn returns to Unconsumed
conn.close();  // OK
```

**Move vs. borrow at function calls.** A function that takes `!T` consumes the value (linear move). A function that takes `@'a T` borrows it. The caller decides which to do at the call site:

```metel
fun inspect(c: @Connection) -> String { ... }   // borrows
fun consume(c: !Connection) { c.close(); }       // consumes

let conn: !Connection = Connection::new(fd);
let info = inspect(@conn);   // borrow — conn stays Unconsumed
consume(conn);               // move — conn transitions to Consumed
```

**The invariant the two systems must agree on.** The lifetime checker must know the scope of every linear binding so it can verify that `@'a T` borrows do not outlive it. The linear checker must know when borrows are active so it can block consumption. The handshake is: lifetime `'a` is bounded above by the scope in which the linear binding is live; the linear checker blocks consumption while `'a` is active.

---

### 4.3 Lifetimes × RC Heap

**The fundamental difference.** In Rust, borrowed references always borrow from a stack-owned value with a definite scope. The "owner" is a named binding, and the borrow expires when the owner's scope ends. In Metel, RC-heap values have no single owner — they are valid as long as any `*T` handle exists, which may be many handles across many scopes.

This makes RC values and lifetimes an awkward combination: there is no "owner scope" to use as the lifetime anchor. The proposal below defines how the two interact in a way that is sound, if more restricted than Rust.

**Proposal: lifetime of a borrow from `*T` is the scope of the handle used to produce it.**

Dereferencing `ptr: *T` to produce `@'a T` is valid for `'a = lifetime_of(ptr)` — the scope in which `ptr` is live as a binding. This is always sound: the RC value is valid as long as any handle exists, and `ptr` is one such handle. As long as `ptr` is in scope, the refcount is at least 1, so the value is not freed.

```metel
let ptr: *Node = get_node_from_somewhere();
let name: @'scope String = @(*ptr).name;   // valid for 'scope: the scope of ptr
println(name);
// ptr goes out of scope here; RC decrements; name is already expired — sound
```

**`@T` from `*mut T` is forbidden.** `*mut T` allows aliased mutation through any clone. A `@'a T` borrow from `*mut T` could be invalidated by a write through another `*mut T` handle during `'a`. Without `@mut T` and an exclusivity checker, this cannot be prevented statically. The rule:

```metel
let mptr: *mut Node = &mut node;
let name = @(*mptr).name;   // TYPE ERROR: cannot form @T from *mut T
                             // downgrade to *T first, or use @mut T (not yet available)

// Correct: downgrade first
let rptr: *T = mptr;
let name: @'scope String = @(*rptr).name;
// WARNING: mptr still exists — a write through mptr during 'scope invalidates name
// The compiler cannot prevent this without exclusivity tracking
```

The warning above points to the residual unsoundness: downgrading `*mut T` to `*T` before borrowing prevents the type error but does not prevent aliased writes through the original `*mut T`. This gap cannot be closed without `@mut T` or a mechanism that suspends all `*mut T` aliases for the duration of the borrow. This is noted as an open question (§5.3).

**Implication: RC borrows are expression-scoped in practice.** Because the lifetime of a borrow from `*T` is bounded by the scope of the handle, and because RC handles are frequently short-lived (created for a lookup, not held long-term), most RC borrows in practice will be expression-scoped — equivalent to the current `@T` semantics. The lifetime system gives them a name, but does not change their reach. The primary value of the lifetime system is for linear values (§4.2) and region-internal pointers (§4.1), not for RC-heap values.

---

## Part 5 — Open Design Questions

### Q1 — `@mut T`: when and how

The biggest gap left by a lifetime system without `@mut T` is in-place mutation through references. The patterns that require it — mutable iterators, in-place sorting, mutation through a borrowed struct cursor — all fall back to consume-and-return.

Adding `@mut T` requires an *exclusivity* rule: at most one `@mut T` to a value at a time, and no `@T` during the `@mut T` borrow. This is a borrow-checker-level invariant. The question is not whether it is possible — it is possible, and the lifetime system provides the infrastructure — but whether the complexity cost is acceptable given Metel's design goal of being simpler than Rust for most programs.

A possible middle ground: introduce `@mut T` as an `unsafe`-only feature (RFC-0026 unsafe blocks), available when the programmer asserts correctness manually. The safe subset of the language keeps consume-and-return; `unsafe` code can use `@mut T` for performance-critical mutation.

### Q2 — Interaction between `Borrowed` linear state and the loop constraint

RFC-0024's loop constraint forbids consuming a linear value created outside a loop body. The borrow state extension (§4.2) adds a new constraint: a linear value in `Borrowed` state also cannot be consumed. The interaction:

Can a linear value transition from `Borrowed` to `Unconsumed` inside a loop — i.e. can a borrow be created and expire within each loop iteration? If the borrow is created from a binding outside the loop, the borrow's lifetime would span multiple iterations, conflicting with the loop constraint in a novel way. This needs explicit rules.

### Q3 — Aliased mutation through `*mut T` during an RC borrow

As noted in §4.3: downgrading `*mut T` to `*T` before borrowing is type-safe but does not prevent a concurrent write through the original `*mut T`. This is a soundness gap for RC values specifically. Options:

- **Forbid `@T` from any value accessible through a live `*mut T` alias** — overly conservative; hard to check.
- **Introduce an exclusivity discipline on `*mut T`:** consuming a `*mut T` into `*T` suspends aliased mutation for the duration of a borrow. This is closer to Rust's `RefCell::borrow()` / `borrow_mut()` pattern — runtime-checked or statically tracked.
- **Accept the unsoundness and document it**, treating `@T` from `*T` borrows as programmer-asserted safety, subject to the same caveats as other aliasing-adjacent patterns.

### Q4 — Lifetime syntax and annotation burden

The design goal of the original `@T` was to avoid lifetime annotations entirely. A full lifetime system reintroduces them. The elision rules (§1.2) reduce the annotation burden for common cases, but the presence of lifetime parameters in struct definitions and function signatures changes the language's surface complexity profile.

Whether this cost is acceptable depends on whether lifetime-parameterized code is expected to be the norm (likely for library authors) or the exception (likely for application code). A possible design position: lifetime annotations are only required for code that crosses ownership boundaries — functions that return borrowed views, structs that hold references. Application code that uses such libraries sees the annotations only in type error messages, not in the code it writes.

### Q5 — Higher-ranked lifetimes

Callbacks and function types that accept borrowed arguments require universally quantified lifetimes: "this function works for any lifetime `'a`." Without HRTBs, passing a `fun(@T) -> Bool` as a predicate to a higher-order function over a borrowed collection is not expressible in the general case. The scope of this problem depends on how common higher-order functions over borrowed data turn out to be in Metel idioms. It can be deferred until the base lifetime system is in use and concrete demand emerges.

---

## References

- `docs/reports/memory-model-programs.md` — memory model programs and limitations analysis (Parts 5–6)
- RFC-0028: Memory and Reference Model — `docs/internal/rfcs/rfc-0028-memory-and-reference-model.md`
- RFC-0025: Region Allocation — `docs/internal/rfcs/rfc-0025-region-allocation.md`
- RFC-0024: Linear Types (superseded) — `docs/internal/rfcs/rfc-0024-linear-types.md`
- RFC-0003: Concurrency Model — `docs/internal/rfcs/rfc-0003-concurrency-model.md`
- RFC cluster: Memory Model — `docs/internal/rfc-cluster-memory-model.md`
- Prior art: Rust reference and lifetime system; Cyclone region-based memory management; Linear Haskell (Bernardy et al. 2018)
