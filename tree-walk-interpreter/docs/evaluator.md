# Evaluator Implementation Notes

> Status: PoC complete (Epic 002, Sprint 2).  
> This evaluator is intentionally the simplest correct implementation. It will be rewritten before production use. Do not over-engineer it; open new issues for correctness gaps instead of adding complexity here.

---

## Pipeline Position

```
TypedProgram  ──►  evaluate()  ──►  side effects / RuntimePanic
```

Entry point: `evaluator::evaluate(program: TypedProgram) -> Result<(), GustError>`

The evaluator operates on the `TypedProgram` produced by `typechecker::check()`. It does not re-check types — if the evaluator panics on a type mismatch, that is a typechecker bug, not an evaluator limitation.

Source: `src/evaluator/mod.rs`

---

## Runtime Values

```rust
pub enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),
    Unit,
    Tuple(Vec<Value>),
    Array(Rc<RefCell<Vec<Value>>>),
    Struct { name: String, fields: HashMap<String, Value> },
    Enum   { name: String, variant: String, fields: HashMap<String, Value> },
    Closure(Rc<ClosureValue>),
    Builtin(String, fn(Vec<Value>, &Span) -> Result<Value, GustError>),
    Perhaps(Option<Box<Value>>),
    YoloResult(Result<Box<Value>, Box<Value>>),
}
```

### Array representation

`Value::Array` uses `Rc<RefCell<Vec<Value>>>` so that mutations through `array_push` and index assignment are visible to all aliases of the same array. This gives reference semantics for arrays in the PoC, which matches the intended behaviour for mutable arrays passed to functions.

**Note:** `Value::Perhaps` and `Value::YoloResult` are currently unused at runtime — `Perhaps<T>` values are represented as `Value::Enum { name: "Perhaps", variant: "Some"/"Nope", .. }` and `Result<T,E>` as `Value::Enum { name: "Result", variant: "Ok"/"Err", .. }`. These variants are relics of an earlier design. They can be removed when the evaluator is rewritten.

### Range representation

`a..b` evaluates to `Value::Struct { name: "Range", fields: { start: Int, end: Int } }`. This is an ad-hoc struct, not a typed `Range` struct — it exists so `for-in` can inspect the fields without a dedicated type. Same pattern for `a..=b` → `"RangeInclusive"`.

---

## Signal-Based Control Flow

All evaluation functions return `Result<Signal, GustError>`:

```rust
pub enum Signal {
    Value(Value),
    Return(Value),
    Break(Value),        // carries the break-expression value
    Continue,
    PropagateErr(Value), // the ? operator
}
```

`Signal::Value` is the normal case. The others implement non-local control flow by propagating up the call stack until handled:

| Signal | Consumed by |
|---|---|
| `Return(v)` | `call_function` — converts to `Signal::Value(v)` at the function boundary |
| `Break(v)` | `Expr::Loop` handler — exits the loop, returns `Signal::Value(v)` |
| `Continue` | `While`, `For`, `ForIn` loop bodies — skips to next iteration |
| `PropagateErr(e)` | `Expr::PropagateError` handler — or `call_function`, which wraps `e` in `Result::Err { error: e }` |

`Signal::into_value()` is a convenience that panics on non-Value signals. It is used at call sites where the typechecker guarantees the expression cannot diverge (e.g., function arguments, struct field expressions). If it panics, that indicates a typechecker bug.

---

## Environment

```rust
pub struct Environment {
    scopes: Vec<HashMap<String, Rc<RefCell<Value>>>>,
}
```

Each binding is stored as an `Rc<RefCell<Value>>`. This has two consequences:

1. **Mutation is visible through the scope chain.** `env.set(name, val)` finds the binding's `Rc` in any enclosing scope and mutates through it. This correctly implements `mut` re-assignment without requiring the caller to traverse scopes differently for reads vs writes.

2. **Closures share mutable state with their definition scope.** `env.clone()` clones the `HashMap`s, but each `Rc<RefCell<Value>>` clone is a shared pointer — not a deep copy. A closure that captures a binding and the enclosing scope that owns that binding share the same `RefCell`. This gives reference semantics for captured mutable variables.

   This is an unintentional consequence of the PoC design. RFC-0006 (closure capture semantics) will establish the intended semantics. For now, any program that relies on closures sharing mutable state with their enclosing scope may produce surprising results, and any program that expects clone-at-definition isolation may also be surprised. The test suite avoids this ambiguity.

---

## Evaluation Entry Point

`evaluate()` runs three passes over the top-level declarations:

**Pass 1a — Define placeholders:**
Every top-level `Fun` and `Impl` method is bound to `Value::Unit` in the root environment. This ensures the names exist before any closure is created, so closures formed in Pass 1b can capture them via shared `Rc`s.

**Pass 1b — Create closures:**
Every top-level `Fun` and `Impl` method clones the full current environment and creates a `Value::Closure`. The clone captures the `Rc`s from Pass 1a, not copies of `Value::Unit`. `env.set()` then mutates those `Rc`s in place.

Because all closures from Pass 1b share the same set of `Rc`s, after Pass 1b completes every closure's captured environment already contains references to every other closure — including those defined after it. This "ties the knot" for mutual recursion without a fixpoint pass or separate reference-resolution step.

**Pass 2 — Evaluate bindings:**
Top-level `let`/`mut` bindings and statements are evaluated in order. `Fun` and `Impl` declarations are skipped (already handled in 1a/1b).

**Call `main()`:**
`main`'s body is executed directly in the root environment so that top-level `let`/`mut` bindings from Pass 2 are visible. `Signal::Return` from `main` is treated as a normal exit.

### Self-recursion inside blocks

`eval_decl` for `Fun` uses the same define-placeholder / clone / set pattern as the top-level pass so that functions defined inside a block can call themselves recursively.

---

## Closure Capture

At closure definition (`TypedExpr::Closure`), the evaluator clones the entire current environment:

```rust
let captured = env.clone();
```

As noted in the Environment section, this clone shares `Rc`s rather than deep-copying values. The captured environment is stored in `ClosureValue.captured`.

At call time (`call_function`), `captured` is cloned again and a new scope is pushed for the parameters:

```rust
let mut call_env = closure.captured.clone();
call_env.push_scope();
```

This means:
- Each call to the same closure gets a fresh parameter scope.
- The captured variable `Rc`s are shared across all calls — mutations to captured variables persist between calls to the same closure.

**This is not the intended permanent semantics.** See RFC-0006.

---

## Pattern Matching

`match_pattern(pattern, value, out)` returns `bool` and writes bindings into `out: &mut HashMap<String, Value>`. It does not mutate the environment directly — the caller pushes a scope and inserts the bindings after a successful match.

Guarded arms: the guard is evaluated in a temporary scope containing the pattern bindings. If the guard returns `false`, the scope is popped and the next arm is tried. Pattern bindings accumulated so far are discarded (the `out` map is not reused between arms).

The evaluator will panic with `"match: no arm matched scrutinee"` if no arm matches at runtime. The typechecker's exhaustiveness check (E0008) is the static guarantee that this panic is unreachable for well-typed programs.

---

## Function Call Dispatch

`call_function(func, args, span)` handles three cases:

- `Value::Builtin(_, f)` — calls the function pointer directly.
- `Value::Closure(rc)` — clones the captured environment, pushes a parameter scope, evaluates the body, and converts `Signal::Return` to `Signal::Value` at the boundary. `Signal::PropagateErr` is also converted: it wraps the error value in `Value::Enum { name: "Result", variant: "Err", .. }` and returns `Signal::Value` of that — so the `?` error appears as a `Result::Err` value to the caller.
- `Value::Unit` — panics with "generic function not supported in v0.1". Top-level generic functions have `FunBody::Generic` and are registered as `Value::Unit` (Pass 1a, never overwritten in 1b). This is the Epic 003 placeholder.

---

## Known Limitations

### Index assignment — identifier only

`arr[expr] = val` only supports an identifier or integer literal as the index expression. Complex index expressions (`arr[f(x)] = v`) are rejected at runtime with a message asking the programmer to assign the index to a variable first. This is a PoC simplification — the typechecker does not enforce this restriction, so a valid type-checked program can produce a runtime error here.

### Field and index assignment — direct variable only

`obj.field = val` and `arr[i] = val` only support a bare identifier on the left-hand side (e.g., `foo.bar = 1` works; `get_foo().bar = 1` panics). The typechecker does not validate this shape, so a well-typed program can reach this panic. Fixing it requires the evaluator to support lvalue paths rather than just names.

### Generic functions — not callable

Generic functions produce `Value::Unit` and calling them panics. This is intentional for Epic 003. No test calls a generic function at the value level.

### `Perhaps` and `YoloResult` variants unused

`Value::Perhaps` and `Value::YoloResult` are defined but never constructed by the evaluator. `Perhaps` values are `Value::Enum { name: "Perhaps", .. }` at runtime. These dead variants should be removed when the evaluator is rewritten.

### Closure/scope mutation semantics unspecified

The PoC's `Rc<RefCell<Value>>` environment gives closures reference semantics for captured variables, which is not the intended permanent behaviour (see RFC-0006). Do not write tests that rely on cross-closure mutation sharing unless they explicitly document the dependency.

---

## Extension Points

### Epic 003 — Generics

Replace the `FunBody::Generic` early-return in `eval_decl` with monomorphization. At call time, specialize the untyped body against the concrete argument types (requires a mini type-check pass or a pre-monomorphized TypedAST).

### Epic 004 — Traits / `?` coercion

`PropagateError` currently requires `Value::Enum { name: "Result", .. }`. Upgrading `?` to use `From<E>` coercion (spec [The ? Operator](../../../public/spec/functions.md#the--operator)) requires looking up a `From` impl at the call site and applying the conversion before wrapping.

### Rewrite

The evaluator is designed to be thrown away. The correct rewrite path is:
1. Decide the permanent value representation (likely a tagged pointer or NaN-boxing scheme).
2. Implement RFC-0006 capture semantics (explicit pointer types for aliasing).
3. Implement the module system (RFC pending) before the evaluator is shared as a library.
