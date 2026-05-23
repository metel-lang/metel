# Gust

A statically typed, expression-oriented language with first-class functions, explicit nullability, and a compiler at the horizon.

## Why?

A sudden gust of inspiration — sometimes a project just takes flight on its own momentum. This one started with a suggestion: write an interpreter. The first iteration was chaotic momentum (hence the predecessor's name). The second iteration aims for clarity and structure, guided by better design up front.

Gust draws heavily from Rust's approach: strong static typing, algebraic data types, explicit error handling — but designed to be learnable without the borrow checker's complexity. The goal is a language that feels safe and expressive without requiring deep knowledge of ownership mechanics.

The implementation is intentionally simple: a tree-walk interpreter in Rust, paired with a living specification. They evolve together in a tight feedback loop, with real programs revealing gaps and design flaws before they ossify in a compiler.

## What?

Gust is a statically typed, expression-oriented programming language. It features:

- **Strong static typing** with local type inference (Hindley-Milner)
- **Algebraic data types** — enums with data-carrying variants
- **Exhaustive pattern matching**
- **Explicit nullability** via `Perhaps<T>` (no null pointers)
- **Explicit error handling** via `Result<T, E>`
- **First-class functions** and closures
- **Generics** with compile-time monomorphization
- **Traits** for ad-hoc polymorphism
- **Memory managed by the runtime** (reference counting)

See the Language Specification for the complete definition.


## How?

The spec and the interpreter are developed in parallel, in a tight loop:

```
Define a feature in the spec
        ↓
Implement it in the interpreter
        ↓
Write real programs using it
        ↓
Observe gaps, wrong assumptions, usability issues
        ↓
Refine the spec  →  implement the refinement  →  next feature
```

The spec is the source of truth within each iteration — no code diverges from it — but the spec itself is a living document expected to evolve through usage. The tree-walk interpreter is the feedback mechanism: fast enough to iterate on, disposable enough not to over-invest in.

## Quick Start

### Prerequisites

- Rust 1.70+
- Cargo

### Build

```bash
cd gust-interpreter
cargo build --release
```

### Run a Program

```bash
cargo run -- path/to/program.gust
```

### Run Tests

```bash
# All tests
cargo test

# Type inference unit tests
cargo test --test lib typeinference_tests

# Typechecking integration tests
cargo test --test lib typechecking_tests
```

## Example

```gust
fun factorial(n: Int) -> Int {
    if (n <= 1) { 1 } else { n * factorial(n - 1) }
}

let result = factorial(5);
```

## Project Structure

```
Gust/
├── gust-interpreter/
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
│   │   ├── lib.rs
│   │   ├── typeinference/  # HM engine unit tests (phases 1–7)
│   │   ├── typechecking/   # Full pipeline integration tests
│   │   └── parsing/        # Parser tests
│   └── Cargo.toml
│
└── docs/           # Spec, RFCs, Changelog
```

## Resources

- **Language Specification:** [`docs/public/spec.md`](docs/public/spec.md)
- **Typechecker Architecture:** [`gust-interpreter/docs/typechecker.md`](gust-interpreter/docs/typechecker.md)
- **Evaluator Design:** [`gust-interpreter/docs/evaluator.md`](gust-interpreter/docs/evaluator.md)
- **RFCs:** [`docs/internal/rfcs/`](docs/internal/rfcs/) — language change proposals and decisions
- **Decision Records:** [`gust-interpreter/docs/decisions/`](gust-interpreter/docs/decisions/) — implementation rationales

## License

Gust is licensed under the Apache License 2.0. See the [LICENSE](LICENSE) file for details.
