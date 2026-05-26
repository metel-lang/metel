# Memory Model Programs — Design Exploration

**Date:** 2026-05-26  
**Status:** Exploratory — not normative  
**Related RFCs:** RFC-0028, RFC-0025, RFC-0003  

---

## Purpose

This report explores what Moonlane programs look like once RFC-0028 (linear types, `@T` read references, `*T`/`*mut T`/`unique *T` pointers), RFC-0025 (region allocation), and RFC-0003 (fibers and channels) are implemented. It is design material, not a spec change. The goal is to surface idioms, validate that the RFC decisions compose well, and identify gaps.

**Notation used throughout:**
- `!T` — use-site linearity sigil (OQ-1, syntax unresolved; candidates: `!T`, `|T`, `linear T`)
- `Box::alloc(v)` / `Box::take(p)` — unique pointer allocation (OQ-2, syntax unresolved)
- `@(*p)` — read reference through a unique pointer (OQ-3, syntax unresolved)

Where open questions affect an example, this is noted inline.

---

## Part 1 — The Three Core Features in Combination

### 1.1 Region allocation

The natural design for `Region::scope` is **implicit allocation**: all heap allocations inside the scope callback go to the region's bump allocator automatically, with no explicit `r.create()` call. The scope itself is unsized and grows as needed.

```moonlane
let ast: Ast = Region::scope(fun() {
    let tokens = tokenise(source);   // allocated in the region
    let ast    = parse(tokens);      // allocated in the region
    ast                              // Ast is a Send value type — copied out on return
});
let ir = lower(ast);   // semantic step on owned data, outside the region
```

This is cleaner than RFC-0025's current `r.create(value)` proposal. The `r` parameter becomes unnecessary. The `Send` return bound remains the safety mechanism: you can only return a type that contains no region-internal pointers, so nothing can dangle after the region is freed.

**`lower()` is only necessary when the return type uses raw pointers.** If `Ast` is a pure value type (nested structs and arrays, no `*T` fields), it is automatically `Send` and can be returned directly via a deep copy on scope exit. The `lower()` call exists for semantic transformation, not for escaping the region — and it belongs outside the scope when both concerns are present.

**Region implementation dependency:** RFC-0025 depends on RFC-0028 (linear types) only in the sense that `Region` is declared as a `linear struct`. Once the `linear` keyword and the LinearEnv pass exist in the language, region allocation can be implemented independently in the runtime/standard library. The internal implementation of the linearity checker does not need to know anything special about `Region`.

### 1.2 Linear types and channels

Channel send (`ch <- value`) is a natural consumption point for a linear value. After a send, the binding is dead — the linearity checker sees it as consumed. This means ownership transfer through channels falls out of two orthogonal features composing naturally, with no extra annotation.

```moonlane
let conn = Connection::new(fd);
ch <- conn;   // conn consumed — cannot be used again
```

For the loop constraint (a linear value created before a loop cannot be consumed inside it), **recursion is the idiomatic solution**. Each recursive call shadows the binding with the new handle returned by the consume-and-return method:

```moonlane
// Correct: each call consumes 'file' and shadows it with the returned handle
fun stream(file: !FileHandle, out: Chan<!Frame>) {
    let (data, file) = file.read_line();
    if (data == "") { file.close(); out.close(); return; }
    out <- Frame { data: data };
    stream(file, out);   // 'file' here is the new handle
}
```

### 1.3 `*T` and `*mut T` are not Send

The `Send` constraint on `*T` and `*mut T` means pointer-based data structures cannot cross fiber boundaries. Code that builds graphs, trees with parent pointers, or doubly-linked lists using RC pointers must stay on a single fiber. Results are communicated to other fibers via channels using `Send` types (plain values, not pointers).

```moonlane
// spawn { bfs(graph_ptr, out) }  ← TYPE ERROR: *GraphNode is not Send
bfs(graph_ptr, out);   // BFS must stay on this fiber; String results flow via channel
```

This is a hard constraint, not a limitation to work around — it is the mechanism that prevents data races on pointer-based structures.

---

## Part 2 — Program Examples

### 2.1 Linear byte-frame pipeline

Three fibers: reader produces linear `Frame` values from a connection, processor filters and counts them using `@T` read references, writer sinks them to another connection.

```moonlane
// !T = use-site linearity sigil (OQ-1, syntax TBD)

linear struct Frame { data: String, seq: Int }

fun is_keepalive(f: @Frame) -> Bool { f.data == "PING" }
fun byte_count(f: @Frame) -> Int    { string_len(f.data) }

// Recursion handles consume-and-return across "iterations".
// A loop cannot consume a linear value created before its body.
fun read_frames(conn: !Conn, out: Chan<!Frame>, seq: Int) {
    let (data, conn) = conn.recv();
    if (data == "") { conn.close(); out.close(); return; }
    out <- Frame { data: data, seq: seq };   // frame consumed by send
    read_frames(conn, out, seq + 1);
}

fun process(input: Chan<!Frame>, output: Chan<!Frame>, stats: Chan<Int>) {
    mut total = 0;
    while let Perhaps::Some { value: f } = <- input {
        if (is_keepalive(@f)) {
            drop(f);   // @f read without consuming; f must still be explicitly consumed
            continue;
        }
        total += byte_count(@f);
        output <- f;   // f consumed by send
    }
    stats <- total;
    output.close();
}

fun write_frames(input: Chan<!Frame>, dest: !Conn) {
    match <- input {
        nope => { dest.close(); }
        Perhaps::Some { value: f } => {
            let Frame { data, seq: _ } = f;    // f consumed by destructure
            let dest = dest.send(data);         // consume-and-return
            write_frames(input, dest);
        }
    }
}

fun main() {
    let frame_ch: Chan<!Frame> = Chan::buffered(32);
    let out_ch:   Chan<!Frame> = Chan::buffered(32);
    let stats_ch: Chan<Int>    = Chan::new();

    spawn { read_frames(Conn::dial("source:9000"), frame_ch, 0); };
    spawn { process(frame_ch, out_ch, stats_ch); };
    write_frames(out_ch, Conn::dial("sink:9001"));

    let total = (<- stats_ch).yolo();
    println("bytes: " + int_to_string(total));
}
```

**What this demonstrates:**
- `@T` for reading a field without consuming (validation, measurement)
- `drop(value)` to explicitly satisfy the linearity checker on an early-exit path
- Channel send as the linear value consumption point
- Destructuring as consumption (`let Frame { data, seq: _ } = f`)
- Consume-and-return method chaining (`conn.recv()` → `(data, conn)`)
- Recursion as the loop-with-linear-value pattern

### 2.2 Graph traversal with `*T` and `*mut T`

An adjacency-list graph built with mutable RC pointers. BFS on a single fiber; results collected via a channel.

```moonlane
// *T and *mut T are not Send — the graph lives on one fiber only.

struct Node { id: Int, label: String, edges: (*mut Node)[] }

fun link(a: *mut Node, b: *mut Node) {
    array_push((*a).edges, b);
    array_push((*b).edges, a);
}

fun bfs(start: *Node, n: Int, out: Chan<String>) {
    mut visited: Bool[] = [];
    for (mut i = 0; i < n; i += 1) { array_push(visited, false); }

    let q: Chan<*Node> = Chan::buffered(n);
    q <- start;

    while let Perhaps::Some { value: node } = <- q {
        let id = (*node).id;
        if (visited[id]) { continue; }
        visited[id] = true;
        out <- (*node).label;   // String is Send — safe to cross fiber boundaries

        for (let e in (*node).edges) {
            let r: *Node = e;   // *mut Node coerces to *Node (downgrade to read-only)
            q <- r;
        }
    }
    out.close();
}

fun main() {
    mut a = Node { id: 0, label: "alpha", edges: [] };
    mut b = Node { id: 1, label: "beta",  edges: [] };
    mut c = Node { id: 2, label: "gamma", edges: [] };

    let pa: *mut Node = &mut a;
    let pb: *mut Node = &mut b;
    let pc: *mut Node = &mut c;
    link(pa, pb);
    link(pb, pc);
    link(pa, pc);

    let out: Chan<String> = Chan::buffered(8);

    // spawn { bfs(pa, 3, out) }  ← TYPE ERROR: *Node is not Send
    bfs(pa, 3, out);

    while let Perhaps::Some { value: label } = <- out {
        println(label);
    }
}
```

**What this demonstrates:**
- `&mut x` to produce `*mut T`
- `(*p).field` — explicit dereference required for field access (no auto-deref, OQ-6)
- `*mut T` → `*T` implicit downgrade (safe; upgrade never allowed)
- `*T` is not `Send` — the type system prevents spawning over a pointer graph
- Channels used for result collection even on a single fiber

### 2.3 Priority dispatcher with `unique *T` and `select`

A pending-job stack as a linear linked list using `unique *T`. A dispatcher fiber uses `select` to prefer high-priority input.

```moonlane
// Box::alloc / Box::take = unique pointer allocation (OQ-2, syntax TBD)

linear struct Job  { id: Int, data: String }
linear struct Node { job: !Job, next: Perhaps<unique *Node> }

fun push(top: Perhaps<unique *Node>, job: !Job) -> unique *Node {
    Box::alloc(Node { job: job, next: top })
}

fun pop(ptr: unique *Node) -> (!Job, Perhaps<unique *Node>) {
    let Node { job, next } = Box::take(ptr);
    (job, next)
}

fun drain(top: Perhaps<unique *Node>) {
    match top {
        nope => {}
        Perhaps::Some { value: ptr } => {
            let (job, rest) = pop(ptr);
            drop(job);
            drain(rest);
        }
    }
}

fun dispatch(hi: Chan<!Job>, lo: Chan<!Job>, out: Chan<!Job>) {
    mut pending: Perhaps<unique *Node> = nope;
    mut running = true;

    while running {
        // select prefers hi-priority; falls back to lo; non-blocking poll
        let got = select {
            j <- hi => j,
            j <- lo => j,
            else    => nope,
        };
        match got {
            nope                                             => { os_yield(); }
            Perhaps::Some { value: nope }                   => { running = false; }
            Perhaps::Some { value: Perhaps::Some { value: job } } => {
                pending = Perhaps::Some { value: push(pending, job) };
            }
        }
        // Flush one job to workers if available
        match pending {
            nope => {}
            Perhaps::Some { value: ptr } => {
                let (job, rest) = pop(ptr);
                out <- job;
                pending = rest;
            }
        }
    }
    drain(pending);
    out.close();
}

fun worker(jobs: Chan<!Job>, results: Chan<String>) {
    while let Perhaps::Some { value: job } = <- jobs {
        let Job { id, data } = job;
        results <- "done:" + int_to_string(id) + " " + data;
    }
}

fun main() {
    let hi_ch:     Chan<!Job>   = Chan::buffered(8);
    let lo_ch:     Chan<!Job>   = Chan::buffered(32);
    let worker_ch: Chan<!Job>   = Chan::buffered(16);
    let result_ch: Chan<String> = Chan::buffered(64);

    spawn { dispatch(hi_ch, lo_ch, worker_ch); };
    for (mut i = 0; i < 4; i += 1) {
        spawn { worker(worker_ch, result_ch); };
    }

    hi_ch <- Job { id: 0, data: "urgent" };
    lo_ch <- Job { id: 1, data: "batch" };
    hi_ch <- Job { id: 2, data: "urgent2" };
    hi_ch.close();
    lo_ch.close();

    while let Perhaps::Some { value: r } = <- result_ch { println(r); }
}
```

**What this demonstrates:**
- `unique *T` for heap-allocated linear nodes in a recursive structure
- `Box::alloc` / `Box::take` for creating and consuming unique pointer handles
- `drain` using recursion to free a linear list (loop constraint applies here too)
- `select` with `else` for non-blocking priority poll
- Linear jobs flowing from dispatcher to workers via channels

---

## Part 3 — The Two Memory Systems

### 3.1 Overview

Moonlane has two memory management systems that operate in parallel:

| | RC heap | Region scope |
|---|---|---|
| Allocation | `*T`, `*mut T` via `&` / `&mut` | Bump allocator, no RC overhead |
| Cycles | Leak — manual breaking required | Free — entire block freed atomically |
| Lifetime tracking | Reference count | Scope boundary |
| Cross-fiber safety | `Send` constraint on pointer types | `Send` return bound on scope result |
| Pointer rules | Conservative — `*T` not `Send`, no cycles | Relaxed inside, `Send` enforced on exit |
| Intended use | Long-lived shared state | Scratch work, temporary complex structures |

The RC heap is the default. It is ergonomic for most code: values live as long as someone holds a reference, and memory is reclaimed when the last reference drops.

The region is an opt-in scratch arena. Its defining property is that the **scope boundary is the lifetime guarantee** — not reference counts. When `Region::scope` returns, the entire backing block is freed in one operation regardless of what was built inside.

### 3.2 Regions as a pointer playground

The most consequential consequence of the region's lifetime model is that **pointer cycles are safe inside a region scope**. RC cycles outside the region cause leaks. Cycles inside the region are free: the scope exits, the backing block is freed, every pointer into it becomes invalid simultaneously. There is nothing to leak.

This means that code that is painful or impossible on the RC heap — graphs with bidirectional edges, trees with parent pointers, doubly-linked lists — is straightforward inside a region scope. You write the same `*T`/`*mut T` code, but without any need for weak pointers, manual cycle-breaking, or a separate GC.

The `Send` return bound enforces the only rule that matters: nothing pointing into the region can escape the scope. Outside the scope, the region is gone.

### 3.3 Example — graph analysis with the two systems working in parallel

The RC heap holds the input data and the final result. The region handles the scratch graph structure, including bidirectional edges that would cause RC cycles outside.

```moonlane
// ── RC heap: holds input and output ──────────────────────────────────────────

struct Edge { from: Int, to: Int }

struct Summary { components: Int, largest: Int }

// ── Region: scratch graph with free pointer cycles ────────────────────────────

struct GraphNode { id: Int, visited: Bool, edges: (*mut GraphNode)[] }

fun build_graph(n: Int, edges: Edge[]) -> (*mut GraphNode)[] {
    mut nodes: (*mut GraphNode)[] = [];
    for (mut i = 0; i < n; i += 1) {
        mut node = GraphNode { id: i, visited: false, edges: [] };
        array_push(nodes, &mut node);
    }
    for (let e in edges) {
        let a: *mut GraphNode = nodes[e.from];
        let b: *mut GraphNode = nodes[e.to];
        array_push((*a).edges, b);
        array_push((*b).edges, a);   // back-edge: RC cycle outside, free inside region
    }
    nodes
}

fun dfs(node: *mut GraphNode) -> Int {
    if ((*node).visited) { return 0; }
    (*node).visited = true;
    mut count = 1;
    for (let e in (*node).edges) { count += dfs(e); }
    count
}

// ── Entry point: two systems working together ─────────────────────────────────

fun analyze(n: Int, edges: Edge[]) -> Summary {
    Region::scope(fun() {
        // Inside: pointer-rich, cycle-safe, no RC overhead
        let nodes = build_graph(n, edges);

        mut components = 0;
        mut largest    = 0;
        for (let node in nodes) {
            if (!(*node).visited) {
                let size = dfs(node);
                components += 1;
                if (size > largest) { largest = size; }
            }
        }

        Summary { components: components, largest: largest }
        // Summary contains only Int — it is Send, allowed to escape.
        // All GraphNode allocations and edge pointers freed with the region.
    })
}

fun main() {
    // Long-lived data on the RC heap
    let edges: Edge[] = [
        Edge { from: 0, to: 1 },
        Edge { from: 1, to: 2 },
        Edge { from: 2, to: 0 },   // cycle: component {0,1,2}
        Edge { from: 3, to: 4 },   // separate component {3,4}
    ];

    let s = analyze(5, edges);
    println("components: " + int_to_string(s.components));   // 2
    println("largest:    " + int_to_string(s.largest));      // 3
}
```

### 3.4 Why `Region` is not `Send`

The relaxed pointer rules inside a region scope are only sound because the scope is single-fiber. `Region` itself is linear and not `Send` — it cannot be passed across a fiber boundary.

If two fibers could allocate into the same region simultaneously, they could build cycles and pointer structures concurrently without any synchronisation — a data race at the allocator level. Restricting the region to one fiber eliminates this class of problem entirely.

Per-request arenas, per-frame scratch allocators, and parser scratch spaces all fit the single-fiber model naturally. For multi-fiber scratch work, each fiber creates its own region.

### 3.5 The `Send` return bound as the only safety rule

Because the scope is single-fiber and its lifetime is bounded by the callback, the single constraint needed for safety is: **the return type of the scope callback must be `Send`**.

Since `*T` and `*mut T` are not `Send`, any type that directly or transitively contains a pointer to region memory cannot be returned. The compiler rejects the scope if the return type would allow a dangling pointer to escape.

Pure value types — structs with only `Int`, `Float`, `Bool`, `String`, and array fields — are automatically `Send` and can be returned freely. The region deep-copies them to the RC heap on scope exit.

This is the only rule the programmer needs to reason about. There are no lifetime annotations, no borrow checker, no explicit `unsafe`. The region scope is the boundary; `Send` is the exit condition.

---

## Part 4 — Design Questions Surfaced

These questions were raised while sketching the programs above. They are not resolved here; they are recorded as input for RFC updates.

### Q1 — Linear values and mutable rebinding in loops

The RFC prohibits consuming a linear value created before a loop body. This forces recursion for any "carry a linear handle through iterations" pattern (file reading, socket streaming). Whether `mut` bindings with explicit rebinding (`file = new_file`) should be treated specially — since the linearity invariant is maintained at each iteration boundary — is unresolved.

**Impact:** all streaming/iteration patterns with linear handles require tail recursion today.

### Q2 — `@T` and `spawn { }` capture

`@T` cannot be stored, so a `spawn { }` block cannot capture a read reference. To pass a linear value to a spawned fiber you must move it through a channel. Whether `spawn { }` should support short-lived `@T` capture with a scoped lifetime (bringing the scope lifetime guarantee into the fiber model) is an open design question.

**Impact:** linear values cannot be "inspected" by a spawned fiber without first sending them through a channel.

### Q3 — Error propagation through linear values

When `?` short-circuits out of a region scope (or any block containing live linear values), all in-scope linear bindings must be consumed before the early exit. Whether `?` should trigger automatic `drop` calls for linear values, or require the programmer to restructure error paths, is the "destructor protocol" open question (RFC-0028 OQ-5).

**Impact:** error-handling code paths with linear values are verbose today unless a `Drop` aspect or `#[auto_drop]` mechanism is introduced.

### Q4 — Region-internal `*T`/`*mut T` vs. RC-heap `*T`/`*mut T`

The two-memory-systems model implies that `*T`/`*mut T` inside a region scope are semantically different from `*T`/`*mut T` on the RC heap: they are bump-allocated, carry no refcount, and may form cycles safely. Whether this difference should be visible in the type system (e.g. a separate `~T` region-pointer type) or invisible to the programmer (same syntax, different runtime behavior depending on allocation context) is an open question.

**Impact:** if they are the same type, the programmer cannot tell from a signature whether a pointer is RC-backed or region-backed. If they are different types, the two systems compose less transparently but are more explicit.

### Q5 — `Region::scope` return value and `Send` bound

RFC-0025 Option A uses `Send` as the return constraint, relying on `*T` being non-`Send` to prevent pointer escape. This works, but is conservative: some types that are safe to return from a region scope (e.g. a struct containing a `unique *T` to heap memory that was not region-allocated) would be incorrectly rejected.

**Impact:** the `Send` constraint may need to be refined as `RegionFree` or similar — "contains no pointers into the current region" — rather than the broader "contains no pointers at all."

---

## References

- RFC-0028: Memory and Reference Model — `docs/internal/rfcs/rfc-0028-memory-and-reference-model.md`
- RFC-0025: Region Allocation — `docs/internal/rfcs/rfc-0025-region-allocation.md`
- RFC-0003: Concurrency Model — `docs/internal/rfcs/rfc-0003-concurrency-model.md`
- RFC cluster: Memory Model — `docs/internal/rfc-cluster-memory-model.md`
