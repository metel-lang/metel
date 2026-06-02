<p align="center">
  <img src="media/metel-logo-dark.svg" alt="Metel" width="600"/>
</p>

An exploration in type-driven memory management.

## Why?

You surely know as well as I do that the world does not need yet another amateur Rust clone, so let me be clear with you: The main reason I built this is that I wanted to.

It began as "Yoloscript," a silly experiment based on Lox from Crafting Interpreters. But somewhere along the way, it got ambitious. Statically typed? Sure. Written in Rust? Why not. Operator overloads, pointers, concurrency? Let's try it all.

The current frontier is linear types—baked in as an opt-in pillar of the memory model. The mission: discover what this system can express and where it breaks.

> Fair warning: this project's development is powered by some serious AI machinery, and it probably shows in a lot of places.

## What?

### Available now (v0.7.0)

- **Strong static typing** with local type inference (Hindley-Milner)
- **Algebraic data types** — structs and enums with data-carrying variants
- **Exhaustive pattern matching**
- **Explicit nullability** via `Perhaps<T>` (no null pointers)
- **Explicit error handling** via `Result<T, E>` with `?` propagation — including cross-type coercion via `From<E>`
- **First-class functions** and closures (`(x) -> T { ... }` syntax)
- **Generics** — generic functions, structs, and enums with full monomorphisation
- **Aspect bounds** — inline (`<T: Comparable>`), multi-bound (`<T: Comparable + Display>`), `where` clauses, and `impl Aspect` anonymous parameters
- **Aspects** — user-defined interfaces (`aspect Foo { ... }`) with `impl Aspect for Type` dispatch, including default methods
- **Explicit receiver semantics** — `&self` (shared read) and `&mut self` (shared mutable) method receivers
- **Pointer types** — `&expr` and `&mut expr` produce `Pointer<T>` and `MutPointer<T>`; assignment through `*ptr` is supported
- **Module system** — multi-file programs via `import` and `export`; visibility enforced with `pub`; `std::core` auto-imported
- **`Iterable<T>`** — implement `for-in` on your own types
- **`From<T>`** — implement `as` casts between any two types
- **`Display`** — `.to_string()` on all built-in types; polymorphic `print`/`println`
- **String interpolation** — `"Hello, ${name}!"` desugars to string concatenation
- **Runtime memory management** via reference counting

### Planned

- **Opt-in linear types** — the `linear` keyword marks a type as use-exactly-once. The type checker statically prevents resource leaks, double-frees, and unconsumed handles. No runtime overhead; in the compiler, linear values are freed deterministically with zero allocator cost.

- **Fiber green threads** — lightweight concurrent tasks launched with `spawn { }`. M:N scheduled by the runtime; no `async`/`await`, no function colouring. A function that blocks inside a fiber does not need a different declaration.

- **Typed channels** — `Chan<T>` is the primary concurrency primitive. Values are transferred between fibers with `ch <- value` (send) and `<- ch` (receive). A `select` expression waits on multiple channels simultaneously. Channels are the natural transport for linear values: sending consumes the value, satisfying the exactly-once rule across fiber boundaries.

- **C FFI** — `extern "C"` blocks declare functions callable via the C ABI. Calls require an `unsafe` block. The primary use case is Rust crate interop: any Rust crate can be exposed to Metel through a thin `#[no_mangle] extern "C"` shim, giving access to the full `crates.io` ecosystem.

See the [Language Specification](docs/public/reference/spec.md) and [RFCs](docs/internal/rfcs/) for the complete design.

## Quick Start

### Prerequisites

- Rust 1.70+
- Cargo

### Build

```bash
cd metel-interpreter
cargo build --release
```

### Run a Program

```bash
cargo run -- path/to/program.mtl
```

### Run Tests

```bash
# All tests
cargo test

# Type inference unit tests
cargo test --test typeinference_tests

# Typechecking integration tests
cargo test --test typechecking_tests

# Evaluator integration tests
cargo test --test evaluator_tests
```

## Examples

**Algebraic types and pattern matching**

```metel
enum Shape {
    Circle    { radius: Float },
    Rectangle { width: Float, height: Float },
}

fun area(s: Shape) -> Float {
    match s {
        Shape::Circle { radius }           => 3.14159 * radius * radius,
        Shape::Rectangle { width, height } => width * height,
    }
}
```

**Generics with aspect bounds**

```metel
fun first<T>(items: T[]) -> Perhaps<T> {
    if (array_len(items) == 0) { None }
    else { items[0] }
}

fun max_of<T: Comparable>(a: T, b: T) -> T {
    if (a > b) { a } else { b }
}
```

**Aspects**

```metel
aspect Summary {
    fun summarize(&self) -> String;
}

struct Article { pub title: String, pub author: String }
struct Tweet   { pub content: String }

impl Summary for Article {
    fun summarize(&self) -> String { self.title + " — " + self.author }
}
impl Summary for Tweet {
    fun summarize(&self) -> String { self.content }
}
```

**Error propagation with `From`-based coercion**

```metel
enum IoError  { FileNotFound {} }
enum AppError { Io { msg: String } }

impl From<IoError> for AppError {
    fun from(e: IoError) -> AppError { AppError::Io { msg: "file not found" } }
}

fun load_config() -> Result<String, AppError> {
    let data = read_file("config.toml")?;  // IoError coerced to AppError via From
    Result::Ok { value: data }
}
```

**User-defined iteration**

```metel
struct Counter { current: Int, max: Int }

impl Iterable<Int> for Counter {
    fun next(&mut self) -> Perhaps<Int> {
        if (self.current >= self.max) { None }
        else {
            self.current = self.current + 1;
            self.current
        }
    }
}

fun main() {
    for (let n in Counter { current: 0, max: 5 }) {
        println(n);  // 1 2 3 4 5
    }
}
```

**String interpolation**

```metel
fun greet(name: String, age: Int) -> String {
    "Hello, ${name}! You are ${age} years old."
}
```

**Modules**

```metel
// shapes.mtl
pub struct Circle { pub radius: Float }

pub fun area(c: Circle) -> Float {
    3.14159 * c.radius * c.radius
}

// main.mtl
import shapes::Circle;
import shapes::area;

fun main() {
    let c = Circle { radius: 2.0 };
    println(area(c));
}
```

## Project Structure

```
Metel/
├── metel-interpreter/
│   ├── src/
│   │   ├── parser/         # PEG grammar (pest) → untyped AST
│   │   ├── ast/            # Untyped AST node definitions
│   │   ├── typeinference/  # HM inference engine
│   │   ├── typechecker/    # Two-pass type checker → typed AST
│   │   ├── typed_ast/      # Typed AST node definitions
│   │   ├── evaluator/      # Tree-walking evaluator
│   │   ├── types/          # Concrete type representation
│   │   └── error/          # Unified error type
│   ├── tests/
│   │   ├── typeinference/  # HM engine unit tests
│   │   ├── typechecking/   # Full pipeline integration tests
│   │   ├── evaluator/      # Runtime behaviour tests
│   │   ├── module_loading/ # Module graph and path resolution tests
│   │   ├── module_semantics/ # Cross-module visibility and import tests
│   │   └── parsing/        # Parser tests
│   └── Cargo.toml
│
└── docs/           # Spec, RFCs, Changelog
```

## Resources

- **Language Specification:** [`docs/public/reference/spec.md`](docs/public/reference/spec.md)
- **Changelog:** [`docs/public/release-notes/changelog.md`](docs/public/release-notes/changelog.md)
- **Typechecker Architecture:** [`metel-interpreter/docs/typechecker.md`](metel-interpreter/docs/typechecker.md)
- **Evaluator Design:** [`metel-interpreter/docs/evaluator.md`](metel-interpreter/docs/evaluator.md)
- **RFCs:** [`docs/internal/rfcs/`](docs/internal/rfcs/) — language change proposals and decisions
- **Decision Records:** [`metel-interpreter/docs/decisions/`](metel-interpreter/docs/decisions/) — implementation rationales

## License

Metel is licensed under the Apache License 2.0. See the [LICENSE](LICENSE) file for details.
