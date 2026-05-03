# Yolang Interpreter Design

> See `decisions/0002-interpreter-architecture.md` for the rationale behind the architectural choices made here.

---

## Pipeline

```
.yolo source file
       │
       ▼
  ┌─────────┐
  │  Parser │  pest PEG grammar → concrete syntax tree (CST)
  └─────────┘
       │
       ▼
  ┌─────────────┐
  │ AST Builder │  CST → untyped abstract syntax tree
  └─────────────┘
       │
       ▼
  ┌──────────────┐
  │ Type Checker │  untyped AST → typed AST  (errors reported here)
  └──────────────┘
       │
       ▼
  ┌─────────────┐
  │  Evaluator  │  typed AST → program output  (tree-walking)
  └─────────────┘
```

Each stage is a separate Rust module. They communicate through well-defined data structures. No stage skips another.

---

## Crate structure

```
yolang/
├── Cargo.toml
└── src/
    ├── main.rs          — CLI entry point: reads a .yolo file, runs the pipeline
    ├── grammar.pest     — pest PEG grammar for the full v0.1 language
    ├── parser/
    │   └── mod.rs       — drives pest, builds the untyped AST from the CST
    ├── ast/
    │   └── mod.rs       — untyped AST node definitions
    ├── types/
    │   └── mod.rs       — type representation used by the type checker and evaluator
    ├── typechecker/
    │   └── mod.rs       — type inference, type checking, monomorphisation
    ├── typed_ast/
    │   └── mod.rs       — typed AST node definitions (AST nodes annotated with resolved types)
    ├── evaluator/
    │   └── mod.rs       — tree-walking evaluator, environment, runtime values
    └── error/
        └── mod.rs       — unified error type covering all pipeline stages
```

---

## AST design

### Untyped AST (`ast/`)

Produced by the parser. Expressions carry a `Span` (byte range in source) for error reporting but no type information.

Key node categories:

```rust
// Top-level
enum Decl {
    Let { name, type_ann, value, span },
    Mut { name, type_ann, value, span },
    Fun { name, generics, params, return_type, body, span },
    Struct { name, generics, fields, span },
    Enum { name, generics, variants, span },
    Impl { trait_name, target_type, methods, span },
    Trait { name, methods, span },
    Stmt(Stmt),
}

// Statements
enum Stmt {
    Expr(Expr),
    Block(Vec<Decl>),
    If { condition, then_branch, else_branch, span },
    While { condition, body, span },
    For { init, condition, step, body, span },
    ForIn { binding, iterable, body, span },
    Loop { body, span },
    Return { value, span },
    Break { value, span },
    Continue { span },
}

// Expressions
enum Expr {
    Literal(Literal),
    Ident(String, Span),
    Tuple(Vec<Expr>, Span),
    Array(Vec<Expr>, Span),
    BinOp { op, left, right, span },
    UnaryOp { op, operand, span },
    Assign { target, op, value, span },
    Call { callee, args, span },
    MethodCall { receiver, method, args, span },
    FieldAccess { object, field, span },
    Index { object, index, span },
    Cast { expr, target_type, span },
    Match { scrutinee, arms, span },
    If { condition, then_branch, else_branch, span },
    Loop { body, span },
    Closure { params, return_type, body, span },
    StructLiteral { path, fields, span },
    PropagateError { expr, span },   // the ? operator
    TupleAccess { object, index, span },
    Path(Vec<String>, Span),         // e.g. Direction::North
}
```

### Typed AST (`typed_ast/`)

Produced by the type checker. Mirrors the untyped AST but every expression node carries a `Type`. Generic functions and types have been monomorphised — there are no type variables in the typed AST.

---

## Type representation (`types/`)

```rust
enum Type {
    Int,
    Float,
    Bool,
    Str,                         // String
    Unit,                        // ()
    Tuple(Vec<Type>),
    Array(Box<Type>),            // T[]
    Fun(Vec<Type>, Box<Type>),   // fun(A, B) -> C
    Named(String, Vec<Type>),    // Struct/Enum name + concrete type args (post-monomorphisation)
    Perhaps(Box<Type>),          // Perhaps<T>  (sugar over Named)
    Result(Box<Type>, Box<Type>),// Result<T,E> (sugar over Named)
}
```

Generics exist only in the type checker as `TypeVar(u32)` unification variables. They are fully resolved before the typed AST is produced.

---

## Type checker (`typechecker/`)

Responsibilities:
1. **Name resolution** — resolve all identifiers to their declarations
2. **Type inference** — infer types for all bindings using Hindley-Milner inference with let-polymorphism; annotations are permitted everywhere but required nowhere (except struct/enum fields and trait method signatures)
3. **Type checking** — verify every expression is used consistently with its type
4. **Trait checking** — verify that trait bounds on generic parameters are satisfied at every call site
5. **Monomorphisation** — for each generic function/type instantiation, produce a concrete specialisation and record it in a monomorphisation table
6. **Exhaustiveness checking** — verify that every `match` covers all cases

The type checker maintains:
- A **type environment** (scope stack mapping names to types)
- A **declaration table** (all top-level struct, enum, trait, impl declarations)
- A **monomorphisation cache** (concrete instantiations already produced, to avoid duplicates)

Error reporting uses the `Span` from the untyped AST to produce source-level error messages.

---

## Evaluator (`evaluator/`)

### Runtime values

```rust
enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),
    Unit,
    Tuple(Vec<Value>),
    Array(Rc<RefCell<Vec<Value>>>),       // mutable, ref-counted
    Struct { name: String, fields: HashMap<String, Value> },
    Enum { name: String, variant: String, fields: HashMap<String, Value> },
    Function(FunctionValue),
    Closure(ClosureValue),
    Perhaps(Option<Box<Value>>),
    Result(std::result::Result<Box<Value>, Box<Value>>),
}

struct FunctionValue {
    params: Vec<String>,
    body: TypedBlock,
    env: Environment,   // captured environment for closures; empty for plain functions
}

struct ClosureValue {
    params: Vec<String>,
    body: TypedBlock,
    env: Environment,   // captured bindings at closure creation time
}
```

### Environment

The environment is a **stack of scopes**, each scope being a `HashMap<String, Rc<RefCell<Value>>>`. Using `Rc<RefCell<Value>>` for all bindings makes `mut` capture by closures work naturally — closures share the same `Rc` as the outer scope.

```rust
struct Environment {
    scopes: Vec<HashMap<String, Rc<RefCell<Value>>>>,
}
```

Operations: `push_scope`, `pop_scope`, `define(name, value)`, `get(name)`, `set(name, value)`.

### Evaluation

The evaluator is a recursive function over typed AST nodes. It takes a node and an `Environment` and returns a `Value` (or a control flow signal).

Control flow (`break`, `continue`, `return`, `?`) is handled via a **signal enum** returned alongside values, rather than Rust exceptions or panics:

```rust
enum Signal {
    Value(Value),
    Return(Value),
    Break(Value),     // carries the break value for `loop { break expr; }`
    Continue,
    PropagateErr(Value),  // the ? operator
}
```

Each evaluation function returns `Result<Signal, RuntimeError>`. The evaluator propagates signals up the call stack until they are consumed by the appropriate construct (`loop` consumes `Break`, function call consumes `Return`, etc.).

### Built-in functions

Built-ins are registered in the root environment before evaluation begins, implemented as a special `Value::Builtin(fn(...) -> Value)` variant. Each built-in from the spec (`print`, `println`, `array_push`, `array_len`, etc.) is one entry.

---

## Error handling

All errors (parse errors, type errors, runtime panics) use a unified `YolangError` type:

```rust
enum YolangError {
    ParseError { message: String, span: Span },
    TypeError { message: String, span: Span },
    RuntimePanic { message: String, span: Span },
}
```

Runtime panics (`.yolo()` on `nope`, out-of-bounds, division by zero) produce `RuntimePanic` and immediately terminate the interpreter with a non-zero exit code, printing the message and source location. This matches the spec's panic semantics.

---

## Memory model

The interpreter uses Rust's `Rc<RefCell<T>>` for all mutable values. This gives reference-counting semantics that match the spec ("memory managed by the runtime — reference counting"). Cycle collection is not in scope for v0.1 (cycles require deliberate effort to create without raw pointers).

---

## Implementation order

Build and validate one pipeline stage at a time, in dependency order. Do not move to the next stage until the current one has passing tests.

1. **Grammar + parser** — get the pest grammar parsing all 10 test files into a valid CST
2. **AST builder** — convert the CST into the untyped AST; verify with hand-checked AST dumps
3. **Type checker (core)** — primitives, variables, functions, arithmetic; no generics yet
4. **Evaluator (core)** — same scope as the type checker; run test files 01 and 02
5. **Type checker + evaluator: structs and enums** — test files 04 and 05
6. **Type checker + evaluator: traits** — test file 06
7. **Type checker + evaluator: generics + monomorphisation** — test files 09, parts of 10
8. **Type checker + evaluator: arrays + tuples** — test file 07
9. **Type checker + evaluator: error handling (`?`, `.yolo()`)** — test file 08
10. **Type checker + evaluator: closures** — test file 03
11. **Full integration** — all 10 test files pass with correct output
12. **Mark spec sections as interpreter-validated**
